use crate::background::BackgroundConfig;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs};
use wlral::input::keyboard::KeyboardConfig;

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
  pub keyboard_layouts: Vec<KeyboardConfig>,
  pub background: BackgroundConfig,
}

impl Config {
  pub fn load() -> Result<Config, Box<dyn Error>> {
    let config_string =
      fs::read_to_string(shellexpand::tilde("~/.config/cascade/config.yaml").to_string())?;
    let mut config: Config = serde_yaml::from_str(&config_string)?;

    for (i, a) in config.keyboard_layouts.iter().enumerate() {
      for (j, b) in config.keyboard_layouts.iter().enumerate() {
        if a == b && i != j {
          return Err(
            format!(
              "keyboard_layouts: Duplicated keyboard layout in index {} and {}: {:?}",
              i, j, a
            )
            .into(),
          );
        }
      }
    }

    BackgroundConfig::validate(&mut config)?;

    Ok(config)
  }
}
