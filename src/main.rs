mod actions;
mod animation;
mod entities;
mod keyboard;
mod pointer;
mod window_manager;

use std::collections::BTreeMap;
use wlral::compositor::Compositor;
use wlral::geometry::Point;

use crate::entities::{Gesture, IdGenerator};
use crate::window_manager::CascadeWindowManager;

fn main() {
  env_logger::init();

  let compositor = Compositor::init();
  let mut window_manager = CascadeWindowManager {
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
  compositor
    .run(window_manager)
    .expect("Could not start compositor");
}
