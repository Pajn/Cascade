mod actions;
mod background;
mod config;
mod entities;
mod keyboard;
mod pointer;
mod window_manager;

use crate::window_manager::CascadeWindowManager;
use background::BackgroundConfig;
use config::Config;
use log::error;
use wlral::compositor::Compositor;

fn main() {
  env_logger::init();
  let config = match Config::load() {
    Ok(config) => config,
    Err(error) => {
      error!("Eror loading config (falling back to default): {}", error);
      Config::default()
    }
  };

  let compositor = Compositor::init();
  compositor.config_manager().update_config(|c| {
    c.keyboard = config.keyboard_layouts.first().cloned().unwrap_or_default();
  });

  let window_manager = CascadeWindowManager::new(config, &compositor);
  BackgroundConfig::init(&window_manager.config, compositor.config_manager());
  compositor
    .run(window_manager)
    .expect("Could not start compositor");
}
