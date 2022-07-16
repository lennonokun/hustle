#![feature(struct_field_attributes)]

use lazy_static::lazy_static;
use std::fs::File;
use std::collections::HashMap;
use std::io::{self, Read};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::env;

use termion::color::Rgb;
use serde::Deserialize;
use toml;

lazy_static! {
  pub static ref CONFIG: Config = {Config::load()};
}

macro_rules! config {
  ($($id1: ident, $ty1: ty, $id2: ident, $ty2: ty, $f: expr);*$(;)?) => {
    #[derive(Deserialize, Debug)]
    struct ConfigTomlLoader { $($id1: Option<$ty1>),* }

    impl ConfigTomlLoader {
      pub fn load(p: &Path) -> io::Result<Self> {
        let mut f = File::open(p)?;
        let mut s = String::new();
        f.read_to_string(&mut s);
        // TODO bad
        let out: Self = toml::from_str(&s).unwrap();
        Ok(out)
      }
    }

    #[derive(Debug, Clone)]
    pub struct Config { $(pub $id2: $ty2),* }

    impl Config {
      /// unwrap one
      pub fn from1(p: &Path) -> io::Result<Self> {
        let ct = ConfigTomlLoader::load(p)?;
        Ok(Self { $($id2: $f(ct.$id1.unwrap())),* })
      }

      /// unwrap left merge
      pub fn from2(p1: &Path, p2: &Path) -> io::Result<Self> {
        let ct1 = ConfigTomlLoader::load(p1)?;
        let ct2 = ConfigTomlLoader::load(p2)?;
        Ok(Self { $($id2: $f(ct2.$id1.or(ct1.$id1).unwrap())),* })
      }
    }
  }
}

pub fn parse_rgb(x: u32) -> Rgb {
  let r = ((x & 0xff0000) >> 16) as u8;
  let g = ((x & 0x00ff00) >> 8) as u8;
  let b = ((x & 0x0000ff) >> 0) as u8;
  Rgb(r, g, b)
}

config! {
  feedback_fg, u32, fb_fg, Rgb, parse_rgb;
  feedback_bgs, [u32; 3], fb_bgs, [Rgb; 3], |xs: [u32; 3]| {
    xs.iter().map(|x| parse_rgb(*x)).collect::<Vec<Rgb>>()
      .try_into().unwrap()
  };
  impossible_fg, u32, imp_fg, Rgb, parse_rgb;
  finished, String, finished, String, |s| s;
  word_banks, HashMap<String, String>, wbps, HashMap<String, String>, |m| m;
}

impl Config {
  pub fn load() -> Self {
    let p1 = Path::new("/usr/share/hustle/config.toml");

    // try to do xdg merged with defaults
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
      let pb: PathBuf = [xdg, "hustle/config.toml".into()].iter().collect();
      let p2 = pb.as_path();
      if p2.exists() {return Config::from2(p1, p2).unwrap()}
    }

    // try to do xdg merged with defaults
    if let Ok(hp) = env::var("HOME") {
      let pb: PathBuf = [hp, ".config/hustle/config.toml".into()].iter().collect();
      let p2 = pb.as_path();
      if p2.exists() {return Config::from2(p1, p2).unwrap()}
    }

    // just use defaults
    return Config::from1(p1).unwrap();
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn default_config() {
    let cfg = Config::from1(Path::new("/usr/share/hustle/config.toml"));
    assert!(cfg.is_ok());
  }
}
