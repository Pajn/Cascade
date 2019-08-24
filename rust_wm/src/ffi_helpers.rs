use mir_rs::*;

#[derive(Clone)]
#[repr(C)]
pub struct SharedPtrWindow(*mut miral::Window);

#[allow(unused)]
extern "C" {
  pub fn window_specification_has_parent(window_info: *const miral::WindowSpecification) -> bool;
  pub fn window_info_has_parent(window_info: *const miral::WindowInfo) -> bool;
  pub fn get_active_window(value: *const miral::WindowManagerTools) -> SharedPtrWindow;
  pub fn select_active_window(
    value: *const miral::WindowManagerTools,
    hont: *const miral::Window,
  ) -> ();
  pub fn rust_get_window(value: SharedPtrWindow) -> *mut miral::Window;
  pub fn rust_drop_window(value: SharedPtrWindow) -> ();
}

#[allow(unused)]
impl SharedPtrWindow {
  pub fn get(&self) -> &mut miral::Window {
    unsafe { &mut *rust_get_window(self.clone()) }
  }
}

impl Drop for SharedPtrWindow {
  fn drop(&mut self) {
    unsafe { rust_drop_window(self.clone()) };
  }
}
