use ipc_channel::ipc::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::mem::transmute;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

#[derive(Serialize, Deserialize)]
pub enum IpcMessage {
  Keymap(String),
  KeymapList(Vec<String>),
}

pub struct IpcServer {
  server_name: String,
  tx: Arc<RwLock<Mutex<IpcSender<IpcMessage>>>>,
}

impl IpcServer {
  pub fn init() -> Result<IpcServer, Box<dyn Error>> {
    let (server, server_name) = IpcOneShotServer::new()?;

    let (dummy_tx, _) = channel()?;
    let boxed_tx = Arc::new(RwLock::new(Mutex::new(dummy_tx)));

    thread::spawn(clone!(boxed_tx => move || {
      match server.accept() {
        Ok((_, tx)) => {
          let tx: IpcSender<IpcMessage> = tx;
          // let state = state.read().unwrap();

          // let result = tx.send(IpcMessage::Keymap((*state).keymap.clone())).and_then(|_|
          //   tx.send(IpcMessage::KeymapList((*state).keymap_list.clone()))
          // );
          // if let Err(error) = result {
          //   eprintln!("IPC error sending initial state: {}", error);
          // }

          *boxed_tx.write().unwrap().get_mut().unwrap() = tx;
          // loop {
          //   match rx.recv() {
          //     Ok(data) => {
          //       match data {
          //       }
          //     },
          //     Err(error) => {
          //       eprintln!("IPC rx error: {}", error);
          //     }
          //   }
          // }
        }
        Err(error) => {
          eprintln!("IPC error: {}", error);
        }
      }
    }));

    Ok(IpcServer {
      server_name,
      tx: boxed_tx,
    })
  }

  pub fn send(&self, message: IpcMessage) {
    let result = self.tx.read().unwrap().lock().unwrap().send(message);

    if let Err(error) = result {
      eprintln!("IPC error sending message {}", error);
    }
  }
}

impl std::fmt::Debug for IpcServer {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
    write!(fmt, "IpcServer {{ server_name: {:?} }}", self.server_name)
  }
}

#[no_mangle]
pub extern "C" fn init_ipc_server() -> *const IpcServer {
  match IpcServer::init() {
    Ok(ipc_server) => unsafe { transmute(Box::new(ipc_server)) },
    Err(error) => panic!("Error initializing IPC server {}", error),
  }
}
