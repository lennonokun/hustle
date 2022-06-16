use rand::prelude::*;
use std::io::{self, Write, StdinLock, StdoutLock};
use std::time::Instant;

use termion::{terminal_size, clear, cursor, color, style};
use termion::raw::{IntoRawMode,RawTerminal};
use termion::input::{TermRead,Keys};
use termion::event::Key;

use crate::ds::*;

const NEXTRA: u16 = 5;
// space
const EMPTY: &'static str = " ";
// edges
const HORZE: &'static str = "─";
const VERTE: &'static str = "│";
// corners
const ULC: &'static str = "┌";
const URC: &'static str = "┐";
const BLC: &'static str = "└";
const BRC: &'static str = "┘";

// #[derive(Debug)]
struct FeedbackCol {
	ans: Word,
	rows: Vec<String>,
	done: bool
}

impl FeedbackCol {
	fn new(ans: Word) -> Self {
		Self {
			ans: ans,
			rows: Vec::<String>::new(),
			done: false
		}
	}

	// returns if newly finished
	fn guess(&mut self, gw: Word) -> bool {
		if self.done {return false}
		let fb = Feedback::from(gw, self.ans);
		let mut s = String::new();
		for i in 0..NLETS {
			if fb.get_g(i) {
				s += &format!("{}{}", color::Rgb(255, 255, 255).fg_string(),
											color::Bg(color::Green));
			} else if fb.get_y(i) {
				s += &format!("{}{}", color::Rgb(255, 255, 255).fg_string(),
											color::Bg(color::Yellow));
			} else {
				s += &format!("{}{}", color::Rgb(255, 255, 255).fg_string(),
											color::Bg(color::Blue));
			}
			s.push(gw.data[i]);
		};
		s += &format!("{}{}", color::Reset.fg_str(), color::Reset.bg_str());
		self.rows.push(s);
		self.done = gw == self.ans;
		return self.done;
	}
}

pub struct Game<'a, R, W> {
	gws: &'a WSet,
	aws: &'a WArr,
	stdin: R,
	stdout: W,
	width: u16,
	height: u16,
	nrows: u16,
	ncols: u16,
	nwords: u16,
	scroll: u16,
	turn: u16,
	ndone: u16,
	empty_string: String,
	t_start: Instant,
	cols: Vec<FeedbackCol>,
	answers: Vec<Word>,
}

impl <'a> Game<'a, Keys<StdinLock<'a>>, RawTerminal<StdoutLock<'a>>> {
	pub fn new(gws: &'a WSet, aws: &'a WArr) -> Self {
		let stdin = io::stdin().lock().keys();
		let stdout = io::stdout().lock().into_raw_mode().unwrap();
		Game {
			gws: gws,
			aws: aws,
			stdin: stdin,
			stdout: stdout,
			width: 0,
			height: 0,
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
		write!(self.stdout, "{}{}", clear::All, cursor::Goto(1,1));

		// top edge
		self.stdout.write(ULC.as_bytes()).unwrap();
		for _ in 1..self.width-1 {
			self.stdout.write(HORZE.as_bytes()).unwrap();
		}
		self.stdout.write(URC.as_bytes()).unwrap();
		self.stdout.write("\r\n".as_bytes()).unwrap();

		// left+right edges
		for _ in 1..self.height-1 {
			self.stdout.write(VERTE.as_bytes()).unwrap();
			for _ in 1..self.width-1 {
				self.stdout.write(EMPTY.as_bytes()).unwrap();
			}
			self.stdout.write(VERTE.as_bytes()).unwrap();
			self.stdout.write("\r\n".as_bytes()).unwrap();
		}

		// bottom edge
		self.stdout.write(BLC.as_bytes()).unwrap();
		for _ in 1..self.width-1 {
			self.stdout.write(HORZE.as_bytes()).unwrap();
		}
		self.stdout.write(BRC.as_bytes()).unwrap();

		self.stdout.flush().unwrap();
	}
	
	fn draw_fbc_row(&mut self, ncol: u16, nrow: u16) {
		let goto = cursor::Goto(ncol*(NLETS as u16 + 1) + 2, nrow + 2);
		let s = self.cols.get((self.ncols * self.scroll + ncol) as usize)
			.and_then(|fbc| fbc.rows.get(nrow as usize))
			.unwrap_or(&self.empty_string);
		write!(self.stdout, "{}{}", goto, s);
	}
	
	fn redraw_fbcols(&mut self) {
		for nrow in 0..self.turn {
			for ncol in 0..self.ncols {
				self.draw_fbc_row(ncol, nrow as u16)
			}
		}
		self.stdout.flush();
	}
	
	fn draw_empty_col(&mut self, ncol: u16) {
		for nrow in 0..self.turn {
			let goto = cursor::Goto(ncol*(NLETS as u16 + 1) + 2, nrow + 2);
			write!(self.stdout, "{}{}", goto, self.empty_string);
		}
	}

	// TODO: add removing cleared cols?
	pub fn start(&mut self, nwords: u16) {

		self.nwords = nwords;
		let termsz = terminal_size().ok();
		self.width = termsz.map(|(w,_)| w).unwrap();
		self.height = termsz.map(|(_,h)| h).unwrap();
		self.ncols = (self.width - 1) / (NLETS + 1) as u16;
		self.nrows = (self.nwords - 1) / self.ncols + 1;
		for _ in 0..NEXTRA {
			self.empty_string.push(' ');
		} if self.nwords + NEXTRA as u16 > self.height - 4 {
			writeln!(self.stdout, "{}{}Window is not tall enough!{}",
							 clear::All, cursor::Goto(1,1), cursor::Goto(1,1));
			return;
		}

		let mut cont = true;
		
		while cont {
			self.ndone = 0;
			self.turn = 0;
			self.scroll = 0;
			self.answers = self.aws
				.choose_multiple(&mut rand::thread_rng(), nwords.into())
				.cloned()
				.collect();
			self.cols = self.answers.iter()
				.map(|ans| FeedbackCol::new(*ans))
				.collect();

			self.draw_base();
			let limit = nwords + NEXTRA;
			let mut quit = false;
			let mut guess = String::new();

			while self.turn < limit && self.ndone < nwords as u16 && !quit {
				write!(self.stdout, "{}", cursor::Goto(guess.len() as u16 + 2, self.height-1));
				self.stdout.flush();
				match self.stdin.next().unwrap().unwrap() {
					Key::Char(c) => if 'a' <= c && c <= 'z' {
						let c2 = (c as u8 - 32) as char;
						guess.push(c2);
						write!(self.stdout, "{}{}",
									 cursor::Goto(guess.len() as u16 + 1, self.height-1),
									 c2.to_string());
					} Key::Backspace => {
						guess.pop();
						write!(self.stdout, "{} ",
									 cursor::Goto(guess.len() as u16 + 2, self.height-1));
					} Key::Esc => {
						quit = true;
					} Key::Up => {
						self.scroll = (self.scroll + self.nrows - 1) % self.nrows;
						self.redraw_fbcols();
					} Key::Down => {
						self.scroll = (self.scroll + 1) % self.nrows;
						self.redraw_fbcols();
					} _ => {}
				}

				if guess.len() == NLETS {
					let gw = Word::from(&guess).unwrap();
					if self.gws.contains(&gw) {
						if self.turn == 0 {self.t_start = Instant::now()}
						let mut i_done: Option<usize> = None;
						for (i, c) in self.cols.iter_mut().enumerate() {
							if c.guess(gw) {i_done = Some(i)}
						}

						self.turn += 1;
						if let Some(i) = i_done {
							// remove finished column and redraw entirely
							self.cols.remove(i);
							self.redraw_fbcols();
						} else {
							// or just draw guesses
							for i in 0..self.ncols {
								self.draw_fbc_row(i, self.turn-1);
							}
						}
					}
					guess = String::new();
					let goto = cursor::Goto(2, self.height - 1);
					write!(self.stdout, "{}{}", goto, self.empty_string);
				}
			}

			self.draw_base();
			write!(self.stdout, "{}", cursor::Goto(2,2));
			if self.ndone == self.nwords {
				write!(self.stdout, "Won in {}/{}, {:.3}!", self.turn,
								self.nwords + NEXTRA as u16,
								self.t_start.elapsed().as_millis() as f64 / 1000.);
			} else {
				write!(self.stdout, "Answers were:");
				for (i, ans) in self.answers.iter().enumerate() {
					write!(self.stdout, "{}{}. {}",
								cursor::Goto(2, i as u16 + 3),
								i + 1, ans.to_string());
				}
			}

			self.stdout.flush();
			let mut quit = false;
			let mut restart = false;
			while !quit && !restart {
				match self.stdin.next().unwrap().unwrap() {
					Key::Char(c) => {
						quit = c == 'q';
						restart = c == 'r';
					}, _ => {}
				}
			}
			cont = restart;
		}
		write!(self.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1,1));
	}
}
