use std::cmp;
use std::io::{self, StdinLock, StdoutLock, Write};
use std::time::Instant;

use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

use crate::ds::*;
use super::fbcol::FeedbackCol;
use super::menu::Menu;
use super::gameio::GameIO;

const NEXTRA: u16 = 5;
const MAXNWORDS: u16 = 2000;

pub struct Game<'a> {
  gwb: WBank,
  awb: WBank,
  gio: GameIO<'a>,
  wlen: u8,
  nrows: u16,
  ncols: u16,
  maxrow: u16,
  nwords: u16,
  scroll: u16,
  turn: u16,
  ndone: u16,
  empty_string: String,
  t_start: Instant,
  cols: Vec<FeedbackCol>,
  answers: Vec<Word>,
}

impl<'a> Game<'a> {
  pub fn new() -> Self {
    let stdin = io::stdin().lock().keys();
    let stdout = io::stdout().lock().into_raw_mode().unwrap();
    Game {
      gwb: WBank {
        wlen: 0,
        data: Vec::new(),
      },
      awb: WBank {
        wlen: 0,
        data: Vec::new(),
      },
      gio: GameIO::new(stdin, stdout),
      wlen: 0,
      maxrow: 0,
      nwords: 0,
      ncols: 0,
      nrows: 0,
      scroll: 0,
      turn: 0,
      ndone: 0,
      empty_string: String::new(),
      t_start: Instant::now(),
      cols: Vec::new(),
      answers: Vec::new(),
    }
  }

  fn draw_status_base(&mut self) {
    self.gio.hcut(1, 3, self.gio.width);
  }

  fn draw_status(&mut self) {
    wrtaf!(self.gio, 2, 2, "guesses: {}/{}, solved: {}/{}, scroll: {}/{}",
           self.turn, self.nwords + NEXTRA,
           self.ndone, self.nwords,
           self.scroll + 1, self.nrows);
  }

  fn draw_fbc_row(&mut self, ncol: u16, nrow: u16) {
    let (x, y) = (ncol * (self.wlen as u16 + 1) + 2, nrow + 4);
    let s = self
      .cols
      .get((self.ncols * self.scroll + ncol) as usize)
      .and_then(|fbc| fbc.rows.get(nrow as usize))
      .unwrap_or(&self.empty_string);
    wrta!(self.gio, x, y, s);
  }

  fn draw_fbcols(&mut self) {
    for nrow in 0..cmp::min(self.turn, self.maxrow) {
      for ncol in 0..self.ncols {
        self.draw_fbc_row(ncol, nrow as u16)
      }
    }
    self.gio.flush();
  }

  fn draw_empty_col(&mut self, ncol: u16) {
    for nrow in 0..cmp::min(self.turn, self.maxrow) {
      let (x, y) = (ncol * (self.wlen as u16 + 1) + 2, nrow + 2);
      wrta!(self.gio, x, y, self.empty_string);
    }
  }

  fn menu_screen(&mut self) -> bool {
    self.gio.rect(1, 1, self.gio.width, self.gio.height);
    let menu = Menu::new(&mut self.gio);
    let res = menu.run();
    if res.quit {return true}
    self.nwords = res.nwords;
    self.wlen = res.wlen;
    (self.gwb, self.awb) = WBank::from2(res.bank, self.wlen).unwrap();
    false
  }

  fn end_screen(&mut self) -> (bool, bool) {
    self.gio.rect(1, 1, self.gio.width, self.gio.height);
    if self.ndone == self.nwords {
      wrtaf!(self.gio, 2, 2,
        "Won n={} in {}/{}, {:.3}!",
        self.nwords,
        self.turn,
        self.nwords + NEXTRA as u16,
        self.t_start.elapsed().as_millis() as f64 / 1000.
      );
    } else {
      wrta!(self.gio, 2, 2, "Answers were:");
      for (i, ans) in self.answers.iter().enumerate() {
        let col = i as u16 % self.ncols;
        let row = i as u16 / self.ncols;
        let x = (self.wlen as u16 + 1) * col + 2;
        let y = row + 4;
        wrta!(self.gio, x, y, ans);
      }
    }

    wrta!(self.gio, 2, self.gio.height - 1,
          "'r': restart, 's': change settings, 'q'/Esc: quit");
    self.gio.flush();

    let mut quit = false;
    let mut restart = false;
    let mut menu = false;
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
    (restart, menu)
  }

  pub fn start(&mut self) {
    let mut cont = true;
    let mut menu = true;

    while cont {
      if menu && self.menu_screen() {break}

      self.ncols = (self.gio.width - 1) / (self.wlen + 1) as u16;
      self.nrows = (self.nwords - 1) / self.ncols + 1;
      self.maxrow = self.gio.height - 5;
      self.empty_string = String::new();
      for _ in 0..self.wlen {
        self.empty_string.push(' ');
      }

      self.ndone = 0;
      self.turn = 0;
      self.scroll = 0;
      self.answers = self.awb.pick(&mut rand::thread_rng(), self.nwords.into());
      self.cols = self
        .answers
        .iter()
        .map(|ans| FeedbackCol::new(*ans))
        .collect();
      let limit = self.nwords + NEXTRA;
      let mut quit = false;
      let mut guess = String::new();

      self.gio.rect(1, 1, self.gio.width, self.gio.height);
      self.draw_status_base();

      while self.turn < limit && self.ndone < self.nwords as u16 && !quit {
        self.draw_status();
        wrta!(self.gio, 2, self.gio.height - 1, self.empty_string);
        wrta!(self.gio, 2, self.gio.height - 1, guess);
        self.gio.flush();

        match self.gio.read() {
          Key::Char(c) => {
            if ('a'..='z').contains(&c) {
              let c2 = (c as u8 - 32) as char;
              guess.push(c2);
            }
          }
          Key::Backspace => {
            guess.pop();
          }
          Key::Esc => {
            quit = true;
          }
          Key::Up => {
            self.scroll = (self.scroll + self.nrows - 1) % self.nrows;
            self.draw_fbcols();
          }
          Key::Down => {
            self.scroll = (self.scroll + 1) % self.nrows;
            self.draw_fbcols();
          }
          _ => {}
        }

        if guess.len() == self.wlen.into() {
          let gw = Word::from(guess).unwrap();
          if self.gwb.contains(gw) {
            if self.turn == 0 {
              self.t_start = Instant::now()
            }
            let mut i_done: Option<usize> = None;
            for (i, c) in self.cols.iter_mut().enumerate() {
              if c.guess(gw) {
                i_done = Some(i);
                self.ndone += 1;
              }
            }

            self.turn += 1;
            if let Some(i) = i_done {
              // remove finished column and redraw entirely
              self.cols.remove(i);
              self.draw_fbcols();
            } else if self.turn <= self.maxrow {
              // or just draw guesses
              for i in 0..self.ncols {
                self.draw_fbc_row(i, self.turn - 1);
              }
            }
          }
          guess = String::new();
          wrta!(self.gio, 2, self.gio.height - 1, self.empty_string);
        }
      }

      (cont, menu) = self.end_screen();
    }

    write!(
      self.gio.stdout,
      "{}{}{}",
      clear::All,
      style::Reset,
      cursor::Goto(1, 1)
    );
  }
}
