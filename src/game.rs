use rand::prelude::*;
use std::io::{self, Read, Write};
use termion::{clear, cursor, color, style};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

use crate::ds::*;

const XCODE: &str = "\x1b[0m";

struct AnswerCol {
	ans: Word,
	disp: Vec<String>,
	done: bool
}

impl AnswerCol {
	fn new(ans: Word) -> Self {
		Self {
			ans: ans,
			disp: Vec::<String>::new(),
			done: false
		}
	}

	// adds to disp and returns colored word
	fn guess(&mut self, gw: Word) -> Option<&String> {
		if self.done {return None}
		let fb = Feedback::from(gw, self.ans);
		let mut s = String::new();
		for i in 0..NLETS {
			if fb.get_g(i) {
				s += &format!("{}", color::Fg(color::Green));
			} else if fb.get_y(i) {
				s += &format!("{}", color::Fg(color::Yellow));
			} else {
				s += &format!("{}", color::Fg(color::Blue));
			}
			s.push(gw.data[i]);
		};
		s.push_str(XCODE);
		self.disp.push(s);
		self.done = gw == self.ans;
		return self.disp.last();
	}
}

pub struct Game<'a, R:Read, W:Write> {
	gws: &'a WSet,
	aws: &'a WArr,
	i: i32,
	ndone: i32,
	cols: Vec<AnswerCol>,
	stdin: R,
	stdout: W,
}

impl <'a, R:Read, W:Write> Game<'a, R, W> {
	pub fn new(gws: &'a WSet, aws: &'a WArr, stdin: R, stdout: W) -> Self {
		Game {
			gws: gws,
			aws: aws,
			i: 0,
			ndone: 0,
			cols: Vec::new(),
			stdin: stdin,
			stdout: stdout
		}
	}
	
	// TODO: add removing cleared cols?
	pub fn start(&mut self, n: usize) {
		self.cols = self.aws
			.choose_multiple(&mut rand::thread_rng(), n)
			.cloned()
			.map(|ans| AnswerCol::new(ans))
			.collect();

		write!(self.stdout, "{}{}", clear::All, cursor::Goto(1,1));
		self.stdout.flush().unwrap();

		while self.i < (n as i32 + 5) && self.ndone < n as i32 {
			let s = self.stdin.read_line().expect("hey!").expect("hey2!");
			if let Some(w) = Word::from_str(&s) {
				if self.gws.contains(&w) {
					for (j, col) in &mut self.cols.iter_mut().enumerate() {
						match col.guess(w) {
							None => self.ndone += 1,
							Some(row) => {
								write!(self.stdout, "{}{}",
											cursor::Goto((6*j+1) as u16, (self.i+1) as u16),
											row);
							}
						}
					}
					write!(self.stdout, "\n");
					self.i += 1;
				}	
			}
			self.stdout.flush().unwrap();
			// go back to guessing zone
			write!(self.stdout, "{}{}",
						 cursor::Goto(1, (self.i+1) as u16),
						 clear::AfterCursor);
		}

		if self.ndone == n as i32 {
			writeln!(self.stdout,"won in {}!", self.i);
		} else {
			writeln!(self.stdout, "answers were:");
			for (i, col) in self.cols.iter().enumerate() {
				writeln!(self.stdout, "{}. {}", i, col.ans.to_string());
			}
		}
	}
}
