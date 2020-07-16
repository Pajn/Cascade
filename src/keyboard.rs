use crate::actions::*;
use crate::window_manager::CascadeWindowManager;
use log::debug;
use std::process::Command;
use wlral::input::events::*;
use xkbcommon::xkb;

pub fn handle_key_press(wm: &mut CascadeWindowManager, event: &KeyboardEvent) -> bool {
  let validate_mod = |mod_names: &Vec<&'static str>, mod_name| {
    mod_names.contains(&mod_name)
      == event
        .xkb_state()
        .mod_name_is_active(mod_name, xkb::STATE_MODS_DEPRESSED)
  };
  let has_mods = |mod_names: Vec<&'static str>| {
    validate_mod(&mod_names, xkb::MOD_NAME_SHIFT)
      && validate_mod(&mod_names, xkb::MOD_NAME_CTRL)
      && validate_mod(&mod_names, xkb::MOD_NAME_LOGO)
      && validate_mod(&mod_names, xkb::MOD_NAME_ALT)
  };

  if event.state() == KeyState::Pressed {
    match event.get_one_sym() {
      xkb::KEY_Home if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        navigate_first(wm);
        true
      }
      xkb::KEY_End if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        navigate_last(wm);
        true
      }
      xkb::KEY_Left if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        navigate(wm, Direction::Left);
        true
      }
      xkb::KEY_Right if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        navigate(wm, Direction::Right);
        true
      }
      xkb::KEY_Up if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        switch_workspace(wm, VerticalDirection::Up);
        true
      }
      xkb::KEY_Down if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        switch_workspace(wm, VerticalDirection::Down);
        true
      }
      xkb::KEY_Left if has_mods(vec![xkb::MOD_NAME_SHIFT, xkb::MOD_NAME_CTRL]) => {
        move_window(wm, Direction::Left);
        true
      }
      xkb::KEY_Right if has_mods(vec![xkb::MOD_NAME_SHIFT, xkb::MOD_NAME_CTRL]) => {
        move_window(wm, Direction::Right);
        true
      }
      xkb::KEY_Left if has_mods(vec![xkb::MOD_NAME_SHIFT, xkb::MOD_NAME_ALT]) => {
        navigate_monitor(wm, Direction::Left, Activation::LastActive);
        true
      }
      xkb::KEY_Right if has_mods(vec![xkb::MOD_NAME_SHIFT, xkb::MOD_NAME_ALT]) => {
        navigate_monitor(wm, Direction::Right, Activation::LastActive);
        true
      }
      xkb::KEY_Left
        if has_mods(vec![
          xkb::MOD_NAME_SHIFT,
          xkb::MOD_NAME_CTRL,
          xkb::MOD_NAME_ALT,
        ]) =>
      {
        move_window_monitor(wm, Direction::Left, Activation::LastActive);
        true
      }
      xkb::KEY_Right
        if has_mods(vec![
          xkb::MOD_NAME_SHIFT,
          xkb::MOD_NAME_CTRL,
          xkb::MOD_NAME_ALT,
        ]) =>
      {
        move_window_monitor(wm, Direction::Right, Activation::LastActive);
        true
      }
      xkb::KEY_a if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        Command::new("ulauncher-toggle")
          .spawn()
          .expect("failed to execute process");
        true
      }
      xkb::KEY_l if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
        Command::new("swaylock")
          .spawn()
          .expect("failed to execute process");
        true
      }
      // xkb::KEY_d if has_mods(vec![xkb::MOD_NAME_SHIFT]) => {
      //   println!("WM: {:?}", wm);
      //   for window in wm.windows.values() {
      //     println!("{}: {}", window.name(), window.state());
      //   }
      //   true
      // }
      xkb::KEY_r if has_mods(vec![xkb::MOD_NAME_CTRL]) => {
        if let Some(active_window) = wm.active_window {
          if let Some(monitor) = wm.monitor_by_window(active_window) {
            let monitor_width = monitor.extents().width();
            let window_width = wm.get_window(active_window).size().width;

            let window = wm.windows.get_mut(&active_window).unwrap();
            if window_width < monitor_width / 3 {
              window.set_size(window.size().with_width(monitor_width / 3));
            } else if window_width < monitor_width / 2 {
              window.set_size(window.size().with_width(monitor_width / 2));
            } else if window_width < ((monitor_width / 3) * 2) {
              window.set_size(window.size().with_width((monitor_width / 3) * 2));
            } else {
              window.set_size(window.size().with_width(monitor_width / 3));
            }
            arrange_windows(wm);
          } else {
            println!("Active window not on a monitor?");
          }
        }
        true
      }
      xkb::KEY_f if has_mods(vec![xkb::MOD_NAME_CTRL]) => {
        if let Some(active_window) = wm.active_window {
          if let Some(monitor) = wm.monitor_by_window(active_window) {
            let monitor_width = monitor.extents().width();

            let window = wm.windows.get_mut(&active_window).unwrap();
            window.set_size(window.size().with_width(monitor_width));
            arrange_windows(wm);
          } else {
            println!("Active window not on a monitor?");
          }
        }
        true
      }
      xkb::KEY_c if has_mods(vec![xkb::MOD_NAME_CTRL]) => {
        if let Some(active_window) = wm.active_window {
          if let Some(monitor) = wm.monitor_by_window(active_window) {
            let monitor_left = monitor.extents().left();
            let monitor_width = monitor.extents().width();
            let window = wm.get_window(active_window);

            if let Some(workspace_id) = window.on_workspace {
              let scroll_left =
                window.top_left().x - monitor_left - monitor_width / 2 + window.size().width / 2;

              let workspace = wm.get_workspace_mut(workspace_id);
              workspace.scroll_left = scroll_left;

              arrange_windows(wm);
            }
          } else {
            println!("Active window not on a monitor?");
          }
        }
        true
      }
      xkb::KEY_BackSpace if has_mods(vec![xkb::MOD_NAME_CTRL]) => {
        if let Some(active_window) = wm.active_window {
          let window = wm.get_window(active_window);
          window.ask_client_to_close();
        }
        true
      }
      xkb::KEY_space if has_mods(vec![xkb::MOD_NAME_CTRL]) => {
        let current_layout = &wm.config_manager.config().keyboard;
        let current_index = wm
          .config
          .keyboard_layouts
          .iter()
          .enumerate()
          .find_map(|(index, layout)| {
            if layout == current_layout {
              Some(index)
            } else {
              None
            }
          })
          // If we didn't find the layout, default to the last so that
          // we increment to the first
          .unwrap_or(wm.config.keyboard_layouts.len() - 1);

        let next_index = (current_index + 1) % wm.config.keyboard_layouts.len();
        let next_layout = wm.config.keyboard_layouts[next_index].clone();
        debug!("Switching keyboard layout to: {:?}", &next_layout);
        wm.config_manager.update_config(move |config| {
          config.keyboard = next_layout;
        });

        true
      }
      _ => false,
    }
  } else {
    false
  }
}
