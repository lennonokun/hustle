use std::io::{self, StdinLock, StdoutLock, Write};

use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

// TODO
// unit tests

// space
const EMPTY: &str = " ";
// edges
const HEDGE: &str = "─";
const VEDGE: &str = "│";
// corners
const ULC: &str = "┌";
const URC: &str = "┐";
const BLC: &str = "└";
const BRC: &str = "┘";
// cuts
const UCUT: &str = "┬";
const RCUT: &str = "┤";
const DCUT: &str = "┴";
const LCUT: &str = "├";

/// write! for GameIO
/// *a: at specific coords
/// *f: formatted

#[macro_use]
macro_rules! wrt {
  ($gio: expr, $( $x: expr ),* ) => {
    $(write!(($gio).stdout, "{}", $x));*
  }
}

#[macro_use]
macro_rules! wrta {
  ($gio: expr, $x: expr, $y: expr, $( $s: expr ),* ) => {
    write!(($gio).stdout, "{}", cursor::Goto($x, $y));
    $(write!(($gio).stdout, "{}", $s));*
  }
}

#[macro_use]
macro_rules! wrtf {
  ($gio: expr, $fmt: expr, $($arg: expr ),*) => {
    {write!(($gio).stdout, $fmt, $($arg),*);}
  }
}

#[macro_use]
macro_rules! wrtaf {
  ($gio: expr, $x: expr, $y: expr, $fmt: expr, $($arg: expr ),* ) => {
    write!(($gio).stdout, "{}", cursor::Goto($x, $y));
    write!(($gio).stdout, $fmt, $($arg),*);
  }
}

type GameIn<'a> = Keys<StdinLock<'a>>;
type GameOut<'a> = RawTerminal<StdoutLock<'a>>;

/// game input and output handler
pub struct GameIO<'a> {
  pub stdin: Keys<StdinLock<'a>>,
  pub stdout: RawTerminal<StdoutLock<'a>>,
  pub width: u16,
  pub height: u16,
}

impl<'a> GameIO<'a> {
  /// construct new GameIO with specified input and output
  pub fn new(stdin: GameIn<'a>, stdout: GameOut<'a>) -> Self {
    let termsz = terminal_size().ok();
    let width = termsz.map(|sz| sz.0).unwrap();
    let height = termsz.map(|sz| sz.1).unwrap();
    Self {stdin, stdout, width, height}
  }

  // update size and return if different
  pub fn resize(&mut self) -> bool {
    if let Ok(termsz) = terminal_size() {
      let diff = (self.width, self.height) != termsz;
      (self.width, self.height) = termsz;
      return diff;
    }
    return false;
  }

  /// read single key from stdin
  pub fn read(&mut self) -> Key {
    self.stdin.next().unwrap().unwrap()
  }

  pub fn read_at(&mut self, x: u16, y: u16) -> Key {
    wrt!(self, cursor::Goto(x, y));
    self.stdin.next().unwrap().unwrap()
  }

  /// flush output
  pub fn flush(&mut self) {
    self.stdout.flush();
  }

  /// draws the empty base
  pub fn empty(&mut self) {
    wrta!(self, 1, 1, clear::All, cursor::Hide, style::Reset);
    for x in 1..=self.width {
      for y in 1..=self.height {wrt!(self, EMPTY);}
      wrt!(self, "\n");
    }
  }

  pub fn clear(&mut self) {
    wrta!(self, 1, 1, clear::All, cursor::Show, style::Reset);
  }

  /// draws a rectangle from (x,y) to (x+w,y+h)
  pub fn rect(&mut self, x: u16, y: u16, w: u16, h: u16) {
    // top
    wrta!(self, x, y, ULC);
    for _ in 1..w-1 {wrt!(self, HEDGE);}
    wrt!(self, URC);

    // middle
    for i in 1..h-1 {
      wrta!(self, x, y+i, VEDGE);
      for _ in 1..w-1 {wrt!(self, EMPTY);}
      wrt!(self, VEDGE);
    }

    // bottom
    wrta!(self, x, y+h-1, BLC);
    for _ in 1..w-1 {wrt!(self, HEDGE);}
    wrt!(self, BRC);
  }

  /// draws a cut from (x,y) to (x+w,y)
  pub fn hcut(&mut self, x: u16, y: u16, w: u16) {
    wrta!(self, x, y, LCUT);
    for _ in 1..w-1 {wrt!(self, HEDGE);}
    wrt!(self, RCUT);
  }

  /// draws a cut from (x,y) to (x,y+h)
  pub fn vcut(&mut self, x: u16, y: u16, h: u16) {
    wrta!(self, x, y, LCUT);
    for i in 1..h-1 {wrta!(self, x+i, y, HEDGE);}
    wrt!(self, RCUT);
  }
}
