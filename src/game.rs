use rand::prelude::*;
use std::io::{self, Read, Write};
use termion::{clear, cursor, color, style};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;

use crate::ds::*;

const XCODE: &str = "\x1b[0m";
const GCODE: &str = "\x1b[32m";
const YCODE: &str = "\x1b[33m";
const BCODE: &str = "\x1b[34m";

pub struct Game<'a, R:Read, W:Write> {
	gws: &'a WSet,
	aws: &'a WArr,
	i: i32,
	disp: String,
	ans: Word,
	done: bool,
	stdin: R,
	stdout: W,
}

fn color_word(gw: Word, fb: Feedback) -> String {
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
	s
}


impl <'a, R:Read, W:Write> Game<'a, R, W> {
	pub fn new(gws: &'a WSet, aws: &'a WArr, stdin: R, stdout: W) -> Self {
		Game {
			gws: gws,
			aws: aws,
			i: 0,
			disp: String::new(),
			ans: Word::from_str("aaaaa").unwrap(),
			done: false,
			stdin: stdin,
			stdout: stdout
		}
	}

	pub fn start(self: &mut Self) {
		self.ans = self.aws[rand::thread_rng().gen_range(0..NWORDS)];
		write!(self.stdout, "{}{}", clear::All, cursor::Goto(1,1));
		self.stdout.flush().unwrap();
		while self.i < 6 && !self.done {
			let s = self.stdin.read_line().expect("hey!").expect("hey2!");
			match Word::from_str(&s) {
				Some(w) => if self.gws.contains(&w) {
					let fb = Feedback::from(w, self.ans);
					self.disp += &color_word(w, fb);
					self.disp += "\n";
					self.done = fb.is_correct();
					self.i += 1;
				}
				None => {}
			}
			write!(self.stdout, "{}{}{}{}", clear::All, cursor::Goto(1,1),
						 self.disp, cursor::Goto(1,(self.i+1) as u16));
		}

		if self.done {
			writeln!(self.stdout,"won in {}!", self.i);
		} else {
			writeln!(self.stdout, "word was {}", self.ans.to_string()); 
		}
	}
}
