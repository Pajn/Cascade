use std::cmp;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use wlral::geometry::*;
use wlral::output::Output;
use wlral::window::Window as WlralWindow;
use wlral::window_management_policy::*;

// use crate::animation::*;
// use crate::input_inhibitor::{focus_exclusive_client, InputInhibitor};

pub type Id = u64;

#[derive(Debug)]
pub struct IdGenerator {
  next_id: Id,
}

impl IdGenerator {
  pub fn new() -> IdGenerator {
    IdGenerator { next_id: 1 }
  }

  pub fn next_id(&mut self) -> Id {
    let id = self.next_id;
    self.next_id = id + 1;
    id
  }
}

#[derive(Debug)]
pub struct Window {
  pub id: Id,
  pub on_workspace: Option<Id>,
  pub window_info: Rc<WlralWindow>,
  /// Position relative the workspace (static when workspace scrolls)
  pub current_position: Rectangle,
  pub pending_position: Option<Rectangle>,
  pub is_dragged: bool,
  // pub old_state: Option<raw::MirWindowState::Type>,
  // pub animation_state: Arc<WindowAnimaitonState>,
  pub animation_status: Arc<RwLock<AnimationStatus>>,
}

impl Window {
  pub fn new(
    id_generator: &mut IdGenerator,
    // animation_state: Arc<WindowAnimaitonState>,
    window_info: Rc<WlralWindow>,
  ) -> Window {
    Window {
      id: id_generator.next_id(),
      on_workspace: None,
      window_info,
      current_position: Rectangle::ZERO,
      pending_position: None,
      is_dragged: false,
      // old_state: None,
      // animation_state,
      animation_status: Arc::new(RwLock::new(AnimationStatus::Still)),
    }
  }

  pub fn name(&self) -> String {
    self.window_info.title().unwrap_or("".to_owned())
  }

  pub fn top_left(&self) -> Point {
    match *self.animation_status.read().unwrap() {
      AnimationStatus::IsAnimating(to) => to,
      AnimationStatus::Still => self
        .pending_position
        .as_ref()
        .map(Rectangle::top_left)
        .unwrap_or(self.current_position.top_left()),
    }
  }

  pub fn size(&self) -> Size {
    self
      .pending_position
      .as_ref()
      .map(Rectangle::size)
      .unwrap_or(self.current_position.size())
  }

  pub fn rendered_pos(&self) -> Rectangle {
    Rectangle {
      top_left: self.rendered_top_left(),
      size: self.rendered_size(),
    }
  }
  pub fn buffer_pos(&self) -> Rectangle {
    self.window_info.buffer_extents()
  }

  pub fn rendered_top_left(&self) -> Point {
    self.window_info.extents().top_left()
  }

  pub fn rendered_size(&self) -> Size {
    self.window_info.extents().size()
  }

  pub fn max_height(&self) -> i32 {
    // unsafe { ((*self.window_info).max_height()).value }
    999999
  }

  pub fn min_height(&self) -> i32 {
    // unsafe { ((*self.window_info).min_height()).value }
    0
  }

  pub fn max_width(&self) -> i32 {
    // unsafe { ((*self.window_info).max_width()).value }
    9999999
  }

  pub fn min_width(&self) -> i32 {
    // unsafe { ((*self.window_info).min_width()).value }
    0
  }

  // pub fn resize(&mut self, size: Size) {
  //   self.window_info.resize(Size {
  //     width: cmp::max(cmp::min(size.width, self.max_width()), self.min_width()),
  //     height: cmp::max(cmp::min(size.height, self.max_height()), self.min_height()),
  //   });
  // }

  // pub fn move_to(&mut self, top_left: Point) {
  //   println!("move_to x={}", top_left.x);
  //   // *self.animation_status.write().unwrap() = AnimationStatus::IsAnimating(top_left);
  //   // self.animation_state.start_animation(
  //   //   MoveWindow::new(self.window_info, self.animation_status.clone())
  //   //     .animate_to(Duration::from_millis(150), top_left),
  //   // );
  //   self.window_info.move_to(top_left);
  // }

  pub fn set_size(&mut self, size: Size) {
    self.set_position(Rectangle {
      top_left: self.top_left(),
      size,
    });
  }
  pub fn set_position(&mut self, position: Rectangle) {
    self.pending_position = Some(Rectangle {
      size: Size {
        width: cmp::max(
          cmp::min(position.size.width, self.max_width()),
          self.min_width(),
        ),
        height: cmp::max(
          cmp::min(position.size.height, self.max_height()),
          self.min_height(),
        ),
      },
      ..position
    });
  }

  pub fn commit_position(&mut self, scroll_left: i32) {
    if let Some(pending_position) = self.pending_position.take() {
      let top_left = pending_position.top_left()
        + Displacement {
          dx: -scroll_left,
          dy: 0,
        };
      if pending_position.size() == self.size() {
        println!("move_to {:?}", top_left);
        self.window_info.move_to(top_left);
        self.current_position.top_left = pending_position.top_left();
      } else {
        println!(
          "set_extents {:?} {:?} != {:?}",
          top_left,
          self.size(),
          self.window_info.extents().size()
        );
        self.window_info.set_extents(&Rectangle {
          top_left,
          size: pending_position.size(),
        });
      }
    }
  }

  // pub fn type_(&self) -> raw::MirWindowType::Type {
  //   unsafe { (*self.window_info).type_() }
  // }

  // pub fn state(&self) -> raw::MirWindowState::Type {
  //   unsafe { (*self.window_info).state() }
  // }

  // pub fn set_state(&self, state: raw::MirWindowState::Type) {
  //   unsafe {
  //     (*self.window_info).state1(state);
  //     configure_window(
  //       self.window_info,
  //       raw::MirWindowAttrib::mir_window_attrib_state,
  //       state as i32,
  //     );
  //   }
  // }

  pub fn has_parent(&self) -> bool {
    // unsafe { window_info_has_parent(self.window_info) }
    // self.window_info
    false
  }

  pub fn hide(&mut self) {
    // self.old_state = Some(self.state());
    // self.set_state(raw::MirWindowState::mir_window_state_hidden);
    // unsafe { hide_window(self.window_info) };
  }
  pub fn show(&mut self) {
    // if let Some(old_state) = self.old_state {
    //   self.old_state = None;
    //   let old_state = match old_state {
    //     raw::MirWindowState::mir_window_state_hidden
    //     | raw::MirWindowState::mir_window_state_minimized => {
    //       raw::MirWindowState::mir_window_state_restored
    //     }
    //     old_state => old_state,
    //   };
    //   self.set_state(old_state)
    // }
    // unsafe { show_window(self.window_info) };
  }

  pub fn minimize(&self) {
    // self.window_info.set_minimized(true);
  }
  pub fn restore(&self) {
    // self.window_info.set_minimized(false);
  }

  pub fn is_tiled(&self) -> bool {
    self.name() != "Ulauncher window title"
      && !self.has_parent()
      // && (self.type_() == raw::MirWindowType::mir_window_type_normal
      //   || self.type_() == raw::MirWindowType::mir_window_type_freestyle)
      && !self.window_info.fullscreen()
      && self.window_info.can_receive_focus()
    // && self.state() != raw::MirWindowState::mir_window_state_attached
  }

  pub fn ask_client_to_close(&self) -> () {
    self.window_info.ask_client_to_close()
  }
}

#[derive(Debug)]
pub enum AnimationStatus {
  Still,
  IsAnimating(Point),
}

// #[derive(Debug)]
// pub struct MoveWindow {
//   window_info: Rc<WlralWindow>,
//   status: Arc<RwLock<AnimationStatus>>,
// }

// unsafe impl Send for MoveWindow {}
// unsafe impl Sync for MoveWindow {}

// impl MoveWindow {
//   pub fn new(window_info: *mut miral::WindowInfo, status: Arc<RwLock<AnimationStatus>>) -> Self {
//     MoveWindow {
//       window_info,
//       status,
//     }
//   }
// }

// impl AnimationTarget<Point> for MoveWindow {
//   fn get_value(&self) -> Point {
//     Point {
//       x: unsafe { (&*(*self.window_info).window()).top_left().x.value },
//       y: unsafe { (*(*self.window_info).window()).top_left().y.value },
//     }
//   }

//   fn set_value(&mut self, top_left: Point) {
//     unsafe { (*(*self.window_info).window()).move_to(top_left.into()) };
//   }

//   fn is_same(&self, other: &Self) -> bool {
//     self.window_info == other.window_info
//   }

//   fn animation_ended(&self) {
//     *self.status.write().unwrap() = AnimationStatus::Still
//   }
// }

// pub type WindowAnimaitonState = AnimaitonState<Point, MoveWindow>;

#[derive(Debug)]
pub struct Workspace {
  pub id: Id,
  pub on_monitor: Option<Id>,
  pub scroll_left: i32,
  pub windows: Vec<Id>,
  pub mru_windows: MruList<Id>,
}

impl Workspace {
  pub fn new(id_generator: &mut IdGenerator) -> Workspace {
    Workspace {
      id: id_generator.next_id(),
      on_monitor: None,
      scroll_left: 0,
      windows: vec![],
      mru_windows: MruList::new(),
    }
  }

  pub fn get_window_index(&self, window: Id) -> Option<usize> {
    self
      .windows
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window)
      .map(|(index, _)| index)
  }

  pub fn swap_windows(&mut self, a: Id, b: Id) {
    let a_raw_index = self.get_window_index(a).unwrap();
    let b_raw_index = self.get_window_index(b).unwrap();
    self.windows.swap(a_raw_index, b_raw_index);
  }

  pub fn active_window(&self) -> Option<Id> {
    self.mru_windows.top().cloned()
  }
}

pub struct Monitor {
  pub id: Id,
  pub workspace: Id,
  pub output: Rc<Output>,
}

impl Monitor {
  pub fn new(id_generator: &mut IdGenerator, workspace: Id, output: Rc<Output>) -> Monitor {
    Monitor {
      id: id_generator.next_id(),
      workspace,
      output,
    }
  }

  pub fn extents(&self) -> Rectangle {
    self.output.extents()
  }
}

pub enum Gesture {
  Move(MoveRequest),
  Resize(ResizeRequest, Rectangle),
  None,
}

#[derive(Debug)]
pub struct MruList<T> {
  items: Vec<T>,
}

impl<T: PartialEq> MruList<T> {
  fn new() -> MruList<T> {
    MruList { items: vec![] }
  }

  pub fn iter(&self) -> std::iter::Rev<std::slice::Iter<'_, T>> {
    self.items.iter().rev()
  }

  pub fn top(&self) -> Option<&T> {
    self.items.last()
  }

  pub fn push(&mut self, item: T) {
    self.remove(&item);
    self.items.push(item);
  }

  pub fn remove(&mut self, item: &T) {
    match self.items.iter().position(|x| *x == *item) {
      Some(pos) => {
        self.items.remove(pos);
      }
      None => {}
    };
  }
}
