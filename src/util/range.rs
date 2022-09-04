use core::str::FromStr;
use std::fmt;
use std::io::{Error, ErrorKind};

use rand::prelude::*;
use rand::Rng;
use rand::distributions::Distribution;
use rand::distributions::uniform::{Uniform, SampleUniform};

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

  pub fn sample<R: Rng>(&self, rng: &mut R) -> X {
    self.dist.sample(rng)
  }
}

impl<X> FromStr for Range<X>
where X: Copy + FromStr + SampleUniform {
  type Err = Error;
  
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Ok(x) = s.parse::<X>() {
      // parse singular (a)
      Ok(Self::new(x, x, true))
    } else {
      // parse multiple (a..=?b)
      let vec = s.split("..").collect::<Vec<&str>>();
      let range = match &vec[..] {
        [a_str, b_str] => {
          // check if inclusive
          let stripped = b_str.strip_prefix("=");
          let inc = stripped.is_some();
          let b_str = stripped.unwrap_or(b_str);

          // parse a and b, and create range
          let a = a_str.parse::<X>().ok();
          let b = b_str.parse::<X>().ok();
          a.zip(b).map(|(a, b)| Self::new(a, b, inc))
        }, _ => {
          None
        }
      };
      
      // return or error
      range.ok_or(Error::new(
        ErrorKind::Other,
        "could not parse range"
      ))
    }
  }
}

impl<X> fmt::Display for Range<X>
where X: Copy + PartialEq + fmt::Display + SampleUniform {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.inc && self.a == self.b {
      write!(f, "{}", self.a)
    } else if self.inc {
      write!(f, "{}..={}", self.a, self.b)
    } else {
      write!(f, "{}..{}", self.a, self.b)
    }
  }
}
