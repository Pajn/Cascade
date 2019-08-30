use crate::ffi_helpers::*;
use mir_rs::*;
use std::cmp;
use std::collections::BTreeMap;

pub type Id = u64;

#[derive(Debug)]
pub struct IdGenerator {
  next_id: Id,
}

impl IdGenerator {
  pub fn new() -> IdGenerator {
    IdGenerator { next_id: 1 }
  }

  pub fn next_id(&mut self) -> Id {
    let id = self.next_id;
    self.next_id = id + 1;
    id
  }
}

#[derive(Debug)]
pub struct Window {
  pub id: Id,
  pub workspace: Id,
  pub window_info: *mut miral::WindowInfo,
  pub x: i32,
  pub y: i32,
  pub size: Size,
  pub has_parent: bool,
}

impl Window {
  pub fn new(
    id_generator: &mut IdGenerator,
    workspace: Id,
    window_info: *mut miral::WindowInfo,
  ) -> Window {
    Window {
      id: id_generator.next_id(),
      workspace,
      window_info,
      x: 0,
      y: 0,
      size: Size {
        width: 0,
        height: 0,
      },
      has_parent: unsafe { window_info_has_parent(window_info) },
    }
  }

  pub fn x(&self) -> i32 {
    unsafe { (&*(*self.window_info).window()).top_left().x.value }
  }

  pub fn y(&self) -> i32 {
    unsafe { (*(*self.window_info).window()).top_left().y.value }
  }

  pub fn width(&self) -> i32 {
    unsafe { (*(*self.window_info).window()).size().width.value }
  }

  pub fn height(&self) -> i32 {
    unsafe { (*(*self.window_info).window()).size().height.value }
  }

  pub fn rendered_size(&self) -> Size {
    Size {
      width: self.width(),
      height: self.height(),
    }
  }

  pub fn max_height(&self) -> i32 {
    unsafe { ((*self.window_info).max_height()).value }
  }

  pub fn min_height(&self) -> i32 {
    unsafe { ((*self.window_info).min_height()).value }
  }

  pub fn max_width(&self) -> i32 {
    unsafe { ((*self.window_info).max_width()).value }
  }

  pub fn min_width(&self) -> i32 {
    unsafe { ((*self.window_info).min_width()).value }
  }

  pub fn set_size(&mut self, mut size: Size) {
    size.width = cmp::max(cmp::min(size.width, self.max_width()), self.min_width());
    size.height = cmp::max(cmp::min(size.height, self.max_height()), self.min_height());
    self.size = size;
  }

  pub fn resize(&mut self, size: Size) {
    self.set_size(size);
    let size = size.into();
    unsafe { (*(*self.window_info).window()).resize(&size) }
  }

  pub fn move_to(&mut self, x: i32, y: i32) {
    unsafe { (*(*self.window_info).window()).move_to(mir::geometry::Point::new(x, y)) }
  }

  pub fn is_tiled(&self) -> bool {
    // unsafe {
    //   println!(
    //     "is_tiled {} {}, {}, {}",
    //     self.id,
    //     (*self.window_info).type_(),
    //     (*self.window_info).state(),
    //     (*self.window_info).type_()
    //   )
    // };
    unsafe {
      !window_info_has_parent(self.window_info)
        && ((*self.window_info).type_() == raw::MirWindowType::mir_window_type_normal
          || (*self.window_info).type_() == raw::MirWindowType::mir_window_type_freestyle)
        && (*self.window_info).state() != raw::MirWindowState::mir_window_state_fullscreen
        && (*self.window_info).state() != raw::MirWindowState::mir_window_state_attached
    }
  }

  pub fn ask_client_to_close(&self, wm: &WindowManager) -> () {
    unsafe { (*wm.tools).ask_client_to_close((*self.window_info).window()) };
  }
}

#[derive(Debug)]
pub struct Workspace {
  pub id: Id,
  pub on_monitor: Option<Id>,
  pub windows: Vec<Id>,
  pub scroll_left: i32,
}

impl Workspace {
  pub fn new(id_generator: &mut IdGenerator) -> Workspace {
    Workspace {
      id: id_generator.next_id(),
      on_monitor: None,
      windows: vec![],
      scroll_left: 0,
    }
  }

  pub fn get_tiled_windows(&self, wm: &WindowManager) -> Vec<Id> {
    self
      .windows
      .iter()
      .filter(|w| wm.get_window(**w).is_tiled())
      .copied()
      .collect()
  }

  pub fn get_window_index(&self, window: Id) -> Option<usize> {
    self
      .windows
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window)
      .map(|(index, _)| index)
  }

  pub fn get_tiled_window_index(&self, wm: &WindowManager, window: Id) -> Option<usize> {
    self
      .get_tiled_windows(wm)
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window)
      .map(|(index, _)| index)
  }
}

#[derive(Debug)]
pub struct Monitor {
  pub id: Id,
  pub extents: Rectangle,
  pub workspace: Id,
}

impl Monitor {
  pub fn new(id_generator: &mut IdGenerator, extents: Rectangle, workspace: Id) -> Monitor {
    Monitor {
      id: id_generator.next_id(),
      extents,
      workspace,
    }
  }
}

#[derive(Debug)]
pub struct WindowManager {
  pub tools: *mut miral::WindowManagerTools,
  pub monitor_id_generator: IdGenerator,
  pub window_id_generator: IdGenerator,
  pub workspace_id_generator: IdGenerator,

  pub monitors: BTreeMap<Id, Monitor>,
  pub windows: BTreeMap<Id, Window>,
  pub workspaces: BTreeMap<Id, Workspace>,
  pub active_window: Option<Id>,
  pub active_workspace: Id,
}

impl WindowManager {
  pub fn get_window(&self, window_id: Id) -> &Window {
    self
      .windows
      .get(&window_id)
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

  pub fn monitor_by_workspace(&self, workspace_id: Id) -> Option<&Monitor> {
    self
      .get_workspace(workspace_id)
      .on_monitor
      .and_then(|monitor_id| self.monitors.get(&monitor_id))
  }

  pub fn monitor_by_window(&self, window_id: Id) -> Option<&Monitor> {
    let workspace_id = self.get_window(window_id).workspace;
    self.monitor_by_workspace(workspace_id)
  }

  pub fn window_by_info(&self, window_info: *const miral::WindowInfo) -> Option<&Window> {
    self
      .windows
      .values()
      .find(|w| w.window_info as *const _ == window_info)
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
        self
          .workspaces
          .insert(second_workspace.id, second_workspace);

        first_workspace_id
      }
      Some(first_workspace) => {
        let first_workspace_id = first_workspace.id;

        // We want there to always be an additional workspace avalible
        if unused_workspaces.len() == 1 {
          let aditional_workspace = Workspace::new(&mut self.workspace_id_generator);
          self
            .workspaces
            .insert(aditional_workspace.id, aditional_workspace);
        }

        first_workspace_id
      }
    }
  }

  pub fn add_window(&mut self, window: Window) -> () {
    println!("WM: {:?}, adding: {:?}", &self, &window);
    let workspace = self.workspaces.get_mut(&self.active_workspace).unwrap();

    if let Some(active_window) = self.active_window {
      let index = workspace
        .get_window_index(active_window)
        .expect("add window workspace");
      workspace.windows.insert(index + 1, window.id);
    } else {
      workspace.windows.push(window.id);
    }

    if !window.has_parent {
      self.active_window = Some(window.id);
      self.active_workspace = window.workspace;
    }

    self.windows.insert(window.id, window);
  }

  pub fn delete_window(&mut self, window_id: Id) -> () {
    let workspace = self.workspaces.get_mut(&self.active_workspace).unwrap();

    let index = workspace
      .windows
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window_id)
      .expect("nowindow in workspace advise_delete_window")
      .0;

    workspace.windows.remove(index);
    self.windows.remove(&window_id);

    if self.active_window == Some(window_id) {
      // Mir will focus a new window for us so we can just unset
      // active_window and wait for the focus event
      self.active_window = None;
    }
  }

  pub fn focus_window(&mut self, window_id: Id) -> () {
    let window = self.get_window(window_id);

    unsafe {
      let window_ptr = (*window.window_info).window();
      select_active_window(self.tools, window_ptr);
    }
  }
}
