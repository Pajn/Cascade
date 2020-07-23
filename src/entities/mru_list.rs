#[derive(Debug, Default)]
pub struct MruList<T: PartialEq> {
  items: Vec<T>,
}

impl<T: PartialEq> MruList<T> {
  pub fn new() -> MruList<T> {
    MruList { items: vec![] }
  }

  pub fn iter(&self) -> impl DoubleEndedIterator<Item = &T> {
    self.items.iter().rev()
  }

  pub fn len(&self) -> usize {
    self.items.len()
  }

  pub fn top(&self) -> Option<&T> {
    self.items.last()
  }

  pub fn push(&mut self, item: T) {
    self.remove(&item);
    self.items.push(item);
  }
  pub fn push_bottom(&mut self, item: T) {
    self.remove(&item);
    self.items.insert(0, item);
  }

  pub fn promote(&mut self, item: &T) {
    if let Some(item) = self.remove(&item) {
      self.items.push(item);
    } else {
      panic!("Promoted non existing item")
    }
  }

  pub fn remove(&mut self, item: &T) -> Option<T> {
    self
      .items
      .iter()
      .position(|x| *x == *item)
      .map(|index| self.items.remove(index))
  }
}
