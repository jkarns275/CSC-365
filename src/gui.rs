extern crate rustbox;
use LOG_FILE;
use std::io::Write;
use rustbox::{Color, RustBox};
use rustbox::Key;

/// A trait containing all the methods needed for a GUI component
pub trait Drawable {
    fn draw(&self, rustbox: &RustBox);
    fn highlight_draw(&self, rustbox: &RustBox);
    fn clear(&self, rustbox: &RustBox);
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn x(&self) -> usize;
    fn y(&self) -> usize;
    fn handle_input(&mut self, k: Key);
    fn component(self, f: Box<FnMut(&mut Self, Key) -> ()>) -> Component<Self> where Self: Sized;
    fn enable(&mut self);
    fn disable(&mut self);
    fn is_enabled(&self) -> bool;

    /// This method is used to read from input components
    fn to_string(&self) -> String;
}

/// Just a box with an x, y, width, and height
pub struct GuiBox {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    fg: Color,
    bg: Color,
    hl: Color,
    enabled: bool,
}

/// Implementation of the Drawable trait for GuiBox
impl Drawable for GuiBox {
    fn to_string(&self) -> String { "".to_string() }
    fn draw(&self, rustbox: &RustBox) {
        self.clear(rustbox);
        let hor = (0..self.width).map(|_| "─").collect::<String>();
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, self.fg, self.bg, &hor);
        rustbox.print(self.x, self.y + self.height - 1, rustbox::RB_BOLD, self.fg, self.bg, &hor);
        for i in self.y..self.y + self.height {
            rustbox.print(self.x, i, rustbox::RB_BOLD, self.fg, Color::Default, "│");
            rustbox.print(self.x + self.width - 1, i, rustbox::RB_BOLD, self.fg, Color::Default, "│");
        }
        rustbox.print(self.x + self.width - 1, self.y + self.height - 1, rustbox::RB_BOLD, self.fg, self.bg, "╯");
        rustbox.print(self.x + self.width - 1, self.y, rustbox::RB_BOLD, self.fg, self.bg, "╮");
        rustbox.print(self.x, self.y + self.height - 1, rustbox::RB_BOLD, self.fg, self.bg, "╰");
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, self.fg, self.bg, "╭");
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        self.clear(rustbox);
        let hor = (0..self.width).map(|_| "─").collect::<String>();
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, self.hl, self.bg, &hor);
        rustbox.print(self.x, self.y + self.height - 1, rustbox::RB_BOLD, self.hl, self.bg, &hor);
        for i in self.y..self.y + self.height {
            rustbox.print(self.x, i, rustbox::RB_BOLD, self.hl, Color::Default, "│");
            rustbox.print(self.x + self.width - 1, i, rustbox::RB_BOLD, self.hl, Color::Default, "│");
        }
        rustbox.print(self.x + self.width - 1, self.y + self.height - 1, rustbox::RB_BOLD, self.hl, self.bg, "╯");
        rustbox.print(self.x + self.width - 1, self.y, rustbox::RB_BOLD, self.hl, self.bg, "╮");
        rustbox.print(self.x, self.y + self.height - 1, rustbox::RB_BOLD, self.hl, self.bg, "╰");
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, self.hl, self.bg, "╭");
    }

    fn clear(&self, rustbox: &RustBox) {
        let s = (0..self.width).map(|_| " ").collect::<String>();
        for i in 0..self.height {
            rustbox.print(self.x, self.y + i, rustbox::RB_BOLD, Color::Default, Color::Default, &s);
        }
    }

    fn component(self, f: Box<FnMut(&mut GuiBox, Key) -> ()>) -> Component<Self> {
        Component { comp: self, handler: f }
    }

    fn height(&self) -> usize { self.height }
    fn width(&self) -> usize { self.width }
    fn x(&self) -> usize { self.x }
    fn y(&self) -> usize { self.y }
    fn handle_input(&mut self, _: Key) {}
    fn enable(&mut self) { self.enabled = true; }
    fn disable(&mut self) { self.enabled = false; }
    fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiBox {
    pub fn new(x: usize, y: usize, width: usize, height: usize, fg: Color, bg: Color, hl: Color) -> GuiBox {
        GuiBox {
            x: x,
            y: y,
            height: height,
            width: width,
            fg: fg,
            bg: bg,
            hl: hl,
            enabled: true,
        }
    }
    pub fn new_default(x: usize, y: usize, width: usize, height: usize) -> GuiBox {
        GuiBox::new(x, y, width, height, Color::White, Color::Default, Color::Green)
    }
}

/// A selector component. It has a list of elements that can be scrolled through.
pub struct GuiSelection {
    choices: Vec<String>,
    cur_choice: usize,
    bg: Color,
    fg: Color,
    selected_bg: Color,
    selected_fg: Color,
    b: GuiBox,
    pub enabled: bool,
}

/// Util min method
fn min(x: usize, y: usize) -> usize { if x > y { y } else { x } }
/// Util max method
fn max(x: usize, y: usize) -> usize { if x < y { y } else { x } }

impl Drawable for GuiSelection {
    /// Returns the content of the current selection, spaces and everything included.
    fn to_string(&self) -> String { self.choices[self.cur_choice].clone() }

    fn draw(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);

        self.b.draw(rustbox);

        if self.choices.len() == 0 { return; }

        let ref s = self.choices[self.cur_choice];
        let end = min(self.b.width - 2, s.len());
        rustbox.print(self.b.x + 3, self.b.y + 1, rustbox::RB_BOLD, self.selected_fg, self.selected_bg, &s[..end]);
        if self.choices.len() == 1 { return; }

        for i in 1..min(self.b.height - 2, self.choices.len()) {
            let ref s = self.choices[(self.cur_choice + i) % self.choices.len()];
            let end = min(self.b.width - 4, s.len());
            rustbox.print(self.b.x + 3, self.b.y + i + 1, rustbox::RB_BOLD, self.fg, self.bg, &s[..end]);
        }
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);

        self.b.highlight_draw(rustbox);

        if self.choices.len() == 0 { return; }

        let ref s = self.choices[self.cur_choice];
        let end = min(self.b.width - 2, s.len());
        rustbox.print(self.b.x + 3, self.b.y + 1, rustbox::RB_BOLD, self.selected_fg, self.selected_bg, &s[..end]);
        rustbox.print(self.b.x + 1, self.b.y + 1, rustbox::RB_BOLD, self.selected_fg, self.selected_bg, "> ");
        if self.choices.len() == 1 { return; }

        for i in 1..min(self.b.height - 2, self.choices.len()) {
            let ref s = self.choices[(self.cur_choice + i) % self.choices.len()];
            let end = min(self.b.width - 4, s.len());
            rustbox.print(self.b.x + 3, self.b.y + i + 1, rustbox::RB_BOLD, self.fg, self.bg, &s[..end]);
        }
    }

    fn clear(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);
    }

    fn component(self, f: Box<FnMut(&mut GuiSelection, Key) -> ()>) -> Component<GuiSelection> {
        Component { comp: self, handler: f }
    }

    fn height(&self)    -> usize { self.b.height }
    fn width(&self)     -> usize { self.b.width }
    fn x(&self)         -> usize { self.b.x }
    fn y(&self)         -> usize { self.b.y }
    fn handle_input(&mut self, _: Key) {}
    fn enable(&mut self) { self.enabled = true; }
    fn disable(&mut self) { self.enabled = false; }
    fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiSelection {
    pub fn new( x: usize, y: usize, width: usize, height: usize,
                fg: Color, bg: Color, selected_fg: Color,
                selected_bg: Color, highlighted_color: Color, dat: Vec<String>) -> GuiSelection {
        GuiSelection {
            b: GuiBox::new(x, y, width, height, fg, bg, highlighted_color),
            fg: fg,
            bg: bg,
            selected_fg: selected_fg,
            selected_bg: selected_bg,
            choices: dat,
            cur_choice: 0,
            enabled: true
        }
    }
    pub fn new_default(x: usize, y: usize, width: usize, height: usize, dat: Vec<String>) -> GuiSelection {
        GuiSelection::new(x, y, width, height, Color::White, Color::Default, Color::Green, Color::Default, Color::Green, dat)
    }
    /// Selector can also be used as a button without me having to add a new struct.
    pub fn button(x: usize, y: usize, text: &str, fg: Color, bg: Color, border_color: Color) -> GuiSelection {
        let mut x = GuiSelection::new(x, y, text.len() + 6, 3, fg, bg, Color::Green, Color::Default, Color::Green, vec![text.to_string() + "  "]);
        x.b.fg = border_color;
        x
    }
    pub fn button_default(x: usize, y: usize, text: &str) -> GuiSelection {
        GuiSelection::button(x, y, text, Color::White, Color::Default, Color::White)
    }
    pub fn up(&mut self) {
        self.cur_choice = (self.cur_choice + 1) % self.choices.len();
    }
    pub fn down(&mut self) {
        if self.cur_choice == 0 { self.cur_choice = self.choices.len() - 1; }
        else { self.cur_choice = (self.cur_choice - 1) % self.choices.len(); }
    }
}

/// Contains a Drawable struct, and a on-input function for it.
pub struct Component<T: Drawable> {
    pub comp: T,
    pub handler: Box<FnMut(&mut T, Key) -> ()>,
}

impl<T: Drawable> Drawable for Component<T> {
    fn to_string(&self) -> String { self.comp.to_string() }

    fn draw(&self, rustbox: &RustBox) {
        self.comp.draw(rustbox);
    }
    fn highlight_draw(&self, rustbox: &RustBox) {
        self.comp.highlight_draw(rustbox);
    }
    fn clear(&self, rustbox: &RustBox) {
        self.comp.clear(rustbox);
    }
    fn width(&self) -> usize {
        self.comp.width()
    }
    fn height(&self) -> usize {
        self.comp.height()
    }
    fn x(&self) -> usize {
        self.comp.x()
    }
    fn y(&self) -> usize {
        self.comp.y()
    }
    fn handle_input(&mut self, k: Key) {
        (self.handler)(&mut self.comp, k)
    }
    fn component(self, f: Box<FnMut(&mut Component<T>, Key) -> ()>) -> Component<Component<T>> { Component { comp: self, handler: f } }
    fn enable(&mut self) { self.comp.enable() }
    fn disable(&mut self) { self.comp.disable() }
    fn is_enabled(&self) -> bool { self.comp.is_enabled() }
}

/// Cotnains multiple Drawable components, with one "selected."
pub struct Container {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub components: Vec<Box<Drawable>>,
    pub selected: i32,
    pub enabled: bool,
}

impl Drawable for Container {
    fn to_string(&self) -> String { self.components[self.selected as usize].to_string() }

    fn draw(&self, rustbox: &RustBox) {
        if self.components.len() == 0 || !self.enabled { return; }
        for i in 0..self.components.len() {
            if self.components[i].is_enabled() {
                self.components[i].draw(rustbox);
            } else {
                self.components[i].clear(rustbox);
            }
        }
        if self.selected != -1 && (self.selected as usize) < self.components.len() {
            let ind = self.selected;
            if self.components[ind as usize].is_enabled() {
                self.components[ind as usize].highlight_draw(rustbox);
            } else {
                self.components[ind as usize].clear(rustbox);
            }
        }
    }

    fn clear(&self, rustbox: &RustBox) {
        let s = (0..self.width).map(|_| " ").collect::<String>();
        for i in 0..self.height {
            rustbox.print(self.x, self.y + i, rustbox::RB_BOLD, Color::Default, Color::Default, &s);
        }
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        if self.components.len() == 0 || !self.enabled { return; }
        for i in 0..self.components.len() {
            if self.components[i].is_enabled() {
                self.components[i].draw(rustbox);
            } else {
                self.components[i].clear(rustbox);
            }
        }
        if self.selected != -1 && (self.selected as usize) < self.components.len() {
            let ind = self.selected;
            if self.components[ind as usize].is_enabled() {
                self.components[ind as usize].highlight_draw(rustbox);
            } else {
                self.components[ind as usize].clear(rustbox);
            }
        }
    }

    fn height(&self) -> usize { self.height }
    fn width(&self) -> usize { self.width }
    fn x(&self)         -> usize { self.x }
    fn y(&self)         -> usize { self.y }
    fn handle_input(&mut self, k: Key) {
        if self.components.len() > 0 && self.selected > -1 {
            self.components[self.selected as usize].handle_input(k);
        }
    }
    fn component(self, f: Box<FnMut(&mut Container, Key) -> ()>) -> Component<Container> {
        Component { comp: self, handler: f }
     }

     fn enable(&mut self) { self.enabled = true; }
     fn disable(&mut self) { self.enabled = false; }
     fn is_enabled(&self) -> bool { self.enabled }
}

impl Container {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Container {
        Container {
            x: x,
            y: y,
            width: width,
            height: height,
            components: vec![],
            enabled: true,
            selected: -1,
        }
    }
    pub fn add(&mut self, element: Box<Drawable>) -> Option<usize> {
        if self.fits(&element) {
            self.components.push(element);
            if self.selected == -1 {
                self.selected = 0;
            }
            Some(self.components.len() - 1)
        } else {
            None
        }
    }
    pub fn fits(&self, element: &Box<Drawable>) -> bool {
        self.x <= element.x() && self.x + self.width() >= element.x() + element.width() &&
        self.y <= element.y() && self.y + self.height() >= element.y() + element.height()
    }
    pub fn next(&mut self) -> Option<usize> {
        if self.components.len() == 0 {
            None
        } else {
            self.selected = ((self.selected + 1) as usize % self.components.len()) as i32;
            Some(self.selected as usize)
        }
    }
}

/// Just like container, except it only displays the selected component
pub struct GuiMux {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub components: Vec<Box<Drawable>>,
    pub selected: i32,
    pub enabled: bool,
}

impl Drawable for GuiMux {
    fn to_string(&self) -> String { "".to_string() }

    fn draw(&self, rustbox: &RustBox) {
        if self.components.len() == 0 || !self.enabled { return; }
        for i in 0..self.components.len() {
            self.components[i].clear(rustbox);
        }
        if self.selected != -1 && (self.selected as usize) < self.components.len() {
            let ind = self.selected;
            if self.components[ind as usize].is_enabled() {
                self.components[ind as usize].draw(rustbox);
            }
        }
    }

    fn clear(&self, rustbox: &RustBox) {
        let s = (0..self.width).map(|_| " ").collect::<String>();
        for i in 0..self.height {
            rustbox.print(self.x, self.y + i, rustbox::RB_BOLD, Color::Default, Color::Default, &s);
        }
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        for element in self.components.iter() {
            element.highlight_draw(rustbox);
        }
    }

    fn height(&self) -> usize { self.height }
    fn width(&self) -> usize { self.width }
    fn x(&self)         -> usize { self.x }
    fn y(&self)         -> usize { self.y }
    fn handle_input(&mut self, k: Key) {
        if self.components.len() > 0 && self.selected > -1 {
            self.components[self.selected as usize].handle_input(k);
        }
    }
    fn component(self, f: Box<FnMut(&mut GuiMux, Key) -> ()>) -> Component<GuiMux> {
        Component { comp: self, handler: f }
     }

     fn enable(&mut self) { self.enabled = true; }
     fn disable(&mut self) { self.enabled = false; }
     fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiMux {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> GuiMux {
        GuiMux {
            x: x,
            y: y,
            width: width,
            height: height,
            components: vec![],
            enabled: true,
            selected: -1,
        }
    }
    pub fn add(&mut self, element: Box<Drawable>) -> Option<usize> {
        if self.fits(&element) {
            self.components.push(element);
            if self.selected == -1 {
                self.selected = 0;
            }
            Some(self.components.len() - 1)
        } else {
            None
        }
    }
    pub fn fits(&self, element: &Box<Drawable>) -> bool {
        self.x <= element.x() && self.x + self.width() >= element.x() + element.width() &&
        self.y <= element.y() && self.y + self.height() >= element.y() + element.height()
    }
    pub fn next(&mut self) -> Option<usize> {
        if self.components.len() == 0 {
            None
        } else {
            self.selected = ((self.selected + 1) as usize % self.components.len()) as i32;
            Some(self.selected as usize)
        }
    }
}

/// Text with a box around it.
pub struct GuiTextBox {
    pub text: String,
    enabled: bool,
    b: GuiBox,
    fg: Color,
    bg: Color,
    hl: Color
}

impl Drawable for GuiTextBox {
    fn to_string(&self) -> String { "".to_string() }

    fn draw(&self, rustbox: &RustBox) {
        self.b.draw(rustbox);
        for i in 0..(self.b.height - 2) {
            if (self.b.width - 2) * i >= self.text.len() { break; }
            let m = if (self.b.width - 2)*(i + 1) > self.text.len() { self.text.len() } else { (self.b.width - 2) * (i + 1)};
            rustbox.print(self.b.x + 1, self.b.y + i + 1, rustbox::RB_BOLD, self.fg, self.bg, &self.text[(self.b.width - 2) * i .. m]);
        }
    }

    fn clear(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        self.b.highlight_draw(rustbox);
        for i in 0..(self.b.height - 2) {
            if (self.b.width - 2) * i >= self.text.len() { break; }
            let m = if (self.b.width - 2)*(i + 1) > self.text.len() { self.text.len() } else { (self.b.width - 2) * (i + 1)};
            rustbox.print(self.b.x + 1, self.b.y + i + 1, rustbox::RB_BOLD, self.hl, self.bg, &self.text[(self.b.width - 2) * i .. m]);
        }
    }

    fn height(&self) -> usize { self.b.height }
    fn width(&self) -> usize { self.b.width }
    fn x(&self)         -> usize { self.b.x }
    fn y(&self)         -> usize { self.b.y }
    fn handle_input(&mut self, _: Key) { }
    fn component(self, f: Box<FnMut(&mut GuiTextBox, Key) -> ()>) -> Component<GuiTextBox> {
        Component { comp: self, handler: f }
     }

     fn enable(&mut self) { self.enabled = true; }
     fn disable(&mut self) { self.enabled = false; }
     fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiTextBox {
    pub fn new(x: usize, y: usize, width: usize, text: &str) -> GuiTextBox {
        GuiTextBox {
            b: GuiBox::new(x, y, width, text.len() / (width - 2) + 3, Color::White, Color::Default, Color::Green),
            text: text.to_string(),
            enabled: true,
            fg: Color::White,
            bg: Color::Default,
            hl: Color::Green,
        }
    }

    pub fn new_string(x: usize, y: usize, width: usize, text: String) -> GuiTextBox {
        GuiTextBox {
            b: GuiBox::new(x, y, width, text.len() / (width - 2) + 3, Color::White, Color::Default, Color::Green),
            text: text,
            enabled: true,
            fg: Color::White,
            bg: Color::Default,
            hl: Color::Green,
        }
    }


    pub fn update_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.b.height = text.len() / (self.b.width - 2) + 3;
    }
}

/// Just text.
pub struct GuiText {
    pub text: String,
    x: usize,
    y: usize,
    enabled: bool,
    fg: Color,
    bg: Color,
    hl: Color
}

impl Drawable for GuiText {
    fn to_string(&self) -> String { "".to_string() }

    fn draw(&self, rustbox: &RustBox) {
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, self.fg, self.bg, &self.text);
    }

    fn clear(&self, rustbox: &RustBox) {
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, Color::Default, Color::Default, &self.text);
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        rustbox.print(self.x, self.y, rustbox::RB_BOLD, self.hl, self.bg, &self.text);
    }

    fn height(&self) -> usize { 1 }
    fn width(&self) -> usize { self.text.len() }
    fn x(&self)         -> usize { self.x }
    fn y(&self)         -> usize { self.y }
    fn handle_input(&mut self, _: Key) { }
    fn component(self, f: Box<FnMut(&mut GuiText, Key) -> ()>) -> Component<GuiText> {
        Component { comp: self, handler: f }
     }

     fn enable(&mut self) { self.enabled = true; }
     fn disable(&mut self) { self.enabled = false; }
     fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiText {
    pub fn new(x: usize, y: usize, text: &str) -> GuiText {
        GuiText {
            x: x,
            y: y,
            text: text.to_string(),
            enabled: true,
            fg: Color::White,
            bg: Color::Default,
            hl: Color::Green,
        }
    }

    pub fn new_string(x: usize, y: usize, text: String) -> GuiText {
        GuiText {
            x: x,
            y: y,
            text: text,
            enabled: true,
            fg: Color::White,
            bg: Color::Default,
            hl: Color::Green,
        }
    }
}

pub struct GuiTextInput {
    pub text: String,
    enabled: bool,
    b: GuiBox,
    fg: Color,
    bg: Color,
    hl: Color
}

impl Drawable for GuiTextInput {
    fn to_string(&self) -> String { self.text.to_string() }

    fn draw(&self, rustbox: &RustBox) {
        self.b.draw(rustbox);
        for i in 0..(self.b.height - 2) {
            if (self.b.width - 2) * i >= self.text.len() { break; }
            let m = if (self.b.width - 2)*(i + 1) > self.text.len() { self.text.len() } else { (self.b.width - 2) * (i + 1)};
            rustbox.print(self.b.x + 1, self.b.y + i + 1, rustbox::RB_BOLD, self.fg, self.bg, &self.text[(self.b.width - 2) * i .. m]);
        }
    }

    fn clear(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);
    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        self.b.highlight_draw(rustbox);
        for i in 0..(self.b.height - 2) {
            if (self.b.width - 2) * i >= self.text.len() { break; }
            let m = if (self.b.width - 2)*(i + 1) > self.text.len() { self.text.len() } else { (self.b.width - 2) * (i + 1)};
            rustbox.print(self.b.x + 1, self.b.y + i + 1, rustbox::RB_BOLD, self.hl, self.bg, &self.text[(self.b.width - 2) * i .. m]);
        }
    }

    fn height(&self) -> usize { self.b.height }
    fn width(&self) -> usize { self.b.width }
    fn x(&self)         -> usize { self.b.x }
    fn y(&self)         -> usize { self.b.y }
    fn handle_input(&mut self, k: Key) {
        match k {
            Key::Char(c) => {
                self.text.push(c);
            },
            Key::Backspace => {
                self.text.pop();
            },
            _ => {}
        }
    }
    fn component(self, f: Box<FnMut(&mut GuiTextInput, Key) -> ()>) -> Component<GuiTextInput> {
        Component { comp: self, handler: f }
     }

     fn enable(&mut self) { self.enabled = true; }
     fn disable(&mut self) { self.enabled = false; }
     fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiTextInput {
    pub fn new(x: usize, y: usize, width: usize, text: &str) -> GuiTextInput {
        GuiTextInput {
            b: GuiBox::new(x, y, width, text.len() / (width - 2) + 3, Color::White, Color::Default, Color::Green),
            text: text.to_string(),
            enabled: true,
            fg: Color::White,
            bg: Color::Default,
            hl: Color::Green,
        }
    }

    pub fn new_string(x: usize, y: usize, width: usize, text: String) -> GuiTextInput {
        GuiTextInput {
            b: GuiBox::new(x, y, width, text.len() / (width - 2) + 3, Color::White, Color::Default, Color::Green),
            text: text,
            enabled: true,
            fg: Color::White,
            bg: Color::Default,
            hl: Color::Green,
        }
    }

    pub fn update_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.b.height = text.len() / (self.b.width - 2) + 3;
    }
}

pub struct GuiSelection2D {
    choices: Vec<Vec<String>>,
    pub cur_choice_x: usize,
    pub cur_choice_y: usize,
    bg: Color,
    fg: Color,
    selected_bg: Color,
    selected_fg: Color,
    b: GuiBox,
    pub enabled: bool,
}
impl Drawable for GuiSelection2D {
    /// Returns the content of the current selection, spaces and everything included.
    fn to_string(&self) -> String { self.choices[self.cur_choice_x][self.cur_choice_y].clone() }

    fn draw(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);

        self.b.draw(rustbox);

        if self.choices.len() == 0 { return; }
        let ref s = self.choices[self.cur_choice_x][self.cur_choice_y];
        let end = min(self.b.width - 2, s.len());
        rustbox.print(self.b.x + 3, self.b.y + 1, rustbox::RB_BOLD, self.selected_fg, self.selected_bg, &s[..end]);
        if self.choices[self.cur_choice_x].len() == 1 { return; }

        for i in 1..min(self.b.height - 2, self.choices[self.cur_choice_x].len()) {
            let ref s = self.choices[self.cur_choice_x][(self.cur_choice_y + i) % self.choices[self.cur_choice_x].len()];
            let end = min(self.b.width - 4, s.len());
            rustbox.print(self.b.x + 3, self.b.y + i + 1, rustbox::RB_BOLD, self.fg, self.bg, &s[..end]);
        }

    }

    fn highlight_draw(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);

        self.b.highlight_draw(rustbox);

        if self.choices.len() == 0 { return; }

        let ref s = self.choices[self.cur_choice_x][self.cur_choice_y];
        let end = min(self.b.width - 2, s.len());
        rustbox.print(self.b.x + 3, self.b.y + 1, rustbox::RB_BOLD, self.selected_fg, self.selected_bg, &s[..end]);
        if self.choices[self.cur_choice_x].len() == 1 { return; }

        for i in 1..min(self.b.height - 2, self.choices[self.cur_choice_x].len()) {
            let ref s = self.choices[self.cur_choice_x][(self.cur_choice_y + i) % self.choices[self.cur_choice_x].len()];
            let end = min(self.b.width - 4, s.len());
            rustbox.print(self.b.x + 3, self.b.y + i + 1, rustbox::RB_BOLD, self.fg, self.bg, &s[..end]);
        }
    }

    fn clear(&self, rustbox: &RustBox) {
        self.b.clear(rustbox);
    }

    fn component(self, f: Box<FnMut(&mut GuiSelection2D, Key) -> ()>) -> Component<GuiSelection2D> {
        Component { comp: self, handler: f }
    }

    fn height(&self)    -> usize { self.b.height }
    fn width(&self)     -> usize { self.b.width }
    fn x(&self)         -> usize { self.b.x }
    fn y(&self)         -> usize { self.b.y }
    fn handle_input(&mut self, _: Key) {}
    fn enable(&mut self) { self.enabled = true; }
    fn disable(&mut self) { self.enabled = false; }
    fn is_enabled(&self) -> bool { self.enabled }
}

impl GuiSelection2D {
    pub fn new( x: usize, y: usize, width: usize, height: usize,
                fg: Color, bg: Color, selected_fg: Color,
                selected_bg: Color, highlighted_color: Color, dat: Vec<Vec<String>>) -> GuiSelection2D {
        GuiSelection2D {
            b: GuiBox::new(x, y, width, height, fg, bg, highlighted_color),
            fg: fg,
            bg: bg,
            selected_fg: selected_fg,
            selected_bg: selected_bg,
            choices: dat,
            cur_choice_x: 0,
            cur_choice_y: 0,
            enabled: true
        }
    }
    pub fn new_default(x: usize, y: usize, width: usize, height: usize, dat: Vec<Vec<String>>) -> GuiSelection2D {
        GuiSelection2D::new(x, y, width, height, Color::White, Color::Default, Color::Green, Color::Default, Color::Green, dat)
    }
    pub fn left(&mut self) {
        if self.cur_choice_x == 0 {
            self.cur_choice_x = self.choices.len() - 1;
        } else {
            self.cur_choice_x -= 1;
        }
        self.cur_choice_y = 0;
        log_file!("Good");
    }
    pub fn right(&mut self) {
        if self.cur_choice_x == self.choices.len() - 1 {
            self.cur_choice_x = 0;
        } else {
            self.cur_choice_x += 1;
        }
        self.cur_choice_y = 0;
        log_file!("Go OD");
    }
    pub fn up(&mut self) {
        let index = self.cur_choice_x;
        self.cur_choice_y = (self.cur_choice_y + 1) % self.choices[index].len();
    }
    pub fn down(&mut self) {
        let index = self.cur_choice_x;
        if self.cur_choice_y == 0 { self.cur_choice_y = self.choices[index].len() - 1; }
        else { self.cur_choice_y = (self.cur_choice_y - 1); }
    }
}
