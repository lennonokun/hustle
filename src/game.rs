use rand::prelude::*;
use std::io::{stdin,stdout,Write};
use crate::ds::*;

const XCODE: &str = "\x1b[0m";
const GCODE: &str = "\x1b[32m";
const YCODE: &str = "\x1b[33m";
const BCODE: &str = "\x1b[34m";
const LNCLR: &str = "\x1b[2K";
const CRSUP: &str = "x1b[1A";
const CLEAR: &str = "\u{001b}c";

fn color_word(gw: Word, fb: Feedback) -> String {
	let mut s = String::new();
	for i in 0..NLETS {
		if fb.get_g(i) {
			s.push_str(GCODE);
		} else if fb.get_y(i) {
			s.push_str(YCODE);
		} else {
			s.push_str(BCODE);
		}
		s.push(gw.data[i]);
	};
	s.push_str(XCODE);
	s
}

pub fn play(gws: &WSet, aws: &WArr) {
	let aw = aws[rand::thread_rng().gen_range(0..NWORDS)];
	let mut disp = String::new();
	print!("{}", CLEAR);
	stdout().flush().unwrap();
	let mut i = 0;
	let mut done = false;
	while i < 6 && !done {
		let mut s = String::new();
		stdin().read_line(&mut s).expect("hey!");
		let s2 = s.strip_suffix("\n").unwrap();
		match Word::from_str(&s2) {
			Some(w) => if gws.contains(&w) {
				let fb = Feedback::from(w, aw);
				disp += &color_word(w, fb);
				disp += "\n";
				done = fb.is_correct();
				i += 1;
			}
			None => {}
		}
		// print!("{}", CRSUP);
		print!("{}{}", CLEAR, disp);
	}
	if done {
		println!("won in {}!", i);
	} else {
		println!("word was {}", aw.to_string()); 
	}
}
