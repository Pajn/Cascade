use crate::animation::*;
use crate::ffi_helpers::*;
use crate::input_inhibitor::{focus_exclusive_client, InputInhibitor};
use crate::ipc_server::*;
use crate::keymap::*;
use mir_rs::*;
use std::cmp;
use std::collections::BTreeMap;
use std::ptr;
use std::sync::{Arc, RwLock};
use std::time::Duration;

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
  pub on_workspace: Option<Id>,
  pub window_info: *mut miral::WindowInfo,
  pub x: i32,
  pub y: i32,
  pub size: Size,
  pub is_dragged: bool,
  pub old_state: Option<raw::MirWindowState::Type>,
  pub animation_state: Arc<WindowAnimaitonState>,
  pub animation_status: Arc<RwLock<AnimationStatus>>,
}

impl Window {
  pub fn new(
    id_generator: &mut IdGenerator,
    animation_state: Arc<WindowAnimaitonState>,
    window_info: *mut miral::WindowInfo,
  ) -> Window {
    Window {
      id: id_generator.next_id(),
      on_workspace: None,
      window_info,
      x: 0,
      y: 0,
      size: Size {
        width: 0,
        height: 0,
      },
      is_dragged: false,
      old_state: None,
      animation_state,
      animation_status: Arc::new(RwLock::new(AnimationStatus::Still)),
    }
  }

  pub fn name(&self) -> String {
    unsafe { window_name(self.window_info).get() }
  }

  pub fn x(&self) -> i32 {
    match *self.animation_status.read().unwrap() {
      AnimationStatus::IsAnimating(to) => to.x,
      AnimationStatus::Still => unsafe { (&*(*self.window_info).window()).top_left().x.value },
    }
  }

  pub fn y(&self) -> i32 {
    match *self.animation_status.read().unwrap() {
      AnimationStatus::IsAnimating(to) => to.y,
      AnimationStatus::Still => unsafe { (*(*self.window_info).window()).top_left().y.value },
    }
  }

  pub fn width(&self) -> i32 {
    unsafe { (*(*self.window_info).window()).size().width.value }
  }

  pub fn height(&self) -> i32 {
    unsafe { (*(*self.window_info).window()).size().height.value }
  }

  pub fn rendered_pos(&self) -> Rectangle {
    Rectangle {
      top_left: self.rendered_top_left(),
      size: self.rendered_size(),
    }
  }

  pub fn rendered_top_left(&self) -> Point {
    Point {
      x: unsafe { (&*(*self.window_info).window()).top_left().x.value },
      y: unsafe { (*(*self.window_info).window()).top_left().y.value },
    }
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

  pub fn move_to(&mut self, top_left: Point) {
    *self.animation_status.write().unwrap() = AnimationStatus::IsAnimating(top_left);
    self.animation_state.start_animation(
      MoveWindow::new(self.window_info, self.animation_status.clone())
        .animate_to(Duration::from_millis(150), top_left),
    );
  }

  pub fn set_position(&mut self, top_left: Point) {
    unsafe { (*(*self.window_info).window()).move_to(top_left.into()) }
  }

  pub fn type_(&self) -> raw::MirWindowType::Type {
    unsafe { (*self.window_info).type_() }
  }

  pub fn state(&self) -> raw::MirWindowState::Type {
    unsafe { (*self.window_info).state() }
  }

  pub fn set_state(&self, state: raw::MirWindowState::Type) {
    unsafe {
      (*self.window_info).state1(state);
      configure_window(
        self.window_info,
        raw::MirWindowAttrib::mir_window_attrib_state,
        state as i32,
      );
    }
  }

  pub fn has_parent(&self) -> bool {
    unsafe { window_info_has_parent(self.window_info) }
  }

  pub fn hide(&mut self) {
    self.old_state = Some(self.state());
    self.set_state(raw::MirWindowState::mir_window_state_hidden);
    unsafe { hide_window(self.window_info) };
  }
  pub fn show(&mut self) {
    if let Some(old_state) = self.old_state {
      self.old_state = None;
      let old_state = match old_state {
        raw::MirWindowState::mir_window_state_hidden
        | raw::MirWindowState::mir_window_state_minimized => {
          raw::MirWindowState::mir_window_state_restored
        }
        old_state => old_state,
      };
      self.set_state(old_state)
    }
    unsafe { show_window(self.window_info) };
  }

  pub fn minimize(&self) {
    unsafe { (*self.window_info).state1(raw::MirWindowState::mir_window_state_minimized) }
  }
  pub fn restore(&self) {
    unsafe { (*self.window_info).state1(raw::MirWindowState::mir_window_state_restored) }
  }

  pub fn is_tiled(&self) -> bool {
    self.name() != "Ulauncher window title"
      && !self.has_parent()
      && (self.type_() == raw::MirWindowType::mir_window_type_normal
        || self.type_() == raw::MirWindowType::mir_window_type_freestyle)
      && self.state() != raw::MirWindowState::mir_window_state_fullscreen
      && self.state() != raw::MirWindowState::mir_window_state_attached
  }

  pub fn ask_client_to_close(&self, wm: &WindowManager) -> () {
    unsafe { (*wm.tools).ask_client_to_close((*self.window_info).window()) };
  }
}

#[derive(Debug)]
pub enum AnimationStatus {
  Still,
  IsAnimating(Point),
}

#[derive(Debug)]
pub struct MoveWindow {
  window_info: *mut miral::WindowInfo,
  status: Arc<RwLock<AnimationStatus>>,
}

unsafe impl Send for MoveWindow {}
unsafe impl Sync for MoveWindow {}

impl MoveWindow {
  pub fn new(window_info: *mut miral::WindowInfo, status: Arc<RwLock<AnimationStatus>>) -> Self {
    MoveWindow {
      window_info,
      status,
    }
  }
}

impl AnimationTarget<Point> for MoveWindow {
  fn get_value(&self) -> Point {
    Point {
      x: unsafe { (&*(*self.window_info).window()).top_left().x.value },
      y: unsafe { (*(*self.window_info).window()).top_left().y.value },
    }
  }

  fn set_value(&mut self, top_left: Point) {
    unsafe { (*(*self.window_info).window()).move_to(top_left.into()) };
  }

  fn is_same(&self, other: &Self) -> bool {
    self.window_info == other.window_info
  }

  fn animation_ended(&self) {
    *self.status.write().unwrap() = AnimationStatus::Still
  }
}

pub type WindowAnimaitonState = AnimaitonState<Point, MoveWindow>;

#[derive(Debug)]
pub struct Workspace {
  pub id: Id,
  pub on_monitor: Option<Id>,
  pub scroll_left: i32,
  pub windows: Vec<Id>,
  pub active_window: Option<Id>,
}

impl Workspace {
  pub fn new(id_generator: &mut IdGenerator) -> Workspace {
    Workspace {
      id: id_generator.next_id(),
      on_monitor: None,
      scroll_left: 0,
      windows: vec![],
      active_window: None,
    }
  }

  pub fn get_window_index(&self, window: Id) -> Option<usize> {
    self
      .windows
      .iter()
      .enumerate()
      .find(|(_, w)| **w == window)
      .map(|(index, _)| index)
  }

  pub fn swap_windows(&mut self, a: Id, b: Id) {
    let a_raw_index = self.get_window_index(a).unwrap();
    let b_raw_index = self.get_window_index(b).unwrap();
    self.windows.swap(a_raw_index, b_raw_index);
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
pub struct ResizeGesture {
  pub window: Id,
  pub buttons: raw::MirPointerButtons,
  pub modifiers: input_event_modifier::Type,
  pub top_left: Point,
  pub size: Size,
  pub edge: raw::MirResizeEdge::Type,
}

#[derive(Debug)]
pub struct MoveGesture {
  pub window: Id,
  pub buttons: raw::MirPointerButtons,
  pub modifiers: input_event_modifier::Type,
  pub top_left: Point,
}

#[derive(Debug)]
pub enum Gesture {
  Resize(ResizeGesture),
  Move(MoveGesture),
  None,
}

#[derive(Debug)]
pub struct WindowManager {
  pub tools: *mut miral::WindowManagerTools,
  pub ipc_server: &'static IpcServer,
  pub keymap: Keymap,
  pub input_inhibitor: Box<InputInhibitor>,
  pub monitor_id_generator: IdGenerator,
  pub window_id_generator: IdGenerator,
  pub workspace_id_generator: IdGenerator,

  pub monitors: BTreeMap<Id, Monitor>,
  pub windows: BTreeMap<Id, Window>,
  pub workspaces: BTreeMap<Id, Workspace>,

  pub old_cursor: Point,
  pub gesture: Gesture,
  pub active_window: Option<Id>,
  pub active_workspace: Id,
  pub new_window_workspace: Id,

  pub animation_state: Arc<WindowAnimaitonState>,
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
    let workspace_id = self.get_window(window_id).on_workspace;
    workspace_id.and_then(|workspace_id| self.monitor_by_workspace(workspace_id))
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
    if let Some(workspace_id) = window.on_workspace {
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
      if self.input_inhibitor.is_allowed(&window) {
        self.activate_window(window_id);
      } else {
        focus_exclusive_client(self);
      }
    }
  }

  pub fn delete_window(&mut self, window_id: Id) -> () {
    self.input_inhibitor.clear_if_dead();

    // Ignore the error as it's normal that windows are not in any workspace
    let _ = self.remove_window_from_workspace(window_id);
    self.windows.remove(&window_id);

    if self.active_window == Some(window_id) {
      // Mir will focus a new window for us so we can just unset
      // active_window and wait for the focus event
      self.active_window = None;
    }
  }

  pub fn activate_window(&mut self, window_id: Id) -> () {
    if let Some(workspace_id) = self.get_window(window_id).on_workspace {
      let workspace = self.workspaces.get_mut(&workspace_id).unwrap();

      if workspace.on_monitor.is_some() {
        workspace.active_window = Some(window_id);
        self.active_workspace = workspace_id;
        self.new_window_workspace = workspace_id;
      }
    }
    self.active_window = Some(window_id);
  }

  pub fn remove_window_from_workspace(&mut self, window: Id) -> Result<(), ()> {
    let workspace = self.get_workspace(self.get_window(window).on_workspace.ok_or(())?);
    let workspace_id = workspace.id;
    if workspace.active_window == Some(window) {
      let active_window = self.get_window(workspace.active_window.unwrap());
      if active_window.is_tiled() {
        let window_index = workspace.get_window_index(window).ok_or(())?;
        let window_index = if window_index > 0 {
          window_index - 1
        } else {
          window_index + 1
        };
        let next_active_window = workspace.windows.get(window_index).copied();
        let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
        workspace.active_window = next_active_window;
      } else {
        let next_active_window = workspace.windows.last().copied();
        let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
        workspace.active_window = next_active_window;
      }
    }
    let workspace = self.workspaces.get_mut(&workspace_id).unwrap();
    let raw_index = workspace.get_window_index(window).ok_or(())?;
    workspace.windows.remove(raw_index);
    Ok(())
  }

  pub fn focus_window(&mut self, window_id: Option<Id>) -> () {
    self.active_window = window_id;
    if let Some(window_id) = window_id {
      let window = self.get_window(window_id);

      if self.input_inhibitor.is_allowed(window) {
        unsafe {
          let window_ptr = (*window.window_info).window();
          select_active_window(self.tools, window_ptr);
        }
      } else {
        focus_exclusive_client(self);
      }
    } else {
      unsafe {
        select_active_window(self.tools, ptr::null());
      }
    }
  }
}
