use std::cmp;
use std::io::{self, StdinLock, StdoutLock, Write};
use std::time::Instant;

use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, color, cursor, style, terminal_size};

use crate::ds::*;

const NEXTRA: u16 = 5;
const MAXNWORDS: u16 = 2000;
// space
const EMPTY: &str = " ";
// edges
const HORZE: &str = "─";
const VERTE: &str = "│";
// corners
const ULC: &str = "┌";
const URC: &str = "┐";
const BLC: &str = "└";
const BRC: &str = "┘";
const MLC: &str = "├";
const MRC: &str = "┤";

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
const WBPREVIEW: [&str; 2] = ["< bank1 >", "< bank2 >"];
const WBPATHS: [&str; 2] = ["/usr/share/hustle/bank1.csv", "/usr/share/hustle/bank2.csv"];

#[derive(Debug)]
struct FeedbackCol {
  ans: Word,
  rows: Vec<String>,
  wlen: u8,
  done: bool,
}

impl FeedbackCol {
  fn new(ans: Word) -> Self {
    Self {
      ans,
      rows: Vec::<String>::new(),
      wlen: ans.wlen,
      done: false,
    }
  }

  // returns if newly finished
  fn guess(&mut self, gw: Word) -> bool {
    if self.done || self.wlen != gw.wlen {
      return false;
    }
    let fb = Feedback::from(gw, self.ans).unwrap();
    let mut s = String::new();
    for i in 0..self.wlen {
      if fb.get_g(i) {
        s += &format!(
          "{}{}",
          color::Rgb(255, 255, 255).fg_string(),
          color::Bg(color::Green)
        );
      } else if fb.get_y(i) {
        s += &format!(
          "{}{}",
          color::Rgb(255, 255, 255).fg_string(),
          color::Bg(color::Yellow)
        );
      } else {
        s += &format!(
          "{}{}",
          color::Rgb(255, 255, 255).fg_string(),
          color::Bg(color::Blue)
        );
      }
      s.push((gw.data[i as usize] + b'A') as char);
    }
    s += &format!("{}{}", color::Reset.fg_str(), color::Reset.bg_str());
    self.rows.push(s);
    self.done = gw == self.ans;
    self.done
  }
}

pub struct Game<R, W> {
  gwb: WBank,
  awb: WBank,
  stdin: R,
  stdout: W,
  wlen: u8,
  width: u16,
  height: u16,
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

impl<'a> Game<Keys<StdinLock<'a>>, RawTerminal<StdoutLock<'a>>> {
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
      stdin,
      stdout,
      wlen: 0,
      width: 0,
      height: 0,
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

  fn draw_base(&mut self) {
    write!(self.stdout, "{}{}", clear::All, cursor::Goto(1, 1));

    // top edge
    self.stdout.write_all(ULC.as_bytes()).unwrap();
    for _ in 1..self.width - 1 {
      self.stdout.write_all(HORZE.as_bytes()).unwrap();
    }
    self.stdout.write_all(URC.as_bytes()).unwrap();
    self.stdout.write_all("\r\n".as_bytes()).unwrap();

    // left+right edges
    for _ in 1..self.height - 1 {
      self.stdout.write_all(VERTE.as_bytes()).unwrap();
      for _ in 1..self.width - 1 {
        self.stdout.write_all(EMPTY.as_bytes()).unwrap();
      }
      self.stdout.write_all(VERTE.as_bytes()).unwrap();
      self.stdout.write_all("\r\n".as_bytes()).unwrap();
    }

    // bottom edge
    self.stdout.write_all(BLC.as_bytes()).unwrap();
    for _ in 1..self.width - 1 {
      self.stdout.write_all(HORZE.as_bytes()).unwrap();
    }
    self.stdout.write_all(BRC.as_bytes()).unwrap();

    write!(self.stdout, "{}", cursor::Hide);
    // self.stdout.flush().unwrap();
  }

  fn draw_status_base(&mut self) {
    write!(self.stdout, "{}", cursor::Goto(1, 3));
    self.stdout.write_all(MLC.as_bytes());
    for _ in 1..self.width - 1 {
      self.stdout.write_all(HORZE.as_bytes()).unwrap();
    }
    self.stdout.write_all(MRC.as_bytes());
  }

  fn draw_status(&mut self) {
    write!(self.stdout, "{}", cursor::Goto(2, 2));
    write!(
      self.stdout,
      "guesses: {}/{}, solved: {}/{}, scroll: {}/{}",
      self.turn,
      self.nwords + NEXTRA,
      self.ndone,
      self.nwords,
      self.scroll + 1,
      self.nrows
    );
  }

  fn draw_fbc_row(&mut self, ncol: u16, nrow: u16) {
    let goto = cursor::Goto(ncol * (self.wlen as u16 + 1) + 2, nrow + 4);
    let s = self
      .cols
      .get((self.ncols * self.scroll + ncol) as usize)
      .and_then(|fbc| fbc.rows.get(nrow as usize))
      .unwrap_or(&self.empty_string);
    write!(self.stdout, "{}{}", goto, s);
  }

  fn draw_fbcols(&mut self) {
    for nrow in 0..cmp::min(self.turn, self.maxrow) {
      for ncol in 0..self.ncols {
        self.draw_fbc_row(ncol, nrow as u16)
      }
    }
    self.stdout.flush();
  }

  fn draw_empty_col(&mut self, ncol: u16) {
    for nrow in 0..cmp::min(self.turn, self.maxrow) {
      let goto = cursor::Goto(ncol * (self.wlen as u16 + 1) + 2, nrow + 2);
      write!(self.stdout, "{}{}", goto, self.empty_string);
    }
  }

  fn menu_screen(&mut self) -> bool {
    let x0 = (self.width - MENUWIDTH) / 2 + 1;
    let y0 = (self.height - MENUHEIGHT) / 2 + 1;
    for i in 0..MENUHEIGHT {
      write!(self.stdout, "{}", cursor::Goto(x0, y0 + i));
      self.stdout.write_all(MENUSCREEN[i as usize].as_bytes());
    }
    self.stdout.flush();

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
      writeln!(self.stdout, "{}*", cursor::Goto(starx, stary));
      match self.stdin.next().unwrap().unwrap() {
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
          writeln!(self.stdout, "{} ", cursor::Goto(starx, stary));
          i = (i + 2) % 3;
        }
        Key::Down | Key::Char('\t') => {
          writeln!(self.stdout, "{} ", cursor::Goto(starx, stary));
          i = (i + 1) % 3;
        }
        Key::Left => {
          if i == 2 {
            j_bank = (j_bank - 1) % 2;
            write!(
              self.stdout,
              "{}{}",
              cursor::Goto(entx, enty),
              WBPREVIEW[j_bank]
            );
            self.stdout.flush();
          }
        }
        Key::Right => {
          if i == 2 {
            j_bank = (j_bank + 1) % 2;
            write!(
              self.stdout,
              "{}{}",
              cursor::Goto(entx, enty),
              WBPREVIEW[j_bank]
            );
            self.stdout.flush();
          }
        }
        Key::Backspace => {
          if i < 2 {
            // pop character
            let mut s = if i == 0 { &mut s_nwords } else { &mut s_wlen };
            s.pop();
            write!(
              self.stdout,
              "{} ",
              cursor::Goto(entx + s.len() as u16, enty)
            );
            self.stdout.flush();
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
            write!(
              self.stdout,
              "{}{}",
              cursor::Goto(entx + s.len() as u16, enty),
              c
            );
            s.push(c);
            self.stdout.flush();
          }
        }
        _ => {}
      }
    }

    if quit {
      return true;
    }

    let bank = bank.unwrap();
    self.nwords = nwords.unwrap();
    self.wlen = wlen.unwrap();
    (self.gwb, self.awb) = WBank::from2(bank, self.wlen).unwrap();
    false
  }

  fn end_screen(&mut self) -> (bool, bool) {
    self.draw_base();
    write!(self.stdout, "{}", cursor::Goto(2, 2));
    if self.ndone == self.nwords {
      write!(
        self.stdout,
        "Won n={} in {}/{}, {:.3}!",
        self.nwords,
        self.turn,
        self.nwords + NEXTRA as u16,
        self.t_start.elapsed().as_millis() as f64 / 1000.
      );
    } else {
      write!(self.stdout, "Answers were:");
      for (i, ans) in self.answers.iter().enumerate() {
        let col = i as u16 % self.ncols;
        let row = i as u16 / self.ncols;
        let x = (self.wlen as u16 + 1) * col + 2;
        let y = row + 4;
        write!(self.stdout, "{}{}", cursor::Goto(x, y), ans.to_string());
      }
    }

    write!(
      self.stdout,
      "{}'r': restart, 's': change settings, 'q'/Esc: quit",
      cursor::Goto(2, self.height - 1)
    );

    self.stdout.flush();
    let mut quit = false;
    let mut restart = false;
    let mut menu = false;
    while !quit && !restart {
      match self.stdin.next().unwrap().unwrap() {
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
    let termsz = terminal_size().ok();
    self.width = termsz.map(|(w, _)| w).unwrap();
    self.height = termsz.map(|(_, h)| h).unwrap();
    self.maxrow = self.height - 5;

    let mut cont = true;
    let mut menu = true;

    while cont {
      if menu {
        self.draw_base();
        if self.menu_screen() {
          break;
        };
      }

      self.ncols = (self.width - 1) / (self.wlen + 1) as u16;
      self.nrows = (self.nwords - 1) / self.ncols + 1;
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

      self.draw_base();
      self.draw_status_base();

      while self.turn < limit && self.ndone < self.nwords as u16 && !quit {
        self.draw_status();
        write!(
          self.stdout,
          "{}",
          cursor::Goto(guess.len() as u16 + 2, self.height - 1)
        );
        self.stdout.flush();
        match self.stdin.next().unwrap().unwrap() {
          Key::Char(c) => {
            if ('a'..='z').contains(&c) {
              let c2 = (c as u8 - 32) as char;
              guess.push(c2);
              write!(
                self.stdout,
                "{}{}",
                cursor::Goto(guess.len() as u16 + 1, self.height - 1),
                c2
              );
            }
          }
          Key::Backspace => {
            guess.pop();
            write!(
              self.stdout,
              "{} ",
              cursor::Goto(guess.len() as u16 + 2, self.height - 1)
            );
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
          let goto = cursor::Goto(2, self.height - 1);
          write!(self.stdout, "{}{}", goto, self.empty_string);
        }
      }

      self.draw_base();
      (cont, menu) = self.end_screen();
    }
    write!(
      self.stdout,
      "{}{}{}",
      clear::All,
      style::Reset,
      cursor::Goto(1, 1)
    );
  }
}
