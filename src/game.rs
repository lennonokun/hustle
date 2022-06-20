use std::io::{self, Write, StdinLock, StdoutLock};
use std::time::Instant;
use std::path::Path;
use std::cmp;

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

const MENUWIDTH: u16 = 17;
const MENUHEIGHT: u16 = 8;
const MENU_OFFX: [u16; 2] = [11, 11];
const MENU_OFFY: [u16; 2] = [4, 5];
const MENU_NX: u16 = 9;
const MENU_NY: u16 = 4;
const MENUSCREEN: [&'static str; MENUHEIGHT as usize] = [
	"┌────────────────┐",
	"│                │",
	"│    WORDLERS    │",
	"│                │",
	"│   nwords:      │",
	"│     wlen:      │",
	"│                │",
	"└────────────────┘",
];

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
			ans: ans,
			rows: Vec::<String>::new(),
			wlen: ans.wlen,
			done: false,
		}
	}

	// returns if newly finished
	fn guess(&mut self, gw: Word) -> bool {
		if self.done || self.wlen != gw.wlen {return false}
		let fb = Feedback::from(gw, self.ans).unwrap();
		let mut s = String::new();
		for i in 0..self.wlen {
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
			s.push((gw.data[i as usize] + 'A' as u8) as char);
		};
		s += &format!("{}{}", color::Reset.fg_str(),
									color::Reset.bg_str());
		self.rows.push(s);
		self.done = gw == self.ans;
		return self.done;
	}
}

pub struct Game<P: AsRef<Path>, R, W> {
	gwp: P,
	awp: P,
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

impl <'a, P: AsRef<Path>>
	Game<P, Keys<StdinLock<'a>>, RawTerminal<StdoutLock<'a>>> {
	pub fn new(gwp: P, awp: P) -> Self {
		let stdin = io::stdin().lock().keys();
		let stdout = io::stdout().lock().into_raw_mode().unwrap();
		Game {
			gwp: gwp,
			awp: awp,
			gwb: WBank {wlen: 0, data: Vec::new()},
			awb: WBank {wlen: 0, data: Vec::new()},
			stdin: stdin,
			stdout: stdout,
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
		write!(self.stdout, "{}", cursor::Hide);

		// self.stdout.flush().unwrap();
	}

	fn menu_screen(&mut self) {
		let x0 = self.width / 2 - 8;
		let y0 = self.height / 2 - 3;
		for i in 0..MENUHEIGHT {
			write!(self.stdout, "{}",
						 cursor::Goto(x0, y0 + i));
			self.stdout.write(MENUSCREEN[i as usize].as_bytes());
		}
		self.stdout.flush();

		let mut cont = true;
		let mut s_arr = [String::new(), String::new()];
		let mut idx = 0usize;
		while cont {
			let x = x0 + MENU_OFFX[idx];
			let y = y0 + MENU_OFFY[idx];
			match self.stdin.next().unwrap().unwrap() {
				Key::Char('\n') => {
					cont = false;
				} Key::Char(c) => if '0' <= c && c <= '9' {
					write!(self.stdout, "{}{}",
									cursor::Goto(x + s_arr[idx].len() as u16, y),
									c.to_string());
					s_arr[idx].push(c);
					self.stdout.flush();
				} Key::Backspace => {
					s_arr[idx].pop();
					write!(self.stdout, "{} ",
									cursor::Goto(x + s_arr[idx].len() as u16, y));
					self.stdout.flush();
				} Key::Up | Key::Down => {
					idx = (idx + 1) % 2;
				} _ => {}
			}
		}

		self.nwords = s_arr[0].parse().unwrap();

		let wlen = s_arr[1].parse().unwrap();
		if self.wlen == wlen {return}
		self.gwb = WBank::from(&self.gwp, wlen).unwrap();
		self.awb = WBank::from(&self.awp, wlen).unwrap();
		eprintln!("wlen: {}, gwb len: {}", self.wlen, self.gwb.data.len());
		self.wlen = wlen;
	}

	fn end_screen(&mut self) -> (bool, bool) {
		self.draw_base();
		write!(self.stdout, "{}", cursor::Goto(2,2));
		if self.ndone == self.nwords {
			write!(self.stdout, "Won n={} in {}/{}, {:.3}!",
							self.nwords,
							self.turn,
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

		write!(self.stdout,
						"{}'r': restart, 's': change settings, 'q'/Esc: quit",
						cursor::Goto(2, self.height-1));

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
				}, Key::Esc => {
					quit = true;
				} _ => {}
			}
		}
		return (restart, menu);
	}
	
	fn draw_fbc_row(&mut self, ncol: u16, nrow: u16) {
		let goto = cursor::Goto(ncol*(self.wlen as u16 + 1) + 2, nrow + 2);
		let s = self.cols.get((self.ncols * self.scroll + ncol) as usize)
			.and_then(|fbc| fbc.rows.get(nrow as usize))
			.unwrap_or(&self.empty_string);
		write!(self.stdout, "{}{}", goto, s);
	}
	
	fn redraw_fbcols(&mut self) {
		for nrow in 0..cmp::min(self.turn, self.maxrow) {
			for ncol in 0..self.ncols {
				self.draw_fbc_row(ncol, nrow as u16)
			}
		}
		self.stdout.flush();
	}
	
	fn draw_empty_col(&mut self, ncol: u16) {
		for nrow in 0..cmp::min(self.turn, self.maxrow) {
			let goto = cursor::Goto(ncol*(self.wlen as u16 + 1) + 2, nrow + 2);
			write!(self.stdout, "{}{}", goto, self.empty_string);
		}
	}

	pub fn start(&mut self) {
		let termsz = terminal_size().ok();
		self.width = termsz.map(|(w,_)| w).unwrap();
		self.height = termsz.map(|(_,h)| h).unwrap();
		self.maxrow = self.height - 3;

		let mut cont = true;
		let mut menu = true;
		
		while cont {
			self.draw_base();
			if menu {self.menu_screen()}

			self.ncols = (self.width - 1) / (self.wlen + 1) as u16;
			self.nrows = (self.nwords - 1) / self.ncols + 1;
			self.empty_string = String::new();
			for _ in 0..self.wlen {
				self.empty_string.push(' ');
			}
			
			self.ndone = 0;
			self.turn = 0;
			self.scroll = 0;
			self.answers = self.awb
				.pick(&mut rand::thread_rng(), self.nwords.into());
			self.cols = self.answers.iter()
				.map(|ans| FeedbackCol::new(*ans))
				.collect();

			self.draw_base();
			let limit = self.nwords + NEXTRA;
			let mut quit = false;
			let mut guess = String::new();

			while self.turn < limit && self.ndone < self.nwords as u16 && !quit {
				write!(self.stdout, "{}",
							 cursor::Goto(guess.len() as u16 + 2, self.height-1));
				self.stdout.flush();
				match self.stdin.next().unwrap().unwrap() {
					Key::Char(c) => if 'a' <= c && c <= 'z' {
						let c2 = (c as u8 - 32) as char;
						guess.push(c2);
						write!(self.stdout, "{}{}",
									 cursor::Goto(guess.len() as u16 + 1,
																self.height-1),
									 c2.to_string());
					} Key::Backspace => {
						guess.pop();
						write!(self.stdout, "{} ",
									 cursor::Goto(guess.len() as u16 + 2,
																self.height-1));
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

				if guess.len() == self.wlen.into() {
					let gw = Word::from(guess).unwrap();
					if self.gwb.contains(gw) {
						if self.turn == 0 {self.t_start = Instant::now()}
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
							self.redraw_fbcols();
						} else if self.turn <= self.maxrow {
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
			(cont, menu) = self.end_screen();
		}
		write!(self.stdout, "{}{}{}", clear::All, style::Reset, cursor::Goto(1,1));
	}
}