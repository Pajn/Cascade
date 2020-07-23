use crate::actions::*;
use crate::entities::*;
use crate::window_manager::CascadeWindowManager;
use wlral::geometry::*;
use wlral::input::events::*;
use wlral::window::*;
use workspace::WorkspacePosition;

pub(crate) fn handle_motion_event(wm: &CascadeWindowManager, event: &MotionEvent) -> bool {
  let new_cursor = event.position().into();

  let over_output = wm
    .output_manager
    .outputs()
    .iter()
    .find(|o| o.extents().contains(&new_cursor))
    .cloned();

  if let Some(ref output) = over_output {
    let workspace = wm
      .output_workspaces
      .borrow()
      .get(output)
      .cloned()
      .expect("Output should be assigned a workspace");
    wm.focus_workspace(&workspace);
  }

  match *wm.gesture.borrow() {
    Gesture::Move(ref gesture) => {
      let window = &gesture.window;
      if new_cursor.y < 100 {
        if let Some(workspace) = wm.workspace_by_window(&gesture.window) {
          let pre_scroll_left = workspace.scroll_left();
          workspace.set_scroll_left(pre_scroll_left - event.delta().dx as i32);
          arrange_windows_workspace_options(wm, workspace.clone(), true);
        }
      } else {
        let window_width = window.size().width();
        if let Some(workspace) = wm.workspace_by_window(&gesture.window) {
          if let Some(ref over_output) = over_output {
            let output_workspace = wm
              .output_workspaces
              .borrow()
              .get(over_output)
              .cloned()
              .expect("Output should be assigned a workspace");
            if output_workspace != workspace {
              move_specified_window_to_workspace(
                wm,
                gesture.window.clone(),
                &output_workspace,
                WorkspacePosition::Coordinate(new_cursor),
              );
              return true;
            }
          }

          if let Some(left_window) = workspace.window_by_direction(&window, Direction::Left) {
            if new_cursor.x
              < left_window.extents().left() + left_window.size().width() / 2 + window_width / 2
            {
              let _ = workspace.move_window(&window, Direction::Left);
              arrange_windows_workspace(wm, workspace);
              return true;
            }
          }
          if let Some(right_window) = workspace.window_by_direction(&window, Direction::Right) {
            if new_cursor.x
              > right_window.extents().left() + right_window.size().width() / 2 - window_width / 2
            {
              let _ = workspace.move_window(&window, Direction::Right);
              arrange_windows_workspace(wm, workspace);
              return true;
            }
          }
        }

        window.move_to((event.position() - gesture.drag_point.as_displacement()).into());
      }

      return true;
    }
    Gesture::Resize(ref gesture, ref original_extents) => {
      let displacement = Displacement::from(event.position() - gesture.cursor_position);
      let mut extents = original_extents.clone();

      if gesture.edges.contains(WindowEdge::TOP) {
        extents.top_left.y += displacement.dy;
        extents.size.height -= displacement.dy;
      } else if gesture.edges.contains(WindowEdge::BOTTOM) {
        extents.size.height += displacement.dy;
      }

      if gesture.edges.contains(WindowEdge::LEFT) {
        extents.top_left.x += displacement.dx;
        extents.size.width -= displacement.dx;

        if let Some(workspace) = wm.workspace_by_window(&gesture.window) {
          let current_extents = gesture.window.extents();
          let delta_x = current_extents.left() - extents.left();
          if delta_x < 0 {
            let mut scroll_left = workspace.scroll_left();
            scroll_left += delta_x;
            workspace.set_scroll_left(scroll_left);
          }
        }
      } else if gesture.edges.contains(WindowEdge::RIGHT) {
        extents.size.width += displacement.dx;
      }

      gesture.window.set_extents(&extents);
      if let Some(workspace) = wm.workspace_by_window(&gesture.window) {
        arrange_windows_workspace(wm, workspace);
      }

      return true;
    }
    _ => {}
  }

  false
}

pub(crate) fn handle_button_event(wm: &CascadeWindowManager, event: &ButtonEvent) -> bool {
  if event.state() == ButtonState::Released {
    let gesture_window = wm.gesture.borrow().window();
    if let Some(window) = gesture_window {
      *wm.gesture.borrow_mut() = Gesture::None;
      if let Some(workspace) = wm.workspace_by_window(&window) {
        arrange_windows_workspace(wm, workspace);
      }
      return true;
    }
  }

  false
}
