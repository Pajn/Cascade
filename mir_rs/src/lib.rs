mod ffi;

pub use ffi::root as raw;
pub use ffi::root::{mir, miral};
use xkbcommon::xkb;

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

impl mir::geometry::Size {
  pub fn new(width: i32, height: i32) -> mir::geometry::Size {
    mir::geometry::Size {
      width: mir::geometry::detail::IntWrapper { value: width },
      height: mir::geometry::detail::IntWrapper { value: height },
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
