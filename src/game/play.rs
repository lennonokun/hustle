use cursive::event::Event;

use super::menu::open_menu;
use super::config::CONFIG;

pub fn play() {
  let mut siv = cursive::default();
  siv.set_theme(CONFIG.theme.clone());
  siv.set_fps(20);
  siv.add_global_callback(Event::CtrlChar('q'), |s| s.quit());

  open_menu(&mut siv);
  siv.run()
}
