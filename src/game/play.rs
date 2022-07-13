use std::cmp;
use std::io::{self, Write};
use std::time::{Instant, Duration};

use termion::event::Key;
use termion::{clear, color, cursor, style, terminal_size};

use crate::ds::*;
use super::fbcol::FeedbackCol;
use super::gameio::GameIO;

// TODO:
// is the dependence between play and end bad
// prettify unknowns + gue
// unknowns should be in top? + truncate

// frequency of letters in standard wordle bank
const FREQ_ORD: &'static str = "EAROTLISNCUYDHPMGBFKWVZXQJ";
const EMPTYUNKNOWNS: &'static str = "                          ";

pub struct PlayScreen<'a, 'b> {
  gio: &'a mut GameIO<'b>,
  gwb: WBank,
  awb: WBank,
  wbp: &'static str,
  wlen: u8,
  nwords: u16,
  nrows: u16,
  ncols: u16,
  maxrow: u16,
  scroll: u16,
  turn: u16,
  ndone: u16,
  empty_string: String,
  t_start: Instant,
  cols: Vec<FeedbackCol>,
  answers: Vec<Word>,
  unknowns: Vec<char>,
}

#[derive(Clone, Default)]
pub struct PlayResults {
  pub won: bool,
  pub wbp: &'static str,
  pub nwords: u16,
  pub wlen: u8,
  pub turn: u16,
  pub answers: Vec<Word>,
  pub time: Duration,
}

impl<'a, 'b> PlayScreen<'a, 'b> {
  pub fn new(gio: &'a mut GameIO<'b>, wbp: &'static str, wlen: u8, nwords: u16) -> Self {
    let (gwb, awb) = WBank::from2(wbp, wlen).unwrap();
    Self {
      gio,
      wbp,
      gwb,
      awb,
      wlen,
      nwords,
      maxrow: 0,
      ncols: 0,
      nrows: 0,
      scroll: 0,
      turn: 0,
      ndone: 0,
      empty_string: String::new(),
      t_start: Instant::now(),
      cols: Vec::new(),
      answers: Vec::new(),
      unknowns: Vec::new(),
    }
  }
  
  /// draw empty base
  fn empty(&mut self) {
    self.gio.rect(1, 1, self.gio.width, self.gio.height);
    self.gio.hcut(1, 4, self.gio.width);
  }

  fn draw_status(&mut self) {
    let limit = self.nwords + NEXTRA as u16;
    let answers_left = self.nwords - self.ndone;
    let turns_left = limit - self.turn;
    let extra_turns = turns_left as i32 - answers_left as i32;
    wrtaf!(self.gio, 2, 2, "solved: {}/{}, {}turns: {}/{} ({:+}){}, scroll: {}/{}",
           self.ndone, self.nwords,
           if extra_turns>=0 {color::Reset.fg_str()} else {color::Red.fg_str()},
           self.turn, limit, extra_turns,
           color::Fg(color::Reset),
           self.scroll + 1, self.nrows);
    let unknowns: String = self.unknowns.iter().cloned().collect();
    wrtaf!(self.gio, 2, 3, "unknowns: {}", EMPTYUNKNOWNS);
    wrtaf!(self.gio, 2, 3, "unknowns: {}", unknowns);
  }

  fn draw_fbc_row(&mut self, ncol: u16, nrow: u16) {
    let (x, y) = (ncol * (self.wlen as u16 + 1) + 2, nrow + 5);
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
  
  fn draw_guess(&mut self, guess: &String) {
    let y = cmp::min(self.turn, self.maxrow) + 5;
    for ncol in 0..self.ncols {
      let x = ncol * (self.wlen as u16 + 1) + 2;
      let n = self.ncols * self.scroll + ncol;
      wrta!(self.gio, x, y, self.empty_string);
      if (n as usize) < self.cols.len() {
        wrta!(self.gio, x, y, guess);
      }
    }
  }

  pub fn run(&mut self) -> PlayResults {
    self.ncols = (self.gio.width - 1) / (self.wlen + 1) as u16;
    self.nrows = (self.nwords - 1) / self.ncols + 1;
    self.maxrow = self.gio.height - 6;
    self.empty_string = String::new();
    for _ in 0..self.wlen {
      self.empty_string.push(' ');
    }
    self.unknowns = FREQ_ORD.chars().collect();

    self.ndone = 0;
    self.turn = 0;
    self.scroll = 0;
    self.answers = self.awb.pick(&mut rand::thread_rng(), self.nwords.into());
    self.cols = self
      .answers
      .iter()
      .map(|ans| FeedbackCol::new(*ans))
      .collect();

    let limit = self.nwords as usize + NEXTRA;
    let mut quit = false;
    let mut guess = String::new();

    self.empty();

    while (self.turn as usize) < limit && self.ndone < self.nwords as u16 && !quit {
      self.draw_status(); // also unecesseary?
      self.draw_guess(&guess);
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
        let gw = Word::from(guess.clone()).unwrap();
        if self.gwb.contains(gw) {
          // remove guessed characters
          let gw2: Vec<char> = guess.to_ascii_uppercase().chars().collect();
          self.unknowns.retain(|&c| !gw2.contains(&c));

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
      }
    }

    PlayResults {
      won: self.ndone == self.nwords,
      wbp: self.wbp,
      nwords: self.nwords,
      wlen: self.wlen,
      turn: self.turn,
      answers: self.answers.clone(),
      time: self.t_start.elapsed(),
    }
  }
}
