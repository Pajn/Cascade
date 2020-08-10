use std::{
  any::Any,
  cell::RefCell,
  collections::HashMap,
  hash::Hash,
  ptr,
  rc::Rc,
  time::{Duration, SystemTime},
};
use wlral::{listener, output_manager::OutputManager};

pub(crate) enum AnimationConflict {
  NoConflict,
  Replace,
  Ignore,
}

pub(crate) trait AnimationDriver {
  fn step(&self, percent: f64);
  fn started(&self) {}
  fn aborted(&self) {}
  fn completed(&self) {}
  fn is_conflict(&self, _other: &Self) -> AnimationConflict
  where
    Self: Sized,
  {
    AnimationConflict::NoConflict
  }
}

impl AnimationDriver for Box<dyn Fn(f64)> {
  fn step(&self, percent: f64) {
    self(percent)
  }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum AnimationState {
  Waiting,
  Running,
  Completed,
  Error,
}

pub(crate) struct Animation<T: ?Sized + AnimationDriver> {
  pub(crate) driver: Box<T>,
  pub(crate) delay: Duration,
  pub(crate) duration: Duration,
}

impl<T: AnimationDriver> Animation<T> {
  pub(crate) fn immediate(duration: Duration, driver: T) -> Animation<T> {
    Animation::delayed(duration, Duration::from_micros(0), driver)
  }

  pub(crate) fn delayed(duration: Duration, delay: Duration, driver: T) -> Animation<T> {
    Animation {
      driver: Box::new(driver),
      delay,
      duration,
    }
  }
}

impl<T: ?Sized + AnimationDriver> Animation<T> {
  fn frame(
    &self,
    now: SystemTime,
    start_time: SystemTime,
    last_state: AnimationState,
  ) -> AnimationState {
    let elapsed = match now.duration_since(start_time) {
      Ok(duration) => duration,
      Err(_) => {
        self.driver.aborted();
        return AnimationState::Error;
      }
    };
    if elapsed <= self.delay {
      return AnimationState::Waiting;
    }
    if last_state == AnimationState::Waiting {
      self.driver.started();
    }

    let percent = (elapsed - self.delay).as_micros() as f64 / (self.duration).as_micros() as f64;

    if percent >= 1.0 {
      self.driver.step(1.0);
      self.driver.completed();
      return AnimationState::Completed;
    }

    self.driver.step(percent);
    AnimationState::Running
  }
}

impl<T: ?Sized + AnimationDriver> PartialEq for Animation<T> {
  fn eq(&self, other: &Self) -> bool {
    ptr::eq(self.driver.as_ref(), other.driver.as_ref())
  }
}
impl<T: ?Sized + AnimationDriver> Eq for Animation<T> {}
impl<T: ?Sized + AnimationDriver> Hash for Animation<T> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    ptr::hash(self.driver.as_ref(), state);
  }
}

pub(crate) struct AnimationManager {
  running_animations:
    RefCell<HashMap<Animation<dyn AnimationDriver>, Option<(SystemTime, AnimationState)>>>,
}

impl AnimationManager {
  pub(crate) fn init(output_manager: Rc<OutputManager>) -> Rc<AnimationManager> {
    let animation_manager = Rc::new(AnimationManager {
      running_animations: RefCell::new(HashMap::new()),
    });
    output_manager
      .on_new_output()
      .subscribe(listener!(animation_manager => move |output| {
        output.on_frame().subscribe(listener!(animation_manager => move || {
          animation_manager.frame();
        }));
      }));
    animation_manager
  }

  pub(crate) fn start<T: 'static + AnimationDriver>(&self, animation: Animation<T>) {
    let mut ignore = false;
    self.running_animations.borrow_mut().retain(|old, _| {
      if let Some(old_driver) = Any::downcast_ref::<T>(&old.driver) {
        match animation.driver.is_conflict(old_driver) {
          AnimationConflict::NoConflict => true,
          AnimationConflict::Replace => {
            old_driver.aborted();
            false
          }
          AnimationConflict::Ignore => {
            ignore = true;
            true
          }
        }
      } else {
        true
      }
    });

    if !ignore {
      self.running_animations.borrow_mut().insert(
        Animation {
          driver: animation.driver as Box<dyn AnimationDriver>,
          delay: animation.delay,
          duration: animation.duration,
        },
        None,
      );
    }
  }

  fn frame(&self) {
    let now = SystemTime::now();
    self
      .running_animations
      .borrow_mut()
      .retain(|animation, animation_state| {
        let (start_time, last_state) =
          animation_state.get_or_insert((now, AnimationState::Waiting));
        let start_time = *start_time;
        let state = animation.frame(now, start_time, *last_state);
        match state {
          AnimationState::Completed | AnimationState::Error => false,
          _ => {
            animation_state.replace((start_time, state));
            true
          }
        }
      });
  }
}
