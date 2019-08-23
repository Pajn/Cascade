use crate::entities::*;

pub fn arange_windows(wm: &mut WindowManager) -> () {
  let positions = wm
    .workspaces
    .get(&wm.active_workspace)
    .unwrap()
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
    .scan(0, |next_x, window| {
      let x = *next_x;
      *next_x = x + window.width();
      Some((window.id, x))
    })
    .collect::<Vec<_>>();

  for (window_id, x) in positions {
    let window = wm.windows.get_mut(&window_id).unwrap();

    let old_x = window.x();

    if old_x != x {
      window.move_to(x, window.y());
    }
  }
}

pub enum Direction {
  Left,
  Right,
}

pub fn naviate_first(wm: &mut WindowManager) {
  if let Some(window_id) = wm.active_workspace().get_tiled_windows(wm).first().copied() {
    wm.focus_window(window_id);
  }
}

pub fn naviate_last(wm: &mut WindowManager) {
  if let Some(window_id) = wm.active_workspace().get_tiled_windows(wm).last().copied() {
    wm.focus_window(window_id);
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
      if index >= 0 {
        if let Some(window_id) = tiled_windows.get(index as usize) {
          wm.focus_window(*window_id);
        }
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
      if tiled_index >= 0 {
        if let Some(new_raw_index) = tiled_windows
          .get(tiled_index as usize)
          .and_then(|id| workspace.get_window_index(*id))
        {
          wm.workspaces
            .get_mut(&wm.active_workspace)
            .expect("move window active workspace")
            .windows
            .swap(raw_index, new_raw_index);
          arange_windows(wm);
        }
      }
    }
  }
}
