//! MTEF version history:
//!
//! MTEF data exists in the following versions:
//! - 0	MathType for Mac 1.x (this format is not described here)
//! - 1	MathType for Mac 2.x and MathType for Windows 1.x
//! - 2	MathType 3.x and Equation Editor 1.x
//! - 3	Equation Editor 3.x (this format is not described here)
//! - 4	MathType 3.5
//! - 5	MathType 4.0 and later


/// Record types:
///
/// The following record types are used:
///
/// | value | symbol | description |
/// | ----- | ------ | ----------- |
/// |0  |END         |end of MTEF, pile, line, embellishment list, or template|
/// |1  |LINE        |line (slot)|
/// |2  |CHAR        |character|
/// |3  |TMPL        |template|
/// |4  |PILE        |pile (vertical stack of lines)|
/// |5  |MATRIX      |matrix|
/// |6  |EMBELL      |character embellishment (e.g. hat, prime)|
/// |7  |RULER       |ruler (tab-stop location)|
/// |8  |FONT_STYLE_DEF |font/char style definition|
/// |9  |SIZE        |general size|
/// |10 |FULL        |full size|
/// |11 |SUB         |subscript size|
/// |12 |SUB2        |sub-subscript size|
/// |13 |SYM         |symbol size|
/// |14 |SUBSYM      |sub-symbol size|
/// |15 |COLOR       |color|
/// |16 |COLOR_DEF   |color definition|
/// |17 |FONT_DEF       |font definition|
/// |18 |EQN_PREFS      |equation preferences (sizes, styles, spacing)|
/// |19 |ENCODING_DEF   |encoding definition|
/// |≥ 100	|FUTURE	    |for future expansion (see below)|
///
/// If the record type is 100 or greater, it represents a record that will be defined in a future version of MTEF.
/// For now, readers can assume that an unsigned integer follows the record type and is the number of bytes following it in the record (i.e. it doesn't include the record type and length).
/// This makes it easy for software that reads MTEF to skip these records. Although it might be handy if all records had such a length value,
/// it will only be present on future expansion records (i.e. those with record types ≥ 100).
pub mod record_types {
    /// end of MTEF, pile, line, embellishment list, or template
    pub const END: u8 = 0;
    /// line (slot)
    pub const LINE: u8 = 1;
    /// character
    pub const CHAR: u8 = 2;
    /// template
    pub const TMPL: u8 = 3;
    /// pile (vertical stack of lines)
    pub const PILE: u8 = 4;
    /// matrix
    pub const MATRIX: u8 = 5;
    /// character embellishment (e.g. hat, prime)
    pub const EMBELL: u8 = 6;
    /// ruler (tab-stop location)
    pub const RULER: u8 = 7;
    /// font/char style definition
    pub const FONT_STYLE_DEF: u8 = 8;
    /// general size
    pub const SIZE: u8 = 9;
    /// full size
    pub const FULL: u8 = 10;
    /// subscript size
    pub const SUB: u8 = 11;
    /// sub-subscript size
    pub const SUB2: u8 = 12;
    /// symbol size
    pub const SYM: u8 = 13;
    /// sub-symbol size
    pub const SUBSYM: u8 = 14;
    /// color
    pub const COLOR: u8 = 15;
    /// color definition
    pub const COLOR_DEF: u8 = 16;
    /// font definition
    pub const FONT_DEF: u8 = 17;
    /// equation preferences (sizes, styles, spacing)
    pub const EQN_PREFS: u8 = 18;
    /// encoding definition
    pub const ENCODING_DEF: u8 = 19;
    /// for future expansion
    pub const FUTURE: u8 = 100;
}

/// Option values:
///
/// Each MTEF 5 record starts with a type byte followed by an option byte.
/// This is different from earlier versions of MTEF where the option flags were stored in the upper 4 bits of the type byte.
///
/// The option flag values are record-dependent:
///
/// |value	|symbol	|description|
/// |-----  |-----  |-----      |
/// |Option flag values for all equation structure records:|
/// |0x08	|MTEF_OPT_NUDGE	|nudge values follow tag|
/// |Option flag values for CHAR records:|
/// |0x01	|MTEF_OPT_CHAR_EMBELL	|character is followed by an embellishment list|
/// |0x02	|MTEF_OPT_CHAR_FUNC_START	|character starts a function (sin, cos, etc.)|
/// |0x04	|MTEF_OPT_CHAR_ENC_CHAR_8	|character is written with an 8-bit encoded value|
/// |0x10	|MTEF_OPT_CHAR_ENC_CHAR_16	|character is written with an 16-bit encoded value|
/// |0x20	|MTEF_OPT_CHAR_ENC_NO_MTCODE	|character is written without an 16-bit MTCode value|
/// |Option flag values for LINE records:|
/// |0x01	|MTEF_OPT_LINE_NULL	|line is a placeholder only (i.e. not displayed)|
/// |0x04	|MTEF_OPT_LINE_LSPACE	|line spacing value follows tag|
/// |Option flag values for LINE and PILE records:|
/// |0x02	|MTEF_OPT_LP_RULER	|RULER record follows LINE or PILE record|
/// |Option flag values for COLOR_DEF records:|
/// |0x01	|MTEF_COLOR_CMYK	|color model is CMYK, else RGB|
/// |0x02	|MTEF_COLOR_SPOT	|color is a spot color, else a process color|
/// |0x04	|MTEF_COLOR_NAME	|color has a name, else no name|
pub mod options {
    pub const MTEF_OPT_NUDGE: u8 = 0x08;
    pub const MTEF_OPT_CHAR_EMBELL: u8 = 0x01;
    pub const MTEF_OPT_CHAR_FUNC_START: u8 = 0x02;
    pub const MTEF_OPT_CHAR_ENC_CHAR_8: u8 = 0x04;
    pub const MTEF_OPT_CHAR_ENC_CHAR_16: u8 = 0x10;
    pub const MTEF_OPT_CHAR_ENC_NO_MTCODE: u8 = 0x20;
    pub const MTEF_OPT_LINE_NULL: u8 = 0x01;
    pub const MTEF_OPT_LINE_LSPACE: u8 = 0x04;
    pub const MTEF_OPT_LP_RULER: u8 = 0x02;

    pub const MTEF_COLOR_CMYK: u8 = 0x01;
    pub const MTEF_COLOR_SPOT: u8 = 0x02;
    pub const MTEF_COLOR_NAME: u8 = 0x04;
}

/// Typeface values:
///
/// CHAR records contain a typeface value (biased by 128), written as a signed integer.
/// If the value is positive, it represents one of MathType’s styles:
pub mod typeface {
    pub const FN_TEXT: u8 = 1;
    pub const FN_FUNCTION: u8 = 2;
    pub const FN_VARIABLE: u8 = 3;
    pub const FN_LCGREEK: u8 = 4;
    pub const FN_UCGREEK: u8 = 5;
    pub const FN_SYMBOL: u8 = 6;
    pub const FN_VECTOR: u8 = 7;
    pub const FN_NUMBER: u8 = 8;
    pub const FN_USER1: u8 = 9;
    pub const FN_USER2: u8 = 10;
    pub const FN_MTEXTRA: u8 = 11;
    pub const FN_TEXT_FE: u8 = 12;
    pub const FN_EXPAND: u8 = 22;
    pub const FN_MARKER: u8 = 23;
    pub const FN_SPACE: u8 = 24;
}

/// Typesize values:
///
/// Typesize values (sometimes referred to as lsizes) are used in several MTEF records.
/// Not all values may be valid in a particular record. Their meaning is as follows:
///
/// |value	|symbol	|description|
/// |-----  |-----  |------ |
/// |0	|SZ_FULL |full|
/// |1	|SZ_SUB |subscript|
/// |2	|SZ_SUB2 |sub-subscript|
/// |3	|SZ_SYM |symbol|
/// |4	|SZ_SUBSYM |sub-symbol|
/// |5	|SZ_USER1 |user 1|
/// |6	|SZ_USER2 |user 2|
/// |7	|SZ_DELTA |delta increment|
pub mod typesize {
    /// full
    pub const SZ_FULL: u8 = 0;
    /// subscript
    pub const SZ_SUB: u8 = 1;
    /// sub-subscript
    pub const SZ_SUB2: u8 = 2;
    /// symbol
    pub const SZ_SYM: u8 = 3;
    /// sub-symbol
    pub const SZ_SUBSYM: u8 = 4;
    /// user 1
    pub const SZ_USER1: u8 = 5;
    /// user 2
    pub const SZ_USER2: u8 = 6;
    /// delta increment
    pub const SZ_DELTA: u8 = 7;
}
