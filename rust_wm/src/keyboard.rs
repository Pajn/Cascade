use crate::actions::*;
use crate::entities::*;
use mir_rs::*;
use xkbcommon::xkb;

pub fn handle_key_press(
  wm: &mut WindowManager,
  key_code: xkb::Keysym,
  modifiers: InputEventModifier::Type,
) -> bool {
  let modifier_mask = InputEventModifier::CTRL_LEFT
    | InputEventModifier::ALT_LEFT
    | InputEventModifier::META_LEFT
    | InputEventModifier::SHIFT_LEFT;

  let modifiers = modifiers & modifier_mask;

  match key_code {
    xkb::KEY_Home if modifiers == InputEventModifier::META_LEFT => {
      naviate_first(wm);
      true
    }
    xkb::KEY_End if modifiers == InputEventModifier::META_LEFT => {
      naviate_last(wm);
      true
    }
    xkb::KEY_Left if modifiers == InputEventModifier::META_LEFT => {
      naviate(wm, Direction::Left);
      true
    }
    xkb::KEY_Right if modifiers == InputEventModifier::META_LEFT => {
      naviate(wm, Direction::Right);
      true
    }
    xkb::KEY_Left if modifiers == InputEventModifier::META_LEFT | InputEventModifier::CTRL_LEFT => {
      move_window(wm, Direction::Left);
      true
    }
    xkb::KEY_Right
      if modifiers == InputEventModifier::META_LEFT | InputEventModifier::CTRL_LEFT =>
    {
      move_window(wm, Direction::Right);
      true
    }
    _ => false,
  }
}
