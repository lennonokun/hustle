use core::str::FromStr;
use core::fmt::{self, Display};
use std::io::{Error, ErrorKind};

use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::Distribution;
use rand::distributions::uniform::{Uniform, SampleUniform};
use rayon::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;

pub struct Range<X> where X: Copy + SampleUniform {
  /// lower bound
  pub a: X,
  /// upper bound
  pub b: X,
  /// inclusive flag
  pub inc: bool,
  /// distribution
  pub dist: Uniform<X>,
}

impl<X> Range<X> where X: Copy + SampleUniform {
  pub fn new(a: X, b: X, inc: bool) -> Self {
    let dist = if inc {
      Uniform::new_inclusive(a, b)
    } else {
      Uniform::new(a, b)
    };
    Self {a, b, inc, dist}
  }

  pub fn sample(&self, rng: &mut ThreadRng) -> X {
    self.dist.sample(rng)
  }
}

impl<X> FromStr for Range<X>
where X: Copy + FromStr + SampleUniform {
  type Err = Error;
  
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    lazy_static! {
      static ref RE_UNIF: Regex = Regex::new(r"^(\d+)..(=?)(\d+)$").unwrap();
    }

    if let Some(caps) = RE_UNIF.captures(s) {
      // FOR NOW UNWRAP
      let a: X = caps.get(1).unwrap().as_str().parse().ok().unwrap();
      let b: X = caps.get(3).unwrap().as_str().parse().ok().unwrap();
      let inc = !caps.get(2).unwrap().as_str().is_empty();
      Ok(Self::new(a, b, inc))
    } else {
      Err(Error::new(
        ErrorKind::Other,
        "metadata does not match!"
      ))
    }
  }
}

impl<X> Display for Range<X>
where X: Copy + Display + SampleUniform {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.inc {
      write!(f, "{}..={}", self.a, self.b)
    } else {
      write!(f, "{}..{}", self.a, self.b)
    }
  }
}
