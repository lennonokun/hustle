use std::cmp;
use std::io::{self, Write};
use std::time::Instant;

use termion::event::Key;
use termion::{clear, color, cursor, style, terminal_size};

use crate::ds::*;
use super::gameio::GameIO;
use super::play::PlayResults;

pub struct EndScreen<'a, 'b> {
  gio: &'a mut GameIO<'b>,
  results: PlayResults, 
}

#[derive(Clone, Copy, Default)]
pub struct EndResults {
  pub restart: bool,
  pub menu: bool,
  // redundant (only one can be true)
  pub quit: bool,
}

impl<'a, 'b> EndScreen<'a, 'b> {
  pub fn new(gio: &'a mut GameIO<'b>, results: PlayResults) -> Self {
    Self {gio, results}
  }

  pub fn run(&mut self) -> EndResults {
    let ncols = (self.gio.width - 1) / (self.results.wlen + 1) as u16;

    self.gio.rect(1, 1, self.gio.width, self.gio.height);
    if self.results.won {
      wrtaf!(self.gio, 2, 2,
        "Won n={} in {}/{}, {:.3}!",
        self.results.nwords,
        self.results.turn,
        self.results.nwords + NEXTRA as u16,
        self.results.time.as_millis() as f64 / 1000.
      );
    } else {
      wrta!(self.gio, 2, 2, "Answers were:");
      for (i, ans) in self.results.answers.iter().enumerate() {
        let col = i as u16 % ncols;
        let row = i as u16 / ncols;
        let x = (self.results.wlen as u16 + 1) * col + 2;
        let y = row + 4;
        wrta!(self.gio, x, y, ans);
      }
    }

    wrta!(self.gio, 2, self.gio.height - 1,
          "'r': restart, 's': change settings, 'q'/Esc: quit");
    self.gio.flush();

    let mut restart = false;
    let mut menu = false;
    let mut quit = false;
    while !quit && !restart {
      match self.gio.read() {
        Key::Char(c) => {
          quit = c == 'q';
          restart = c == 'r' || c == 's';
          menu = c == 's';
        }
        Key::Esc => {
          quit = true;
        }
        _ => {}
      }
    }

    EndResults {restart, menu, quit}
  }
}
