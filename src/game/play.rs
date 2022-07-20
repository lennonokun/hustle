use std::io;
use std::env;
use std::path::{Path, PathBuf};

use cursive::Cursive;
use cursive::views::*;
use cursive::traits::*;
use cursive::event::{Event, Key};
use cursive::theme::PaletteColor::*;
use cursive::theme::Color::*;
use cursive::theme::BaseColor::*;
use cursive::theme::{Theme, Palette, BorderStyle};

use super::menu::open_menu;
use super::config::CONFIG;

pub fn play() {
  let mut siv = cursive::default();
  siv.set_theme(CONFIG.theme.clone());
  siv.set_fps(20);
  siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());

  open_menu(&mut siv);
  siv.run()
}
