use cursive::Cursive;
use cursive::view::Nameable;
use cursive::views::*;
use cursive::theme::{Color, BaseColor, ColorStyle, Effect};
use cursive::traits::*;
use cursive::event::{Event, EventResult, Key};
use cursive::direction::Direction;
use cursive::{Printer, Vec2};
use cursive::view::CannotFocus;

use crate::ds::*;
use super::config::Config;
use super::gameview::GameView;

pub fn game_open(s: &mut Cursive, wbp: String, wlen: u8, nwords: usize) {
  let wbp = "/usr/share/hustle/bank1.csv"; // TODO for now
  let mut fbview = GameView::new(wbp, wlen, nwords);

  s.add_fullscreen_layer(fbview);
}
