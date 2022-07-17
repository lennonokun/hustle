use cursive::Cursive;
use cursive::views::*;
use cursive::traits::*;
use cursive::event::{Event, Key};

use super::game::game_open;
use super::config::CONFIG;
use super::hselectview::HSelectView;

pub fn menu_open(s: &mut Cursive) {
  let mut bank_select = HSelectView::new();
  for (k,v) in CONFIG.word_banks.iter() {
    bank_select.add_item(k.to_string(), v.to_string());
  }

  let menu_input = LinearLayout::vertical()
    .child(PaddedView::lrtb(0,0,1,1, TextView::new("HUSTLE").center()))
    .child(LinearLayout::horizontal()
           .child(TextView::new("nwords")
                  .fixed_width(10))
           .child(EditView::new()
                  .with_name("nwords")
                  .fixed_width(15)))
    .child(LinearLayout::horizontal()
           .child(TextView::new("wlen")
                  .fixed_width(10))
           .child(EditView::new()
                  .with_name("wlen")
                  .fixed_width(15)))
    .child(LinearLayout::horizontal()
           .child(TextView::new("wbank")
                  .fixed_width(10))
           .child(bank_select
                  .with_name("wbank")
                  .fixed_width(15)));

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
  let wbn = s.call_on_name(
    "wbank",
    |view: &mut HSelectView<String>| view.selected_label());

  if let (Some(nwords), Some(wlen), Some(Some(wbn))) = (nwords, wlen, wbn) {
    s.pop_layer();
    game_open(s, &wbn, wlen, nwords);
  }
}

