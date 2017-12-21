//taken from https://doc.rust-lang.org/src/std/ascii.rs.html
//can be abandoned as soon it gets stabilized https://doc.rust-lang.org/std/primitive.char.html#method.is_ascii_graphic

enum AsciiCharacterClass {
    C,  // control
    Cw, // control whitespace
    W,  // whitespace
    D,  // digit
    L,  // lowercase
    Lx, // lowercase hex digit
    U,  // uppercase
    Ux, // uppercase hex digit
    P,  // punctuation
}

use self::AsciiCharacterClass::*;

static ASCII_CHARACTER_CLASS: [AsciiCharacterClass; 128] = [
//  _0 _1 _2 _3 _4 _5 _6 _7 _8 _9 _a _b _c _d _e _f
    C, C, C, C, C, C, C, C, C, Cw,Cw,C, Cw,Cw,C, C, // 0_
    C, C, C, C, C, C, C, C, C, C, C, C, C, C, C, C, // 1_
    W, P, P, P, P, P, P, P, P, P, P, P, P, P, P, P, // 2_
    D, D, D, D, D, D, D, D, D, D, P, P, P, P, P, P, // 3_
    P, Ux,Ux,Ux,Ux,Ux,Ux,U, U, U, U, U, U, U, U, U, // 4_
    U, U, U, U, U, U, U, U, U, U, U, P, P, P, P, P, // 5_
    P, Lx,Lx,Lx,Lx,Lx,Lx,L, L, L, L, L, L, L, L, L, // 6_
    L, L, L, L, L, L, L, L, L, L, L, P, P, P, P, C, // 7_
];

/// Checks if the value is an ASCII graphic character:
/// U+0021 '@' ... U+007E '~'.
/// For strings, true if all characters in the string are
/// ASCII punctuation.
///
/// # Examples
///
/// ```
/// #![feature(ascii_ctype)]
/// # #![allow(non_snake_case)]
/// use std::ascii::AsciiExt;
/// let A = 'A';
/// let G = 'G';
/// let a = 'a';
/// let g = 'g';
/// let zero = '0';
/// let percent = '%';
/// let space = ' ';
/// let lf = '\n';
/// let esc = '\u{001b}';
///
/// assert!(A.is_ascii_graphic());
/// assert!(G.is_ascii_graphic());
/// assert!(a.is_ascii_graphic());
/// assert!(g.is_ascii_graphic());
/// assert!(zero.is_ascii_graphic());
/// assert!(percent.is_ascii_graphic());
/// assert!(!space.is_ascii_graphic());
/// assert!(!lf.is_ascii_graphic());
/// assert!(!esc.is_ascii_graphic());
/// ```
pub fn is_ascii_graphic(c: u8) -> bool {
    let c = c as usize;
    if c >= 0x80 {
        return false;
    }
    match ASCII_CHARACTER_CLASS[c] {
        Ux | U | Lx | L | D | P => true,
        _ => false,
    }
}
