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
  size: Vec2,
  style1: Style,
  style2: Style,
}

impl EditView {
  pub fn new() -> Self {
    Self {
      content: Rc::new(String::new()),
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
      printer.print_hline((0,0), self.size.x, "_");
      printer.print((0,0), &self.content);
    });
  }

  fn on_event(&mut self, event: Event) -> EventResult {
    match event {
      Event::Char(c) => {
        Rc::make_mut(&mut self.content).push(c);
        return EventResult::Consumed(None);
      } Event::Key(Key::Backspace) => {
        Rc::make_mut(&mut self.content).pop();
        return EventResult::Consumed(None);
      } Event::CtrlChar('w') | Event::Ctrl(Key::Backspace) => {
        Rc::make_mut(&mut self.content).clear();
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
