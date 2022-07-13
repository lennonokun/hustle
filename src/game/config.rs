use std::fs::File;
use std::io::{self, Read};
use std::error::Error;
use std::path::Path;

use termion::color::Rgb;
use serde::Deserialize;
use toml;

#[derive(Deserialize)]
struct RawConfig {
  feedback_colors: [u32; 3],
  impossible_colors: u32,
  finished: String,
}

#[derive(Debug)]
pub struct Config {
  pub fbcolors: [Rgb; 3],
  pub impcolor: Rgb,
  pub finished: String,
}

pub fn parse_rgb(x: u32) -> Rgb {
  let r = ((x & 0xff0000) >> 16) as u8;
  let g = ((x & 0x00ff00) >> 8) as u8;
  let b = ((x & 0x0000ff) >> 0) as u8;
  Rgb(r, g, b)
}

impl Config {
  pub fn load(p: &Path) -> io::Result<Self> {
    let mut f = File::open(p)?;
    let mut s = "".to_string();
    f.read_to_string(&mut s);
    let rawcfg: RawConfig = toml::from_str(&s)?;
    let fbcolors = rawcfg.feedback_colors
      .iter().map(|x| parse_rgb(*x))
      .collect::<Vec<Rgb>>()
      .try_into().unwrap();
    let impcolor = parse_rgb(rawcfg.impossible_colors);
    
    Ok(Config {fbcolors, impcolor, finished: rawcfg.finished})
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn default_config() {
    let cfg = Config::load(Path::new("data/config.toml"));
    assert!(cfg.is_ok());
    println!("{:?}", cfg);
    assert!(false);
  }
}
