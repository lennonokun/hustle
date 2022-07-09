use std::io::{self, StdinLock, StdoutLock, Write};

use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

use crate::ds::{MINWLEN, MAXWLEN};
use super::gameio::GameIO;

const MENUWIDTH: u16 = 25;
const MENUHEIGHT: u16 = 9;
const MENUSTARX: [u16; 3] = [2, 2, 2];
const MENUSTARY: [u16; 3] = [4, 5, 6];
const MENUENTX: [u16; 3] = [12, 12, 12];
const MENUENTY: [u16; 3] = [4, 5, 6];
const MENUSCREEN: [&str; MENUHEIGHT as usize] = [
  "┌────────────────────────┐",
  "│                        │",
  "│         HUSTLE         │",
  "│                        │",
  "│   nwords:              │",
  "│     wlen: 5            │",
  "│     bank: < bank1 >    │",
  "│                        │",
  "└────────────────────────┘",
];

const NBANKS: u8 = 2;
const MAXNWORDS: u16 = 2000;
const WBPREVIEW: [&str; 2] = ["< bank1 >", "< bank2 >"];
const WBPATHS: [&str; 2] = ["/usr/share/hustle/bank1.csv", "/usr/share/hustle/bank2.csv"];

#[derive(Clone, Copy, Default)]
pub struct MenuResults {
  pub quit: bool,
  pub nwords: u16,
  pub wlen: u8,
  pub bank: &'static str,
}

pub struct MenuScreen<'a, 'b> {
  gio: &'a mut GameIO<'b>
}

type LockedIn<'a> = Keys<StdinLock<'a>>;
type LockedOut<'a> = RawTerminal<StdoutLock<'a>>;

impl<'a, 'b> MenuScreen<'a, 'b> {
  pub fn new(gio: &'a mut GameIO<'b>) -> Self {
    Self {gio}
  }

  pub fn run(self) -> MenuResults {
    let x0 = (self.gio.width - MENUWIDTH) / 2 + 1;
    let y0 = (self.gio.height - MENUHEIGHT) / 2 + 1;
    
    self.gio.empty();
    self.gio.rect(x0, y0, MENUWIDTH, MENUHEIGHT);
    for i in 0..MENUHEIGHT {
      wrta!(self.gio, x0, y0+i, MENUSCREEN[i as usize]);
    }
    self.gio.flush();

    let mut cont = true;
    let mut quit = false;

    let mut i = 0usize;
    let mut s_nwords = String::new();
    let mut nwords: Option<u16> = None;
    let mut s_wlen = String::from("5");
    let mut wlen: Option<u8> = None;
    let mut j_bank: usize = 0;
    let mut bank: Option<&str> = None;

    while cont {
      let entx = x0 + MENUENTX[i];
      let enty = y0 + MENUENTY[i];
      let starx = x0 + MENUSTARX[i];
      let stary = y0 + MENUSTARY[i];
      wrt!(self.gio, cursor::Goto(starx, stary), "*");
      match self.gio.read() {
        Key::Char('\n') => {
          // stop if valid
          nwords = s_nwords.parse().ok();
          wlen = s_wlen.parse().ok();
          bank = Some(WBPATHS[j_bank]);
          if let (Some(nwords), Some(wlen), Some(bank)) = (nwords, wlen, bank) {
            cont = !((1..=MAXNWORDS).contains(&nwords)
              && (MINWLEN..=MAXWLEN).contains(&(wlen as usize)));
          }
        }
        Key::Up | Key::BackTab => {
          wrta!(self.gio, starx, stary, " ");
          i = (i + 2) % 3;
        }
        Key::Down | Key::Char('\t') => {
          wrta!(self.gio, starx, stary, " ");
          i = (i + 1) % 3;
        }
        Key::Left => {
          if i == 2 {
            j_bank = (j_bank - 1) % 2;
            wrta!(self.gio, entx, enty, WBPREVIEW[j_bank]);
            self.gio.flush();
          }
        }
        Key::Right => {
          if i == 2 {
            j_bank = (j_bank + 1) % 2;
            wrta!(self.gio, entx, enty, WBPREVIEW[j_bank]);
            self.gio.flush();
          }
        }
        Key::Backspace => {
          if i < 2 {
            // pop character
            let mut s = if i == 0 { &mut s_nwords } else { &mut s_wlen };
            s.pop();
            wrta!(self.gio, entx + s.len() as u16, enty, " ");
            self.gio.flush();
          }
        }
        Key::Esc => {
          cont = false;
          quit = true;
        }
        Key::Char(c) => {
          if i < 2 && '0' <= c && c <= '9' {
            // push character
            let mut s = if i == 0 { &mut s_nwords } else { &mut s_wlen };
            wrta!(self.gio, entx + s.len() as u16, enty, c);
            self.gio.flush();
            s.push(c);
          }
        }
        _ => {}
      }
    }

    if quit {
      MenuResults{quit, nwords: 0, wlen: 0, bank: ""}
    } else {
      let nwords = nwords.unwrap();
      let wlen = wlen.unwrap();
      let bank = bank.unwrap();
      MenuResults{quit, nwords, wlen, bank}
    }
  }
}
