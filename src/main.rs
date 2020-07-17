mod actions;
mod animation;
mod background;
mod config;
mod entities;
mod keyboard;
mod pointer;
mod window_manager;

use crate::entities::{Gesture, IdGenerator};
use crate::window_manager::CascadeWindowManager;
use background::BackgroundConfig;
use config::Config;
use log::error;
use std::collections::BTreeMap;
use wlral::compositor::Compositor;
use wlral::geometry::Point;

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

  let mut window_manager = CascadeWindowManager {
    config,

    config_manager: compositor.config_manager(),
    output_manager: compositor.output_manager(),
    window_manager: compositor.window_manager(),

    gesture: Gesture::None,
    restore_size: BTreeMap::new(),

    // tools,
    // input_inhibitor,
    monitor_id_generator: IdGenerator::new(),
    window_id_generator: IdGenerator::new(),
    workspace_id_generator: IdGenerator::new(),

    monitors: BTreeMap::new(),
    windows: BTreeMap::new(),
    workspaces: BTreeMap::new(),

    old_cursor: Point { x: 0, y: 0 },
    active_window: None,
    active_workspace: 0,
    new_window_workspace: 0,
    // animation_state: animation_state.clone(),
  };
  window_manager.init();
  BackgroundConfig::init(&window_manager.config, compositor.config_manager());
  compositor
    .run(window_manager)
    .expect("Could not start compositor");
}
