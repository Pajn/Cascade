use crate::ffi_helpers::*;
use crate::ipc_server::*;
use mir_rs::*;
use std::ffi::CString;

#[derive(Debug)]
pub struct Keymap {
  ipc_server: &'static IpcServer,
  keymap_ctrl: *mut miral::Keymap,
  keymap: String,
  keymap_list: Vec<String>,
}

impl Keymap {
  pub fn new(ipc_server: &'static IpcServer, keymap_ctrl: *mut miral::Keymap) -> Self {
    Keymap {
      ipc_server,
      keymap_ctrl,
      keymap: "us".to_owned(),
      keymap_list: vec!["us".to_owned(), "se".to_owned()],
    }
  }

  pub fn set_keymap<S: ToString>(&mut self, keymap: S) {
    let keymap = keymap.to_string();
    println!("set_keymap {}", &keymap);
    unsafe {
      set_keymap(
        self.keymap_ctrl,
        CString::new(keymap.as_str()).unwrap().as_ptr(),
      );
    }
    self.keymap = keymap.clone();
    self.ipc_server.send(IpcMessage::Keymap(keymap));
  }

  pub fn set_keymap_list(&mut self, keymap_list: Vec<String>) {
    self.keymap_list = keymap_list.clone();
    self.ipc_server.send(IpcMessage::KeymapList(keymap_list));
  }

  pub fn set_next_keymap(&mut self) {
    let index = self
      .keymap_list
      .iter()
      .enumerate()
      .find(|(_, l)| *l == &self.keymap)
      .map(|(i, _)| i)
      .unwrap_or(0);
    let next_index = (index + 1) % self.keymap_list.len();
    if let Some(next_keymap) = self.keymap_list.get(next_index).cloned() {
      self.set_keymap(next_keymap);
    }
  }
}
