#![allow(unused)]

#[cfg(feature="play")]
extern crate lazy_static;

pub mod command;
pub mod util;
#[cfg(feature = "play")]
pub mod game;
#[cfg(feature = "solve")]
pub mod solve;
#[cfg(feature = "gen")]
pub mod analysis;
