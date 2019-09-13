use mir_rs::*;
use std::any::Any;
use std::sync::{Arc, RwLock};
use std::time::Duration;

pub trait AnimatableValue: Copy {
  fn intermediate_value(step: f64, from: &Self, to: &Self) -> Self;
}

impl AnimatableValue for i32 {
  fn intermediate_value(step: f64, from: &Self, to: &Self) -> Self {
    ((to - from) as f64 * step + (*from as f64)) as Self
  }
}

impl AnimatableValue for Point {
  fn intermediate_value(step: f64, from: &Self, to: &Self) -> Self {
    Point {
      x: i32::intermediate_value(step, &from.x, &to.x),
      y: i32::intermediate_value(step, &from.y, &to.y),
    }
  }
}

pub trait AnimationTarget<T: AnimatableValue> {
  fn get_value(&self) -> T;
  fn set_value(&mut self, value: T) -> ();
  fn is_same(&self, _other: &Self) -> bool
  where
    Self: Sized,
  {
    false
  }

  fn animation_ended(&self) {}

  fn animate_to(self, duration: Duration, to: T) -> Animation<T, Self>
  where
    Self: 'static + Sized,
  {
    Animation {
      duration,
      step: 0.0,
      from: self.get_value(),
      to,
      target: Box::new(self),
    }
  }
}

#[derive(Debug)]
pub struct Animation<T: AnimatableValue + Sized, A: AnimationTarget<T>> {
  duration: Duration,
  step: f64,
  from: T,
  to: T,
  target: Box<A>,
}

impl<T: AnimatableValue, A: AnimationTarget<T>> Animation<T, A> {
  fn step(&mut self, elapsed_time: Duration) {
    let elapsed_step = elapsed_time.as_millis() as f64 / self.duration.as_millis() as f64;
    self.step = (self.step + elapsed_step).min(1.0);
    let value = T::intermediate_value(self.step, &self.from, &self.to);
    self.target.set_value(value);
  }
}

#[derive(Debug)]
pub struct AnimaitonState<T: AnimatableValue, A: AnimationTarget<T>> {
  running_animations: Arc<RwLock<Vec<Box<Animation<T, A>>>>>,
}

impl<T: AnimatableValue, A: 'static + AnimationTarget<T>> AnimaitonState<T, A> {
  pub fn new() -> Self {
    AnimaitonState {
      running_animations: Arc::new(RwLock::new(vec![])),
    }
  }

  pub fn step(&self, elapsed_time: Duration) {
    for animation in self.running_animations.write().unwrap().iter_mut() {
      animation.step(elapsed_time);
    }
    self
      .running_animations
      .write()
      .unwrap()
      .retain(|animation| {
        let retain = animation.step < 1.0;

        if !retain {
          animation.target.animation_ended();
        }

        retain
      });
  }

  pub fn start_animation(&self, animation: Animation<T, A>) {
    // TODO: Maybe set elapsed time when having old animation
    self.running_animations.write().unwrap().retain(|old| {
      if let Some(old_target) = Any::downcast_ref::<A>(&old.target) {
        !animation.target.is_same(old_target)
      } else {
        true
      }
    });
    self
      .running_animations
      .write()
      .unwrap()
      .push(Box::new(animation));
  }
}
