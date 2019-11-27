use log::debug;
use std::cmp;
use std::cmp::Ordering;
use wlral::geometry::*;

use crate::entities::*;
use crate::window_manager::CascadeWindowManager;

fn update_cached_positions(wm: &mut CascadeWindowManager, workspace_id: Id) -> () {
  let monitor = wm.monitor_by_workspace(workspace_id);
  let positions = wm
    .get_workspace(workspace_id)
    .windows
    .iter()
    .filter_map(|window_id| {
      let window = wm.windows.get(window_id).unwrap();

      if window.is_tiled() {
        Some(window)
      } else {
        None
      }
    })
    .scan(
      monitor.map_or(0, |m| m.extents().left()),
      |next_x, window| {
        let x = *next_x;
        *next_x = x + window.size().width;
        Some((window.id, x))
      },
    )
    .collect::<Vec<_>>();

  for (window_id, x) in positions {
    let monitor = wm.monitor_by_window(window_id);
    let window = wm.get_window(window_id);

    let height = match monitor {
      Some(monitor) => cmp::min(monitor.extents().height(), window.max_height()),
      None => window.size().height,
    };

    let y = match monitor {
      Some(monitor) => monitor.extents().top() + (monitor.extents().height() - height) / 2,
      None => window.top_left().y,
    };

    let window = wm.windows.get_mut(&window_id).unwrap();

    println!("x={}", x);

    window.set_position(Rectangle {
      top_left: Point { x, y },
      size: window.size().with_height(height),
    })
  }
}

pub fn ensure_window_visible(wm: &mut CascadeWindowManager, window_id: Id) -> () {
  if let Some(monitor) = wm.monitor_by_window(window_id) {
    let extents_left = monitor.extents().left();
    let extents_right = monitor.extents().right();
    let window = wm.get_window(window_id);
    let window_x = window.top_left().x;
    let window_width = window.size().width();
    let workspace_id = window.on_workspace.unwrap();
    let workspace = wm.workspaces.get_mut(&workspace_id).unwrap();

    let x_window_left = window_x;
    let x_window_right = window_x + window_width;

    let x_workspace_left = workspace.scroll_left + extents_left;
    let x_workspace_right = workspace.scroll_left + extents_right;

    println!(
      "window_x={} window_width={} x_workspace_left={} x_workspace_right={}",
      window_x, window_width, x_workspace_left, x_workspace_right
    );
    println!(
      "extents_left={} extents_right={}",
      extents_left, extents_right
    );

    if x_window_left < x_workspace_left {
      workspace.scroll_left = x_window_left - extents_left;
      debug!("Scrolling left to {}", workspace.scroll_left);
    } else if x_window_right > x_workspace_right {
      workspace.scroll_left = x_window_right - extents_right;
      debug!("Scrolling right to {}", workspace.scroll_left);
    }
  } else {
    println!(
      "ensure_window_visible on window \"{}\" not on any monitor",
      window_id
    );
  }
}

pub fn update_window_positions(wm: &mut CascadeWindowManager, workspace_id: Id) -> () {
  let workspace = wm.get_workspace(workspace_id);
  let scroll_left = workspace.scroll_left;

  for window_id in workspace.windows.clone() {
    let window = wm.get_window_mut(window_id);

    if window.is_dragged {
      continue;
    }

    window.commit_position(scroll_left);
  }
}

pub fn arrange_windows_workspace(wm: &mut CascadeWindowManager, workspace_id: Id) -> () {
  update_cached_positions(wm, workspace_id);
  let workspace = wm.get_workspace(workspace_id);
  if let Some(active_window_id) = workspace.active_window() {
    let active_window = wm.get_window(active_window_id);
    if active_window.is_tiled() {
      ensure_window_visible(wm, active_window_id);
    }
  }
  update_window_positions(wm, workspace_id);
}

pub fn arrange_windows(wm: &mut CascadeWindowManager) -> () {
  arrange_windows_workspace(wm, wm.active_workspace);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
  Left,
  Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VerticalDirection {
  Up,
  Down,
}

pub fn navigate_first(wm: &mut CascadeWindowManager) {
  if let Some(window_id) = wm.active_workspace().windows.first().copied() {
    wm.focus_window(Some(window_id));
  }
}

pub fn navigate_last(wm: &mut CascadeWindowManager) {
  if let Some(window_id) = wm.active_workspace().windows.last().copied() {
    wm.focus_window(Some(window_id));
  }
}

pub fn get_tiled_window(
  wm: &CascadeWindowManager,
  window_id: Id,
  direction: Direction,
) -> Option<Id> {
  let window = wm.get_window(window_id);
  let workspace = wm.get_workspace(window.on_workspace.unwrap());
  let index = workspace
    .get_window_index(window_id)
    .expect("Active window not found in active workspace") as isize;
  let index = match direction {
    Direction::Left => index - 1,
    Direction::Right => index + 1,
  };

  if index < 0 {
    None
  } else {
    workspace.windows.get(index as usize).cloned()
  }
}

pub fn navigate(wm: &mut CascadeWindowManager, direction: Direction) {
  if let Some(active_window) = wm.active_window {
    if wm.get_window(active_window).is_tiled() {
      if let Some(other_window) = get_tiled_window(wm, active_window, direction) {
        wm.focus_window(Some(other_window));
      } else {
        navigate_monitor(wm, direction, Activation::FromDirection);
      }
    }
  } else {
    match direction {
      Direction::Left => navigate_first(wm),
      Direction::Right => navigate_last(wm),
    }
  }
}

pub fn move_window(wm: &mut CascadeWindowManager, direction: Direction) {
  if let Some(active_window) = wm.active_window {
    if let Some(workspace_id) = wm.get_window(active_window).on_workspace {
      if let Some(other_window) = get_tiled_window(wm, active_window, direction) {
        wm.workspaces
          .get_mut(&workspace_id)
          .unwrap()
          .swap_windows(active_window, other_window);
        arrange_windows(wm);
      } else {
        move_window_monitor(wm, direction, Activation::FromDirection);
      }
    }
  }
}

pub fn monitor_x_position(a: &&Monitor, b: &&Monitor) -> Ordering {
  a.extents().left().cmp(&b.extents().left())
}

pub enum Activation {
  LastActive,
  FromDirection,
}

pub fn navigate_monitor(
  wm: &mut CascadeWindowManager,
  direction: Direction,
  activation: Activation,
) {
  if let Some(current_monitor) = wm.active_workspace().on_monitor {
    let mut monitors = wm.monitors.values().collect::<Vec<_>>();
    monitors.sort_by(monitor_x_position);

    let index = monitors
      .iter()
      .enumerate()
      .find(|(_, m)| m.id == current_monitor)
      .map(|(index, _)| index)
      .expect("current_monitor not found") as isize;

    let index = match direction {
      Direction::Left => index - 1,
      Direction::Right => index + 1,
    };

    if index >= 0 {
      if let Some(monitor) = monitors.get(index as usize) {
        wm.active_workspace = monitor.workspace;
        wm.new_window_workspace = wm.active_workspace;
        let window = match (activation, direction) {
          (Activation::LastActive, _) => wm
            .get_workspace(monitor.workspace)
            .active_window()
            .or_else(|| wm.get_workspace(monitor.workspace).windows.last().cloned()),
          (Activation::FromDirection, Direction::Left) => {
            wm.get_workspace(monitor.workspace).windows.last().cloned()
          }
          (Activation::FromDirection, Direction::Right) => {
            wm.get_workspace(monitor.workspace).windows.first().cloned()
          }
        };
        wm.focus_window(window);
      }
    }
  } else {
    // TODO: This should not happen
    println!("Active workspace was not on any monitor")
  }
}

pub fn find_index_by_cursor(wm: &CascadeWindowManager, workspace_id: Id, cursor: &Point) -> usize {
  let workspace = wm.get_workspace(workspace_id);
  let mut max_x = None;
  for (index, window_id) in workspace.windows.iter().cloned().enumerate() {
    let window = wm.get_window(window_id);
    if window.is_tiled() {
      let pos = window.rendered_pos();

      if pos.left() <= cursor.x && pos.right() >= cursor.x {
        if cursor.x < pos.right() - pos.width() / 2 {
          return index;
        } else {
          return index + 1;
        }
      }

      if max_x == None || Some(pos.right()) > max_x {
        max_x = Some(pos.top_left().x());
      }
    }
  }

  if Some(cursor.x) > max_x {
    return workspace.windows.len();
  } else {
    return 0;
  }
}

pub fn move_window_workspace(
  wm: &mut CascadeWindowManager,
  window_id: Id,
  to_workspace_id: Id,
  index: usize,
) {
  let to_workspace = wm.workspaces.get_mut(&to_workspace_id).unwrap();

  to_workspace.windows.insert(index, window_id);
  to_workspace.mru_windows.push(window_id);
  wm.active_workspace = to_workspace_id;
  wm.new_window_workspace = to_workspace_id;

  wm.remove_window_from_workspace(window_id)
    .expect("Window to move not found in its workspace");

  let window = wm.windows.get_mut(&window_id).unwrap();
  let from_workspace_id = window.on_workspace;
  window.on_workspace = Some(to_workspace_id);

  arrange_windows_workspace(wm, to_workspace_id);
  if let Some(from_workspace_id) = from_workspace_id {
    arrange_windows_workspace(wm, from_workspace_id);
  }
}

pub fn move_window_monitor(
  wm: &mut CascadeWindowManager,
  direction: Direction,
  activation: Activation,
) {
  if let Some(active_window) = wm.active_window {
    let window = wm.get_window(active_window);
    if let Some(from_workspace_id) = window.on_workspace {
      let from_workspace = wm.get_workspace(from_workspace_id);
      if let Some(monitor) = from_workspace.on_monitor {
        let mut monitors = wm.monitors.values().collect::<Vec<_>>();
        monitors.sort_by(monitor_x_position);

        let index = monitors
          .iter()
          .enumerate()
          .find(|(_, m)| m.id == monitor)
          .map(|(index, _)| index)
          .expect("current_monitor not found") as isize;

        let index = match direction {
          Direction::Left => index - 1,
          Direction::Right => index + 1,
        };

        if index >= 0 {
          if let Some(monitor) = monitors.get(index as usize) {
            let to_workspace_id = monitor.workspace;
            let to_workspace = wm.get_workspace(to_workspace_id);

            let index = match (activation, direction) {
              (Activation::LastActive, _) => to_workspace
                .active_window()
                .and_then(|w| to_workspace.get_window_index(w).map(|i| i + 1))
                .unwrap_or(to_workspace.windows.len()),
              (Activation::FromDirection, Direction::Left) => to_workspace.windows.len(),
              (Activation::FromDirection, Direction::Right) => 0,
            };

            move_window_workspace(wm, active_window, to_workspace_id, index);
          }
        }
      } else {
        // TODO: This should not happen
        println!("Active window was on workspace that was not on any monitor")
      }
    }
  }
}

pub fn switch_workspace(wm: &mut CascadeWindowManager, direction: VerticalDirection) {
  let mut hidden_workspaces = wm
    .workspaces
    .values()
    .filter(|workspace| workspace.on_monitor.is_none())
    .collect::<Vec<_>>();
  hidden_workspaces.sort_by(|a, b| a.id.cmp(&b.id));

  let current_workspace = wm.active_workspace();
  let current_workspace_id = current_workspace.id;

  for window_id in current_workspace.windows.clone() {
    let window = wm.windows.get_mut(&window_id).unwrap();
    window.hide();
  }

  let next_workspace = match direction {
    VerticalDirection::Up => hidden_workspaces
      .iter()
      .find(|w| w.id > current_workspace_id)
      .or(hidden_workspaces.first())
      .unwrap(),
    VerticalDirection::Down => hidden_workspaces
      .iter()
      .rfind(|w| w.id < current_workspace_id)
      .or(hidden_workspaces.last())
      .unwrap(),
  };
  let next_workspace_id = next_workspace.id;
  let next_active_window = next_workspace.active_window();

  let current_workspace = wm.workspaces.get_mut(&current_workspace_id).unwrap();
  let monitor_id = current_workspace.on_monitor.unwrap();
  current_workspace.on_monitor = None;

  let next_workspace = wm.workspaces.get_mut(&next_workspace_id).unwrap();
  next_workspace.on_monitor = Some(monitor_id);

  let monitor = wm.monitors.get_mut(&monitor_id).unwrap();
  monitor.workspace = next_workspace_id;

  wm.active_workspace = next_workspace_id;
  wm.new_window_workspace = next_workspace_id;
  wm.focus_window(next_active_window);
  arrange_windows_workspace(wm, next_workspace_id);

  let next_workspace = wm.get_workspace(next_workspace_id);
  for window_id in next_workspace.windows.clone() {
    let window = wm.windows.get_mut(&window_id).unwrap();
    window.show();
  }
}

// pub fn apply_resize_by(wm: &mut CascadeWindowManager, displacement: Displacement) -> () {
//   if let Gesture::Resize(ref gesture, ref rect) = wm.gesture {
//     let old_pos = rect.top_left;
//     let old_size = rect.size;
//     let mut new_pos = old_pos.clone();
//     let mut new_size = old_size.clone();
//     let window_id = match wm.window_by_info(gesture.window.clone()) {
//       Some(window) => window.id,
//       None => return,
//     };

//     if gesture.edges.contains(WindowEdge::RIGHT) {
//       new_size.width = old_size.width + displacement.dx;
//     }

//     if gesture.edges.contains(WindowEdge::LEFT) {
//       let requested_width = old_size.width - displacement.dx;
//       let window = wm.get_window(window_id);

//       new_size.width = cmp::max(
//         cmp::min(requested_width, window.max_width()),
//         window.min_width(),
//       );
//       new_pos.x = old_pos.x + displacement.dx + (requested_width - new_size.width);

//       if let Some(workspace_id) = window.on_workspace {
//         let workspace = wm.workspaces.get_mut(&workspace_id).unwrap();
//         workspace.scroll_left -= displacement.dx;
//       }
//     }

//     if gesture.edges.contains(WindowEdge::TOP) {
//       new_size.height = old_size.height - displacement.dy;
//       new_pos.y = old_pos.y + displacement.dy;
//     }

//     if gesture.edges.contains(WindowEdge::BOTTOM) {
//       new_size.height = old_size.height + displacement.dy;
//     }

//     if let Some(workspace_id) = wm.get_window(window_id).on_workspace {
//       if new_pos != old_pos || new_size != old_size {
//         if new_size != old_size {
//           let window = wm.windows.get_mut(&window_id).unwrap();
//           window.resize(new_size);
//         }
//         arrange_windows_workspace(wm, workspace_id);
//       }
//     }
//     if wm.active_window != Some(window_id) {
//       wm.focus_window(Some(window_id));
//     }
//     if let Gesture::Resize(_, ref mut rect) = wm.gesture {
//       rect.top_left = new_pos;
//       rect.size = new_size;
//     }
//   }
// }