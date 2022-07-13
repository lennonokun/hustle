use std::io;
use std::env;
use std::path::{Path, PathBuf};

use termion::input::TermRead;
use termion::raw::IntoRawMode;

use super::config::Config;
use super::end::EndScreen;
use super::gameio::GameIO;
use super::menu::{MenuResults, MenuScreen};
use super::play::PlayScreen;

pub fn game() {
  let stdin = io::stdin().lock().keys();
  let stdout = io::stdout().lock().into_raw_mode().unwrap();
  let mut gio = GameIO::new(stdin, stdout);
  let cfg = Config::load();

  let mut cont = true;
  let mut screen = "menu";
  let mut m_results = MenuResults::default();

  while cont {
    if screen == "menu" {
      let menu = MenuScreen::new(&mut gio, &cfg);
      m_results = menu.run();
      cont = !m_results.quit;
      screen = "play";
    } else if screen == "play" {
      let mut play = PlayScreen::new(&mut gio, &cfg, m_results.bank, m_results.wlen, m_results.nwords);
      let p_results = play.run();

      let mut end = EndScreen::new(&mut gio, &cfg, p_results);
      let e_results = end.run();

      if e_results.quit {
        cont = false;
      } else if e_results.restart {
        screen = "play";
      } else if e_results.menu {
        screen = "menu";
      }
    }
  }

  gio.clear();
}
