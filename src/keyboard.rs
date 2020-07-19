use crate::actions::*;
use crate::window_manager::CascadeWindowManager;
use log::{debug, error, trace};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{collections::BTreeMap, process::Command};
use wlral::input::events::*;
use xkbcommon::xkb;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
#[serde(rename_all = "snake_case")]
pub enum ActionShortcut {
  NavigateToFirst,
  NavigateToLast,
  Navigate { direction: Direction },
  NavigateWorkspace { direction: VerticalDirection },
  NavigateMonitor { direction: Direction },

  MoveWindow { direction: Direction },
  MoveWindowWorkspace { direction: VerticalDirection },
  MoveWindowMonitor { direction: Direction },

  ResizeWindow { steps: Vec<f32> },
  CenterWindow,
  CloseWindow,

  SwitchKeyboardLayout,

  DebugPrintWindows,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CommandShortcut {
  cmd: String,
  #[serde(default)]
  args: Vec<String>,
}
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyboardShortcut {
  Action(ActionShortcut),
  Command(CommandShortcut),
}

#[derive(Default, Debug, Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Keybinding {
  alt: bool,
  ctrl: bool,
  logo: bool,
  shift: bool,
  key: xkb::Keysym,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcutsConfig(BTreeMap<Keybinding, KeyboardShortcut>);

impl Default for KeyboardShortcutsConfig {
  fn default() -> Self {
    let mut default = BTreeMap::new();
    default.insert(
      Keybinding {
        key: xkb::KEY_Home,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::NavigateToFirst),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_End,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::NavigateToLast),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Left,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::Navigate {
        direction: Direction::Left,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Right,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::Navigate {
        direction: Direction::Right,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Up,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::NavigateWorkspace {
        direction: VerticalDirection::Up,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Down,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::NavigateWorkspace {
        direction: VerticalDirection::Down,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Left,
        alt: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::NavigateMonitor {
        direction: Direction::Left,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Right,
        alt: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::NavigateMonitor {
        direction: Direction::Right,
      }),
    );

    default.insert(
      Keybinding {
        key: xkb::KEY_Left,
        ctrl: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::MoveWindow {
        direction: Direction::Left,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Right,
        ctrl: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::MoveWindow {
        direction: Direction::Right,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Up,
        ctrl: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::MoveWindowWorkspace {
        direction: VerticalDirection::Up,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Down,
        ctrl: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::MoveWindowWorkspace {
        direction: VerticalDirection::Down,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Left,
        alt: true,
        ctrl: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::MoveWindowMonitor {
        direction: Direction::Left,
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_Right,
        alt: true,
        ctrl: true,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::MoveWindowMonitor {
        direction: Direction::Right,
      }),
    );

    default.insert(
      Keybinding {
        key: xkb::KEY_r,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::ResizeWindow {
        steps: vec![0.33, 0.5, 0.66],
      }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_f,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::ResizeWindow { steps: vec![1.0] }),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_c,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::CenterWindow),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_BackSpace,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::CloseWindow),
    );
    default.insert(
      Keybinding {
        key: xkb::KEY_space,
        logo: true,
        ..Keybinding::default()
      },
      KeyboardShortcut::Action(ActionShortcut::SwitchKeyboardLayout),
    );

    KeyboardShortcutsConfig(default)
  }
}

impl Serialize for Keybinding {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut keys = vec![];
    if self.alt {
      keys.push("alt");
    }
    if self.ctrl {
      keys.push("ctrl");
    }
    if self.shift {
      keys.push("shift");
    }
    if self.logo {
      keys.push("super");
    }
    let key = xkb::keysym_get_name(self.key);
    keys.push(&key);
    keys.join("+").serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for Keybinding {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    use serde::de::Error;
    let keys: String = Deserialize::deserialize(deserializer)?;
    let mut keys = keys.split("+").map(str::trim).collect::<Vec<_>>();
    let key = keys.pop().ok_or(Error::custom("No key specified"))?;
    let mods = keys;
    let mut binding = Keybinding {
      key: xkb::keysym_from_name(key, xkb::KEYSYM_CASE_INSENSITIVE),
      ..Keybinding::default()
    };
    if binding.key == xkb::KEY_NoSymbol {
      return Err(Error::custom(format!("Invalid key \"{}\" specified", key)));
    }
    for modifier in mods {
      match &modifier.to_ascii_lowercase() as &str {
        "alt" => {
          binding.alt = true;
        }
        "ctrl" => {
          binding.ctrl = true;
        }
        "shift" => {
          binding.shift = true;
        }
        "super" | "logo" => {
          binding.logo = true;
        }
        _ => {
          return Err(Error::custom(format!(
            "Invalid modifier \"{}\" specified",
            modifier
          )));
        }
      }
    }
    Ok(binding)
  }
}

impl ActionShortcut {
  fn triggered(&self, wm: &mut CascadeWindowManager) {
    match self {
      ActionShortcut::NavigateToFirst => {
        navigate_first(wm);
      }
      ActionShortcut::NavigateToLast => {
        navigate_last(wm);
      }
      ActionShortcut::Navigate { direction } => {
        navigate(wm, *direction);
      }
      ActionShortcut::NavigateWorkspace { direction } => {
        switch_workspace(wm, *direction);
      }
      ActionShortcut::NavigateMonitor { direction } => {
        navigate_monitor(wm, *direction, Activation::LastActive);
      }
      ActionShortcut::MoveWindow { direction } => {
        move_window(wm, *direction);
      }
      ActionShortcut::MoveWindowWorkspace { direction } => {
        move_active_window_workspace(wm, *direction);
      }
      ActionShortcut::MoveWindowMonitor { direction } => {
        move_window_monitor(wm, *direction, Activation::LastActive);
      }
      ActionShortcut::ResizeWindow { steps } => {
        resize_window(wm, steps);
      }
      ActionShortcut::CenterWindow => {
        center_window(wm);
      }
      ActionShortcut::CloseWindow => {
        if let Some(active_window) = wm.active_window {
          let window = wm.get_window(active_window);
          window.ask_client_to_close();
        }
      }
      ActionShortcut::SwitchKeyboardLayout => {
        switch_keyboard_layout(wm);
      }
      ActionShortcut::DebugPrintWindows => {
        println!("DEBUG: Windows: {:?}", &wm.windows);
      }
    }
  }
}

impl CommandShortcut {
  fn triggered(&self, _wm: &mut CascadeWindowManager) {
    let result = Command::new(&self.cmd).args(&self.args).spawn();

    if let Err(error) = result {
      error!("Failed to execute command in shortcut: {}", error);
    }
  }
}

impl KeyboardShortcut {
  fn triggered(&self, wm: &mut CascadeWindowManager) {
    match self {
      KeyboardShortcut::Action(shortcut) => {
        shortcut.triggered(wm);
      }
      KeyboardShortcut::Command(shortcut) => {
        shortcut.triggered(wm);
      }
    }
  }
}

pub fn handle_key_press(wm: &mut CascadeWindowManager, event: &KeyboardEvent) -> bool {
  if event.state() == KeyState::Pressed {
    let binding = Keybinding {
      alt: event
        .xkb_state()
        .mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_DEPRESSED),
      ctrl: event
        .xkb_state()
        .mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_DEPRESSED),
      logo: event
        .xkb_state()
        .mod_name_is_active(xkb::MOD_NAME_LOGO, xkb::STATE_MODS_DEPRESSED),
      shift: event
        .xkb_state()
        .mod_name_is_active(xkb::MOD_NAME_SHIFT, xkb::STATE_MODS_DEPRESSED),
      key: xkb::keysym_from_name(
        &xkb::keysym_get_name(event.get_one_sym()),
        xkb::KEYSYM_CASE_INSENSITIVE,
      ),
    };
    let shortcut = wm.config.keyboard_shortcuts.0.get(&binding).cloned();

    trace!(
      "Pressed key {}, binding: {:?}",
      xkb::keysym_get_name(event.get_one_sym()),
      &binding
    );
    if let Some(shortcut) = shortcut {
      debug!("Triggering shortcut");
      shortcut.triggered(wm);
      true
    } else {
      false
    }
  } else {
    false
  }
}
