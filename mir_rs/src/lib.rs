mod ffi;

pub use ffi::root as raw;
pub use ffi::root::{mir, miral};
use xkbcommon::xkb;
use std::ops::Sub;

pub trait AsOption<T> {
  fn as_ref(&self) -> Option<&T>;
  fn as_mut(&mut self) -> Option<&mut T>;
}

impl<T> AsOption<T> for mir::optional_value<T> {
  fn as_ref(&self) -> Option<&T> {
    if self.is_set_ {
      Some(&self.value_)
    } else {
      None
    }
  }

  fn as_mut(&mut self) -> Option<&mut T> {
    if self.is_set_ {
      Some(&mut self.value_)
    } else {
      None
    }
  }
}

impl mir::geometry::Point {
  pub fn new(x: i32, y: i32) -> mir::geometry::Point {
    mir::geometry::Point {
      x: mir::geometry::detail::IntWrapper { value: x },
      y: mir::geometry::detail::IntWrapper { value: y },
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Point {
  pub x: i32,
  pub y: i32,
}

impl Point {
  pub fn x(&self) -> i32 {
    self.x
  }

  pub fn y(&self) -> i32 {
    self.y
  }
}

impl From<mir::geometry::Point> for Point {
  fn from(point: mir::geometry::Point) -> Point {
    Point {
      x: point.x.value,
      y: point.y.value,
    }
  }
}

impl From<Point> for mir::geometry::Point {
  fn from(point: Point) -> mir::geometry::Point {
    mir::geometry::Point {
      x: mir::geometry::detail::IntWrapper { value: point.x },
      y: mir::geometry::detail::IntWrapper { value: point.y },
    }
  }
}

impl mir::geometry::Size {
  pub fn new(width: i32, height: i32) -> mir::geometry::Size {
    mir::geometry::Size {
      width: mir::geometry::detail::IntWrapper { value: width },
      height: mir::geometry::detail::IntWrapper { value: height },
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Size {
  pub width: i32,
  pub height: i32,
}

impl Size {
  pub fn width(&self) -> i32 {
    self.width
  }

  pub fn height(&self) -> i32 {
    self.height
  }

  pub fn with_width(&self, width: i32) -> Size {
    Size {
      width,
      height: self.height,
    }
  }

  pub fn with_height(&self, height: i32) -> Size {
    Size {
      width: self.width,
      height,
    }
  }
}

impl From<mir::geometry::Size> for Size {
  fn from(size: mir::geometry::Size) -> Size {
    Size {
      width: size.width.value,
      height: size.height.value,
    }
  }
}

impl From<Size> for mir::geometry::Size {
  fn from(size: Size) -> mir::geometry::Size {
    mir::geometry::Size {
      width: mir::geometry::detail::IntWrapper { value: size.width },
      height: mir::geometry::detail::IntWrapper { value: size.height },
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Rectangle {
  pub top_left: Point,
  pub size: Size,
}

impl Rectangle {
  pub fn left(&self) -> i32 {
    self.top_left.x
  }

  pub fn top(&self) -> i32 {
    self.top_left.y
  }

  pub fn right(&self) -> i32 {
    self.bottom_right().x
  }

  pub fn bottom(&self) -> i32 {
    self.bottom_right().y
  }

  pub fn width(&self) -> i32 {
    self.size.width
  }

  pub fn height(&self) -> i32 {
    self.size.height
  }

  pub fn bottom_right(&self) -> Point {
    Point {
      x: self.left() + self.width(),
      y: self.top() + self.height(),
    }
  }

  pub fn contains(&self, point: &Point) -> bool {
    self.left() <= point.x && self.right() > point.x() &&
    self.top() <= point.y && self.bottom() > point.y
  }
}

impl From<mir::geometry::Rectangle> for Rectangle {
  fn from(rectangle: mir::geometry::Rectangle) -> Rectangle {
    Rectangle {
      top_left: rectangle.top_left.into(),
      size: rectangle.size.into(),
    }
  }
}

impl From<Rectangle> for mir::geometry::Rectangle {
  fn from(rectangle: Rectangle) -> mir::geometry::Rectangle {
    mir::geometry::Rectangle {
      top_left: rectangle.top_left.into(),
      size: rectangle.size.into(),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Displacement {
  pub dx: i32,
  pub dy: i32,
}

impl From<mir::geometry::Displacement> for Displacement {
  fn from(displacement: mir::geometry::Displacement) -> Displacement {
    Displacement {
      dx: displacement.dx.value,
      dy: displacement.dy.value,
    }
  }
}

impl From<Displacement> for mir::geometry::Displacement {
  fn from(displacement: Displacement) -> mir::geometry::Displacement {
    mir::geometry::Displacement {
      dx: mir::geometry::detail::IntWrapper { value: displacement.dx },
      dy: mir::geometry::detail::IntWrapper { value: displacement.dy },
    }
  }
}

impl Sub<Point> for Point {
  type Output = Displacement;

  fn sub(self, other: Self) -> Self::Output {
    Displacement {
      dx: self.x - other.x,
      dy: self.y - other.y,
    }
  }
}

pub mod input_event_modifier {
  use crate::ffi;

  pub type Type = ffi::root::MirInputEventModifier::Type;

  pub const NONE: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_none;
  pub const ALT: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_alt;
  pub const ALT_LEFT: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_alt_left;
  pub const ALT_RIGHT: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_alt_right;
  pub const SHIFT: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_shift;
  pub const SHIFT_LEFT: Type =
    ffi::root::MirInputEventModifier::mir_input_event_modifier_shift_left;
  pub const SHIFT_RIGHT: Type =
    ffi::root::MirInputEventModifier::mir_input_event_modifier_shift_right;
  pub const SYM: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_sym;
  pub const FUNCTION: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_function;
  pub const CTRL: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_ctrl;
  pub const CTRL_LEFT: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_ctrl_left;
  pub const CTRL_RIGHT: Type =
    ffi::root::MirInputEventModifier::mir_input_event_modifier_ctrl_right;
  pub const META: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_meta;
  pub const META_LEFT: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_meta_left;
  pub const META_RIGHT: Type =
    ffi::root::MirInputEventModifier::mir_input_event_modifier_meta_right;
  pub const CAPS_LOCK: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_caps_lock;
  pub const NUM_LOCK: Type = ffi::root::MirInputEventModifier::mir_input_event_modifier_num_lock;
  pub const SCROLL_LOCK: Type =
    ffi::root::MirInputEventModifier::mir_input_event_modifier_scroll_lock;
}

pub fn keyboard_event_key_code(event: *const ffi::root::MirKeyboardEvent) -> xkb::Keysym {
  unsafe { ffi::root::mir_keyboard_event_key_code(event) }
}

pub fn keyboard_event_modifiers(
  event: *const ffi::root::MirKeyboardEvent,
) -> input_event_modifier::Type {
  unsafe { ffi::root::mir_keyboard_event_modifiers(event) }
}
