use std::io::{self, Write};

use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

use super::gameio::GameIO;
use super::menu::{Menu, MenuResults};
use super::play::{Play, PlayResults};
use super::end::{End, EndResults};

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
      let mut menu = Menu::new(&mut gio);
      m_results = menu.run();
      cont = !m_results.quit;
      screen = "play";
    } else if screen == "play" {
      let mut play = Play::new(&mut gio, m_results.bank, m_results.wlen, m_results.nwords);
      let p_results = play.run();

      let mut end = End::new(&mut gio, p_results);
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

  wrta!(gio, 1, 1, clear::All, cursor::Show, style::Reset);
}
