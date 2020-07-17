use crate::config::Config;
use log::error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{error::Error, fs, process::Command, rc::Rc};
use wlral::config::ConfigManager;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageMode {
  Stretch,
  Fit,
  Fill,
  Center,
  Tile,
}

impl Default for ImageMode {
  fn default() -> Self {
    ImageMode::Fill
  }
}

impl ToString for ImageMode {
  fn to_string(&self) -> String {
    match self {
      ImageMode::Stretch => "stretch".to_string(),
      ImageMode::Fit => "fit".to_string(),
      ImageMode::Fill => "fill".to_string(),
      ImageMode::Center => "center".to_string(),
      ImageMode::Tile => "tile".to_string(),
    }
  }
}

#[derive(Default, Debug, PartialEq, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BackgroundConfig {
  pub color: Option<String>,
  #[serde(skip)]
  pub parsed_color: [f32; 3],
  pub image: Option<String>,
  pub image_mode: ImageMode,
}

impl BackgroundConfig {
  pub fn validate(config: &mut Config) -> Result<(), Box<dyn Error>> {
    let color_re = Regex::new(r"^\s*#([0-9a-fA-F]{6})\s*$").unwrap();

    if let Some(ref image_path) = config.background.image {
      let image_path = shellexpand::tilde(image_path).to_string();
      match fs::metadata(&image_path) {
        Ok(metadata) => {
          if !metadata.is_file() {
            return Err(format!("background.image: \"{}\" is a directory", image_path).into());
          }
          config.background.image = Some(image_path);
        }
        Err(error) => {
          return Err(format!("background.image: Can't read \"{}\": {}", image_path, error).into());
        }
      }
    } else {
      // Default to a gray color if there is no background image
      config.background.parsed_color = [0.3, 0.3, 0.3];
    }

    if let Some(ref color) = config.background.color {
      if let Some(m) = color_re.captures_iter(color).next().and_then(|c| c.get(1)) {
        let red = u8::from_str_radix(&m.as_str()[0..2], 16)? as f32;
        let green = u8::from_str_radix(&m.as_str()[2..4], 16)? as f32;
        let blue = u8::from_str_radix(&m.as_str()[4..6], 16)? as f32;
        let byte_max = u8::MAX as f32;
        config.background.parsed_color = [red / byte_max, green / byte_max, blue / byte_max];
      } else {
        return Err("background.color must be in the format #000000".into());
      }
    }

    Ok(())
  }

  pub fn init(config: &Config, config_manager: Rc<ConfigManager>) {
    println!("background config: {:?}", config.background);
    config_manager.update_config(|c| c.background_color = config.background.parsed_color);
    if let Some(ref image) = config.background.image {
      let result = Command::new("swaybg")
        .args(&["-i", image])
        .args(&["-m", &config.background.image_mode.to_string()])
        .spawn();

      if let Err(error) = result {
        error!("swaybg failed to start: {}", error);
      }
    }
  }
}
