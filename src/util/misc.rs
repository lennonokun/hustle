// constants
pub const DEFTURNS: u32 = 6;
pub const NEXTRA: usize = 5;
pub const DEFWLEN: u8 = 5;
pub const MAXWLEN: usize = 11;

pub const DEFWBP: &'static str = "/usr/share/hustle/bank1.csv";
pub const DEFWBP2: &'static str = "/usr/share/hustle/bank2.csv";
pub const DEFHDP: &'static str = "/usr/share/hustle/happrox.csv";
pub const DEFLDP: &'static str = "/usr/share/hustle/lbounds.csv";

pub fn is_alpha(c: char) -> bool {
  ('a'..='z').contains(&c) || ('A'..='Z').contains(&c)
}

// assumes c is alpha 
pub fn upper(c: char) -> char {
  if ('a'..='z').contains(&c) {
    (c as u8 + b'A' - b'a') as char
  } else {
    c
  }
}
