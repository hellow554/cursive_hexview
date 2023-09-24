#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications,
    missing_docs,
    rustdoc::all
)]

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

use std::borrow::Borrow;
use std::cmp::min;

use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key, MouseEvent};
use cursive::theme::{ColorStyle, Effect};
use cursive::vec::Vec2;
use cursive::view::{CannotFocus, View};
use cursive::{Printer, With};
use itertools::Itertools;
use std::fmt::{self, Write};

/// Controls the possible interactions with a [`HexView`].
///
/// This enum is used for the [`set_display_state`] method
/// and controls the interaction inside of the cursive environment.
///
/// [HexView]: struct.HexView.html
/// [`set_display_state`]: struct.HexView.html#method.set_display_state
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DisplayState {
    /// The view can neither be focused, nor edited
    Disabled,
    /// The view can be focused but not edited
    Enabled,
    /// The view can be focused and edited
    Editable,
}

/// Controls the visual output of the `HexView` struct.
///
/// There are various options which can be altered. For a detailed description of them, please see the fields below.
/// For the changes to apply, you need to use [`set_config`].
///
/// [`set_config`]: struct.HexView.html#method.set_config
#[derive(Debug, Clone, Copy)]
pub struct HexViewConfig {
    /// Controls the number of bytes per line.
    ///
    /// It needs to be greater than 0 and equal or higher than the `bytes_per_group` value.
    /// Default is `16`
    pub bytes_per_line: usize,
    /// Controls the number of bytes per group.
    ///
    /// It needs to be greater than 0 equal or lower than `bytes_per_line`.
    /// Default is `1`
    pub bytes_per_group: usize,
    /// Controls the separator between the hex groups in the data output.
    ///
    /// Default is ` ` (0x20)
    pub byte_group_separator: &'static str,
    /// Controls the separator between the address label and the hex output of the data.
    ///
    /// Default is `: `
    pub addr_hex_separator: &'static str,
    /// Controls the separator between the hex output and the ASCII representation of the data.
    ///
    /// Default is ` | `
    pub hex_ascii_separator: &'static str,
    /// Controls if the ASCII representation of the data should be shown.
    ///
    /// Default is `true`
    pub show_ascii: bool,
    /// Controls the address of the first byte
    ///
    /// Default is 0
    pub start_addr: usize,
    /// Controls number of bytes used for displaying address.
    /// When 0, the value is computed automatically.
    /// Default is 0
    pub bytes_per_addr: usize,
}

impl Default for HexViewConfig {
    fn default() -> Self {
        Self {
            bytes_per_line: 16,
            bytes_per_group: 1,
            byte_group_separator: " ",
            addr_hex_separator: ": ",
            hex_ascii_separator: " | ",
            show_ascii: true,
            start_addr: 0,
            bytes_per_addr: 0,
        }
    }
}

/// Hexadecimal viewer.
///
/// This is a classic hexview which can be used to view and manipulate data which resides inside
/// this struct. There are severeal states in which the view can be operatered, see [`DisplayState`].
/// You should consider the corresponding method docs for each state.
///
/// [`DisplayState`]: enum.DisplayState.html
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
///     let mut cur = cursive::dummy();
///
///     cur.add_layer(cursive::views::Dialog::around(view).title("HexView"));
///
///     // cur.run();
/// }
/// ```
pub struct HexView {
    data: Vec<u8>,
    config: HexViewConfig,
    cursor: Vec2,
    state: DisplayState,
}

impl Default for HexView {
    /// Creates a new, default `HexView` with an empty databuffer and disabled state.
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for HexView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HexView")
            .field("config", &self.config)
            .field("cursor", &self.cursor)
            .field("state", &self.state)
            .finish_non_exhaustive()
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
    #[must_use]
    pub fn new() -> Self {
        Self::new_from_iter(Vec::<u8>::new())
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
    pub fn new_from_iter<B: Borrow<u8>, I: IntoIterator<Item = B>>(data: I) -> Self {
        Self {
            cursor: Vec2::zero(),
            data: data.into_iter().map(|u| *u.borrow()).collect(),
            state: DisplayState::Disabled,
            config: HexViewConfig::default(),
        }
    }

    /// This function allows the customization of the `HexView` output.
    ///
    /// For options and explanation of every possible option, see the `HexViewConfig` struct.
    ///
    /// #Examples
    ///
    /// ```
    /// # use cursive_hexview::{HexView,HexViewConfig};
    /// let mut view = HexView::new();
    /// view.set_config(HexViewConfig {
    ///     bytes_per_line: 8,
    ///     ..Default::default()
    /// });
    /// ```
    pub fn set_config(&mut self, config: HexViewConfig) {
        self.config = config;
    }

    /// [`set_config`](#method.set_config)
    #[must_use]
    pub fn with_config(self, config: HexViewConfig) -> Self {
        self.with(|s| s.set_config(config))
    }

    /// Returns a reference to the internal data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let data = vec![3, 4, 9, 1];
    /// let view = HexView::new_from_iter(&data);
    /// assert_eq!(view.data(), &data);
    /// ```
    #[must_use]
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Sets the data during the lifetime of this instance.
    ///
    /// For insance to update the data due to an external event.
    ///
    /// ```
    /// # use cursive_hexview::HexView;
    /// let mut view = HexView::new();
    /// view.set_data(b"Hello, World!".to_owned().iter());
    /// ```
    pub fn set_data<B: Borrow<u8>, I: IntoIterator<Item = B>>(&mut self, data: I) {
        self.data = data.into_iter().map(|u| *u.borrow()).collect();
    }

    /// [`set_display_state`](#method.set_display_state)
    #[must_use]
    pub fn display_state(self, state: DisplayState) -> Self {
        self.with(|s| s.set_display_state(state))
    }

    /// Returns the currenct config for this view
    ///
    /// You have a read access to the current config.
    /// In order to modify them, use [`config_mut`].
    #[must_use]
    pub fn config(&self) -> &HexViewConfig {
        &self.config
    }

    /// Returns the current config for this view.
    ///
    /// You can modify and update all values of that config. To make those
    /// changes visible, you must redraw this view.
    #[must_use]
    pub fn config_mut(&mut self) -> &mut HexViewConfig {
        &mut self.config
    }

    /// Sets the state of the view to one of the variants from `DisplayState`.
    ///
    /// This will alter the behavior of the view accrodingly to the set state.
    ///
    /// If the state is set to `Disabled` this view can neither be focused nor edited. If the state
    /// is set to `Enabled` it can be focused and the cursor can be moved around, but no data can
    /// be altered. If set to `Editable` this view behaves like `Enabled` but the data *can* be altered.
    ///
    /// # Note
    ///
    /// This has nothing to do with rusts type system, which means even when this instance is set to
    /// `Disabled` you still can alter the data through [`set_data`](#method.set_data) but you cannot
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
    #[must_use]
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
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Sets the length of the data which this view displays.
    ///
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

/// calcs the position in a line with spacing
fn get_cursor_offset(vec: Vec2, config: &HexViewConfig) -> Vec2 {
    (
        ((vec.x as f32 / (2 * config.bytes_per_group) as f32).floor() as usize) * config.byte_group_separator.len(),
        0,
    )
        .into()
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
    if c.is_ascii_graphic() {
        c as char
    } else {
        '.'
    }
}

// implements helper functions for this struct
impl HexView {
    /// Counts how many digits we need to align the addresses evenly.
    ///
    /// E.g. we need 2 digits for 20 elements (0x14), but only 1 for 10 elements (0xA)
    fn get_addr_digit_length(&self) -> usize {
        match self.data.len() {
            0..=1 => 1,
            e => std::cmp::max(
                ((e + self.config.start_addr) as f64).log(16.0).ceil() as usize,
                self.config.bytes_per_addr,
            ),
        }
    }

    /// Counts how many rows we need to display the complete data
    fn get_widget_height(&self) -> usize {
        match self.data.len() {
            0 => 1,
            e => (e as f64 / self.config.bytes_per_line as f64).ceil() as usize,
        }
    }

    /// calcs the offset to the current position to match the spacing we insert to group the hex chars.
    ///
    /// e.g. cursor (5, 0) will result in (6, 0) because of the 1 space spacing after the fourth char
    /// and cursor (9, 0) will result in (11, 0) because of the 1+1 spacing after the fourth and eighth char
    fn get_cursor_offset(&self) -> Vec2 {
        self.cursor + get_cursor_offset(self.cursor, &self.config)
    }

    /// gets the amount of nibbles in the current row
    fn get_elements_in_current_row(&self) -> usize {
        get_elements_in_row(self.data.len(), self.cursor.y, self.config.bytes_per_line)
    }

    /// gets the max cursor-x position in the current row
    fn get_max_x_in_current_row(&self) -> usize {
        get_max_x_in_row(self.data.len(), self.cursor.y, self.config.bytes_per_line)
    }

    /// advances the x position by one
    ///
    /// Returns either an `EventResult::Ignored` if the end of
    /// the line is reached or `EventResult::Consumed(None)` if it was successful.
    fn cursor_x_advance(&mut self) -> EventResult {
        let max_pos = self.get_max_x_in_current_row();
        if self.cursor.x == max_pos {
            return EventResult::Ignored;
        }

        self.cursor.x = min(self.cursor.x + 1, max_pos);
        EventResult::Consumed(None)
    }

    /// Gets the element under the cursor
    ///
    /// (which points to a nibble, but we are interested in the
    /// whole u8)
    ///
    /// Returns none if the cursor is out of range.
    fn get_element_under_cursor(&self) -> Option<u8> {
        let elem = self.cursor.y * self.config.bytes_per_line + self.cursor.x / 2;
        self.data.get(elem).copied()
    }

    /// Converts the visual position to a non spaced one.
    ///
    /// This function is used to convert the
    /// point where the mouse clicked to the real cursor position without padding
    fn convert_visual_to_real_cursor(&self, pos: Vec2) -> Vec2 {
        let mut res = pos;
        let hex_offset = self.get_field_length(Field::Addr) + self.get_field_length(Field::AddrSep);

        res.y = min(self.get_widget_height() - 1, pos.y);
        res.x = res.x.saturating_sub(hex_offset);
        res.x = res.x.saturating_sub(get_cursor_offset(res, &self.config).x);
        res.x = min(
            get_max_x_in_row(self.data.len(), res.y, self.config.bytes_per_line),
            res.x,
        );

        res
    }

    /// returns the displayed characters per field
    #[allow(unknown_lints, clippy::needless_pass_by_value)]
    fn get_field_length(&self, field: Field) -> usize {
        match field {
            Field::Addr => self.get_addr_digit_length(),
            Field::AddrSep => self.config.addr_hex_separator.len(),
            Field::Hex => {
                (((2 * self.config.bytes_per_group) + self.config.byte_group_separator.len())
                    * (self.config.bytes_per_line / self.config.bytes_per_group))
                    - self.config.byte_group_separator.len()
            }
            Field::AsciiSep => self.config.hex_ascii_separator.len(),
            Field::Ascii => self.config.bytes_per_line * 2,
        }
    }
}

// implements draw-helper functions
// it will look as follows
// addr: hexehex hexhex hexhex ... | asciiiiiii
// the addr field will be padded, so that all addresses are equal in length
// the hex field will be grouped by 4 character (nibble) and seperated by 1 space
// the seperator is a special pipe, which is longer and connects with the lower and bottom "pipe" (BOX DRAWINGS LIGHT VERTICAL \u{2502})
// the ascii part is just the ascii char of the coressponding hex value if it is [graphical](https://doc.rust-lang.org/std/primitive.u8.html#method.is_ascii_graphic), if not it will be displayed as a dot (.)
impl HexView {
    /// draws the addr field into the printer
    fn draw_addr(&self, printer: &Printer) {
        let digits_len = self.get_addr_digit_length();
        for lines in 0..self.get_widget_height() {
            printer.print(
                (0, lines),
                &format!(
                    "{:0len$X}",
                    self.config.start_addr + lines * self.config.bytes_per_line,
                    len = digits_len
                ),
            );
        }
    }

    fn draw_addr_hex_sep(&self, printer: &Printer) {
        printer.print_vline((0, 0), self.get_widget_height(), self.config.addr_hex_separator);
    }

    /// draws the hex fields between the addr and ascii representation
    fn draw_hex(&self, printer: &Printer) {
        for (i, c) in self.data.chunks(self.config.bytes_per_line).enumerate() {
            let hex = c
                .chunks(self.config.bytes_per_group)
                .map(|c| {
                    let mut s = String::new();
                    for &b in c {
                        write!(&mut s, "{:02X}", b).expect("Unable to write hex values");
                    }
                    s
                })
                .format(self.config.byte_group_separator);
            printer.print((0, i), &format!("{}", hex));
        }
    }

    /// draws the ascii seperator between the hex and ascii representation
    fn draw_ascii_sep(&self, printer: &Printer) {
        printer.print_vline((0, 0), self.get_widget_height(), self.config.hex_ascii_separator);
    }

    /// draws the ascii chars
    fn draw_ascii(&self, printer: &Printer) {
        for (i, c) in self.data.chunks(self.config.bytes_per_line).enumerate() {
            let ascii: String = c.iter().map(make_printable).collect();
            printer.print((0, i), &ascii);
        }
    }

    /// this highlights the complete hex byte under the cursor
    #[allow(clippy::similar_names)]
    fn highlight_current_hex(&self, printer: &Printer) {
        if let Some(elem) = self.get_element_under_cursor() {
            let high = self.cursor.x % 2 == 0;
            let hpos = self.get_cursor_offset();
            let dpos = hpos.map_x(|x| if high { x + 1 } else { x - 1 });

            let fem = format!("{:02X}", elem);
            let s = fem.split_at(1);
            let ext = |hl| if hl { s.0 } else { s.1 };

            printer.with_color(ColorStyle::highlight(), |p| p.print(hpos, ext(high)));
            printer.with_color(ColorStyle::secondary(), |p| {
                p.with_effect(Effect::Reverse, |p| p.print(dpos, ext(!high)));
            });
        }
    }

    /// this highlights the corresponding ascii value of the hex which is under the cursor
    fn highlight_current_ascii(&self, printer: &Printer) {
        if let Some(elem) = self.get_element_under_cursor() {
            let pos = self.cursor.map_x(|x| x / 2);
            let ascii = make_printable(&elem);
            printer.with_color(ColorStyle::highlight(), |p| p.print(pos, &ascii.to_string()));
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
                Key::Left => {
                    if self.cursor.x == 0 {
                        return EventResult::Ignored;
                    }

                    self.cursor.x = self.cursor.x.saturating_sub(1);
                }
                Key::Right => {
                    return self.cursor_x_advance();
                }
                Key::Up => {
                    if self.cursor.y == 0 {
                        return EventResult::Ignored;
                    }

                    self.cursor.y = self.cursor.y.saturating_sub(1);
                }
                Key::Down => {
                    if self.cursor.y == self.get_widget_height().saturating_sub(1) {
                        return EventResult::Ignored;
                    }

                    let max_pos = min(self.data.len(), self.cursor.y / 2 + 16).saturating_sub(1);
                    self.cursor.y = min(self.cursor.y + 1, max_pos);
                    self.cursor.x = min(self.cursor.x, self.get_elements_in_current_row().saturating_sub(1) * 2);
                }
                Key::Home => self.cursor.x = 0,
                Key::End => self.cursor.x = self.get_max_x_in_current_row(),
                _ => {
                    return EventResult::Ignored;
                }
            },
            Event::Shift(Key::Home) => self.cursor = (0, 0).into(),
            Event::Shift(Key::End) => {
                self.cursor = (
                    get_max_x_in_row(self.data.len(), self.get_widget_height() - 1, 16),
                    self.get_widget_height() - 1,
                )
                    .into();
            }

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
                                let elem = realpos.y * self.config.bytes_per_line + realpos.x / 2;
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
            Event::Mouse {
                offset,
                position,
                event: MouseEvent::Press(_),
            } => {
                if let Some(position) = position.checked_sub(offset) {
                    self.cursor = self.convert_visual_to_real_cursor(position);
                } else {
                    return EventResult::Ignored;
                }
            }
            _ => {
                return EventResult::Ignored;
            }
        };

        EventResult::Consumed(None)
    }

    fn required_size(&mut self, _: Vec2) -> Vec2 {
        let length = self.get_field_length(Field::Addr)
            + self.get_field_length(Field::AddrSep)
            + self.get_field_length(Field::Hex)
            + self.get_field_length(Field::AsciiSep)
            + self.get_field_length(Field::Ascii);

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

        self.draw_addr(&printer.offset((addr.0, 0)).cropped((addr.1, height)));
        self.draw_addr_hex_sep(&printer.offset((addr_sep.0, 0)).cropped((addr_sep.1, height)));
        self.draw_hex(&printer.offset((hex.0, 0)).cropped((hex.1, height)));
        if self.config.show_ascii {
            self.draw_ascii_sep(&printer.offset((ascii_sep.0, 0)).cropped((ascii_sep.1, height)));
            self.draw_ascii(&printer.offset((ascii.0, 0)).cropped((ascii.1, height)));
        }

        if self.state != DisplayState::Disabled {
            self.highlight_current_hex(&printer.offset((hex.0, 0)).cropped((hex.1, height)).focused(true));
            if self.config.show_ascii {
                self.highlight_current_ascii(&printer.offset((ascii.0, 0)).cropped((ascii.1, height)).focused(true));
            }
        }
    }

    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        (self.state != DisplayState::Disabled)
            .then(EventResult::consumed)
            .ok_or(CannotFocus)
    }
}

//TODO: needs_relayout: only when cursor moved or data has been updated (either internally or externally)
//      required_size:  support different views (e.g. wihtout ascii, without addr, hex only)
