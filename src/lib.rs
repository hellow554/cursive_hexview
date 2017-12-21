#![deny(missing_docs, missing_copy_implementations, trivial_casts, trivial_numeric_casts,
unsafe_code, unused_import_braces, unused_qualifications)]

//! A simple `HexView` for [cursive](https://crates.io/crates/cursive).
//!
//! It is meant to display a data of u8 and format them like e.g. hexdump does.
//! You can interact with the view with your keyboard.
//! Currently the following keys are implemented:
//!
//! | Key                                 | Action                                                                                                                                                                                                                                                 |
//! |-------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
//! | <kbd>&leftarrow;</kbd>              | Move the cursor to the next, left nibble. If already on the left edge of the view, the event will be "ignored", which means the outer view will handle the event (e.g. focus the view next to this one)                                                |
//! | <kbd>&rightarrow;</kbd>             | Move the cursor to the next, right nibble. If already on the right edge of the view, the event will be "ignored", which means the outer view will handle the event (e.g. focus the view next to this one)                                              |
//! | <kbd>&uparrow;</kbd>                | Move the cursor to the previous line. If already on the top edge of the view, the event will be "ignored", which means the outer view will handle the event (e.g. focus the view next to this one)                                                     |
//! | <kbd>&downarrow;</kbd>              | Move the cursor to the next line. If already on the bottom edge of the,view, the event will be "ignored", which means the outer view will handle the event (e.g. focus the view next to this one)                                                      |
//! | <kbd>Home</kbd>                     | Move the cursor to the beginning of the current line.                                                                                                                                                                                                  |
//! | <kbd>End</kbd>                      | Move the cursor to the end of the current line.                                                                                                                                                                                                        |
//! | <kbd>Shift</kbd> + <kbd>Home</kbd>  | Move the cursor to position (0 ,0) which means to the beginning of the view.                                                                                                                                                                           |
//! | <kbd>Shift</kbd> + <kbd>End</kbd>   | Move the cursor to the last nibble in the view.                                                                                                                                                                                                        |
//! | <kbd>+</kbd>                        | Increase the amount of data by one byte. It will be filled up with `0`.                                                                                                                                                                                |
//! | <kbd>-</kbd>                        | Decrease the amount of data by one. Any data that will leave the viewable area, will be permanantly lost.                                                                                                                                              |
//! | <kbd>0-9</kbd>, <kbd>a-f</kbd>      | Set the nibble under the cursor to the corresponding hex value. Note, that this is only available in the editable state, see [`DisplayState`](enum.DisplayState.html#Editable) and [`set_display_state`](struct.HexView.html#method.set_display_state) |


extern crate cursive;
extern crate itertools;

mod asciihelp;

use std::borrow::Borrow;
use std::cmp::min;

use cursive::{Printer, With};
use cursive::theme::{ColorStyle, Effect};
use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key, MouseEvent};
use cursive::view::View;
use cursive::vec::Vec2;
use cursive::traits::*;
use itertools::Itertools;

/// This enum is used for the [`set_display_state`](struct.HexView.html#method.set_display_state) method
/// and controls the interaction inside of the cursive environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayState {
    /// The view can neither be focused, nor edited
    Disabled,
    /// The view can be focused but not edited
    Enabled,
    /// The view can be focused and edited
    Editable,
}


/// This is a classic hexview which can be used to view and manipulate data which resides inside
/// this struct. There are severeal states in which the view can be operatered, see [`DisplayState`](enum.DisplayState.html).
/// You should consider the corresponding method docs for each state.
///
/// # Examples
///
/// ```
/// extern crate cursive;
/// extern crate cursive_hexview;
///
/// use cursive_hexview::{DisplayState,HexView};
///
/// fn main() {
///     let view = HexView::new().display_state(DisplayState::Editable);
///     let mut cur = cursive::Cursive::new();
///
///     cur.add_layer(cursive::views::Dialog::around(view).title("HexView"));
///
///     // cur.run();
/// }
/// ```
///
pub struct HexView {
    cursor: Vec2,
    data: Vec<u8>,
    state: DisplayState,
}

impl Default for HexView {
    /// Creates a new, default `HexView` with an empty databuffer and disabled state.
    fn default() -> Self {
        HexView::new()
    }
}

impl HexView {
    /// Creates a new, default `HexView` with an empty databuffer and disabled state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new();
    /// ```
    pub fn new() -> HexView {
        HexView {
            cursor: (0, 0).into(),
            data: Vec::new(),
            state: DisplayState::Disabled,
        }
    }

    /// Crates a new `HexView` with the given data and disabled state.
    ///
    /// # Examples
    ///
    /// With data from a `Vec`:
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new_from_iter(vec![3, 6, 1, 8, 250]);
    /// ```
    ///
    /// With data from a byte string literal:
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new_from_iter(b"Hello, World!");
    /// ```
    ///
    /// Or with a slice
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new_from_iter(&[5, 6, 2, 89]);
    /// ```
    pub fn new_from_iter<B: Borrow<u8>, I: IntoIterator<Item=B>>(data: I) -> HexView {
        HexView {
            cursor: (0, 0).into(),
            data: data.into_iter().map(|u| *u.borrow()).collect(),
            state: DisplayState::Disabled,
        }
    }

    /// Returns a reference to the internal data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let data = vec![3, 4, 9, 1];
    /// let view = HexView::new_from_iter(data.clone());
    /// assert_eq!(view.data(), &data);
    /// ```
    ///
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    /// This methods lets set you data during the lifetime of this instance (e.g. the data has been
    /// updated due to an external event).
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let mut view = cursive_hexview::HexView::new();
    /// view.set_data(b"Hello, World!".to_owned().iter());
    /// ```
    pub fn set_data<B: Borrow<u8>, I: IntoIterator<Item=B>>(&mut self, data: I) {
        self.data = data.into_iter().map(|u| *u.borrow()).collect();
    }

    /// [set_display_state](#method.set_display_state)
    pub fn display_state(self, state: DisplayState) -> Self {
        self.with(|s| s.set_display_state(state))
    }

    /// Sets the state of the view to one of the variants from `DisplayState`. This will alter the
    /// behavoir of the view accrodingly to the set state.
    ///
    /// If the state is set to `Disabled` this view can neither be focused nor edited. If the state
    /// is set to `Enabled` it can be focused and the cursor can be moved around, but no data can
    /// be altered. If set to `Editable` this view behaves like `Enabled` but the data *can* be altered.
    ///
    /// # Note
    ///
    /// This has nothing to do with rusts type system, which means even when this instance is set to
    /// `Disabled` you still can alter the data through [set_data](#method.set_data) but you cannot
    /// alter it with the keyboard commands (<kbd>+</kbd>, <kbd>-</kbd>, <kbd>#hexvalue</kbd>).
    ///
    /// # Examples
    ///
    /// ```
    /// # use cursive_hexview::{DisplayState,HexView};
    /// let view = HexView::new().display_state(DisplayState::Editable);
    /// ```
    pub fn set_display_state(&mut self, state: DisplayState) {
        self.state = state;
    }

    /// Returns the length of the data.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new_from_iter(vec![0, 1, 2, 3]);
    /// assert_eq!(4, view.len());
    /// ```
    /// 
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks whether the data is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new();
    /// assert!(view.is_empty());
    /// ```
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let view = HexView::new_from_iter(b"ABC");
    /// assert!(!view.is_empty());
    /// ```
    ///
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Sets the length of the data which this view displays.
    /// If the new length is greater than the current one, 0's will be appended to the data.
    /// If the new length is less than the current one, the data will be truncated and is lost.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let mut view = HexView::new();
    /// view.set_len(3);
    /// 
    /// assert_eq!(view.len(), 3);
    /// assert_eq!(view.data(), &vec![0u8, 0u8, 0u8]);
    /// ```
    ///
    pub fn set_len(&mut self, length: usize) {
        if self.data.len() != length {
            let oldlen = self.data.len();
            self.data.resize(length, 0);
            if oldlen > length {
                self.cursor.y = min(self.cursor.y, self.get_widget_height());
                self.cursor.x = min(self.cursor.x, (self.data.len() * 2).saturating_sub(1));
            }
        }
    }
}


enum Field {
    Addr,
    AddrSep,
    Hex,
    AsciiSep,
    Ascii,
}

//TODO: Maybe make them configurable anyhow. Ideas?!
const U8S_PER_LINE: usize = 16;
const NIBBLES_PER_LINE: usize = U8S_PER_LINE * 2;
const CHARS_PER_GROUP: usize = 4;
const CHARS_PER_SPACING: usize = 1;
const ADDR_HEX_SEPERATOR: &str = ": ";
const HEX_ASCII_SEPERATOR: &str = " │ ";

/// calcs the position in a line with spacing
fn get_cursor_offset(vec: Vec2) -> Vec2 {
    (((vec.x as f32 / CHARS_PER_GROUP as f32).floor() as usize) * CHARS_PER_SPACING, 0).into()
}

/// returns the number of elements in `row` for a given `datalen` with `elements_per_line`
fn get_elements_in_row(datalen: usize, row: usize, elements_per_line: usize) -> usize {
    min(datalen.saturating_sub(elements_per_line * row), elements_per_line)
}

/// returns the maximal cursor position in a `row` for a `datalen` and `elements_per_line`
fn get_max_x_in_row(datalen: usize, row: usize, elements_per_line: usize) -> usize {
    (get_elements_in_row(datalen, row, elements_per_line) * 2).saturating_sub(1)
}

/// converts the character either to itself if it `is_ascii_graphic`
fn make_printable<T: Borrow<u8>>(c: T) -> char {
    let c = *c.borrow();
    if asciihelp::is_ascii_graphic(c) {
        c as char
    } else {
        '.'
    }
}

// implements helper functions for this struct
impl HexView {
    /// Counts how many digits we need to align the addresses evenly.
    /// E.g. we need 2 digits for 20 elements (0x14), but only 1 for 10 elements (0xA)
    fn get_addr_digit_length(&self) -> usize {
        match self.data.len() {
            0 ... 1 => 1,
            e => (e as f64).log(16.0).ceil() as usize
        }
    }

    /// Counts how many rows we need to display the complete data
    fn get_widget_height(&self) -> usize {
        match self.data.len() {
            0 => 1,
            e => (e as f64 / 16.0).ceil() as usize
        }
    }

    /// calcs the offset to the current position to match the spacing we insert to group the hex chars,
    /// e.g. cursor (5, 0) will result in (6, 0) because of the 1 space spacing after the fourth char
    /// and cursor (9, 0) will result in (11, 0) because of the 1+1 spacing after the fourth and eighth char
    fn get_cursor_offset(&self) -> Vec2 {
        self.cursor + get_cursor_offset(self.cursor)
    }

    /// gets the amount of nibbles in the current row
    fn get_elements_in_current_row(&self) -> usize {
        get_elements_in_row(self.data.len(), self.cursor.y, U8S_PER_LINE)
    }

    /// gets the max cursor-x position in the current row
    fn get_max_x_in_current_row(&self) -> usize {
        get_max_x_in_row(self.data.len(), self.cursor.y, U8S_PER_LINE)
    }

    /// advances the x position by one and returns either an `EventResult::Ignored` if the end of
    /// the line is reached or `EventResult::Consumed(None)` if it was successful.
    fn cursor_x_advance(&mut self) -> EventResult {
        let max_pos = self.get_max_x_in_current_row();
        if self.cursor.x == max_pos {
            return EventResult::Ignored;
        } else {
            self.cursor.x = min(self.cursor.x + 1, max_pos);
        }
        EventResult::Consumed(None)
    }

    /// Gets the element under the cursor (which points to a nibble, but we are interested in the
    /// whole u8), none if the cursor is out of range.
    fn get_element_under_cursor(&self) -> Option<u8> {
        let elem = self.cursor.y * U8S_PER_LINE + self.cursor.x / 2;
        if let Some(d) = self.data.get(elem) {
            Some(*d)
        } else {
            None
        }
    }

    /// Converts the visual position to a non spaced one. This function is used to convert the
    /// point where the mouse clicked to the real cursor position without padding
    fn convert_visual_to_real_cursor(&self, pos: Vec2) -> Vec2 {
        let mut res = pos;
        let hex_offset = self.get_field_length(Field::Addr) + self.get_field_length(Field::AddrSep);

        res.y = min(self.get_widget_height() - 1, pos.y);
        res.x = res.x.saturating_sub(hex_offset);
        res.x = res.x.saturating_sub(get_cursor_offset(res).x);
        res.x = min(get_max_x_in_row(self.data.len(), res.y, U8S_PER_LINE), res.x);

        res
    }

    /// returns the displayed characters per field
    #[allow(unknown_lints, needless_pass_by_value)]
    fn get_field_length(&self, field: Field) -> usize {
        match field {
            Field::Addr => self.get_addr_digit_length(),
            Field::AddrSep => ADDR_HEX_SEPERATOR.len(),
            Field::Hex => (CHARS_PER_GROUP + CHARS_PER_SPACING) * U8S_PER_LINE / 2,
            Field::AsciiSep => HEX_ASCII_SEPERATOR.len(),
            Field::Ascii => NIBBLES_PER_LINE,
        }
    }
}

/// implements draw-helper functions
/// it will look as follows
/// addr: hexehex hexhex hexhex ... | asciiiiiii
/// the addr field will be padded, so that all addresses are equal in length
/// the hex field will be grouped by 4 character (nibble) and seperated by 1 space
/// the seperator is a special pipe, which is longer and connects with the lower and bottom "pipe" (BOX DRAWINGS LIGHT VERTICAL \u{2502})
/// the ascii part is just the ascii char of the coressponding hex value if it is graphical (see asciihelp), if not it will be displayed as a dot (.)
impl HexView {
    /// draws the addr field into the printer
    fn draw_addr(&self, printer: &Printer) {
        let digits_len = self.get_addr_digit_length();
        for lines in 0..self.get_widget_height() {
            printer.print((0, lines), &format!("{:0len$X}", lines * U8S_PER_LINE, len = digits_len));
        }
    }

    fn draw_addr_hex_sep(&self, printer: &Printer) {
        printer.print_vline((0, 0), self.get_widget_height(), ADDR_HEX_SEPERATOR);
    }

    /// draws the hex fields between the addr and ascii representation
    fn draw_hex(&self, printer: &Printer) {
        for (i, c) in self.data.chunks(U8S_PER_LINE).enumerate() {
            let hex = c.chunks(2).map(|c| if c.len() == 2 { format!("{:02X}{:02X}", c[0], c[1]) } else { format!("{:02X}", c[0]) }).format(" ");
            printer.print((0, i), &format!("{}", hex));
        }
    }

    /// draws the ascii seperator between the hex and ascii representation
    fn draw_ascii_sep(&self, printer: &Printer) {
        printer.print_vline((0, 0), self.get_widget_height(), HEX_ASCII_SEPERATOR);
    }

    /// draws the ascii chars
    fn draw_ascii(&self, printer: &Printer) {
        for (i, c) in self.data.chunks(16).enumerate() {
            let ascii: String = c.iter().map(make_printable).collect();
            printer.print((0, i), &ascii);
        }
    }

    /// this highlights the complete hex byte under the cursor
    fn highlight_current_hex(&self, printer: &Printer) {
        if let Some(elem) = self.get_element_under_cursor() {
            let high = self.cursor.x % 2 == 0;
            let hpos = self.get_cursor_offset();
            let dpos = hpos.map_x(|x| if high { x + 1 } else { x - 1 });

            let fem = format!("{:02X}", elem);
            let s = fem.split_at(1);
            let ext = |hl| if hl { s.0 } else { s.1 };

            printer.with_color(ColorStyle::Highlight, |p| p.print(hpos, ext(high)));
            printer.with_color(ColorStyle::Secondary, |p| p.with_effect(Effect::Reverse, |p| p.print(dpos, ext(!high))));
        }
    }

    /// this highlights the corresponding ascii value of the hex which is under the cursor
    fn highlight_current_ascii(&self, printer: &Printer) {
        if let Some(elem) = self.get_element_under_cursor() {
            let pos = self.cursor.map_x(|x| x / 2);
            let ascii = make_printable(&elem);
            printer.with_color(ColorStyle::Highlight, |p| p.print(pos, &ascii.to_string()));
        }
    }
}

impl View for HexView {
    fn on_event(&mut self, event: Event) -> EventResult {
        if self.state == DisplayState::Disabled {
            return EventResult::Ignored;
        }

        match event {
            //view keys
            Event::Key(k) => match k {
                Key::Left => if self.cursor.x == 0 {
                    return EventResult::Ignored;
                } else {
                    self.cursor.x = self.cursor.x.saturating_sub(1)
                },
                Key::Right => {
                    return self.cursor_x_advance();
                }
                Key::Up => {
                    if self.cursor.y == 0 {
                        return EventResult::Ignored;
                    } else {
                        self.cursor.y = self.cursor.y.saturating_sub(1)
                    }
                }
                Key::Down => {
                    if self.cursor.y == self.get_widget_height().saturating_sub(1) {
                        return EventResult::Ignored;
                    } else {
                        let max_pos = min(self.data.len(), self.cursor.y / 2 + 16).saturating_sub(1);
                        self.cursor.y = min(self.cursor.y + 1, max_pos);
                        self.cursor.x = min(self.cursor.x, self.get_elements_in_current_row().saturating_sub(1) * 2);
                    }
                }
                Key::Home => self.cursor.x = 0,
                Key::End => self.cursor.x = self.get_max_x_in_current_row(),
                _ => { return EventResult::Ignored; }
            }
            Event::Shift(Key::Home) => self.cursor = (0, 0).into(),
            Event::Shift(Key::End) => self.cursor = (get_max_x_in_row(self.data.len(), self.get_widget_height() - 1, 16), self.get_widget_height() - 1).into(),

            //edit keys
            Event::Char(c) => {
                if self.state != DisplayState::Editable {
                    return EventResult::Ignored;
                }

                match c {
                    '+' => {
                        let datalen = self.data.len();
                        self.set_len(datalen + 1);
                    }
                    '-' => {
                        let datalen = self.data.len();
                        self.set_len(datalen.saturating_sub(1));
                    }
                    _ => {
                        if let Some(val) = c.to_digit(16) {
                            if let Some(dat) = self.get_element_under_cursor() {
                                let realpos = self.cursor;
                                let elem = realpos.y * 16 + realpos.x / 2;
                                let high = self.cursor.x % 2 == 0;
                                let mask = 0xF << if high { 4 } else { 0 };

                                self.data[elem] = (dat & !mask) | ((val as u8) << if high { 4 } else { 0 });
                                self.cursor_x_advance();
                            }
                        } else {
                            return EventResult::Ignored;
                        }
                    }
                }
            }
            Event::Mouse { offset, position, event } => {
                match event {
                    MouseEvent::Press(_) => {
                        if offset.fits_in(position) {
                            self.cursor = self.convert_visual_to_real_cursor(position - offset);
                        } else {
                            return EventResult::Ignored;
                        }
                    }
                    _ => { return EventResult::Ignored; }
                }
            }
            _ => { return EventResult::Ignored; }
        };

        EventResult::Consumed(None)
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        let length = self.get_field_length(Field::Addr) +
            self.get_field_length(Field::AddrSep) +
            self.get_field_length(Field::Hex) +
            self.get_field_length(Field::AsciiSep) +
            self.get_field_length(Field::Ascii);

        (length, self.get_widget_height()).into()
    }

    fn draw(&self, printer: &Printer) {
        let height = self.get_widget_height();
        //they are a tuple of (offset, len)
        let addr = (0usize, self.get_field_length(Field::Addr));
        let addr_sep = (addr.0 + addr.1, self.get_field_length(Field::AddrSep));
        let hex = (addr_sep.0 + addr_sep.1, self.get_field_length(Field::Hex));
        let ascii_sep = (hex.0 + hex.1, self.get_field_length(Field::AsciiSep));
        let ascii = (ascii_sep.0 + ascii_sep.1, self.get_field_length(Field::Ascii));

        self.draw_addr(&printer.sub_printer((addr.0, 0), (addr.1, height), false));
        self.draw_addr_hex_sep(&printer.sub_printer((addr_sep.0, 0), (addr_sep.1, height), false));
        self.draw_hex(&printer.sub_printer((hex.0, 0), (hex.1, height), false));
        self.draw_ascii_sep(&printer.sub_printer((ascii_sep.0, 0), (ascii_sep.1, height), false));
        self.draw_ascii(&printer.sub_printer((ascii.0, 0), (ascii.1, height), false));

        if self.state != DisplayState::Disabled {
            self.highlight_current_hex(&printer.sub_printer((hex.0, 0), (hex.1, height), true));
            self.highlight_current_ascii(&printer.sub_printer((ascii.0, 0), (ascii.1, height), true));
        }
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        self.state != DisplayState::Disabled
    }
}


//TODO: needs_relayout: only when cursor moved or data has been updated (either internally or externally)
//      required_size:  support different views (e.g. wihtout ascii, without addr, hex only)

