use log::debug;
use std::collections::BTreeMap;
use std::rc::Rc;
use wlral::geometry::*;
use wlral::input::event_filter::EventFilter;
use wlral::input::events::*;
use wlral::output::Output;
use wlral::output_manager::OutputManager;
use wlral::window::Window as WlralWindow;
use wlral::window_management_policy::*;
use wlral::window_manager::WindowManager;

use crate::actions::*;
use crate::entities::{Gesture, Id, IdGenerator, Monitor, Window, Workspace};
use crate::keyboard::handle_key_press;
use crate::pointer;

pub struct CascadeWindowManager {
  pub output_manager: Rc<OutputManager>,
  pub window_manager: Rc<WindowManager>,

  pub gesture: Gesture,
  pub restore_size: BTreeMap<usize, Rectangle>,

  pub monitor_id_generator: IdGenerator,
  pub window_id_generator: IdGenerator,
  pub workspace_id_generator: IdGenerator,

  pub monitors: BTreeMap<Id, Monitor>,
  pub windows: BTreeMap<Id, Window>,
  pub workspaces: BTreeMap<Id, Workspace>,

  pub old_cursor: Point,
  pub active_window: Option<Id>,
  pub active_workspace: Id,
  pub new_window_workspace: Id,
  // pub animation_state: Arc<WindowAnimaitonState>,
}

impl CascadeWindowManager {
  pub fn init(&mut self) {
    let workspace_id = self.get_or_create_unused_workspace();
    self.new_window_workspace = workspace_id;
    self.active_workspace = workspace_id;
  }

  pub fn get_window(&self, window_id: Id) -> &Window {
    self
      .windows
      .get(&window_id)
      // .expect(format!("Window with id {} not found", window_id))
      .expect("Window with id {} not found")
  }

  pub fn get_window_mut(&mut self, window_id: Id) -> &mut Window {
    self
      .windows
      .get_mut(&window_id)
      // .expect(format!("Window with id {} not found", window_id))
      .expect("Window with id {} not found")
  }

  pub fn get_workspace(&self, workspace_id: Id) -> &Workspace {
    self
      .workspaces
      .get(&workspace_id)
      // .expect(format!("Workspace with id {} not found", workspace_id))
      .expect("Workspace with id {} not found")
  }

  pub fn get_workspace_mut(&mut self, workspace_id: Id) -> &mut Workspace {
    self
      .workspaces
      .get_mut(&workspace_id)
      // .expect(format!("Workspace with id {} not found", workspace_id))
      .expect("Workspace with id {} not found")
  }

  pub fn monitor_by_workspace(&self, workspace_id: Id) -> Option<&Monitor> {
    self
      .get_workspace(workspace_id)
      .on_monitor
      .and_then(|monitor_id| self.monitors.get(&monitor_id))
  }

  pub fn monitor_by_window(&self, window_id: Id) -> Option<&Monitor> {
    let workspace_id = self.get_window(window_id).on_workspace;
    workspace_id.and_then(|workspace_id| self.monitor_by_workspace(workspace_id))
  }

  pub fn window_by_info(&self, window_info: Rc<WlralWindow>) -> Option<&Window> {
    self.windows.values().find(|w| w.window_info == window_info)
  }

  pub fn get_window_at(&self, point: &Point) -> Option<&Window> {
    println!("get_window_at, {:?}", point);
    self
      .monitors
      .values()
      .find(|m| m.extents().contains(point))
      .and_then(|m| {
        println!(
          "windows, {:?}",
          self
            .get_workspace(m.workspace)
            .mru_windows
            .iter()
            .map(|w| self.get_window(*w).buffer_pos())
            .collect::<Vec<_>>()
        );
        self
          .get_workspace(m.workspace)
          .mru_windows
          .iter()
          .map(|w| self.get_window(*w))
          .find(|w| w.buffer_pos().contains(point))
      })
  }

  pub fn active_window(&self) -> Option<&Window> {
    self.active_window.and_then(|id| self.windows.get(&id))
  }

  pub fn active_workspace(&self) -> &Workspace {
    self
      .workspaces
      .get(&self.active_workspace)
      .expect("Active workspace not found")
  }

  pub fn new_window_workspace(&self) -> &Workspace {
    self
      .workspaces
      .get(&self.new_window_workspace)
      .expect("New window workspace not found")
  }

  pub fn get_or_create_unused_workspace(&mut self) -> Id {
    let unused_workspaces = self
      .workspaces
      .values()
      .filter(|w| w.on_monitor == None)
      .collect::<Vec<_>>();

    match unused_workspaces.first() {
      Option::None => {
        let first_workspace = Workspace::new(&mut self.workspace_id_generator);
        let first_workspace_id = first_workspace.id;
        self.workspaces.insert(first_workspace.id, first_workspace);
        let second_workspace = Workspace::new(&mut self.workspace_id_generator);
        let second_workspace_id = second_workspace.id;
        self
          .workspaces
          .insert(second_workspace.id, second_workspace);

        debug!(
          "Created workspaces as there were no unused first={}, extra={}",
          first_workspace_id, second_workspace_id
        );

        first_workspace_id
      }
      Some(first_workspace) => {
        let first_workspace_id = first_workspace.id;

        // We want there to always be an additional workspace avalible
        if unused_workspaces.len() == 1 {
          let aditional_workspace = Workspace::new(&mut self.workspace_id_generator);
          let aditional_workspace_id = aditional_workspace.id;
          self
            .workspaces
            .insert(aditional_workspace.id, aditional_workspace);

          debug!(
            "Created workspace as there were no extras {}",
            aditional_workspace_id
          );
        }

        first_workspace_id
      }
    }
  }

  pub fn delete_window(&mut self, window_id: Id) -> () {
    // Ignore the error as it's normal that windows are not in any workspace
    let _ = self.remove_window_from_workspace(window_id);
    self.windows.remove(&window_id);

    if self.active_window == Some(window_id) {
      // TODO: WLRAL does not do this!
      // Mir will focus a new window for us so we can just unset
      // active_window and wait for the focus event
      self.active_window = None;
    }
  }

  pub fn focus_window(&mut self, window_id: Option<Id>) -> () {
    if let Some(window_id) = window_id {
      if let Some(workspace_id) = self.get_window(window_id).on_workspace {
        let workspace = self.get_workspace_mut(workspace_id);

        workspace.mru_windows.push(window_id);

        if workspace.on_monitor.is_some() {
          self.active_workspace = workspace_id;
          self.new_window_workspace = workspace_id;
        }
      }
      self.active_window = Some(window_id);
      self
        .window_manager
        .focus_window(self.get_window(window_id).window_info.clone());
      arrange_windows(self);
    }
  }

  pub fn remove_window_from_workspace(&mut self, window_id: Id) -> Result<(), ()> {
    let workspace = self.get_workspace(self.get_window(window_id).on_workspace.ok_or(())?);
    let workspace_id = workspace.id;
    // if workspace.active_window() == Some(window_id) {
    //   let active_window = self.get_window(window_id);
    //   if active_window.is_tiled() {
    //     let window_index = workspace.get_window_index(window_id).ok_or(())?;
    //     let window_index = if window_index > 0 {
    //       window_index - 1
    //     } else {
    //       window_index + 1
    //     };
    //     let next_active_window = workspace.windows.get(window_index).copied();
    //     let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
    //     workspace.active_window = next_active_window;
    //   } else {
    //     let next_active_window = workspace.windows.last().copied();
    //     let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
    //     workspace.active_window = next_active_window;
    //   }
    // }
    let workspace = self.get_workspace_mut(workspace_id);
    let raw_index = workspace.get_window_index(window_id).ok_or(())?;
    workspace.windows.remove(raw_index);
    workspace.mru_windows.remove(&window_id);
    Ok(())
  }
}

impl WindowManagementPolicy for CascadeWindowManager {
  fn handle_window_ready(&mut self, window_info: Rc<WlralWindow>) {
    let mut window = Window::new(
      &mut self.window_id_generator,
      // wm.animation_state.clone(),
      window_info,
    );
    // window.x = window.x();
    // window.y = window.y();
    window.current_position = window.window_info.extents();
    println!("handle_window_ready name: {:?}", window.name());

    // let type_ = unsafe { window_info.type_() };
    // let has_parent = unsafe { window_info_has_parent(window_info) };
    if window.is_tiled() {
      window.on_workspace = Some(self.new_window_workspace);
      // println!(
      //   "handle_window_ready tiled type_ {}, has_parent {}",
      //   type_, has_parent
      // );
      // } else {
      //   println!(
      //     "handle_window_ready not tiled type_ {}, has_parent {}",
      //     type_, has_parent
      //   );
    }

    // println!("WM: {:?}, adding: {:?}", &self, &window);
    if let Some(workspace_id) = window.on_workspace {
      println!("workspace_id {}", workspace_id);
      println!("workspaces {:?}", self.workspaces);
      let workspace = self.workspaces.get_mut(&workspace_id).unwrap();

      if let Some(index) = self
        .active_window
        .and_then(|active_window| workspace.get_window_index(active_window))
      {
        workspace.windows.insert(index + 1, window.id);
      } else {
        workspace.windows.push(window.id);
      }
    }

    let window_id = window.id;
    self.windows.insert(window.id, window);

    let window = self.get_window(window_id);
    if !window.has_parent() {
      self.focus_window(Some(window_id));
    }

    arrange_windows(self);
  }

  fn advise_configured_window(&mut self, window_info: Rc<WlralWindow>) {
    if let Some(window) = self.window_by_info(window_info.clone()) {
      let window_id = window.id;
      let new_position = window_info.extents()
        + Displacement {
          dx: window
            .on_workspace
            .map(|w| self.get_workspace(w).scroll_left)
            .unwrap_or(0),
          dy: 0,
        };
      self.get_window_mut(window_id).current_position = new_position;
    } else {
      println!(
        "nowindow in windows advise_configured_window, title: {:?}",
        window_info.title()
      );
    }
  }

  fn advise_delete_window(&mut self, window_info: Rc<WlralWindow>) {
    if let Some(window) = self.window_by_info(window_info.clone()) {
      let window_id = window.id;
      self.delete_window(window_id);
    } else {
      println!(
        "nowindow in windows advise_delete_window, title: {:?}",
        window_info.title()
      );
    }

    // println!("advise_delete_window {:?}", &self);
  }

  fn advise_output_create(&mut self, output: Rc<Output>) {
    let workspace_id = self.get_or_create_unused_workspace();
    println!("advise_output_create, workspace={}", workspace_id);
    let monitor = Monitor::new(&mut self.monitor_id_generator, workspace_id, output);
    self.get_workspace_mut(workspace_id).on_monitor = Some(monitor.id);
    self.monitors.insert(monitor.id, monitor);
  }

  fn advise_output_update(&mut self, output: Rc<Output>) {
    let monitor = self
      .monitors
      .iter_mut()
      .find(|(_, m)| &m.output == &output)
      .expect("monitor advise_output_update")
      .1;

    println!("advise_output_update {:?}", output.extents());

    let workspace_id = monitor.workspace;
    arrange_windows_workspace(self, workspace_id);
  }

  fn advise_output_delete(&mut self, output: Rc<Output>) {
    let monitor = self
      .monitors
      .iter_mut()
      .find(|(_, m)| &m.output == &output)
      .expect("monitor advise_output_delete")
      .1;
    let workspace = self
      .workspaces
      .get_mut(&monitor.workspace)
      .expect("workspace advise_output_delete");
    workspace.on_monitor = None;
    let monitor_id = monitor.id;
    self.monitors.remove(&monitor_id);

    arrange_windows(self);
  }

  fn handle_request_move(&mut self, request: MoveRequest) {
    if !self.window_manager.window_has_focus(&request.window) {
      // Deny move requests from unfocused clients
      return;
    }

    if request.window.maximized() {
      request.window.set_maximized(false);
    }
    if request.window.fullscreen() {
      request.window.set_fullscreen(false);
    }

    self.gesture = Gesture::Move(request)
  }
  fn handle_request_resize(&mut self, request: ResizeRequest) {
    if !self.window_manager.window_has_focus(&request.window) {
      // Deny resize requests from unfocused clients
      return;
    }

    if !request.window.resizing() {
      request.window.set_resizing(true);
    }

    let original_extents = request.window.extents();
    self.gesture = Gesture::Resize(request, original_extents)
  }
  // fn handle_request_maximize(&mut self, request: MaximizeRequest) {
  //   let output = self.output_for_window(&request.window);

  //   if let Some(output) = output {
  //     if request.maximize {
  //       self.restore_size.insert(
  //         request.window.wlr_surface() as usize,
  //         request.window.extents(),
  //       );
  //       request.window.set_maximized(true);
  //       request.window.set_extents(&Rectangle {
  //         top_left: output.top_left(),
  //         size: output.size(),
  //       });
  //     } else {
  //       request.window.set_maximized(false);
  //       if let Some(extents) = self
  //         .restore_size
  //         .get(&(request.window.wlr_surface() as usize))
  //       {
  //         request.window.set_extents(extents);
  //       }
  //     }
  //   }
  // }
  // fn handle_request_fullscreen(&mut self, request: FullscreenRequest) {
  //   let output = request
  //     .output
  //     .clone()
  //     .or_else(|| self.output_for_window(&request.window));

  //   if let Some(output) = output {
  //     if request.fullscreen {
  //       self.restore_size.insert(
  //         request.window.wlr_surface() as usize,
  //         request.window.extents(),
  //       );
  //       request.window.set_fullscreen(true);
  //       request.window.set_extents(&Rectangle {
  //         top_left: output.top_left(),
  //         size: output.size(),
  //       });
  //     } else {
  //       request.window.set_fullscreen(false);
  //       if let Some(extents) = self
  //         .restore_size
  //         .get(&(request.window.wlr_surface() as usize))
  //       {
  //         request.window.set_extents(extents);
  //       }
  //     }
  //   }
  // }
}

impl EventFilter for CascadeWindowManager {
  fn handle_keyboard_event(&mut self, event: &KeyboardEvent) -> bool {
    handle_key_press(self, event)
  }
  fn handle_pointer_motion_event(&mut self, event: &MotionEvent) -> bool {
    pointer::handle_motion_event(self, event)
  }
  fn handle_pointer_button_event(&mut self, event: &ButtonEvent) -> bool {
    pointer::handle_button_event(self, event)
  }
}
