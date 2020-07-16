use wlral::geometry::*;
use wlral::input::events::*;
use wlral::window::*;

use crate::actions::*;
use crate::entities::*;
use crate::window_manager::CascadeWindowManager;

pub fn handle_motion_event(wm: &mut CascadeWindowManager, event: &MotionEvent) -> bool {
  let new_cursor = event.position().into();

  let over_monitor = wm
    .monitors
    .values()
    .find(|m| m.extents().contains(&new_cursor));

  if let Some(monitor) = over_monitor {
    if monitor.workspace != wm.active_workspace {
      wm.active_workspace = monitor.workspace;
      wm.new_window_workspace = monitor.workspace;
    }
  }

  let consume_event = match wm.gesture {
    Gesture::Move(ref gesture) => {
      let window_id = wm
        .window_by_info(gesture.window.clone())
        .expect("moved window")
        .id;
      if new_cursor.y < 100 {
        let window = wm.windows.get_mut(&window_id).unwrap();
        if window.is_dragged {
          window.is_dragged = false;
        }
        let window = wm.get_window(window_id);
        if let Some(workspace_id) = window.on_workspace {
          let workspace = wm.workspaces.get_mut(&workspace_id).unwrap();
          workspace.scroll_left -= new_cursor.x - wm.old_cursor.x;
          println!(
            "update workspace.scroll_left {}: {}",
            new_cursor.x - wm.old_cursor.x,
            workspace.scroll_left
          );
          arrange_windows_workspace(wm, workspace_id);
        }
        wm.focus_window(Some(window_id));
      } else {
        let window = wm.windows.get_mut(&window_id).unwrap();
        if !window.is_dragged {
          window.is_dragged = true;
        }

        let window = wm.get_window(window_id);
        let window_width = window.size().width();
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
            if new_cursor.x
              < left_window.top_left().x() + left_window.size().width() / 2 + window_width / 2
            {
              wm.workspaces
                .get_mut(&workspace_id)
                .unwrap()
                .swap_windows(window_id, left_window_id);
              arrange_windows_workspace(wm, workspace_id);
              return true;
            }
          }
          if let Some(right_window_id) = get_tiled_window(wm, window_id, Direction::Right) {
            let right_window = wm.get_window(right_window_id);
            if new_cursor.x
              > right_window.top_left().x() + right_window.size().width() / 2 - window_width / 2
            {
              wm.workspaces
                .get_mut(&workspace_id)
                .unwrap()
                .swap_windows(window_id, right_window_id);
              arrange_windows_workspace(wm, workspace_id);
              return true;
            }
          }
        }

        window
          .window_info
          .move_to((event.position() - gesture.drag_point.as_displacement()).into());
      }

      true
    }
    Gesture::Resize(ref gesture, ref original_extents) => {
      // apply_resize_by(wm, new_cursor - wm.old_cursor);
      let displacement = Displacement::from(event.position() - gesture.cursor_position);
      let mut extents = original_extents.clone();
      let (window_id, workspace_id) = match wm.window_by_info(gesture.window.clone()) {
        Some(window) => (window.id, window.on_workspace),
        None => return false,
      };

      if gesture.edges.contains(WindowEdge::TOP) {
        extents.top_left.y += displacement.dy;
        extents.size.height -= displacement.dy;
      } else if gesture.edges.contains(WindowEdge::BOTTOM) {
        extents.size.height += displacement.dy;
      }

      if gesture.edges.contains(WindowEdge::LEFT) {
        extents.top_left.x += displacement.dx;
        extents.size.width -= displacement.dx;

        if let Some(workspace_id) = workspace_id {
          let workspace = wm.workspaces.get_mut(&workspace_id).unwrap();
          workspace.scroll_left -= displacement.dx;
        }
      } else if gesture.edges.contains(WindowEdge::RIGHT) {
        extents.size.width += displacement.dx;
      }

      if let Some(workspace_id) = workspace_id {
        let window = wm.windows.get_mut(&window_id).unwrap();
        window.set_position(extents);
        arrange_windows_workspace(wm, workspace_id);
      }

      true
    }
    _ => false,
  };

  wm.old_cursor = new_cursor;
  consume_event
}

pub fn handle_button_event(wm: &mut CascadeWindowManager, event: &ButtonEvent) -> bool {
  let consume_event = event.state() == ButtonState::Released
    && match wm.gesture {
      Gesture::Move(ref gesture) => {
        let window_id = wm
          .window_by_info(gesture.window.clone())
          .expect("moved window")
          .id;
        let window = wm.windows.get_mut(&window_id).unwrap();
        window.is_dragged = false;
        wm.gesture = Gesture::None;
        if let Some(workspace_id) = window.on_workspace {
          arrange_windows_workspace(wm, workspace_id);
        }
        true
      }
      Gesture::Resize(_, _) => {
        wm.gesture = Gesture::None;
        true
      }
      _ => false,
    };

  if !consume_event && event.state() == ButtonState::Pressed {
    let window_id = wm.get_window_at(&event.position().into()).map(|w| w.id);
    if let Some(window_id) = window_id {
      wm.focus_window(Some(window_id));
    }
  }

  consume_event
}
