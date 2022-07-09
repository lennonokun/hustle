use std::io::{self, Write};

use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

use super::gameio::GameIO;
use super::menu::{MenuScreen, MenuResults};
use super::play::{PlayScreen, PlayResults};
use super::end::{EndScreen, EndResults};

const NEXTRA: u16 = 5;
const MAXNWORDS: u16 = 2000;

pub fn game() {
  let stdin = io::stdin().lock().keys();
  let stdout = io::stdout().lock().into_raw_mode().unwrap();
  let mut gio = GameIO::new(stdin, stdout);

  let mut cont = true;
  let mut screen = "menu";
  let mut m_results = MenuResults::default();
  
  while cont {
    if screen == "menu" {
      let mut menu = MenuScreen::new(&mut gio);
      m_results = menu.run();
      cont = !m_results.quit;
      screen = "play";
    } else if screen == "play" {
      let mut play = PlayScreen::new(&mut gio, m_results.bank, m_results.wlen, m_results.nwords);
      let p_results = play.run();

      let mut end = EndScreen::new(&mut gio, p_results);
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
