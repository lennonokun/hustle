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

use super::menu::menu_open;

// TODO screens? 

pub fn play() {
  let mut siv = cursive::default();
  let mut palette = Palette::default();
//  palette[Background] = TerminalDefault;
//  palette[Primary] = TerminalDefault;
//  palette[Secondary] = TerminalDefault;
//  palette[Tertiary] = TerminalDefault;
//  palette[View] = TerminalDefault;
//  palette[Highlight] = TerminalDefault;
  let theme = Theme {shadow: false, borders: BorderStyle::Simple, palette};
  
  siv.set_theme(theme);
  siv.set_fps(20);
  siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());
  menu_open(&mut siv);
  siv.run()
}

//pub fn game() {
//  let gin = io::stdin().lock().keys();
//  let gout = io::stdout().lock().into_raw_mode().unwrap();
//  let mut gio = GameIO::new(gin, gout);
//  let cfg = Config::load();
//
//  let mut cont = true;
//  let mut screen = "menu";
//  let mut m_results = MenuResults::default();
//
//  while cont {
//    if screen == "menu" {
//      let menu = MenuScreen::new(&mut gio, &cfg);
//      m_results = menu.run();
//      cont = !m_results.quit;
//      screen = "play";
//    } else if screen == "play" {
//      let mut play = PlayScreen::new(&mut gio, &cfg, m_results.bank, m_results.wlen, m_results.nwords);
//      let p_results = play.run();
//
//      let mut end = EndScreen::new(&mut gio, &cfg, p_results);
//      let e_results = end.run();
//
//      if e_results.quit {
//        cont = false;
//      } else if e_results.restart {
//        screen = "play";
//      } else if e_results.menu {
//        screen = "menu";
//      }
//    }
//  }
//
//  gio.clear();
//}
