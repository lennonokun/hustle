#![allow(unused)]

extern crate lazy_static;

pub mod util;
pub mod command;
#[cfg(feature = "gen")]
pub mod analysis;
#[cfg(feature = "solve")]
pub mod solve;
#[cfg(feature = "play")]
pub mod game;
