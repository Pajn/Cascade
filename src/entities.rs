pub(crate) mod mru_list;
pub(crate) mod workspace;

pub(crate) use mru_list::MruList;
use std::rc::Rc;
use wlral::geometry::*;
use wlral::window::Window;
use wlral::window_management_policy::*;
pub(crate) use workspace::Workspace;

pub(crate) enum Gesture {
  Move(MoveRequest),
  Resize(ResizeRequest, Rectangle),
  None,
}

impl Gesture {
  pub(crate) fn window(&self) -> Option<Rc<Window>> {
    match self {
      Gesture::Move(request) => Some(request.window.clone()),
      Gesture::Resize(request, _) => Some(request.window.clone()),
      _ => None,
    }
  }
}
