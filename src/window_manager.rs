use crate::config::Config;
use crate::{
  actions::{arrange_windows_all_workspaces, arrange_windows_workspace},
  animation::AnimationManager,
  entities::{
    workspace::{Workspace, WorkspacePosition},
    Gesture, MruList,
  },
  keyboard::handle_key_press,
  pointer,
};
use log::warn;
use std::{
  cell::{Ref, RefCell},
  cmp,
  collections::BTreeMap,
  rc::Rc,
};
use wlral::{
  compositor::Compositor,
  config::ConfigManager,
  input::{
    event_filter::EventFilter,
    events::{ButtonEvent, KeyboardEvent, MotionEvent},
  },
  output::Output,
  output_manager::OutputManager,
  window::Window,
  window_management_policy::{MoveRequest, ResizeRequest, WindowManagementPolicy},
  window_manager::WindowManager,
};

pub(crate) struct CascadeWindowManager {
  pub(crate) config: Config,
  pub(crate) config_manager: Rc<ConfigManager>,
  pub(crate) output_manager: Rc<OutputManager>,
  pub(crate) window_manager: Rc<WindowManager>,
  pub(crate) animation_manager: Rc<AnimationManager>,
  mru_windows: RefCell<MruList<Rc<Window>>>,
  mru_workspaces: RefCell<MruList<Rc<Workspace>>>,
  pub(crate) output_workspaces: RefCell<BTreeMap<Rc<Output>, Rc<Workspace>>>,

  pub(crate) gesture: RefCell<Gesture>,
}

impl CascadeWindowManager {
  pub(crate) fn init(config: Config, compositor: &Compositor) -> CascadeWindowManager {
    let animation_manager = AnimationManager::init(compositor.output_manager());
    CascadeWindowManager {
      config,
      config_manager: compositor.config_manager(),
      output_manager: compositor.output_manager(),
      window_manager: compositor.window_manager(),
      animation_manager,
      mru_windows: RefCell::new(MruList::new()),
      mru_workspaces: RefCell::new(MruList::new()),
      output_workspaces: RefCell::new(BTreeMap::new()),

      gesture: RefCell::new(Gesture::None),
    }
  }

  pub(crate) fn focus_workspace(&self, workspace: &Rc<Workspace>) {
    if self.output_by_workspace(workspace).is_none() {
      let output = self
        .mru_workspaces()
        .top()
        .and_then(|active_workspace| self.output_by_workspace(active_workspace));
      if let Some(output) = output {
        self
          .output_workspaces
          .borrow_mut()
          .insert(output, workspace.clone());
      } else {
        warn!("Focusing workspace not on any monitor");
      }
    }
    self.mru_workspaces.borrow_mut().promote(workspace);
    let top_window = workspace.mru_windows().top().cloned();
    if let Some(window) = top_window {
      self.window_manager.focus_window(window);
    } else {
      self.window_manager.blur();
    }
  }

  pub(crate) fn active_window(&self) -> Option<Rc<Window>> {
    self.mru_windows.borrow().top().cloned()
  }
  pub(crate) fn mru_workspaces(&self) -> Ref<MruList<Rc<Workspace>>> {
    self.mru_workspaces.borrow()
  }
  pub(crate) fn workspace_by_window(&self, window: &Window) -> Option<Rc<Workspace>> {
    self
      .mru_workspaces()
      .iter()
      .find(|w| w.has_window(&window))
      .cloned()
  }
  pub(crate) fn output_by_window(&self, window: &Window) -> Option<Rc<Output>> {
    self
      .workspace_by_window(window)
      .and_then(|workspace| self.output_by_workspace(&workspace))
  }
  pub(crate) fn output_by_workspace(&self, workspace: &Workspace) -> Option<Rc<Output>> {
    self
      .output_workspaces
      .borrow()
      .iter()
      .find_map(|(output, w)| {
        if w.as_ref() == workspace {
          Some(output.clone())
        } else {
          None
        }
      })
  }
}

impl WindowManagementPolicy for CascadeWindowManager {
  fn handle_window_ready(&self, window: Rc<Window>) {
    if window.can_receive_focus() {
      self.mru_windows.borrow_mut().push(window.clone());

      let active_workspace = self
        .mru_workspaces()
        .top()
        .cloned()
        .expect("There should be at least one workspace");
      active_workspace.add_window(window.clone(), WorkspacePosition::ActiveWindow);

      self.window_manager.focus_window(window);
    }
  }
  fn advise_configured_window(&self, window: Rc<Window>) {
    let workspace = self.workspace_by_window(&window);
    if let Some(workspace) = workspace {
      arrange_windows_workspace(self, workspace);
    }
  }
  fn advise_focused_window(&self, window: Rc<Window>) {
    self.mru_windows.borrow_mut().promote(&window);
    let workspace = self.workspace_by_window(&window);
    if let Some(workspace) = workspace {
      workspace.promote_window(&window);
      self.focus_workspace(&workspace);
      arrange_windows_workspace(self, workspace.clone());
    }
  }
  fn advise_delete_window(&self, window: Rc<Window>) {
    self.mru_windows.borrow_mut().remove(&window);

    let workspace = self
      .mru_workspaces()
      .iter()
      .find(|w| w.has_window(&window))
      .cloned();
    if let Some(workspace) = workspace {
      workspace.remove_window(&window);
    }

    let next_window = self
      .mru_workspaces()
      .iter()
      .filter_map(|w| w.mru_windows().top().cloned())
      .next();
    if let Some(window) = next_window {
      self.window_manager.focus_window(window)
    }
  }

  fn advise_output_create(&self, output: Rc<Output>) {
    let mut mru_workspaces = self.mru_workspaces.borrow_mut();
    let expected_extra_workspaces = cmp::max(self.config.extra_workspaces, 1);
    while mru_workspaces.len() - self.output_workspaces.borrow().len() <= expected_extra_workspaces
    {
      mru_workspaces.push_bottom(Rc::new(Workspace::new()));
    }
    let first_unused_workspace = mru_workspaces
      .iter()
      .find(|w| self.output_by_workspace(w).is_none())
      .cloned()
      .expect("There should be at least one unused workspace");

    self
      .output_workspaces
      .borrow_mut()
      .insert(output, first_unused_workspace.clone());
    arrange_windows_workspace(self, first_unused_workspace);
  }
  fn advise_output_update(&self, output: Rc<Output>) {
    let workspace = self
      .output_workspaces
      .borrow()
      .get(&output)
      .cloned()
      .expect("Output should have an assigned workspace");
    arrange_windows_workspace(self, workspace);
  }
  fn advise_output_delete(&self, output: Rc<Output>) {
    let mru_workspaces = self.mru_workspaces();
    let mru_outputs = mru_workspaces
      .iter()
      .filter_map(|w| self.output_by_workspace(w))
      .filter(|o| *o != output);

    let output_workspaces = mru_outputs
      .zip(mru_workspaces.iter().cloned())
      .collect::<BTreeMap<_, _>>();
    *self.output_workspaces.borrow_mut() = output_workspaces;
    arrange_windows_all_workspaces(self);
  }

  fn handle_request_move(&self, request: MoveRequest) {
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

    *self.gesture.borrow_mut() = Gesture::Move(request)
  }
  fn handle_request_resize(&self, request: ResizeRequest) {
    if !self.window_manager.window_has_focus(&request.window) {
      // Deny resize requests from unfocused clients
      return;
    }

    if !request.window.resizing() {
      request.window.set_resizing(true);
    }

    let original_extents = request.window.extents();
    *self.gesture.borrow_mut() = Gesture::Resize(request, original_extents)
  }
}

impl EventFilter for CascadeWindowManager {
  fn handle_keyboard_event(&self, event: &KeyboardEvent) -> bool {
    handle_key_press(self, event)
  }
  fn handle_pointer_motion_event(&self, event: &MotionEvent) -> bool {
    pointer::handle_motion_event(self, event)
  }
  fn handle_pointer_button_event(&self, event: &ButtonEvent) -> bool {
    pointer::handle_button_event(self, event)
  }
}
