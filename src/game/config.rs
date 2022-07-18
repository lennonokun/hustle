use config::{Config as ConfigMod, File};
use lazy_static::lazy_static;
use std::io::{self, Read};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::env;
use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;
use cursive::theme::{Color, Palette};

lazy_static! {
  // find config on start up
  pub static ref CONFIG: Config = {Config::find().unwrap()};
}

pub struct Config {
  pub palette: Palette,
  pub word_banks: IndexMap<String, String>,
  pub column_finish: String,
}

// loader config
#[derive(Debug, Deserialize)]
struct RawConfig {
  pub theme: RawTheme,
  pub word_banks: IndexMap<String, String>,
  pub behavior: RawBehavior,
}

#[derive(Debug, Deserialize)]
struct RawTheme {
  status: RawStatusTheme,
  feedback: RawFbThemes,
}

#[derive(Debug, Deserialize)]
struct RawStatusTheme {
  impossible_fg: String
}

#[derive(Debug, Deserialize)]
struct RawFbThemes {
  unsolved: RawFbTheme,
  solved: RawFbTheme,
}

#[derive(Debug, Deserialize)]
struct RawFbTheme {
  fg: String,
  absent_bg: String,
  present_bg: String,
  correct_bg: String,
}

#[derive(Debug, Deserialize)]
struct RawBehavior {
  pub column_finish: String,
}

macro_rules! add_src {
  ($builder: expr, $($buf: expr),+) => {
    let mut pb = PathBuf::new();
    $(pb.push($buf));+;
    $builder = $builder.add_source(File::from(pb))
  };
}


fn set_color(palette: &mut Palette, name: &str, string: &String) {
}

impl Config {
  pub fn find() -> Option<Self> {
    // default < home < xdg
    let mut builder = ConfigMod::builder();
    add_src!(builder, "/usr/share/hustle/config.toml");
    if let Ok(homep) = env::var("HOME") {
      add_src!(builder, homep, ".config/hustle/config.toml");
    } if let Ok(xdgp) = env::var("XDG_CONFIG_HOME") {
      add_src!(builder, xdgp, "hustle/config.toml");
    }

    let rawcfg: RawConfig = builder
      .build().expect("couldn't build config")
      .try_deserialize().ok()?;

    Self::process(rawcfg)
  }

  fn process(rawcfg: RawConfig) -> Option<Self> {
    macro_rules! set_color {
      ($palette: expr, $name: expr, $string: expr) => {
        $palette.set_color($name, Color::parse($string)?)
      }
    }
    // todo add namespaces
    let mut palette = Palette::default();
    let theme = rawcfg.theme;
    set_color!(palette, "ufb_fg", &theme.feedback.unsolved.fg);
    set_color!(palette, "ufb_abg", &theme.feedback.unsolved.absent_bg);
    set_color!(palette, "ufb_pbg", &theme.feedback.unsolved.present_bg);
    set_color!(palette, "ufb_cbg", &theme.feedback.unsolved.correct_bg);
    set_color!(palette, "sfb_fg", &theme.feedback.solved.fg);
    set_color!(palette, "sfb_abg", &theme.feedback.solved.absent_bg);
    set_color!(palette, "sfb_pbg", &theme.feedback.solved.present_bg);
    set_color!(palette, "sfb_cbg", &theme.feedback.solved.correct_bg);
    set_color!(palette, "stat_imp_fg", &theme.status.impossible_fg);

    Some(Config {
      palette,
      word_banks: rawcfg.word_banks,
      column_finish: rawcfg.behavior.column_finish,
    })
  }

  pub fn color(&self, name: &str) -> Color {
    *self.palette.custom(name).unwrap()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn find_some() {
    assert!(Config::find().is_some());
  }
}
