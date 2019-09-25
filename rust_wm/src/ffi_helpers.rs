use mir_rs::*;
use std::ffi::CStr;
use std::os::raw::c_char;

#[allow(unused)]
extern "C" {
  pub fn window_specification_name(window_info: *const miral::WindowSpecification)
    -> *const c_char;
  pub fn configure_window(
    window_info: *const miral::WindowInfo,
    attrib: raw::MirWindowAttrib::Type,
    value: i32,
  ) -> i32;
  pub fn hide_window(window_info: *const miral::WindowInfo) -> ();
  pub fn show_window(window_info: *const miral::WindowInfo) -> ();
  pub fn window_name(window_info: *const miral::WindowInfo) -> SharedPtrString;
  pub fn rust_get_string(value: SharedPtrString) -> *const c_char;
  pub fn rust_drop_string(value: SharedPtrString) -> ();
  pub fn window_specification_has_parent(window_info: *const miral::WindowSpecification) -> bool;
  pub fn window_info_has_parent(window_info: *const miral::WindowInfo) -> bool;
  pub fn get_active_window(tools: *const miral::WindowManagerTools) -> SharedPtrWindow;
  pub fn get_window_at(
    tools: *const miral::WindowManagerTools,
    cursor: mir::geometry::Point,
  ) -> SharedPtrWindow;
  pub fn select_active_window(
    value: *const miral::WindowManagerTools,
    hont: *const miral::Window,
  ) -> ();
  pub fn rust_get_window(value: SharedPtrWindow) -> *mut miral::Window;
  pub fn rust_drop_window(value: SharedPtrWindow) -> ();
  pub fn client_is_alive(client: *const raw::wl_client) -> bool;
  pub fn client_owns_window(
    client: *const raw::wl_client,
    window: *const miral::WindowInfo,
  ) -> bool;
  pub fn set_keymap(keymap_ctrl: *mut miral::Keymap, keymap: *const c_char) -> ();
}

#[derive(Clone)]
#[repr(C)]
pub struct SharedPtrString(*const raw::std::string);

#[allow(unused)]
impl SharedPtrString {
  pub fn get(&self) -> String {
    unsafe { to_string(rust_get_string(self.clone())) }
  }
}

impl Drop for SharedPtrString {
  fn drop(&mut self) {
    unsafe { rust_drop_string(self.clone()) };
  }
}

#[derive(Clone)]
#[repr(C)]
pub struct SharedPtrWindow(*mut miral::Window);

#[allow(unused)]
impl SharedPtrWindow {
  pub fn get(&self) -> &mut miral::Window {
    unsafe { &mut *rust_get_window(self.clone()) }
  }

  pub fn get_opt(&self) -> Option<&mut miral::Window> {
    unsafe { rust_get_window(self.clone()).as_mut() }
  }
}

impl Drop for SharedPtrWindow {
  fn drop(&mut self) {
    unsafe { rust_drop_window(self.clone()) };
  }
}

pub fn to_string(string: *const c_char) -> String {
  unsafe { CStr::from_ptr(string).to_string_lossy().to_string() }
}
