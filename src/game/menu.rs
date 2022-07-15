use cursive::Cursive;
use cursive::views::*;
use cursive::traits::*;
use cursive::event::{Event, Key};

use super::game::game_open;
use super::config::Config;

pub fn menu_open(s: &mut Cursive) {
  let menu_input = LinearLayout::vertical()
    .child(PaddedView::lrtb(0,0,1,1, TextView::new("HUSTLE").center()))
    .child(LinearLayout::horizontal()
           .child(TextView::new("nwords")
                  .fixed_width(10))
           .child(EditView::new()
                  .with_name("nwords")
                  .fixed_width(10)))
    .child(LinearLayout::horizontal()
           .child(TextView::new("wlen")
                  .fixed_width(10))
           .child(EditView::new()
                  .with_name("wlen")
                  .fixed_width(10)))
    .child(LinearLayout::horizontal()
           .child(TextView::new("wbank")
                  .fixed_width(10))
           .child(EditView::new()
                  .with_name("wbank")
                  .fixed_width(10)));

  let menu = Dialog::around(menu_input)
    .title("Menu")
    .button("Ok", menu_submit);

  s.add_layer(menu);
  s.add_global_callback(Key::Enter, menu_submit);
}

fn menu_submit(s: &mut Cursive) {
  let nwords = s.call_on_name(
    "nwords",
    |view: &mut EditView| view.get_content())
    .and_then(|a| a.parse::<usize>().ok());
  let wlen = s.call_on_name(
    "wlen",
    |view: &mut EditView| view.get_content())
    .and_then(|a| a.parse::<u8>().ok());
  let wbank = s.call_on_name(
    "wbank",
    |view: &mut EditView| view.get_content());
  if let (Some(nwords), Some(wlen), Some(wbank)) = (nwords, wlen, wbank) {
    s.pop_layer();
    game_open(s, wbank.to_string(), wlen, nwords);
  }
}

