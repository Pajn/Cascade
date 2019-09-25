use crate::actions::*;
use crate::entities::*;
use crate::ffi_helpers::*;
use mir_rs::*;

pub extern "C" fn handle_pointer_event(
  wm: &mut WindowManager,
  event: *const raw::MirPointerEvent,
) -> bool {
  let action = unsafe { raw::mir_pointer_event_action(event) };

  let new_cursor = Point {
    x: unsafe { raw::mir_pointer_event_axis_value(event, raw::MirPointerAxis::mir_pointer_axis_x) }
      as i32,
    y: unsafe { raw::mir_pointer_event_axis_value(event, raw::MirPointerAxis::mir_pointer_axis_y) }
      as i32,
  };
  let buttons = unsafe { raw::mir_pointer_event_buttons(event) };
  let modifiers = unsafe { raw::mir_pointer_event_modifiers(event) };

  if wm.input_inhibitor.is_inhibited() {
    let window = unsafe { get_window_at(wm.tools, new_cursor.into()) };
    let window = unsafe { (*wm.tools).info_for2(window.get()) };

    if let Some(window) = wm.window_by_info(window) {
      if !wm.input_inhibitor.is_allowed(window) {
        return true;
      }
    }
  }

  let over_monitor = wm
    .monitors
    .values()
    .find(|m| m.extents.contains(&new_cursor));

  if action == raw::MirPointerAction::mir_pointer_action_motion {
    if let Some(monitor) = over_monitor {
      if monitor.workspace != wm.active_workspace {
        wm.active_workspace = monitor.workspace;
        wm.new_window_workspace = monitor.workspace;
      }
    }
  }

  let consume_event = match wm.gesture {
    Gesture::Move(ref gesture) => {
      if action == raw::MirPointerAction::mir_pointer_action_motion
        && buttons == gesture.buttons
        && modifiers == gesture.modifiers
      {
        let window_id = gesture.window;
        let displacement = new_cursor - wm.old_cursor;
        if new_cursor.y < 100 {
          let window = wm.windows.get_mut(&window_id).unwrap();
          if window.is_dragged {
            window.is_dragged = false;
          }
          let window = wm.get_window(window_id);
          if let Some(workspace_id) = window.on_workspace {
            let workspace = wm.workspaces.get_mut(&workspace_id).unwrap();
            workspace.scroll_left -= displacement.dx;
            arrange_windows_workspace(wm, workspace_id);
          }
          wm.focus_window(Some(window_id));
        } else {
          let window = wm.windows.get_mut(&window_id).unwrap();
          if !window.is_dragged {
            window.is_dragged = true;
            unsafe {
              (*window.window_info).clip_area1(&optional_none_rectangle());
            }
          }

          let window = wm.get_window(window_id);
          let window_width = window.width();
          if let Some(workspace_id) = window.on_workspace {
            if let Some(over_monitor) = over_monitor {
              if over_monitor.workspace != workspace_id {
                let to_workspace_id = over_monitor.workspace;
                let index = find_index_by_cursor(wm, to_workspace_id, &new_cursor);
                move_window_workspace(wm, window_id, to_workspace_id, index);
                return true;
              }
            }

            if let Some(left_window_id) = get_tiled_window(wm, window_id, Direction::Left) {
              let left_window = wm.get_window(left_window_id);
              if new_cursor.x < left_window.x() + left_window.width() / 2 + window_width / 2 {
                wm.workspaces
                  .get_mut(&workspace_id)
                  .unwrap()
                  .swap_windows(window_id, left_window_id);
                arrange_windows(wm);
                return true;
              }
            }
            if let Some(right_window_id) = get_tiled_window(wm, window_id, Direction::Right) {
              let right_window = wm.get_window(right_window_id);
              if new_cursor.x > right_window.x() + right_window.width() / 2 - window_width / 2 {
                wm.workspaces
                  .get_mut(&workspace_id)
                  .unwrap()
                  .swap_windows(window_id, right_window_id);
                arrange_windows(wm);
                return true;
              }
            }
          }

          unsafe {
            let window = wm.get_window(window_id);
            let window_raw = (*window.window_info).window();
            (*wm.tools).drag_window(window_raw, displacement.into());
          }
        }

        true
      } else {
        let window = wm.windows.get_mut(&gesture.window).unwrap();
        window.is_dragged = false;
        wm.gesture = Gesture::None;
        if let Some(workspace_id) = window.on_workspace {
          arrange_windows_workspace(wm, workspace_id);
        }
        false
      }
    }
    Gesture::Resize(ref gesture) => {
      if action == raw::MirPointerAction::mir_pointer_action_motion
        && buttons == gesture.buttons
        && modifiers == gesture.modifiers
      {
        apply_resize_by(wm, new_cursor - wm.old_cursor);
        true
      } else {
        wm.gesture = Gesture::None;
        false
      }
    }
    _ => false,
  };

  if !consume_event && action == raw::MirPointerAction::mir_pointer_action_button_down {
    let window = unsafe { get_window_at(wm.tools, new_cursor.into()) };
    if let Some(window) = window.get_opt() {
      unsafe { select_active_window(wm.tools, window) };
    }
  }

  wm.old_cursor = new_cursor;
  consume_event
}

pub enum GestureType {
  Move,
  Resize,
}

pub fn handle_pointer_request(
  wm: &mut WindowManager,
  window_info: *mut miral::WindowInfo,
  input_event: *const raw::MirInputEvent,
  edge: raw::MirResizeEdge::Type,
  gesture_type: GestureType,
) -> bool {
  if wm.input_inhibitor.is_inhibited() {
    return false;
  }

  let input_event_type = unsafe { raw::mir_input_event_get_type(input_event) };
  if input_event_type != raw::MirInputEventType::mir_input_event_type_pointer {
    return false;
  }

  if let Some(window) = wm.window_by_info(window_info) {
    if window.has_parent() {
      return false;
    }
    if window.state() != raw::MirWindowState::mir_window_state_restored {
      return false;
    }

    let pointer_event = unsafe { raw::mir_input_event_get_pointer_event(input_event) };
    let buttons = unsafe { raw::mir_pointer_event_buttons(pointer_event) };
    let modifiers = unsafe { raw::mir_pointer_event_modifiers(pointer_event) };

    match gesture_type {
      GestureType::Move => {
        wm.gesture = Gesture::Move(MoveGesture {
          window: window.id,
          buttons,
          modifiers,
          top_left: window.rendered_top_left(),
        });
      }
      GestureType::Resize => {
        wm.gesture = Gesture::Resize(ResizeGesture {
          window: window.id,
          buttons,
          modifiers,
          top_left: window.rendered_top_left(),
          size: window.rendered_size(),
          edge,
        });
      }
    }
    true
  } else {
    println!("handle_request_move window not found");
    false
  }
}
