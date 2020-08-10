use crate::animation::{Animation, AnimationConflict, AnimationDriver, AnimationManager};
use std::{rc::Rc, time::Duration};
use wlral::{
  geometry::{Displacement, FPoint, Point},
  window::Window,
};

const WINDOW_ANIMATION_SPEED: f64 = 15.0;
const MAX_WINDOW_ANIMATION_DURATION_MS: u64 = 300;

pub(crate) trait WindowAnimations {
  fn set_window_position(&self, window: Rc<Window>, to_top_left: Point);
  fn animate_window_position(&self, window: Rc<Window>, to_top_left: Point);
}

impl WindowAnimations for AnimationManager {
  fn set_window_position(&self, window: Rc<Window>, to_top_left: Point) {
    let start: FPoint = window.extents().top_left().into();
    let end: FPoint = to_top_left.into();
    self.start(Animation::immediate(
      Duration::from_millis(0),
      WindowAnimationDriver { window, start, end },
    ));
  }

  fn animate_window_position(&self, window: Rc<Window>, to_top_left: Point) {
    let start: FPoint = window.extents().top_left().into();
    let end: FPoint = to_top_left.into();
    if start == end {
      return;
    }
    let distance = (start - end).length();
    self.start(Animation::immediate(
      Duration::from_millis(
        ((distance * WINDOW_ANIMATION_SPEED) as u64).min(MAX_WINDOW_ANIMATION_DURATION_MS),
      ),
      WindowAnimationDriver { window, start, end },
    ));
  }
}

struct WindowAnimationDriver {
  window: Rc<Window>,
  start: FPoint,
  end: FPoint,
}

impl AnimationDriver for WindowAnimationDriver {
  fn step(&self, percent: f64) {
    self.window.set_translate(Displacement {
      dx: ((self.start.x - self.end.x) * (1.0 - percent)) as i32,
      dy: ((self.start.y - self.end.y) * (1.0 - percent)) as i32,
    });
  }
  fn started(&self) {
    self.window.move_to(self.end.into());
  }
  fn aborted(&self) {
    self.window.set_translate(Displacement::ZERO);
  }
  fn is_conflict(&self, other: &Self) -> AnimationConflict {
    if self.window == other.window {
      if self.end == other.end {
        AnimationConflict::Ignore
      } else {
        AnimationConflict::Replace
      }
    } else {
      AnimationConflict::NoConflict
    }
  }
}
