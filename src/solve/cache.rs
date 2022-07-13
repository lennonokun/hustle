use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};

use super::state::State;
use crate::ds::*;

// TODO: cache tests don't need to actually solve

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Cache {
  n: usize, // number of sets
  m: usize, // max set length
  table: Vec<VecDeque<Entry>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Entry {
  state: State,
  dt: DTree,
}

impl Cache {
  pub fn new(n: usize, m: usize) -> Self {
    let mut table = Vec::with_capacity(n);
    for _i in 0..n {
      table.push(VecDeque::new())
    }
    Self { n, m, table }
  }

  pub fn get_row(&mut self, state: &State) -> Option<&mut VecDeque<Entry>> {
    let mut h = DefaultHasher::new();
    state.hash(&mut h);
    let hash = h.finish();
    let idx = hash & (self.n as u64 - 1);
    self.table.get_mut(idx as usize)
  }

  pub fn read(&mut self, state: &State) -> Option<&DTree> {
    let row = self.get_row(state).unwrap();
    for (i, ent) in row.iter().enumerate() {
      if ent.check(state) {
        // promote to front
        let ent = row.remove(i).unwrap();
        row.push_front(ent);
        return Some(&row.front().unwrap().dt);
      }
    }
    None
  }

  // assumes state not already in table
  pub fn add(&mut self, state: State, dt: DTree) {
    let m = self.m; // hacky way to avoid double borrow
    let row = self.get_row(&state).unwrap();
    row.push_front(Entry { state, dt });
    if row.len() > m {
      row.pop_back();
    }
  }
}

impl Entry {
  // check if equal
  pub fn check(&self, state: &State) -> bool {
    self.state.n == state.n && self.state.aws == state.aws
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::solve::{SData, State};

  fn add_garbage<'a>(n: usize, cache: &mut Cache, sd: &mut SData) {
    let mut i = 0;
    while i < n {
      let state = State::random(20);
      if let Some(dt) = state.solve(sd, u32::MAX) {
        cache.add(state.clone(), dt.clone());
        i += 1;
      }
    }
  }

  #[test]
  fn add_read() {
    let mut sd = SData::new2(2);
    let state = State::random(20);
    let dt = state.solve(&mut sd, u32::MAX).unwrap();

    // fully associative 5-way cache
    let mut cache = Cache::new(1, 5);
    assert!(cache.read(&state).is_none());
    cache.add(state.clone(), dt.clone());
    assert_eq!(cache.read(&state).unwrap().clone(), dt);

    // add less than m garbage, state should still be there
    add_garbage(4, &mut cache, &mut sd);
    assert_eq!(cache.read(&state).unwrap().clone(), dt);

    // original state brought back to top
    // after m garbage, it should be gone
    add_garbage(5, &mut cache, &mut sd);
    println!("row size: {}", cache.table.get(0).unwrap().len());
    assert!(cache.read(&state).is_none());
  }
}
