use super::MruList;
use crate::actions::Direction;
use log::trace;
use std::{
  cell::{Ref, RefCell},
  rc::Rc,
};
use wlral::{geometry::Point, window::Window};

pub(crate) enum WorkspacePosition {
  ActiveWindow,
  Start,
  End,
  Coordinate(Point),
}

#[derive(Debug)]
pub(crate) struct Workspace {
  scroll_left: RefCell<i32>,
  windows: RefCell<Vec<Rc<Window>>>,
  mru_windows: RefCell<MruList<Rc<Window>>>,
}

impl PartialEq<Workspace> for Workspace {
  fn eq(&self, other: &Workspace) -> bool {
    RefCell::as_ptr(&self.windows) == RefCell::as_ptr(&other.windows)
  }
}

impl Workspace {
  pub(crate) fn new() -> Workspace {
    Workspace {
      scroll_left: RefCell::new(0),
      windows: RefCell::new(vec![]),
      mru_windows: RefCell::new(MruList::new()),
    }
  }

  pub(crate) fn scroll_left(&self) -> i32 {
    *self.scroll_left.borrow()
  }
  pub(crate) fn set_scroll_left(&self, scroll_left: i32) {
    *self.scroll_left.borrow_mut() = scroll_left;
  }

  pub(crate) fn has_window(&self, window: &Window) -> bool {
    self.windows.borrow().iter().any(|w| w.as_ref() == window)
  }

  pub(crate) fn windows(&self) -> Ref<Vec<Rc<Window>>> {
    self.windows.borrow()
  }
  pub(crate) fn mru_windows(&self) -> Ref<MruList<Rc<Window>>> {
    self.mru_windows.borrow()
  }
  fn index_of_window(&self, window: &Rc<Window>) -> usize {
    self
      .windows()
      .iter()
      .enumerate()
      .find(|(_, w)| *w == window)
      .map(|(index, _)| index)
      .expect("window not found in workspace")
  }
  pub(crate) fn window_by_direction(
    &self,
    from: &Rc<Window>,
    direction: Direction,
  ) -> Option<Rc<Window>> {
    let from_index = self.index_of_window(from);
    trace!(
      "window_by_direction from: \"{:?}\", direction: {:?}, index: {}",
      from.title(),
      direction,
      from_index
    );

    let index = match direction {
      Direction::Left => {
        if from_index == 0 {
          return None;
        }
        from_index - 1
      }
      Direction::Right => from_index + 1,
    };

    trace!(
      "window_by_direction index: {}, num windows: {}",
      index,
      self.windows().len(),
    );

    self.windows().get(index).cloned()
  }
  pub(crate) fn move_window(&self, window: &Rc<Window>, direction: Direction) -> Result<(), ()> {
    let from_index = self.index_of_window(window);

    let to_index = match direction {
      Direction::Left => {
        if from_index == 0 {
          return Err(());
        }
        from_index - 1
      }
      Direction::Right => from_index + 1,
    };

    if to_index == self.windows().len() {
      return Err(());
    }

    self.windows.borrow_mut().swap(from_index, to_index);
    trace!(
      "Moved window \"{:?}\" {:?} to index {}",
      window.title(),
      direction,
      to_index
    );
    Ok(())
  }

  pub(crate) fn promote_window(&self, window: &Rc<Window>) {
    self.mru_windows.borrow_mut().promote(window);
  }
  pub(crate) fn add_window(&self, window: Rc<Window>, position: WorkspacePosition) {
    assert!(window.can_receive_focus());
    let index = match position {
      WorkspacePosition::ActiveWindow => {
        let active_window = self.mru_windows().top().cloned();
        if let Some(active_window) = active_window {
          let index = self
            .windows
            .borrow()
            .iter()
            .enumerate()
            .find_map(|(index, window)| {
              if *window == active_window {
                Some(index)
              } else {
                None
              }
            })
            .expect("Window should be in both mru_windows and windows");

          index + 1
        } else {
          0
        }
      }
      WorkspacePosition::Start => 0,
      WorkspacePosition::End => self.windows.borrow().len(),
      WorkspacePosition::Coordinate(point) => {
        self
          .windows()
          .iter()
          .enumerate()
          .fold(0, |last_index, (current_index, window)| {
            let extents = window.extents();
            if point.x < extents.left() {
              last_index
            } else if point.x < extents.center_x() {
              current_index
            } else {
              current_index + 1
            }
          })
      }
    };

    trace!(
      "Adding window \"{:?}\" to workspace at index {}",
      window.title(),
      index
    );
    self.windows.borrow_mut().insert(index, window.clone());
    self.mru_windows.borrow_mut().push(window);
  }
  pub(crate) fn remove_window(&self, window: &Rc<Window>) {
    trace!("Removing window \"{:?}\" from workspace", window.title());
    self.windows.borrow_mut().retain(|w| w != window);
    self.mru_windows.borrow_mut().remove(window);
  }
}
