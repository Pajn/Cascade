use crate::entities::*;
use std::cmp;
use std::cmp::Ordering;

fn update_cached_positions(wm: &mut WindowManager, workspace_id: Id) -> () {
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
    .scan(monitor.map_or(0, |m| m.extents.left()), |next_x, window| {
      let x = *next_x;
      *next_x = x + window.size.width;
      Some((window.id, x))
    })
    .collect::<Vec<_>>();

  for (window_id, x) in positions {
    let monitor = wm.monitor_by_window(window_id);
    let window = wm.get_window(window_id);

    let height = match monitor {
      Some(monitor) => cmp::min(monitor.extents.height(), window.max_height()),
      None => window.height(),
    };

    let y = match monitor {
      Some(monitor) => monitor.extents.top() + (monitor.extents.height() - height) / 2,
      None => window.y(),
    };

    let window = wm.windows.get_mut(&window_id).unwrap();

    window.x = x;
    window.y = y;
    window.size.height = height;
  }
}

pub fn ensure_window_visible(wm: &mut WindowManager, window_id: Id) -> () {
  if let Some(monitor) = wm.monitor_by_window(window_id) {
    let extents_left = monitor.extents.left();
    let extents_right = monitor.extents.right();
    let window = wm.get_window(window_id);
    let window_x = window.x;
    let window_width = window.width();
    let workspace = wm.workspaces.get_mut(&wm.active_workspace).unwrap();

    let x_window_left = window_x;
    let x_window_right = window_x + window_width;

    let x_workspace_left = workspace.scroll_left + extents_left;
    let x_workspace_right = workspace.scroll_left + extents_right;

    if x_window_left < x_workspace_left {
      workspace.scroll_left = x_window_left - extents_left;
    } else if x_window_right > x_workspace_right {
      workspace.scroll_left = x_window_right - extents_right;
    }
  } else {
    println!(
      "ensure_window_visible on window \"{}\" not on any monitor",
      window_id
    );
  }
}

pub fn update_window_positions(wm: &mut WindowManager, workspace_id: Id) -> () {
  let workspace = wm.get_workspace(workspace_id);
  let scroll_left = workspace.scroll_left;
  let windows = workspace.get_tiled_windows(wm);

  for window_id in windows {
    let window = wm.get_window(window_id);

    let old_x = window.x();
    let old_y = window.y();
    let old_size = window.rendered_size();

    let x = window.x - scroll_left;
    let y = window.y;
    let size = window.size.clone();

    if size != old_size {
      let window = wm.windows.get_mut(&window_id).unwrap();
      window.resize(size);
    }

    if old_x != x || old_y != y {
      let window = wm.windows.get_mut(&window_id).unwrap();
      window.move_to(x, y);
    }
  }
}

pub fn arrange_windows_workspace(wm: &mut WindowManager, workspace_id: Id) -> () {
  update_cached_positions(wm, workspace_id);
  if let Some(active_window) = wm.active_window() {
    if active_window.workspace == workspace_id && active_window.is_tiled() {
      let id = active_window.id;
      ensure_window_visible(wm, id);
    }
  }
  update_window_positions(wm, workspace_id);
}

pub fn arrange_windows(wm: &mut WindowManager) -> () {
  arrange_windows_workspace(wm, wm.active_workspace);
}

pub enum Direction {
  Left,
  Right,
}

pub fn naviate_first(wm: &mut WindowManager) {
  if let Some(window_id) = wm.active_workspace().get_tiled_windows(wm).first().copied() {
    wm.focus_window(Some(window_id));
  }
}

pub fn naviate_last(wm: &mut WindowManager) {
  if let Some(window_id) = wm.active_workspace().get_tiled_windows(wm).last().copied() {
    wm.focus_window(Some(window_id));
  }
}

pub fn naviate(wm: &mut WindowManager, direction: Direction) {
  if let Some(active_window) = wm.active_window {
    if wm.get_window(active_window).is_tiled() {
      let workspace = wm.active_workspace();
      let tiled_windows = workspace.get_tiled_windows(wm);
      let index = workspace
        .get_tiled_window_index(wm, active_window)
        .expect("Active window not found in active workspace") as isize;
      let index = match direction {
        Direction::Left => index - 1,
        Direction::Right => index + 1,
      };
      if index < 0 || index > tiled_windows.len() as isize - 1 {
        naviate_monitor(wm, direction, Activation::FromDirection);
      } else if let Some(window_id) = tiled_windows.get(index as usize) {
        wm.focus_window(Some(*window_id));
      }
    }
  } else {
    match direction {
      Direction::Left => naviate_first(wm),
      Direction::Right => naviate_last(wm),
    }
  }
}

pub fn move_window(wm: &mut WindowManager, direction: Direction) {
  if let Some(active_window) = wm.active_window {
    if wm.get_window(active_window).is_tiled() {
      let workspace = wm.active_workspace();
      let tiled_windows = workspace.get_tiled_windows(wm);
      let raw_index = workspace
        .get_window_index(active_window)
        .expect("Active window not found in active workspace");
      let tiled_index = workspace
        .get_tiled_window_index(wm, active_window)
        .expect("Active window not found in active workspace") as isize;
      let tiled_index = match direction {
        Direction::Left => tiled_index - 1,
        Direction::Right => tiled_index + 1,
      };
      if tiled_index < 0 || tiled_index > tiled_windows.len() as isize - 1 {
        move_window_monitor(wm, direction, Activation::FromDirection);
      } else if let Some(new_raw_index) = tiled_windows
        .get(tiled_index as usize)
        .and_then(|id| workspace.get_window_index(*id))
      {
        wm.workspaces
          .get_mut(&wm.active_workspace)
          .expect("move window active workspace")
          .windows
          .swap(raw_index, new_raw_index);
        arrange_windows(wm);
      }
    }
  }
}

pub fn monitor_x_position(a: &&Monitor, b: &&Monitor) -> Ordering {
  a.extents.left().cmp(&b.extents.left())
}

pub enum Activation {
  LastActive,
  FromDirection,
}

pub fn naviate_monitor(wm: &mut WindowManager, direction: Direction, activation: Activation) {
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
        let window = match (activation, direction) {
          (Activation::LastActive, _) => wm
            .get_workspace(monitor.workspace)
            .active_window
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

pub fn move_window_monitor(wm: &mut WindowManager, direction: Direction, activation: Activation) {
  if let Some(active_window) = wm.active_window {
    let window = wm.get_window(active_window);
    if window.is_tiled() {
      let from_workspace = wm.get_workspace(window.workspace);
      let from_workspace_id = from_workspace.id;
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
            let to_workspace = wm.workspaces.get_mut(&monitor.workspace).unwrap();

            match (activation, direction) {
              (Activation::LastActive, _) => {
                let index = to_workspace
                  .active_window
                  .and_then(|w| to_workspace.get_window_index(w).map(|i| i + 1))
                  .unwrap_or(to_workspace.windows.len());

                to_workspace.windows.insert(index, active_window)
              }
              (Activation::FromDirection, Direction::Left) => {
                to_workspace.windows.push(active_window)
              }
              (Activation::FromDirection, Direction::Right) => {
                to_workspace.windows.insert(0, active_window)
              }
            };
            wm.active_workspace = to_workspace.id;

            wm.remove_window_from_workspace(active_window)
              .expect("Active window not found in active workspace");

            let window = wm.windows.get_mut(&active_window).unwrap();
            window.workspace = wm.active_workspace;

            arrange_windows(wm);
            arrange_windows_workspace(wm, from_workspace_id);
          }
        }
      } else {
        // TODO: This should not happen
        println!("Active window was on workspace that was not on any monitor")
      }
    }
  }
}
