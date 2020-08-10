use crate::entities::*;
use crate::window_manager::CascadeWindowManager;
use log::{debug, error, trace, warn};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::{cmp::Ordering, rc::Rc};
use wlral::{geometry::*, output::Output, window::Window};
use workspace::WorkspacePosition;

pub(crate) fn arrange_windows_workspace_options(
  wm: &CascadeWindowManager,
  workspace: Rc<Workspace>,
  force_set: bool,
) {
  if let Some(output) = wm.output_by_workspace(&workspace) {
    let positions = workspace
      .windows()
      .iter()
      .cloned()
      .scan(output.extents().left(), |next_x, window| {
        let x = *next_x;
        *next_x = x + window.size().width();
        Some((window, x))
      })
      .collect::<Vec<_>>();

    let mut scroll_left = workspace.scroll_left();
    let active_window = wm.active_window();
    // Enusure that the focused window is visble
    if let Some(window) = active_window {
      for (w, window_x) in positions.iter() {
        if *w == window {
          let extents_left = output.extents().left();
          let extents_right = output.extents().right();
          let window_width = window.size().width();

          let x_window_left = *window_x;
          let x_window_right = window_x + window_width;

          let x_workspace_left = scroll_left + extents_left;
          let x_workspace_right = scroll_left + extents_right;

          if x_window_left < x_workspace_left {
            scroll_left = x_window_left - extents_left;
            trace!("Scrolling left to {}", scroll_left);
          } else if x_window_right > x_workspace_right {
            scroll_left = x_window_right - extents_right;
            trace!("Scrolling right to {}", scroll_left);
          }
        }
      }
    }
    workspace.set_scroll_left(scroll_left);

    let gesture_window = wm.gesture.borrow().window();

    for (window, x) in positions {
      if !force_set && Some(&window) == gesture_window.as_ref() {
        continue;
      }

      let height = cmp::min(
        output.extents().height(),
        window.max_height().unwrap_or(i32::MAX as u32) as i32,
      );

      let y = output.extents().top() + (output.extents().height() - height) / 2;

      let extents = Rectangle {
        top_left: Point { x, y },
        size: window.size().with_height(height),
      } + Displacement {
        dx: -scroll_left,
        dy: 0,
      };

      if extents.size() == window.size() {
        window.move_to(extents.top_left);
      } else {
        window.set_extents(&extents);
      }
    }
  }
}

pub(crate) fn arrange_windows_workspace(wm: &CascadeWindowManager, workspace: Rc<Workspace>) {
  arrange_windows_workspace_options(wm, workspace, false)
}

pub(crate) fn arrange_windows_all_workspaces(wm: &CascadeWindowManager) {
  for workspace in wm.mru_workspaces().iter().cloned() {
    arrange_windows_workspace(wm, workspace);
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Direction {
  Left,
  Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum VerticalDirection {
  Up,
  Down,
}

pub(crate) fn navigate_first(wm: &CascadeWindowManager) {
  if let Some(active_workspace) = wm.mru_workspaces().top() {
    if let Some(window) = active_workspace.windows().first() {
      wm.window_manager.focus_window(window.clone());
    }
  }
}

pub(crate) fn navigate_last(wm: &CascadeWindowManager) {
  if let Some(active_workspace) = wm.mru_workspaces().top() {
    if let Some(window) = active_workspace.windows().last() {
      wm.window_manager.focus_window(window.clone());
    }
  }
}

pub(crate) fn navigate(wm: &CascadeWindowManager, direction: Direction) {
  let next_window = wm.active_window().and_then(|active_window| {
    wm.workspace_by_window(&active_window)
      .and_then(|workspace| workspace.window_by_direction(&active_window, direction))
  });

  if let Some(next_window) = next_window {
    wm.window_manager.focus_window(next_window.clone());
  } else {
    match direction {
      Direction::Left => {
        trace!("Navigate to left monitor");
        navigate_monitor(wm, Direction::Left, WorkspacePosition::End);
      }
      Direction::Right => {
        trace!("Navigate to right monitor");
        navigate_monitor(wm, Direction::Right, WorkspacePosition::Start);
      }
    }
  }
}

pub(crate) fn move_window(wm: &CascadeWindowManager, direction: Direction) {
  if let Some(active_window) = wm.active_window() {
    if let Some(workspace) = wm.workspace_by_window(&active_window) {
      let result = workspace.move_window(&active_window, direction);

      if result.is_ok() {
        arrange_windows_workspace(wm, workspace);
      } else {
        match direction {
          Direction::Left => {
            trace!("Move window to left monitor");
            move_window_monitor(wm, Direction::Left, WorkspacePosition::End);
          }
          Direction::Right => {
            trace!("Move window to right monitor");
            move_window_monitor(wm, Direction::Right, WorkspacePosition::Start);
          }
        }
      }
    }
  }
}

fn get_workspace_by_direction(
  wm: &CascadeWindowManager,
  direction: VerticalDirection,
) -> Option<Rc<Workspace>> {
  if let Some(active_workspace) = wm.mru_workspaces().top() {
    let hidden_workspaces = wm
      .mru_workspaces()
      .iter()
      .filter(|workspace| {
        *workspace == active_workspace || wm.output_by_workspace(workspace).is_none()
      })
      .cloned()
      .collect::<Vec<_>>();

    let index = hidden_workspaces
      .iter()
      .enumerate()
      .find(|(_, workspace)| *workspace == active_workspace)
      .map(|(index, _)| index)
      .expect("active_workspace not found") as isize;

    let index = match direction {
      VerticalDirection::Up => index - 1,
      VerticalDirection::Down => index + 1,
    };

    if index >= 0 {
      return hidden_workspaces.get(index as usize).cloned();
    }
  }
  None
}

fn get_output_by_direction(wm: &CascadeWindowManager, direction: Direction) -> Option<Rc<Output>> {
  let active_workspace = wm.mru_workspaces().top().cloned();
  let active_output = active_workspace.and_then(|workspace| wm.output_by_workspace(&workspace));

  if let Some(active_output) = active_output {
    let mut outputs = wm
      .output_workspaces
      .borrow()
      .keys()
      .cloned()
      .collect::<Vec<_>>();

    fn output_x_position(a: &Rc<Output>, b: &Rc<Output>) -> Ordering {
      a.extents().left().cmp(&b.extents().left())
    }
    outputs.sort_by(output_x_position);

    let index = outputs
      .iter()
      .enumerate()
      .find(|(_, output)| **output == active_output)
      .map(|(index, _)| index)
      .expect("active_output not found") as isize;

    let index = match direction {
      Direction::Left => index - 1,
      Direction::Right => index + 1,
    };

    if index >= 0 {
      return outputs.get(index as usize).cloned();
    }
  } else {
    error!("get_output_by_direction: Active workspace is not on any monitor")
  }
  None
}

pub(crate) fn navigate_workspace(wm: &CascadeWindowManager, direction: VerticalDirection) {
  let next_workspace = get_workspace_by_direction(wm, direction);

  if let Some(workspace) = next_workspace {
    trace!("Focusing workspace by direction {:?}", direction);
    wm.focus_workspace(&workspace);
  }
}

pub(crate) fn navigate_monitor(
  wm: &CascadeWindowManager,
  direction: Direction,
  workspace_position: WorkspacePosition,
) {
  let next_output = get_output_by_direction(wm, direction);

  if let Some(output) = next_output {
    let workspace = wm
      .output_workspaces
      .borrow()
      .get(&output)
      .cloned()
      .expect("Output should have an assigned workspace");
    let window = match workspace_position {
      WorkspacePosition::ActiveWindow => workspace.mru_windows().top().cloned(),
      WorkspacePosition::End => workspace.windows().last().cloned(),
      WorkspacePosition::Start => workspace.windows().first().cloned(),
      WorkspacePosition::Coordinate(_) => {
        error!("Can not navigate monitor by coordinate");
        return;
      }
    };

    trace!(
      "Focusing monitor \"{}\" by direction {:?}",
      output.name(),
      direction
    );
    if let Some(window) = window {
      wm.window_manager.focus_window(window);
    } else {
      wm.focus_workspace(&workspace);
    }
  }
}

pub(crate) fn move_specified_window_to_workspace(
  wm: &CascadeWindowManager,
  window: Rc<Window>,
  to_workspace: &Rc<Workspace>,
  position: WorkspacePosition,
) {
  if let Some(from_workspace) = wm.workspace_by_window(&window) {
    from_workspace.remove_window(&window);
    to_workspace.add_window(window.clone(), position);
    wm.focus_workspace(to_workspace);
    arrange_windows_all_workspaces(wm);
  }
}

pub(crate) fn move_active_window_to_workspace(
  wm: &CascadeWindowManager,
  to_workspace: &Rc<Workspace>,
  position: WorkspacePosition,
) {
  if let Some(active_window) = wm.active_window() {
    move_specified_window_to_workspace(wm, active_window, to_workspace, position);
  }
}

pub(crate) fn move_window_workspace(wm: &CascadeWindowManager, direction: VerticalDirection) {
  let next_workspace = get_workspace_by_direction(wm, direction);

  if let Some(to_workspace) = next_workspace {
    trace!(
      "Moving active window to workspace by direction {:?}",
      direction
    );
    move_active_window_to_workspace(wm, &to_workspace, WorkspacePosition::ActiveWindow);
  }
}

pub(crate) fn move_window_monitor(
  wm: &CascadeWindowManager,
  direction: Direction,
  workspace_position: WorkspacePosition,
) {
  let next_output = get_output_by_direction(wm, direction);

  if let Some(output) = next_output {
    let to_workspace = wm
      .output_workspaces
      .borrow()
      .get(&output)
      .cloned()
      .expect("Output should have an assigned workspace");

    trace!(
      "Moving active window to monitor \"{}\" by direction {:?}",
      output.name(),
      direction
    );
    move_active_window_to_workspace(wm, &to_workspace, workspace_position)
  }
}

pub(crate) fn resize_window(wm: &CascadeWindowManager, steps: &Vec<f32>) {
  if let Some(active_window) = wm.window_manager.focused_window() {
    if let Some(output) = wm.output_by_window(&active_window) {
      let output_width = output.extents().width() as f32;
      let window_width = active_window.size().width as f32;

      let mut did_resize = false;
      for step in steps.iter().cloned() {
        if window_width < (output_width * step).floor() {
          trace!(
            "resize_window: {} < {} * {}",
            window_width,
            output_width,
            step
          );
          active_window.resize(
            active_window
              .size()
              .with_width((output_width * step).round() as i32),
          );
          did_resize = true;
          break;
        } else {
          trace!(
            "resize_window: {} >= {} * {}",
            window_width,
            output_width,
            step
          );
        }
      }
      if !did_resize {
        if let Some(first_step) = steps.first().cloned() {
          active_window.resize(
            active_window
              .size()
              .with_width((output_width * first_step).round() as i32),
          );
        } else {
          error!("resize_window needs at least one step defined");
        }
      }
      arrange_windows_all_workspaces(wm);
    } else {
      error!("resize_window: Active window is not on a monitor");
    }
  }
}

pub(crate) fn center_window(wm: &CascadeWindowManager) {
  if let Some(window) = wm.active_window() {
    if let Some(workspace) = wm.workspace_by_window(&window) {
      if let Some(output) = wm.output_by_workspace(&workspace) {
        let output_left = output.extents().left();
        let output_width = output.extents().width();
        let current_scroll_left = workspace.scroll_left();

        let scroll_left =
          window.extents().left() + current_scroll_left - output_left - output_width / 2
            + window.size().width / 2;

        workspace.set_scroll_left(scroll_left);

        arrange_windows_workspace(wm, workspace);
      } else {
        warn!("center_window: Active window not on a output?");
      }
    }
  }
}

pub(crate) fn switch_keyboard_layout(wm: &CascadeWindowManager) {
  let current_layout = &wm.config_manager.config().keyboard;
  let current_index = wm
    .config
    .keyboard_layouts
    .iter()
    .enumerate()
    .find_map(|(index, layout)| {
      if layout == current_layout {
        Some(index)
      } else {
        None
      }
    })
    // If we didn't find the layout, default to the last so that
    // we increment to the first
    .unwrap_or(wm.config.keyboard_layouts.len() - 1);

  let next_index = (current_index + 1) % wm.config.keyboard_layouts.len();
  let next_layout = wm.config.keyboard_layouts[next_index].clone();
  debug!("Switching keyboard layout to: {:?}", &next_layout);
  wm.config_manager.update_config(move |config| {
    config.keyboard = next_layout;
  });
}
