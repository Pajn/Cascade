mod actions;
mod entities;
mod ffi_helpers;
mod keyboard;

use crate::actions::arange_windows;
use crate::entities::*;
use crate::keyboard::*;
use mir_rs::*;
use std::collections::BTreeMap;
use std::mem::transmute;

fn is_tiled(window: &miral::WindowSpecification) -> bool {
  let type_ = unsafe {
    (*(*window).type_())
      .as_ref()
      .copied()
      .unwrap_or(raw::MirWindowType::mir_window_type_normal)
  };
  let state = unsafe {
    (*(*window).state())
      .as_ref()
      .copied()
      .unwrap_or(raw::MirWindowState::mir_window_state_unknown)
  };

  (type_ == raw::MirWindowType::mir_window_type_normal
    || type_ == raw::MirWindowType::mir_window_type_freestyle)
    && state != raw::MirWindowState::mir_window_state_fullscreen
}

#[no_mangle]
pub extern "C" fn init_wm(tools: *mut miral::WindowManagerTools) -> *mut WindowManager {
  let mut wm = WindowManager {
    tools,
    window_id_generator: IdGenerator::new(),
    workspace_id_generator: IdGenerator::new(),
    windows: BTreeMap::new(),
    workspaces: BTreeMap::new(),
    active_window: None,
    active_workspace: 0,
  };

  let first_workspace_id = wm.workspace_id_generator.next_id();
  wm.active_workspace = first_workspace_id;
  wm.workspaces.insert(
    first_workspace_id,
    Workspace {
      id: first_workspace_id,
      windows: vec![],
    },
  );

  unsafe { transmute(Box::new(wm)) }
}

#[no_mangle]
pub extern "C" fn place_new_window(
  wm: *mut WindowManager,
  window_specification: *mut miral::WindowSpecification,
) -> () {
  let wm = unsafe { &mut *wm };
  let window_specification = unsafe { &mut *window_specification };

  if is_tiled(window_specification) {
    if let Some(mut point) = unsafe { &mut *window_specification.top_left1() }.as_mut() {
      point.x.value = wm
        .active_window
        .and_then(|id| wm.windows.get(&id))
        .map_or(0, |window| window.x() + window.width());
      point.y.value = 0;
    }
  }
}

#[no_mangle]
pub extern "C" fn handle_window_ready(
  wm: *mut WindowManager,
  window_info: *mut miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };
  let window_info = unsafe { &mut *window_info };

  let window = Window::new(
    &mut wm.window_id_generator,
    wm.active_workspace,
    window_info,
  );
  wm.add_window(window);

  arange_windows(wm);

  println!("handle_window_ready {:?}", &wm);
}

#[no_mangle]
pub extern "C" fn advise_focus_gained(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };

  if let Some(window_id) = wm.window_by_info(window_info).map(|w| w.id) {
    wm.active_window = Some(window_id);
  }
}

#[no_mangle]
pub extern "C" fn handle_modify_window(
  wm: *mut WindowManager,
  _window_info: *const miral::WindowInfo,
  _modifications: *const miral::WindowSpecification,
) -> () {
  let wm = unsafe { &mut *wm };

  arange_windows(wm);
}

#[no_mangle]
pub extern "C" fn advise_delete_window(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };

  let window_id = wm
    .window_by_info(window_info)
    .expect("nowindow in windows advise_delete_window")
    .id;

  wm.delete_window(window_id);

  println!("advise_delete_window {:?}", &wm);
}

#[no_mangle]
pub extern "C" fn handle_keyboard_event(
  wm: *mut WindowManager,
  event: *const raw::MirKeyboardEvent,
) -> bool {
  let wm = unsafe { &mut *wm };

  let action = unsafe { raw::mir_keyboard_event_action(event) };
  let key_code = keyboard_event_key_code(event);
  let modifiers = keyboard_event_modifiers(event);

  if action == raw::MirKeyboardAction::mir_keyboard_action_down {
    handle_key_press(wm, key_code, modifiers)
  } else {
    false
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
