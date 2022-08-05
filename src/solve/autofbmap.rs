use std::collections::{hash_map, HashMap};
use std::default::Default;
use std::slice;
use std::vec;
use std::iter::{Enumerate, IntoIterator};

use crate::util::*;

// decided through benchmarks
const AFBMAP_WLEN_CUT: u8 = 7;
const AFBMAP_ALEN_CUT: usize = 25;

pub struct AutoFbMap<T> {
  wlen: u8,
  default: T,
  data: AutoFbMapData<T>,
}

enum AutoFbMapData<T> {
  Vec(Vec<T>),
  HMap(HashMap<Feedback, T>),
}

pub struct IterMut<'a, T> {
  wlen: u8,
  data: IterMutData<'a, T>,
}

enum IterMutData<'a, T> {
  Vec(Enumerate<slice::IterMut<'a, T>>),
  HMap(hash_map::IterMut<'a, Feedback, T>),
}

pub struct IntoIter<T> {
  wlen: u8,
  data: IntoIterData<T>,
}

enum IntoIterData<T> {
  Vec(Enumerate<vec::IntoIter<T>>),
  HMap(hash_map::IntoIter<Feedback, T>),
}

impl<T> AutoFbMap<T> where T: Clone {
  pub fn new(wlen: u8, alen: usize, default: T) -> Self {
    let data = if wlen <= AFBMAP_WLEN_CUT && alen >= AFBMAP_ALEN_CUT {
      AutoFbMapData::<T>::Vec(vec![default.clone(); 3usize.pow(wlen as u32)])
    } else {
      AutoFbMapData::<T>::HMap(HashMap::new())
    };
    Self { wlen, default, data }
  }

  pub fn get_mut(&mut self, gw: Word, aw: Word) -> &mut T {
    match &mut self.data {
      AutoFbMapData::<T>::Vec(vec) => {
        vec.get_mut(fb_id(gw, aw) as usize).unwrap()
      }, AutoFbMapData::<T>::HMap(map) => {
        let fb = Feedback::from(gw, aw).unwrap();
        map.entry(fb).or_insert_with(|| self.default.clone())
      }
    }
  }

  pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
    let data = match &mut self.data {
      AutoFbMapData::<T>::Vec(ref mut vec) => IterMutData::Vec(vec.iter_mut().enumerate()),
      AutoFbMapData::<T>::HMap(ref mut map) => IterMutData::HMap(map.iter_mut())
    };
    IterMut { wlen: self.wlen, data }
  }

  pub fn into_iter(self) -> IntoIter<T> {
    let data = match self.data {
      AutoFbMapData::<T>::Vec(vec) => IntoIterData::Vec(vec.into_iter().enumerate()),
      AutoFbMapData::<T>::HMap(map) => IntoIterData::HMap(map.into_iter()),
    };
    IntoIter { wlen: self.wlen, data }
  }
}

impl<'a, T> Iterator for IterMut<'a, T> {
  type Item = (Feedback, &'a mut T);
  
  fn next(&mut self) -> Option<Self::Item> {
    match &mut self.data {
      IterMutData::Vec(viter) => {
        viter.next().map(|(id, t)| (Feedback::from_id(id as u32, self.wlen), t))
      }, IterMutData::HMap(hiter) => {
        hiter.next().map(|(fb, t)| (*fb, t))
      }
    }
  }
}

// impl<'a, T> IntoIterator for &'a mut AutoFbMap<T> where T: Clone {
//   type Item = (Feedback, &'a T);
//   type IntoIter = IterMut<'a, T>;
// 
//   fn iter(self) -> Self::IntoIter {self.iter_mut()}
// }

impl<T> Iterator for IntoIter<T> {
  type Item = (Feedback, T);

  fn next(&mut self) -> Option<Self::Item> {
    match &mut self.data {
      IntoIterData::Vec(viter) => {
        viter.next().map(|(id, t)| (Feedback::from_id(id as u32, self.wlen), t))
      }, IntoIterData::HMap(hiter) => {
        hiter.next()
      }
    }
  }
}
