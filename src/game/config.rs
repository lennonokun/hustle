use config::{Config as ConfigMod, File};
use lazy_static::lazy_static;
use std::io::{self, Read};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::env;
use std::default::Default;

use indexmap::IndexMap;
use serde::Deserialize;
use cursive::theme::Color;

lazy_static! {
  // find config on start up
  pub static ref CONFIG: Config = {Config::find().unwrap()};
}

pub struct Config {
  // pub feedback_theme: FeedbackTheme,
  pub feedback_fg: Color,
  pub feedback_absent_bg: Color,
  pub feedback_present_bg: Color,
  pub feedback_correct_bg: Color,
  pub impossible_fg: Color,
  pub word_banks: IndexMap<String, String>,
  pub column_finish: String,
}

// loader config
#[derive(Deserialize)]
struct RawConfig {
  // pub feedback_theme: Option<LFeedbackTheme>,
  pub feedback_fg: String,
  pub feedback_absent_bg: String,
  pub feedback_present_bg: String,
  pub feedback_correct_bg: String,
  pub impossible_fg: String,
  pub word_banks: IndexMap<String, String>,
  pub column_finish: String,
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

    Some(Config {
      feedback_fg: Color::parse(&rawcfg.feedback_fg)?,
      feedback_absent_bg: Color::parse(&rawcfg.feedback_absent_bg)?,
      feedback_present_bg: Color::parse(&rawcfg.feedback_present_bg)?,
      feedback_correct_bg: Color::parse(&rawcfg.feedback_correct_bg)?,
      impossible_fg: Color::parse(&rawcfg.impossible_fg)?,
      word_banks: rawcfg.word_banks,
      column_finish: rawcfg.column_finish,
    })
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
