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
    monitor_id_generator: IdGenerator::new(),
    window_id_generator: IdGenerator::new(),
    workspace_id_generator: IdGenerator::new(),
    monitors: BTreeMap::new(),
    windows: BTreeMap::new(),
    workspaces: BTreeMap::new(),
    active_window: None,
    active_workspace: 0,
  };

  wm.active_workspace = wm.get_or_create_unused_workspace();

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

    if let Some(mut size) = unsafe { &mut *window_specification.size1() }.as_mut() {
      if let Some(monitor) = wm.monitor_by_workspace(wm.active_workspace) {
        size.height.value = monitor.size.height;
      }
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
    wm.active_workspace = wm.get_window(window_id).workspace;
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
pub extern "C" fn advise_output_create(wm: *mut WindowManager, output: *const miral::Output) -> () {
  let wm = unsafe { &mut *wm };
  let output = unsafe { &*output };

  let size = Size {
    width: unsafe { output.extents().size.width.value },
    height: unsafe { output.extents().size.height.value },
  };

  let workspace = wm.get_or_create_unused_workspace();
  let monitor = Monitor::new(&mut wm.monitor_id_generator, size, workspace, output);
  wm.workspaces.get_mut(&workspace).unwrap().on_monitor = Some(monitor.id);
  wm.monitors.insert(monitor.id, monitor);
}

#[no_mangle]
pub extern "C" fn advise_output_update(
  wm: *mut WindowManager,
  updated: *const miral::Output,
  original: *const miral::Output,
) -> () {
  let wm = unsafe { &mut *wm };
  let updated = unsafe { &*updated };

  let new_size = Size {
    width: unsafe { updated.extents().size.width.value },
    height: unsafe { updated.extents().size.height.value },
  };

  let mut monitor = wm
    .monitors
    .iter_mut()
    .find(|(_, m)| m.output == original)
    .expect("monitor advise_output_update")
    .1;
  monitor.size = new_size;
}

#[no_mangle]
pub extern "C" fn advise_output_delete(wm: *mut WindowManager, output: *const miral::Output) -> () {
  let wm = unsafe { &mut *wm };

  let monitor = wm
    .monitors
    .iter_mut()
    .find(|(_, m)| m.output == output)
    .expect("monitor advise_output_delete")
    .1;
  let workspace = wm
    .workspaces
    .get_mut(&monitor.workspace)
    .expect("workspacee advise_output_delete");
  workspace.on_monitor = None;
  let monitor_id = monitor.id;
  wm.monitors.remove(&monitor_id);
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
