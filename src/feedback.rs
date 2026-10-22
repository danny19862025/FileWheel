use std::{
    cell::{Cell, RefCell},
    ffi::{CString, c_int, c_void},
    sync::mpsc::{self, SyncSender},
    time::{Duration, Instant},
};

pub struct HoverSound {
    sender: RefCell<Option<SyncSender<Instant>>>,
    worker: RefCell<Option<std::thread::JoinHandle<()>>>,
    last: Cell<Option<Instant>>,
}

impl HoverSound {
    pub fn new(silent: bool) -> Self {
        let (sender, worker) = if silent {
            (None, None)
        } else {
            let (sender, receiver) = mpsc::sync_channel::<Instant>(1);
            // Audio backends and file preparation never block GTK's main thread.
            let worker = std::thread::spawn(move || {
                let Ok(sound) = Sound::open() else {
                    return;
                };
                while let Ok(when) = receiver.recv() {
                    if when.elapsed() < Duration::from_millis(200) {
                        let _ = sound.play();
                    }
                }
            });
            (Some(sender), Some(worker))
        };
        Self {
            sender: RefCell::new(sender),
            worker: RefCell::new(worker),
            last: Cell::new(None),
        }
    }
    pub fn play(&self) {
        let now = Instant::now();
        if self
            .last
            .get()
            .is_some_and(|last| now.duration_since(last) < Duration::from_millis(125))
        {
            return;
        }
        if let Some(sender) = self.sender.borrow().as_ref() {
            let _ = sender.try_send(now);
        }
        self.last.set(Some(now));
    }
    pub fn preview(&self) {
        // An explicit click should replay even if entering the button just
        // played a hover cue. Passive hover events remain rate-limited.
        self.last.set(None);
        self.play();
    }
    pub fn stop(&self) {
        self.sender.borrow_mut().take();
        if let Some(worker) = self.worker.borrow_mut().take() {
            let _ = worker.join();
        }
    }
}
impl Drop for HoverSound {
    fn drop(&mut self) {
        self.stop();
    }
}

type Create = unsafe extern "C" fn(*mut *mut c_void) -> c_int;
type Open = unsafe extern "C" fn(*mut c_void) -> c_int;
type Destroy = unsafe extern "C" fn(*mut c_void) -> c_int;
type Play = unsafe extern "C" fn(*mut c_void, u32, ...) -> c_int;

struct Library(*mut c_void);
impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            libc::dlclose(self.0);
        }
    }
}

struct Sound {
    context: *mut c_void,
    play: Play,
    destroy: Destroy,
    file: CString,
    _work: crate::engine::WorkDir,
    _library: Library,
}
impl Sound {
    fn open() -> Result<Self, String> {
        // libcanberra is optional. Keep its handle alive until after the context
        // is destroyed, and use exactly the C ABI declared in canberra.h.
        unsafe {
            let handle = libc::dlopen(
                c"libcanberra.so.0".as_ptr(),
                libc::RTLD_NOW | libc::RTLD_LOCAL,
            );
            if handle.is_null() {
                return Err("Install libcanberra to enable hover sounds.".into());
            }
            let library = Library(handle);
            let symbol = |name: &std::ffi::CStr| -> Result<*mut c_void, String> {
                let pointer = libc::dlsym(handle, name.as_ptr());
                if pointer.is_null() {
                    Err("The installed libcanberra is incompatible.".into())
                } else {
                    Ok(pointer)
                }
            };
            let create: Create = std::mem::transmute(symbol(c"ca_context_create")?);
            let open: Open = std::mem::transmute(symbol(c"ca_context_open")?);
            let destroy: Destroy = std::mem::transmute(symbol(c"ca_context_destroy")?);
            let play: Play = std::mem::transmute(symbol(c"ca_context_play")?);
            let work = crate::engine::WorkDir::new(&std::env::temp_dir())?;
            let path = work.path.join("hover.wav");
            std::fs::write(&path, include_bytes!("../data/hover.wav"))
                .map_err(|e| e.to_string())?;
            use std::os::unix::ffi::OsStrExt;
            let file = CString::new(path.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
            let mut context = std::ptr::null_mut();
            if create(&mut context) < 0 {
                return Err("Could not create the desktop sound context.".into());
            }
            let sound = Self {
                context,
                play,
                destroy,
                file,
                _work: work,
                _library: library,
            };
            if open(context) < 0 {
                return Err("No desktop event-sound backend is available.".into());
            }
            Ok(sound)
        }
    }
    fn play(&self) -> Result<(), String> {
        let result = unsafe {
            (self.play)(
                self.context,
                1,
                c"media.filename".as_ptr(),
                self.file.as_ptr(),
                c"media.name".as_ptr(),
                c"FileWheel hover".as_ptr(),
                c"media.role".as_ptr(),
                c"event".as_ptr(),
                c"canberra.cache-control".as_ptr(),
                c"permanent".as_ptr(),
                std::ptr::null::<std::ffi::c_char>(),
            )
        };
        if result < 0 {
            Err(format!("Desktop sound playback failed ({result})."))
        } else {
            Ok(())
        }
    }
}
impl Drop for Sound {
    fn drop(&mut self) {
        unsafe {
            (self.destroy)(self.context);
        }
    }
}

pub fn self_test() -> Result<(), String> {
    let sound = Sound::open()?;
    sound.play()?;
    std::thread::sleep(Duration::from_millis(100));
    Ok(())
}
