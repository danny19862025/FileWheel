//! Windows integration: process trees, the always-on-top drop target, the
//! notification-area icon, Explorer reveal, dark mode and the hover sound.
use std::cell::RefCell;
use std::ffi::{OsStr, c_void};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command};
use std::rc::Rc;

use gtk::prelude::*;
use windows_sys::Win32::Foundation::{
    CloseHandle, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT, STILL_ACTIVE, WPARAM,
};
use windows_sys::Win32::Graphics::Dwm::{
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND, DwmSetWindowAttribute,
};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};
use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_HIDDEN, GetFileAttributesW, INVALID_FILE_ATTRIBUTES, SetFileAttributesW,
};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject, TerminateJobObject,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows_sys::Win32::System::Threading::{
    CREATE_NO_WINDOW, GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconFromResourceEx, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
    DestroyIcon, DestroyMenu, DestroyWindow, GWL_EXSTYLE, GetCursorPos, GetSystemMetrics,
    GetWindowLongPtrW, GetWindowRect, HICON, HWND_TOPMOST, IMAGE_ICON, LR_DEFAULTCOLOR, LoadImageW,
    MF_SEPARATOR, MF_STRING, PostMessageW, RegisterClassW, RegisterWindowMessageW, SM_CXSMICON,
    SM_CYSMICON, SWP_NOACTIVATE, SWP_NOSIZE, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, WM_APP, WM_CONTEXTMENU, WM_LBUTTONUP, WM_NULL,
    WM_RBUTTONUP, WNDCLASSW, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW, WS_OVERLAPPED,
};

fn wide(text: impl AsRef<OsStr>) -> Vec<u16> {
    text.as_ref().encode_wide().chain(Some(0)).collect()
}

// ---------------------------------------------------------------- processes

/// A Job Object holding a converter and everything it starts. LibreOffice's
/// soffice.com, for example, runs the real work in a child soffice.bin.
pub struct Job(HANDLE);
// The handle is only used by the thread that runs the converter.
unsafe impl Send for Job {}

impl Job {
    pub fn prepare(command: &mut Command) {
        // FileWheel is a GUI program: engines must not flash console windows.
        command.creation_flags(CREATE_NO_WINDOW);
    }
    pub fn attach(child: &Child) -> Option<Self> {
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return None;
            }
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let ok = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            ) != 0
                && AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) != 0;
            if ok {
                Some(Self(job))
            } else {
                CloseHandle(job);
                None
            }
        }
    }
    pub fn kill(&self) {
        unsafe {
            TerminateJobObject(self.0, 1);
        }
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        // KILL_ON_JOB_CLOSE also stops helpers left running after the engine exits.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn pid_alive(pid: u32) -> bool {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return false;
        }
        let mut code = 0u32;
        let alive = GetExitCodeProcess(process, &mut code) != 0 && code == STILL_ACTIVE as u32;
        CloseHandle(process);
        alive
    }
}

/// Work folders beside the user's files start with a dot, which only hides
/// them on Unix; mark them hidden for Explorer too.
pub fn hide(path: &Path) {
    let name = wide(path);
    unsafe {
        let attributes = GetFileAttributesW(name.as_ptr());
        if attributes != INVALID_FILE_ATTRIBUTES {
            SetFileAttributesW(name.as_ptr(), attributes | FILE_ATTRIBUTE_HIDDEN);
        }
    }
}

// ------------------------------------------------------------- desktop bits

pub fn reveal(path: &Path) {
    // Explorer parses its own command line; the quoted path keeps spaces intact.
    let _ = Command::new("explorer.exe")
        .raw_arg(format!("/select,\"{}\"", path.display()))
        .spawn();
}

/// Windows "app mode": `Some(true)` for dark, `None` if the setting is absent.
pub fn system_dark() -> Option<bool> {
    let key = wide(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize");
    let value = wide("AppsUseLightTheme");
    let mut data = 0u32;
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            value.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            (&mut data as *mut u32).cast(),
            &mut size,
        )
    };
    (status == 0).then_some(data == 0)
}

pub fn play_sound(wav: &'static [u8]) -> Result<(), String> {
    let ok = unsafe {
        PlaySoundW(
            wav.as_ptr().cast(),
            std::ptr::null_mut(),
            SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
        )
    };
    if ok != 0 {
        Ok(())
    } else {
        Err("Windows could not play the hover sound.".into())
    }
}

// ------------------------------------------------------- floating windows

unsafe extern "C" {
    fn gdk_win32_surface_get_handle(surface: *mut c_void) -> HWND;
}

fn hwnd(window: &gtk::Window) -> Option<HWND> {
    let surface = window.surface()?;
    let handle = unsafe { gdk_win32_surface_get_handle(surface.as_ptr().cast()) };
    (!handle.is_null()).then_some(handle)
}

/// Keep a borderless window above other windows, out of the taskbar, and
/// vertically centred against the right edge of its monitor's work area —
/// the Windows counterpart of the Wayland layer-shell placement.
pub fn pin_right_edge(window: &gtk::Window, right_margin: i32) {
    let margin = right_margin;
    window.connect_map(move |window| {
        let window = window.clone();
        // Let GTK finish its own placement first.
        gtk::glib::idle_add_local_once(move || place(&window, margin));
    });
}

fn place(window: &gtk::Window, right_margin: i32) {
    let Some(hwnd) = hwnd(window) else {
        return;
    };
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let style = (style | WS_EX_TOOLWINDOW as isize) & !(WS_EX_APPWINDOW as isize);
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style);
        let corner = DWMWCP_ROUND;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE as u32,
            (&corner as *const i32).cast(),
            std::mem::size_of::<i32>() as u32,
        );
        let mut rect: RECT = std::mem::zeroed();
        GetWindowRect(hwnd, &mut rect);
        let mut info: MONITORINFO = std::mem::zeroed();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut info) == 0 {
            return;
        }
        let scale = GetDpiForWindow(hwnd).max(96) as f64 / 96.;
        let work = info.rcWork;
        let (width, height) = (rect.right - rect.left, rect.bottom - rect.top);
        let x = work.right - width - (right_margin as f64 * scale).round() as i32;
        let y = work.top + ((work.bottom - work.top) - height) / 2;
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, 0, 0, SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

// --------------------------------------------------------------- tray icon

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    ToggleTarget,
    Quit,
}

const TRAY_MESSAGE: u32 = WM_APP + 1;
const MENU_OPEN: usize = 1;
const MENU_TARGET: usize = 2;
const MENU_QUIT: usize = 3;

struct Tray {
    window: HWND,
    icon: HICON,
    taskbar_created: u32,
    handler: Rc<dyn Fn(TrayAction)>,
}

thread_local! {
    // Window procedures are plain functions; the tray lives on GTK's thread.
    static TRAY: RefCell<Option<Tray>> = const { RefCell::new(None) };
}

fn notify_data(window: HWND, icon: HICON) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = window;
    data.uID = 1;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = TRAY_MESSAGE;
    data.hIcon = icon;
    for (slot, unit) in data.szTip.iter_mut().zip("FileWheel".encode_utf16()) {
        *slot = unit;
    }
    data
}

fn small_icon(png: &'static [u8]) -> HICON {
    unsafe {
        let (cx, cy) = (GetSystemMetrics(SM_CXSMICON), GetSystemMetrics(SM_CYSMICON));
        // Resource 1 is the .exe icon embedded by build.rs.
        let icon = LoadImageW(
            GetModuleHandleW(std::ptr::null()),
            1 as _,
            IMAGE_ICON,
            cx,
            cy,
            LR_DEFAULTCOLOR,
        );
        if !icon.is_null() {
            return icon;
        }
        CreateIconFromResourceEx(png.as_ptr(), png.len() as u32, 1, 0x0003_0000, cx, cy, 0)
    }
}

unsafe extern "system" fn tray_proc(window: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    let state = TRAY.with(|tray| {
        tray.borrow()
            .as_ref()
            .map(|t| (t.taskbar_created, t.icon, t.handler.clone()))
    });
    if let Some((taskbar_created, icon, handler)) = state {
        if message == taskbar_created {
            // Explorer restarted: put the icon back.
            unsafe { Shell_NotifyIconW(NIM_ADD, &notify_data(window, icon)) };
            return 0;
        }
        if message == TRAY_MESSAGE {
            let action = match (l & 0xffff) as u32 {
                WM_LBUTTONUP => Some(TrayAction::Open),
                WM_RBUTTONUP | WM_CONTEXTMENU => menu(window),
                _ => None,
            };
            if let Some(action) = action {
                handler(action);
            }
            return 0;
        }
    }
    unsafe { DefWindowProcW(window, message, w, l) }
}

fn menu(window: HWND) -> Option<TrayAction> {
    unsafe {
        let menu = CreatePopupMenu();
        if menu.is_null() {
            return None;
        }
        let open = wide("Open FileWheel");
        let target = wide("Show/hide drop target");
        let quit = wide("Quit");
        AppendMenuW(menu, MF_STRING, MENU_OPEN, open.as_ptr());
        AppendMenuW(menu, MF_STRING, MENU_TARGET, target.as_ptr());
        AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(menu, MF_STRING, MENU_QUIT, quit.as_ptr());
        let mut point = POINT { x: 0, y: 0 };
        GetCursorPos(&mut point);
        // Required so the menu closes when the user clicks elsewhere.
        SetForegroundWindow(window);
        let chosen = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_RIGHTBUTTON,
            point.x,
            point.y,
            0,
            window,
            std::ptr::null(),
        ) as usize;
        PostMessageW(window, WM_NULL, 0, 0);
        DestroyMenu(menu);
        match chosen {
            MENU_OPEN => Some(TrayAction::Open),
            MENU_TARGET => Some(TrayAction::ToggleTarget),
            MENU_QUIT => Some(TrayAction::Quit),
            _ => None,
        }
    }
}

/// Add the notification-area icon. Returns false if Windows refused it.
pub fn install_tray(png: &'static [u8], handler: impl Fn(TrayAction) + 'static) -> bool {
    unsafe {
        let instance = GetModuleHandleW(std::ptr::null());
        let class = wide("FileWheelTray");
        let mut wc: WNDCLASSW = std::mem::zeroed();
        wc.lpfnWndProc = Some(tray_proc);
        wc.hInstance = instance;
        wc.lpszClassName = class.as_ptr();
        RegisterClassW(&wc);
        // A hidden top-level window (not message-only) so it also receives
        // the "TaskbarCreated" broadcast after Explorer restarts.
        let window = CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        if window.is_null() {
            return false;
        }
        let icon = small_icon(png);
        let taskbar_created = RegisterWindowMessageW(wide("TaskbarCreated").as_ptr());
        if Shell_NotifyIconW(NIM_ADD, &notify_data(window, icon)) == 0 {
            DestroyWindow(window);
            if !icon.is_null() {
                DestroyIcon(icon);
            }
            return false;
        }
        TRAY.with(|tray| {
            *tray.borrow_mut() = Some(Tray {
                window,
                icon,
                taskbar_created,
                handler: Rc::new(handler),
            })
        });
        true
    }
}

pub fn remove_tray() {
    if let Some(tray) = TRAY.with(|tray| tray.borrow_mut().take()) {
        unsafe {
            Shell_NotifyIconW(NIM_DELETE, &notify_data(tray.window, tray.icon));
            DestroyWindow(tray.window);
            if !tray.icon.is_null() {
                DestroyIcon(tray.icon);
            }
        }
    }
}
