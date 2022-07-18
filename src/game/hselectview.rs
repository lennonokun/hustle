use std::fmt::Display;
use std::rc::Rc;
use std::cmp;

use cursive::Cursive;
use cursive::view::Nameable;
use cursive::views::*;
use cursive::theme::{Color, BaseColor, Style, ColorStyle, Effect};
use cursive::reexports::enumset::{EnumSet,enum_set};
use cursive::traits::*;
use cursive::event::{Event, EventResult, Key};
use cursive::direction::Direction;
use cursive::{Printer, Vec2, Rect};
use cursive::align::*;
use cursive::view::CannotFocus;

// todo make util.rs
fn make_style(color: ColorStyle, effects: EnumSet<Effect>) -> Style {
  Style {color, effects}
}

#[derive(Debug)]
struct Item<T> {
  pub label: String,
  pub value: Rc<T>,
}

impl<T> Item<T> {
  fn new(label: String, value: T) -> Self {
    let value = Rc::new(value);
    Item { label, value }
  }
}

#[derive(Debug)]
pub struct HSelectView<T> {
  items: Vec<Item<T>>,
  index: usize, 
  size: Vec2,
  style1: Style,
  style2: Style,
  align: Align,
}

impl<T> HSelectView<T> {
  pub fn new() -> Self {
    Self {
      items: Vec::new(),
      index: 0,
      size: Vec2::zero(),
      style1: make_style(ColorStyle::highlight(), enum_set!()),
      style2: make_style(ColorStyle::highlight_inactive(), enum_set!()),
      align: Align::center(),
    }
  }

  pub fn add_item(&mut self, label: String, value: T) {
    self.items.push(Item::new(label, value));
  }

  // todo is cloning bad? have to clone in draw
  pub fn selected_label(&self) -> Option<String> {
    self.items.get(self.index).map(|item| item.label.clone())
  }

  pub fn selection(&self) -> Option<Rc<T>> {
    eprintln!("selection!");
    self.items.get(self.index).map(|item| item.value.clone())
  }

  pub fn get_style(&self, printer: &Printer) -> Style {
    if printer.focused {
      self.style1
    } else {
      self.style2
    }
  }
}

impl<T: 'static> View for HSelectView<T> {
  // TODO add needs redraw
  
  fn layout(&mut self, size: Vec2) {
    self.size = size;
  }

  fn draw(&self, printer: &Printer) {
    if let Some(label) = self.selected_label() {
      printer.print((0,0), "< ");
      printer.print((self.size.x-2,0), " >");

      let w = self.size.x-4;
      printer.with_style(self.get_style(printer), |printer| {
        printer.print_hline((2,0), w, " ");
        let printer = printer.shrinked_centered((4,0));
        let x = self.align.h.get_offset(label.len(), w);
        printer.print((x,0), &label);
      });
    }
  }

  fn on_event(&mut self, event: Event) -> EventResult {
    match event {
      Event::Key(Key::Left) => {
        self.index = (self.index + 1) % self.items.len();
      } Event::Key(Key::Right) => {
        self.index = (self.index + self.items.len() - 1) % self.items.len();
      } _ => {
        return EventResult::Ignored;
      }
    }
    EventResult::Consumed(None)
  }

  fn take_focus(&mut self, source: Direction) -> Result<EventResult, CannotFocus> {
    Ok(EventResult::Consumed(None))
  }
}
