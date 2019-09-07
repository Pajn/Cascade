use crate::actions::*;
use crate::entities::*;
use mir_rs::*;
use std::process::Command;
use xkbcommon::xkb;

pub fn handle_key_press(
  wm: &mut WindowManager,
  key_code: xkb::Keysym,
  modifiers: input_event_modifier::Type,
) -> bool {
  let modifier_mask = input_event_modifier::CTRL_LEFT
    | input_event_modifier::ALT_LEFT
    | input_event_modifier::META_LEFT
    | input_event_modifier::SHIFT_LEFT;

  let modifiers = modifiers & modifier_mask;

  if wm.input_inhibitor.is_inhibited() {
    return false;
  }

  match key_code {
    xkb::KEY_Home if modifiers == input_event_modifier::META_LEFT => {
      naviate_first(wm);
      true
    }
    xkb::KEY_End if modifiers == input_event_modifier::META_LEFT => {
      naviate_last(wm);
      true
    }
    xkb::KEY_Left if modifiers == input_event_modifier::META_LEFT => {
      naviate(wm, Direction::Left);
      true
    }
    xkb::KEY_Right if modifiers == input_event_modifier::META_LEFT => {
      naviate(wm, Direction::Right);
      true
    }
    xkb::KEY_Left
      if modifiers == input_event_modifier::META_LEFT | input_event_modifier::CTRL_LEFT =>
    {
      move_window(wm, Direction::Left);
      true
    }
    xkb::KEY_Right
      if modifiers == input_event_modifier::META_LEFT | input_event_modifier::CTRL_LEFT =>
    {
      move_window(wm, Direction::Right);
      true
    }
    xkb::KEY_Left
      if modifiers == input_event_modifier::META_LEFT | input_event_modifier::SHIFT_LEFT =>
    {
      naviate_monitor(wm, Direction::Left, Activation::LastActive);
      true
    }
    xkb::KEY_Right
      if modifiers == input_event_modifier::META_LEFT | input_event_modifier::SHIFT_LEFT =>
    {
      naviate_monitor(wm, Direction::Right, Activation::LastActive);
      true
    }
    xkb::KEY_Left
      if modifiers
        == input_event_modifier::META_LEFT
          | input_event_modifier::CTRL_LEFT
          | input_event_modifier::SHIFT_LEFT =>
    {
      move_window_monitor(wm, Direction::Left, Activation::LastActive);
      true
    }
    xkb::KEY_Right
      if modifiers
        == input_event_modifier::META_LEFT
          | input_event_modifier::CTRL_LEFT
          | input_event_modifier::SHIFT_LEFT =>
    {
      move_window_monitor(wm, Direction::Right, Activation::LastActive);
      true
    }
    xkb::KEY_a if modifiers == input_event_modifier::META_LEFT => {
      Command::new("ulauncher-toggle")
        .spawn()
        .expect("failed to execute process");
      true
    }
    xkb::KEY_d if modifiers == input_event_modifier::META_LEFT => {
      println!("WM: {:?}", wm);
      true
    }
    xkb::KEY_r if modifiers == input_event_modifier::META_LEFT => {
      if let Some(active_window) = wm.active_window {
        if let Some(monitor) = wm.monitor_by_window(active_window) {
          let monitor_width = monitor.extents.width();
          let window_width = wm.get_window(active_window).size.width;

          let window = wm.windows.get_mut(&active_window).unwrap();
          if window_width < monitor_width / 3 {
            window.set_size(window.size.with_width(monitor_width / 3));
          } else if window_width < monitor_width / 2 {
            window.set_size(window.size.with_width(monitor_width / 2));
          } else if window_width < ((monitor_width / 3) * 2) {
            window.set_size(window.size.with_width((monitor_width / 3) * 2));
          } else {
            window.set_size(window.size.with_width(monitor_width / 3));
          }
          arrange_windows(wm);
        } else {
          println!("Active window not on a monitor?");
        }
      }
      true
    }
    xkb::KEY_f if modifiers == input_event_modifier::META_LEFT => {
      if let Some(active_window) = wm.active_window {
        if let Some(monitor) = wm.monitor_by_window(active_window) {
          let monitor_width = monitor.extents.width();

          let window = wm.windows.get_mut(&active_window).unwrap();
          window.set_size(window.size.with_width(monitor_width));
          arrange_windows(wm);
        } else {
          println!("Active window not on a monitor?");
        }
      }
      true
    }
    xkb::KEY_c if modifiers == input_event_modifier::META_LEFT => {
      if let Some(active_window) = wm.active_window {
        if let Some(monitor) = wm.monitor_by_window(active_window) {
          let monitor_left = monitor.extents.left();
          let monitor_width = monitor.extents.width();
          let window = wm.get_window(active_window);

          if let Some(workspace_id) = window.on_workspace {
            let scroll_left = window.x - monitor_left - monitor_width / 2 + window.size.width / 2;

            let workspace = wm.workspaces.get_mut(&workspace_id).unwrap();
            workspace.scroll_left = scroll_left;

            arrange_windows(wm);
          }
        } else {
          println!("Active window not on a monitor?");
        }
      }
      true
    }
    xkb::KEY_BackSpace if modifiers == input_event_modifier::META_LEFT => {
      if let Some(active_window) = wm.active_window {
        let window = wm.get_window(active_window);
        window.ask_client_to_close(wm);
      }
      true
    }
    _ => false,
  }
}
