use config::{Config as ConfigMod, File};
use lazy_static::lazy_static;
use std::io::{self, Read};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::env;
use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;
use cursive::theme::{Color, Palette, Theme, BorderStyle};

lazy_static! {
  // find config on start up
  pub static ref CONFIG: Config = {Config::find().unwrap()};
}

pub struct Config {
  pub theme: Theme,
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
  view: RawViewTheme,
  status: RawStatusTheme,
  feedback: RawFbThemes,
}

#[derive(Debug, Deserialize)]
struct RawViewTheme {
  pub do_shadow: bool,
  pub border_style: String,
  pub background: String,
  pub shadow: String,
  pub view: String,
  pub primary: String,
  pub secondary: String,
  pub tertiary: String,
  pub title_primary: String,
  pub title_secondary: String,
  pub highlight: String,
  pub highlight_text: String,
  pub highlight_inactive: String,
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
    let rawtheme = rawcfg.theme;
    set_color!(palette, "Background", rawtheme.view.background);
    set_color!(palette, "Shadow", rawtheme.view.shadow);
    set_color!(palette, "View", rawtheme.view.view);
    set_color!(palette, "Primary", rawtheme.view.primary);
    set_color!(palette, "Secondary", rawtheme.view.secondary);
    set_color!(palette, "Tertiary", rawtheme.view.tertiary);
    set_color!(palette, "TitlePrimary", rawtheme.view.title_primary);
    set_color!(palette, "TitleSecondary", rawtheme.view.title_secondary);
    set_color!(palette, "Highlight", rawtheme.view.highlight);
    set_color!(palette, "HighlightInactive", rawtheme.view.highlight_inactive);
    set_color!(palette, "HighlightText", rawtheme.view.highlight_text);
    set_color!(palette, "dfb_fg", rawtheme.feedback.desaturated.fg);
    set_color!(palette, "dfb_abg", rawtheme.feedback.desaturated.absent_bg);
    set_color!(palette, "dfb_pbg", rawtheme.feedback.desaturated.present_bg);
    set_color!(palette, "dfb_cbg", rawtheme.feedback.desaturated.correct_bg);
    set_color!(palette, "sfb_fg", rawtheme.feedback.saturated.fg);
    set_color!(palette, "sfb_abg", rawtheme.feedback.saturated.absent_bg);
    set_color!(palette, "sfb_pbg", rawtheme.feedback.saturated.present_bg);
    set_color!(palette, "sfb_cbg", rawtheme.feedback.saturated.correct_bg);
    set_color!(palette, "stat_imp_fg", rawtheme.status.impossible_fg);

    let borders = if rawtheme.view.border_style == "simple" {
      BorderStyle::Simple
    } else if rawtheme.view.border_style == "outset" {
      BorderStyle::Outset
    } else {
      BorderStyle::None
    };

    let theme = Theme {
      shadow: rawtheme.view.do_shadow,
      borders,
      palette,
    };

    Some(Config {
      theme,
      word_banks: rawcfg.word_banks,
      column_finish: rawcfg.behavior.column_finish,
      column_desaturate: rawcfg.behavior.column_desaturate,
      quick_guess: rawcfg.behavior.quick_guess,
    })
  }

  pub fn color(&self, name: &str) -> Color {
    *self.theme.palette.custom(name).unwrap()
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
