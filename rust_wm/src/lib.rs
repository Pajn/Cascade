mod actions;
mod entities;
mod ffi_helpers;
mod input_inhibitor;
mod keyboard;
mod pointer;

use crate::actions::*;
use crate::entities::*;
use crate::ffi_helpers::*;
use crate::input_inhibitor::{focus_exclusive_client, InputInhibitor};
use crate::keyboard::*;
use crate::pointer::*;
use mir_rs::*;
use std::collections::BTreeMap;
use std::mem::transmute;

fn is_tiled(window: &miral::WindowSpecification) -> bool {
  let has_parent = unsafe { window_specification_has_parent(window) };
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
  let name = unsafe {
    window_specification_name(window)
      .as_ref()
      .map(|a| to_string(a))
  };
  println!("name: {:?}", name);

  name != Some("Ulauncher window title".to_owned())
    && !has_parent
    && (type_ == raw::MirWindowType::mir_window_type_normal
      || type_ == raw::MirWindowType::mir_window_type_freestyle)
    && state != raw::MirWindowState::mir_window_state_fullscreen
    && state != raw::MirWindowState::mir_window_state_attached
}

#[no_mangle]
pub extern "C" fn init_wm(
  tools: *mut miral::WindowManagerTools,
  input_inhibitor: *mut InputInhibitor,
) -> *mut WindowManager {
  let input_inhibitor = unsafe { Box::from_raw(input_inhibitor) };
  let mut wm = WindowManager {
    tools,
    input_inhibitor,
    monitor_id_generator: IdGenerator::new(),
    window_id_generator: IdGenerator::new(),
    workspace_id_generator: IdGenerator::new(),

    monitors: BTreeMap::new(),
    windows: BTreeMap::new(),
    workspaces: BTreeMap::new(),

    old_cursor: Point { x: 0, y: 0 },
    gesture: Gesture::None,
    active_window: None,
    active_workspace: 0,
    new_window_workspace: 0,
  };

  wm.active_workspace = wm.get_or_create_unused_workspace();
  wm.new_window_workspace = wm.active_workspace;

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
    println!("place_new_window tiled");
    if let Some(mut point) = unsafe { &mut *window_specification.top_left1() }.as_mut() {
      point.x.value = wm
        .new_window_workspace()
        .active_window
        .and_then(|id| wm.windows.get(&id))
        .map_or_else(
          || {
            wm.new_window_workspace()
              .on_monitor
              .and_then(|monitor_id| wm.monitors.get(&monitor_id))
              .map_or(0, |monitor| monitor.extents.left())
          },
          |window| window.x() + window.width(),
        );
      point.y.value = 0;
    }

    if let Some(mut size) = unsafe { &mut *window_specification.size1() }.as_mut() {
      if let Some(monitor) = wm.monitor_by_workspace(wm.new_window_workspace) {
        size.height.value = monitor.extents.height();
      }
    }
  } else {
    println!(
      "place_new_window not tiled, output_id({:?}), top_left({:?})",
      unsafe { window_specification.output_id().as_ref() },
      unsafe { window_specification.top_left().as_ref() }
    );
  }
}

#[no_mangle]
pub extern "C" fn handle_window_ready(
  wm: *mut WindowManager,
  window_info: *mut miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };
  let window_info = unsafe { &mut *window_info };

  let mut window = Window::new(&mut wm.window_id_generator, window_info);
  window.x = window.x();
  window.y = window.y();
  window.size = window.rendered_size();
  println!("handle_window_ready name: {:?}", window.name());

  let type_ = unsafe { window_info.type_() };
  let has_parent = unsafe { window_info_has_parent(window_info) };
  if window.is_tiled() {
    window.on_workspace = Some(wm.new_window_workspace);
    println!(
      "handle_window_ready tiled type_ {}, has_parent {}",
      type_, has_parent
    );
  } else {
    println!(
      "handle_window_ready not tiled type_ {}, has_parent {}",
      type_, has_parent
    );
  }

  wm.add_window(window);

  arrange_windows(wm);
}

#[no_mangle]
pub extern "C" fn handle_raise_window(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };

  if let Some(window) = wm.window_by_info(window_info) {
    if wm.input_inhibitor.is_allowed(window) {
      let window_id = window.id;
      wm.focus_window(Some(window_id));
    } else {
      focus_exclusive_client(wm);
    }
  } else {
    wm.focus_window(None);
  }
}

#[no_mangle]
pub extern "C" fn advise_focus_gained(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };

  if let Some(window) = wm.window_by_info(window_info) {
    if wm.input_inhibitor.is_allowed(window) {
      let window_id = window.id;
      wm.activate_window(window_id);

      if let Some(workspace_id) = wm.get_window(window_id).on_workspace {
        ensure_window_visible(wm, window_id);
        update_window_positions(wm, workspace_id);
      }
    } else {
      focus_exclusive_client(wm);
    }
  }
}

#[no_mangle]
pub extern "C" fn pre_handle_modify_window(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
  modifications: *mut miral::WindowSpecification,
) -> () {
  let wm = unsafe { &mut *wm };
  let window_info = unsafe { &*window_info };
  let modifications = unsafe { &mut *modifications };

  if let Some(state) = unsafe { (*modifications.state()).as_ref() } {
    if *state == raw::MirWindowState::mir_window_state_maximized {
      unsafe { (*modifications.state1()).value_ = window_info.state() };
    }
    // Hack: Something (probably basic window manager in miral) tries
    // to unset hidden. For now, just make sure that it can't but
    // later on figure out why this is needed and make sure it isn't.
    if *state == raw::MirWindowState::mir_window_state_restored {
      if let Some(window) = wm.window_by_info(window_info) {
        if window.state() == raw::MirWindowState::mir_window_state_hidden
          && window.old_state.is_some()
        {
          println!("Tried to unset hidden");
          unsafe { (*modifications.state1()).value_ = window_info.state() };
        }
      }
    }
  }
}

#[no_mangle]
pub extern "C" fn post_handle_modify_window(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
  _modifications: *const miral::WindowSpecification,
) -> () {
  let wm = unsafe { &mut *wm };

  if let Some(window) = wm
    .window_by_info(window_info)
    .map(|w| w.id)
    .and_then(|id| wm.windows.get_mut(&id))
  {
    window.size = window.rendered_size();

    arrange_windows(wm);
  } else {
    println!("Could not get modified window");
  }
}

#[no_mangle]
pub extern "C" fn advise_delete_window(
  wm: *mut WindowManager,
  window_info: *const miral::WindowInfo,
) -> () {
  let wm = unsafe { &mut *wm };

  if let Some(window) = wm.window_by_info(window_info) {
    let window_id = window.id;
    wm.delete_window(window_id);
  } else {
    println!(
      "nowindow in windows advise_delete_window, info: {:?}",
      window_info
    );
  }

  println!("advise_delete_window {:?}", &wm);
}

#[no_mangle]
pub extern "C" fn advise_output_create(
  _wm: *mut WindowManager,
  _output: *const miral::Output,
) -> () {
  // let wm = unsafe { &mut *wm };
  // let output = unsafe { &*output };

  // let size = Size {
  //   width: unsafe { output.extents().size.width.value },
  //   height: unsafe { output.extents().size.height.value },
  // };

  // let workspace = wm.get_or_create_unused_workspace();
  // let monitor = Monitor::new(&mut wm.monitor_id_generator, size, workspace, output);
  // wm.workspaces.get_mut(&workspace).unwrap().on_monitor = Some(monitor.id);
  // wm.monitors.insert(monitor.id, monitor);
}

#[no_mangle]
pub extern "C" fn advise_output_update(
  _wm: *mut WindowManager,
  _updated: *const miral::Output,
  _original: *const miral::Output,
) -> () {
  // let wm = unsafe { &mut *wm };
  // let updated = unsafe { &*updated };

  // let new_size = Size {
  //   width: unsafe { updated.extents().size.width.value },
  //   height: unsafe { updated.extents().size.height.value },
  // };

  // let mut monitor = wm
  //   .monitors
  //   .iter_mut()
  //   .find(|(_, m)| m.output == original)
  //   .expect("monitor advise_output_update")
  //   .1;
  // monitor.output = updated;
  // monitor.size = new_size;
}

#[no_mangle]
pub extern "C" fn advise_output_delete(
  _wm: *mut WindowManager,
  _output: *const miral::Output,
) -> () {
  // let wm = unsafe { &mut *wm };

  // let monitor = wm
  //   .monitors
  //   .iter_mut()
  //   .find(|(_, m)| m.output == output)
  //   .expect("monitor advise_output_delete")
  //   .1;
  // let workspace = wm
  //   .workspaces
  //   .get_mut(&monitor.workspace)
  //   .expect("workspacee advise_output_delete");
  // workspace.on_monitor = None;
  // let monitor_id = monitor.id;
  // wm.monitors.remove(&monitor_id);
}

#[no_mangle]
pub extern "C" fn advise_application_zone_create(
  wm: *mut WindowManager,
  zone: *const miral::Zone,
) -> () {
  let wm = unsafe { &mut *wm };
  let zone = unsafe { (*zone).extents().into() };

  println!("advise_application_zone_create, {:?}", zone);

  let workspace = wm.get_or_create_unused_workspace();
  let monitor = Monitor::new(&mut wm.monitor_id_generator, zone, workspace);
  wm.workspaces.get_mut(&workspace).unwrap().on_monitor = Some(monitor.id);
  wm.monitors.insert(monitor.id, monitor);
}

#[no_mangle]
pub extern "C" fn advise_application_zone_update(
  wm: *mut WindowManager,
  updated: *const miral::Zone,
  original: *const miral::Zone,
) -> () {
  let wm = unsafe { &mut *wm };
  let updated = unsafe { (*updated).extents().into() };
  let original = unsafe { (*original).extents().into() };

  println!(
    "advise_application_zone_update, from {:?} to {:?}",
    original, updated
  );

  let mut monitor = wm
    .monitors
    .iter_mut()
    .find(|(_, m)| m.extents == original)
    .expect("monitor advise_application_zone_update")
    .1;
  monitor.extents = updated;
}

#[no_mangle]
pub extern "C" fn advise_application_zone_delete(
  wm: *mut WindowManager,
  zone: *const miral::Zone,
) -> () {
  let mut wm = unsafe { &mut *wm };
  let zone = unsafe { (*zone).extents().into() };
  println!("advise_application_zone_delete");

  let monitor = wm
    .monitors
    .iter_mut()
    .find(|(_, m)| m.extents == zone)
    .expect("monitor advise_application_zone_delete")
    .1;
  let workspace = wm
    .workspaces
    .get_mut(&monitor.workspace)
    .expect("workspacee advise_application_zone_delete");
  workspace.on_monitor = None;
  let monitor_id = monitor.id;
  wm.monitors.remove(&monitor_id);

  arrange_windows(&mut wm);
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

#[no_mangle]
pub extern "C" fn handle_pointer_event(
  wm: *mut WindowManager,
  event: *const raw::MirPointerEvent,
) -> bool {
  let wm = unsafe { &mut *wm };

  pointer::handle_pointer_event(wm, event)
}

#[no_mangle]
pub extern "C" fn handle_request_move(
  wm: *mut WindowManager,
  window_info: *mut miral::WindowInfo,
  input_event: *const raw::MirInputEvent,
) -> () {
  let wm = unsafe { &mut *wm };

  handle_pointer_request(
    wm,
    window_info,
    input_event,
    raw::MirResizeEdge::mir_resize_edge_none,
    GestureType::Move,
  );
}

#[no_mangle]
pub extern "C" fn handle_request_resize(
  wm: *mut WindowManager,
  window_info: *mut miral::WindowInfo,
  input_event: *const raw::MirInputEvent,
  edge: raw::MirResizeEdge::Type,
) -> () {
  let wm = unsafe { &mut *wm };

  handle_pointer_request(wm, window_info, input_event, edge, GestureType::Resize);
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
