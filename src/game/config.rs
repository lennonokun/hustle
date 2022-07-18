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
  pub column_desaturate: bool,
  pub quick_guess: bool,
}

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
  desaturated: RawFbTheme,
  saturated: RawFbTheme,
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
  pub column_desaturate: bool,
  pub quick_guess: bool,
}

macro_rules! add_src {
  ($builder: expr, $($buf: expr),+) => {
    let mut pb = PathBuf::new();
    $(pb.push($buf));+;
    $builder = $builder.add_source(File::from(pb))
  };
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
        $palette.set_color($name, Color::parse(&$string)?)
      }
    }
    let mut palette = Palette::default();
    let theme = rawcfg.theme;
    set_color!(palette, "dfb_fg", theme.feedback.desaturated.fg);
    set_color!(palette, "dfb_abg", theme.feedback.desaturated.absent_bg);
    set_color!(palette, "dfb_pbg", theme.feedback.desaturated.present_bg);
    set_color!(palette, "dfb_cbg", theme.feedback.desaturated.correct_bg);
    set_color!(palette, "sfb_fg", theme.feedback.saturated.fg);
    set_color!(palette, "sfb_abg", theme.feedback.saturated.absent_bg);
    set_color!(palette, "sfb_pbg", theme.feedback.saturated.present_bg);
    set_color!(palette, "sfb_cbg", theme.feedback.saturated.correct_bg);
    set_color!(palette, "stat_imp_fg", theme.status.impossible_fg);

    Some(Config {
      palette,
      word_banks: rawcfg.word_banks,
      column_finish: rawcfg.behavior.column_finish,
      column_desaturate: rawcfg.behavior.column_desaturate,
      quick_guess: rawcfg.behavior.quick_guess,
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
