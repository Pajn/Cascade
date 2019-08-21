use mir_rs::{
  mir_geometry_Point, mir_geometry_detail_IntWrapper, mir_keyboard_event_action,
  mir_keyboard_event_modifiers, mir_keyboard_event_scan_code, miral_WindowManagerTools,
  miral_WindowSpecification, MirInputEventModifier, MirKeyboardAction, MirKeyboardEvent,miral_Window
};

#[no_mangle]
pub extern "C" fn place_new_window(window_specification: *mut miral_WindowSpecification) -> () {
  let window_specification = unsafe { &mut *window_specification };

  unsafe { &mut *window_specification.top_left1() }
    .value_
    .x
    .value = 0;
  unsafe { &mut *window_specification.top_left1() }
    .value_
    .y
    .value = 0;
}

#[no_mangle]
pub extern "C" fn handle_keyboard_event(
  tools: *const miral_WindowManagerTools,
  window: *mut miral_Window,
  event: *const MirKeyboardEvent,
) -> bool {
  let modifier_mask = MirInputEventModifier::mir_input_event_modifier_ctrl_left
    | MirInputEventModifier::mir_input_event_modifier_alt_left;

  let action = unsafe { mir_keyboard_event_action(event) };
  let scan_code = unsafe { mir_keyboard_event_scan_code(event) };
  let modifiers = unsafe { mir_keyboard_event_modifiers(event) & modifier_mask };

  if action == MirKeyboardAction::mir_keyboard_action_down {
    println!(
      "action: {:?}, scan_code: {}, modifiers: {}",
      action, scan_code, modifiers
    );

    if scan_code == 105 && modifiers == MirInputEventModifier::mir_input_event_modifier_ctrl_left {
      println!("move right");

      let window = unsafe { &mut *window };
      let current_x = unsafe { window.top_left() }.x.value;
      unsafe {
        window.move_to(mir_geometry_Point {
          x: mir_geometry_detail_IntWrapper {
            value: current_x - 10,
          },
          y: mir_geometry_detail_IntWrapper { value: 0 },
        })
      };
      return true
    }
    if scan_code == 106 && modifiers == MirInputEventModifier::mir_input_event_modifier_ctrl_left {
      println!("move left");
      println!("tools {:?}", tools);
      let tools = unsafe { &*tools };
      println!("tools2 {:?}", tools);
      
      let count = unsafe { tools.count_applications() };
      println!("got window count {}", count);

      // for some reason does tools.active_window from Rust cause a NPE
      // in miral::WindowManagerTools::active_window()
      // let mut window = unsafe { tools.active_window() };

      let window = unsafe { &mut *window };

      println!("got window {:?}", window);
      println!("top_left {:?}", unsafe { window.top_left() });

      let current_x = unsafe { window.top_left() }.x.value;
      unsafe {
        window.move_to(mir_geometry_Point {
          x: mir_geometry_detail_IntWrapper {
            value: current_x + 10,
          },
          y: mir_geometry_detail_IntWrapper { value: 0 },
        })
      };
      return true
    }
  }

  false
}

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
