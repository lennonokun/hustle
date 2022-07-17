#![feature(struct_field_attributes)]
use lazy_static::lazy_static;
use std::fs::File;
use std::collections::HashMap;
use std::io::{self, Read};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::env;
use toml_loader::{self,Loadable};

use serde::Deserialize;
use toml;
use cursive::theme::Color;

lazy_static! {
  pub static ref CONFIG: Config = {Config::find()};
}

// TODO REMEMBER TO FLIP ORDER OF P1/P2 after migrating to macro

pub trait Loadable {
  fn load(vec: Vec<&Path>) -> Option<Self> where Self: Sized;
}

#[derive(Loadable, Debug)]
pub struct Config {
  #[color]
  pub feedback_fg: Color,
  #[color]
  pub feedback_absent_bg: Color,
  #[color]
  pub feedback_present_bg: Color,
  #[color]
  pub feedback_correct_bg: Color,
  #[color]
  pub impossible_fg: Color,
  pub word_banks: HashMap<String, String>,
}

impl Config {
  pub fn find() -> Self {
    let p1 = Path::new("/usr/share/hustle/config.toml");

    // try to do xdg merged with defaults
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
      let pb: PathBuf = [xdg, "hustle/config.toml".into()].iter().collect();
      let p2 = pb.as_path();
      if p2.exists() {return Config::load(vec![p2, p1]).unwrap()}
    }

    // try to do xdg merged with defaults
    if let Ok(hp) = env::var("HOME") {
      let pb: PathBuf = [hp, ".config/hustle/config.toml".into()].iter().collect();
      let p2 = pb.as_path();
      if p2.exists() {return Config::load(vec![p2, p1]).unwrap()}
    }

    // just use defaults
    return Config::load(vec![p1]).unwrap();
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn default_config() {
    let cfg = Config::load(vec![Path::new("/usr/share/hustle/config.toml")]);
    assert!(cfg.is_some());
  }

  #[test]
  pub fn test_config2() {
    let cfg = Config::load(
      vec![
      Path::new("/home/lokun/.config/hustle/config.toml"),
      Path::new("/usr/share/hustle/config.toml"),
      ]
    );
    println!("{:?}", cfg);
    assert!(false);
  }
}
