//! Pins the floating drop target and wheel to the right screen edge:
//! gtk4-layer-shell on Wayland, Win32 always-on-top placement on Windows.
#[cfg(not(windows))]
use gtk::prelude::*;
#[cfg(not(windows))]
use std::ffi::c_void;

#[cfg(windows)]
pub fn attach(window: &gtk::Window, right_margin: i32) -> bool {
    crate::win32::pin_right_edge(window, right_margin);
    true
}

#[cfg(not(windows))]
#[link(name = "gtk4-layer-shell")]
unsafe extern "C" {
    fn gtk_layer_is_supported() -> i32;
    fn gtk_layer_init_for_window(window: *mut c_void);
    fn gtk_layer_set_namespace(window: *mut c_void, name: *const std::ffi::c_char);
    fn gtk_layer_set_layer(window: *mut c_void, layer: i32);
    fn gtk_layer_set_anchor(window: *mut c_void, edge: i32, anchor: i32);
    fn gtk_layer_set_margin(window: *mut c_void, edge: i32, margin: i32);
    fn gtk_layer_set_keyboard_mode(window: *mut c_void, mode: i32);
    fn gtk_layer_set_exclusive_zone(window: *mut c_void, zone: i32);
}
#[cfg(not(windows))]
pub fn attach(window: &gtk::Window, right_margin: i32) -> bool {
    // The compositor connection and GTK objects are only accessed on the GTK thread.
    unsafe {
        if gtk_layer_is_supported() == 0 {
            return false;
        }
        let ptr = window.as_ptr().cast();
        gtk_layer_init_for_window(ptr);
        gtk_layer_set_namespace(ptr, c"filewheel".as_ptr());
        // TOP stays above normal windows but below fullscreen video and games.
        gtk_layer_set_layer(ptr, 2);
        gtk_layer_set_anchor(ptr, 1, 1);
        gtk_layer_set_margin(ptr, 1, right_margin);
        gtk_layer_set_keyboard_mode(ptr, 2);
        gtk_layer_set_exclusive_zone(ptr, -1);
    }
    true
}
