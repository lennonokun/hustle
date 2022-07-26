use std::fmt::Display;
use std::rc::Rc;
use std::cmp;

use cursive::Cursive;
use cursive::view::Nameable;
use cursive::views::*;
use cursive::theme::{Color, BaseColor, Style, ColorStyle, Effect};
use cursive::traits::*;
use cursive::event::{Event, EventResult, Key};
use cursive::direction::Direction;
use cursive::{Printer, Vec2};
use cursive::reexports::enumset::{EnumSet,enum_set};
use cursive::view::CannotFocus;

fn make_style(color: ColorStyle, effects: EnumSet<Effect>) -> Style {
  Style {color, effects}
}

#[derive(Debug)]
pub struct EditView {
  content: Rc<String>,
  cursor: usize,
  size: Vec2,
  style1: Style,
  style2: Style,
}

impl EditView {
  pub fn new() -> Self {
    Self {
      content: Rc::new(String::new()),
      cursor: 0,
      size: Vec2::zero(),
      style1: make_style(ColorStyle::highlight(), enum_set!()),
      style2: make_style(ColorStyle::highlight_inactive(), enum_set!()),
    }
  }

  pub fn get_content(&self) -> Rc<String> {
    Rc::clone(&self.content)
  }

  pub fn get_style(&self, printer: &Printer) -> Style {
    if printer.focused {
      self.style1
    } else {
      self.style2
    }
  }
}

impl View for EditView {
  fn layout(&mut self, size: Vec2) {
    self.size = size;
  }

  fn draw(&self, printer: &Printer) {
    printer.with_style(self.get_style(printer), |printer| {
      // draw background + content
      printer.print_hline((0,0), self.size.x, "_");
      printer.print((0,0), &self.content);

      // draw cursor
      if printer.focused {
        let c_cursor: char = self.content.chars()
          .nth(self.cursor).unwrap_or('_');
        printer.with_effect(Effect::Reverse, |printer| {
          printer.print((self.cursor, 0), &c_cursor.to_string());
        });
      }
    });
  }

  fn on_event(&mut self, event: Event) -> EventResult {
    match event {
      Event::Char(c) => {
        Rc::make_mut(&mut self.content).insert(self.cursor, c);
        self.cursor += 1;
        return EventResult::Consumed(None);
      } Event::Key(Key::Backspace) => {
        Rc::make_mut(&mut self.content).pop();
        if self.cursor > 0 { self.cursor -= 1}
        eprintln!("{:?} {:?}", self.content, self.cursor);
        return EventResult::Consumed(None);
      } Event::CtrlChar('w') | Event::Ctrl(Key::Backspace) => {
        Rc::make_mut(&mut self.content).clear();
        self.cursor = 0;
        return EventResult::Consumed(None);
      } Event::Key(Key::Left) => {
        self.cursor = cmp::max(self.cursor-1, 0);
        return EventResult::Consumed(None);
      } Event::Key(Key::Right) => {
        self.cursor = cmp::min(self.cursor+1, self.content.len());
        return EventResult::Consumed(None);
      } _ => {
        return EventResult::Ignored;
      }
    }
  }

  fn take_focus(&mut self, source: Direction) -> Result<EventResult, CannotFocus> {
    Ok(EventResult::Consumed(None))
  }
}
