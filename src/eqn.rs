use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::BufRead;
use encoding::{Encoding, DecoderTrap};
use encoding::all::{GBK, UTF_8};
use std::borrow::Cow;


#[derive(Debug)]
pub struct MTEquation {
    m_mtef_ver: u8,
    m_platform: u8,
    m_product: u8,
    m_version: u8,
    m_version_sub: u8,
    m_application: String,
    m_inline: u8,

    encoding_defs: Vec<MTRecords>,
    records: Vec<MTRecords>,
}

#[derive(Debug)]
enum MTRecords {
    END,
    LINE(MTLine),
    CHAR(MTChar),
    TMPL(MTTmpl),
    ENCODING_DEF(String),
    FONT_DEF { enc_def_index: u8, name: String },
    FONT_STYLE_DEF { font_def_index: u8, char_style: u8 },
    EQN_PREFS { sizes: Vec<String>, spaces: Vec<String>, styles: Vec<Option<u8>> },
    FULL, SUB, SUB2, SYM, SUBSYM,
    FUTURE,
}


#[derive(Debug)]
struct MTLine {
    nudge: (u16, u16),
    line_spacing: u8,
    null: bool,
}

#[derive(Debug)]
struct MTTmpl {
    nudge: (u16, u16),
    selector: u8,
    variation: u16,
    options: u8
}

#[derive(Debug)]
struct MTChar {
    nudge: (u16, u16),
    typeface: u8,
    mtcode: u16,
    fp8: u8,
    fp16: u16,
}

impl MTEquation {
    /// How MTEF is stored in files and objects
    /// https://docs.wiris.com/en/mathtype/mathtype_desktop/mathtype-sdk/mtefstorage
    pub fn from_ole(path: &str) -> Result<MTEquation, super::error::Error> {
        let reader = ole::Reader::from_path(path).unwrap();
        for entry in reader.iterate() {
            if entry.name() == "Equation Native" {
                let mut slice = reader.get_entry_slice(entry).unwrap();
                let mut buf = vec![0; slice.len()];
                slice.read(&mut buf).unwrap();
                let hdr = EqnOleFileHdr::parse_ole_hdr(&buf).unwrap();
                let body = buf[hdr.cb_hdr as usize..(hdr.cb_hdr as usize + hdr.size as usize)].to_vec();
                let t = MTEquation::parse(body).unwrap();
                return Ok(t);
            }
        }
        Err(super::error::Error::InvalidOLEFile)
    }

    /// Introduction
    /// This document is describes the binary equation format used by MathType 4.0 (all platforms).
    /// Although MTEF is not the most friendly medium for defining equations,
    /// there have been so many requests for this information, we decided to publish it anyway.
    /// We must warn the reader that it is not an easy format to understand and, more importantly,
    /// MathType is not at all forgiving in its processing of it.
    /// This means that if you send MathType MTEF with errors, it might crash.
    /// At a minimum, you will get an equation with formatting problems. Also, it is a binary format.
    /// This means that you can't use character strings to represent equations and it makes creating MTEF a little harder
    /// with programming languages like Visual Basic.
    ///
    /// How MathType stores an equation description in an OLE equation object,
    /// a file, or on the clipboard is not described here.
    /// Please see the document on MathType MTEF Storage for more information on this subject.
    ///
    /// This document sometimes refers to MathType's internal names for values (e.g. parmLINESPACE).
    /// These are given for reference purposes and are handy for reducing error when such values are communicated by humans.
    pub fn parse(buf: Vec<u8>) -> Result<MTEquation, super::error::Error> {
        let mut cur = Cursor::new(buf);
        let mut eqn = MTEquation {
            m_mtef_ver: cur.read_u8().unwrap(),
            m_platform: cur.read_u8().unwrap(),
            m_product: cur.read_u8().unwrap(),
            m_version: cur.read_u8().unwrap(),
            m_version_sub: cur.read_u8().unwrap(),
            m_application: read_null_terminated_string(&mut cur).unwrap(),
            m_inline: cur.read_u8().unwrap(),
            encoding_defs: vec![
                MTRecords::ENCODING_DEF("MTCode".to_string()),
                MTRecords::ENCODING_DEF("Unknown".to_string()),
                MTRecords::ENCODING_DEF("Symbol".to_string()),
                MTRecords::ENCODING_DEF("MTExtra".to_string()),
            ],
            records: vec![],
        };
        loop {
            match cur.read_u8() {
                Ok(END) => eqn.records.push(MTRecords::END),
                Ok(LINE) => {
                    let options = cur.read_u8().unwrap();
                    let mut line = MTLine {
                        nudge: (0, 0),
                        line_spacing: 0,
                        null: false,
                    };
                    if MTEF_OPT_NUDGE == MTEF_OPT_NUDGE & options {
                        line.nudge = read_nudge_values(&mut cur)
                    }
                    if MTEF_OPT_LINE_LSPACE == MTEF_OPT_LINE_LSPACE & options {
                        line.line_spacing = cur.read_u8().unwrap()
                    }
                    if MTEF_OPT_LINE_NULL == MTEF_OPT_LINE_NULL & options {
                        line.null = true
                    }
                    eqn.records.push(MTRecords::LINE(line))
                }
                Ok(CHAR) => {
                    let mut ch = MTChar { nudge: (0, 0), typeface: 0, mtcode: 0, fp8: 0, fp16: 0 };
                    let options = cur.read_u8().unwrap();
                    if MTEF_OPT_NUDGE == MTEF_OPT_NUDGE & options {
                        ch.nudge = read_nudge_values(&mut cur)
                    }
                    ch.typeface = cur.read_u8().unwrap();

                    if MTEF_OPT_CHAR_ENC_NO_MTCODE != MTEF_OPT_CHAR_ENC_NO_MTCODE & options {
                        ch.mtcode = cur.read_u16::<LittleEndian>().unwrap()
                    }
                    if MTEF_OPT_CHAR_ENC_CHAR_8 == MTEF_OPT_CHAR_ENC_CHAR_8 & options {
                        ch.fp8 = cur.read_u8().unwrap();
                    }
                    if MTEF_OPT_CHAR_ENC_CHAR_16 == MTEF_OPT_CHAR_ENC_CHAR_16 & options {
                        ch.fp16 = cur.read_u16::<LittleEndian>().unwrap();
                    }
                    let record = MTRecords::CHAR(ch);
                    eqn.records.push(record)
                }
                Ok(TMPL) => {
                    let mut tmpl = MTTmpl { nudge: (0, 0), selector: 0, variation: 0, options: 0 };
                    let options = cur.read_u8().unwrap();
                    if MTEF_OPT_NUDGE == MTEF_OPT_NUDGE & options {
                        tmpl.nudge = read_nudge_values(&mut cur)
                    }
                    tmpl.selector = cur.read_u8().unwrap();

                    // variation, 1 or 2 bytes
                    let byte1 = cur.read_u8().unwrap() as u16;
                    tmpl.variation = match 0x80 == byte1 & 0x80 {
                        true => {
                            let byte2 = cur.read_u8().unwrap() as u16;
                            (byte1 & 0x7F) | (byte2 << 8)
                        },
                        false => { byte1 }
                    };
                    tmpl.options = cur.read_u8().unwrap();
                    let record = MTRecords::TMPL(tmpl);
                    eqn.records.push(record)
                }
                Ok(PILE) => { println!("PILE") }
                Ok(EMBELL) => { println!("EMBELL") }
                Ok(MATRIX) => { println!("MATRIX") }
                Ok(RULER) => { println!("RULER") }
                Ok(FONT_STYLE_DEF) => {
                    let record = MTRecords::FONT_STYLE_DEF {
                        font_def_index: cur.read_u8().unwrap(),
                        char_style: cur.read_u8().unwrap()
                    };
                    eqn.records.push(record)
                }
                Ok(SIZE) => { println!("SIZE") }
                Ok(FULL) => eqn.records.push(MTRecords::FULL),
                Ok(SUB) => eqn.records.push(MTRecords::SUB),
                Ok(SUB2) => eqn.records.push(MTRecords::SUB2),
                Ok(SYM) => eqn.records.push(MTRecords::SYM),
                Ok(SUBSYM) => eqn.records.push(MTRecords::SUBSYM),
                Ok(COLOR) => { println!("COLOR") }
                Ok(COLOR_DEF) => { println!("COLOR_DEF") }
                Ok(FONT_DEF) => {
                    let record = MTRecords::FONT_DEF {
                        enc_def_index: cur.read_u8().unwrap(),
                        name: read_null_terminated_string(&mut cur).unwrap(),
                    };
                    eqn.records.push(record)
                }
                Ok(EQN_PREFS) => {
                    let _options = cur.read_u8().unwrap();

                    // sizes
                    let size = cur.read_u8().unwrap();
                    let sizes = read_dimension_arrays(&mut cur, size).unwrap();

                    // spaces
                    let size = cur.read_u8().unwrap();
                    let spaces = read_dimension_arrays(&mut cur, size).unwrap();

                    // styles
                    let size = cur.read_u8().unwrap();
                    let mut styles = vec![];
                    for _i in 0..size {
                        let c = cur.read_u8().unwrap();
                        match c == 0 {
                            true => { styles.push(None) },
                            false => { styles.push(Some(cur.read_u8().unwrap())) }
                        }
                    }
                    let record = MTRecords::EQN_PREFS { sizes, spaces, styles };
                    eqn.records.push(record)
                }
                Ok(ENCODING_DEF) => eqn.records.push(
                    MTRecords::ENCODING_DEF(read_null_terminated_string(&mut cur).unwrap())),
                Ok(FUTURE) => eqn.records.push(MTRecords::FUTURE),
                Ok(_) => eqn.records.push(MTRecords::FUTURE),
                Err(_e) => break
            }
        }
        Ok(eqn)
    }
}


impl MTEquation {
    pub fn translate(&self) -> Result<String, super::error::Error> {
        for record in &self.records {
            println!("{:?}", record);
        }
        Ok("hello".to_string())
    }
}
/// How MTEF is Stored in Files and Objects
/// http://web.archive.org/web/20010304111449/http://mathtype.com/support/tech/MTEF_storage.htm#OLE%20Objects
/// OLE Equation Objects
/// MTEF data is saved as the native data format of the object.
/// Whenever an equation object is to be written to an OLE "stream", a 28- byte header is written, followed by the MTEF data.
#[derive(Debug)]
struct EqnOleFileHdr {
    // length of header, sizeof(EQNOLEFILEHDR) = 28 bytes
    cb_hdr: u16,
    // hiword = 2, loword = 0
    version: u32,
    cf: u16,
    size: u32,
    reserved1: u32,
    reserved2: u32,
    reserved3: u32,
    reserved4: u32,
}


impl EqnOleFileHdr {
    fn parse_ole_hdr(buf: &Vec<u8>) -> Result<EqnOleFileHdr, super::error::Error> {
        let mut cur = Cursor::new(buf);
        let hdr = EqnOleFileHdr {
            cb_hdr: cur.read_u16::<LittleEndian>().unwrap(),
            version: cur.read_u32::<LittleEndian>().unwrap(),
            cf: cur.read_u16::<LittleEndian>().unwrap(),
            size: cur.read_u32::<LittleEndian>().unwrap(),
            reserved1: cur.read_u32::<LittleEndian>().unwrap(),
            reserved2: cur.read_u32::<LittleEndian>().unwrap(),
            reserved3: cur.read_u32::<LittleEndian>().unwrap(),
            reserved4: cur.read_u32::<LittleEndian>().unwrap(),
        };
        if 28u16 != hdr.cb_hdr && 131072u32 != hdr.version {
            Err(super::error::Error::InvalidOLEFile)
        } else {
            Ok(hdr)
        }
    }
}


/// value 	symbol 	description
/// 0 	END 	end of MTEF, pile, line, embellishment list, or template
const END: u8 = 0;
/// 1 	LINE 	line (slot)
const LINE: u8 = 1;
/// 2 	CHAR 	character
const CHAR: u8 = 2;
/// 3 	TMPL 	template
const TMPL: u8 = 3;
/// 4 	PILE 	pile (vertical stack of lines)
const PILE: u8 = 4;
/// 5 	MATRIX 	matrix
const EMBELL: u8 = 5;
/// 6 	EMBELL 	character embellishment (e.g. hat, prime)
const MATRIX: u8 = 6;
/// 7 	RULER 	ruler (tab-stop location)
const RULER: u8 = 7;
/// 8 	FONT_STYLE_DEF 	font/char style definition
const FONT_STYLE_DEF: u8 = 8;
/// 9 	SIZE 	general size
const SIZE: u8 = 9;
/// 10 	FULL 	full size
const FULL: u8 = 10;
/// 11 	SUB 	subscript size
const SUB: u8 = 11;
/// 12 	SUB2 	sub-subscript size
const SUB2: u8 = 12;
/// 13 	SYM 	symbol size
const SYM: u8 = 13;
/// 14 	SUBSYM 	sub-symbol size
const SUBSYM: u8 = 14;
/// 15 	COLOR 	color
const COLOR: u8 = 15;
/// 16 	COLOR_DEF 	color definition
const COLOR_DEF: u8 = 16;
/// 17 	FONT_DEF 	font definition
const FONT_DEF: u8 = 17;
/// 18 	EQN_PREFS 	equation preferences (sizes, styles, spacing)
const EQN_PREFS: u8 = 18;
/// 19 	ENCODING_DEF 	encoding definition
const ENCODING_DEF: u8 = 19;
/// >= 100 	FUTURE 	for future expansion (see below)
const FUTURE: u8 = 100;

/// nudge values follow tag
const MTEF_OPT_NUDGE: u8 = 0x08;
// line is a placeholder only (i.e. not displayed)
const MTEF_OPT_LINE_NULL: u8 = 0x01;
// line spacing value follows tag
const MTEF_OPT_LINE_LSPACE: u8 = 0x04;
// RULER record follows LINE or PILE record
const MTEF_OPT_LP_RULER: u8 = 0x02;
/// Option flag values for CHAR records:
// character is followed by an embellishment list
const MTEF_OPT_CHAR_EMBELL: u8 = 0x01;
// character starts a function (sin, cos, etc.)
const MTEF_OPT_CHAR_FUNC_START: u8 = 0x02;
// character is written with an 8-bit encoded value
const MTEF_OPT_CHAR_ENC_CHAR_8: u8 = 0x04;
// character is written with an 16-bit encoded value
const MTEF_OPT_CHAR_ENC_CHAR_16: u8 = 0x10;
// character is written without an 16-bit MTCode value
const MTEF_OPT_CHAR_ENC_NO_MTCODE: u8 = 0x20;

fn read_null_terminated_string(cur: &mut Cursor<Vec<u8>>) -> Result<String, Cow<'static, str>> {
    let mut buf = vec![];
    cur.read_until(b'\0', &mut buf).unwrap();
    buf.pop();
    // TODO: or UTF_8 encase of Windows English version.
    GBK.decode(buf.as_slice(), DecoderTrap::Strict)
}

fn read_dimension_arrays(cur: &mut Cursor<Vec<u8>>, size: u8) -> Result<Vec<String>, super::error::Error> {
    let mut count = 0;
    let mut new_str = true;
    let mut tmp_str = String::new();
    let mut vec = vec![];

    let mut fx = |x: u8, s: &mut String, flag: &bool| -> Result<(), super::error::Error> {
        match flag {
            true => match x {
                0x00 => s.push_str("in"),
                0x01 => s.push_str("cm"),
                0x02 => s.push_str("pt"),
                0x03 => s.push_str("pc"),
                0x04 => s.push_str("%"),
                _ => {
                    return Err(super::error::Error::InvalidOLEFile);
                }
            },
            false => match x {
                0x00 => s.push('0'),
                0x01 => s.push('1'),
                0x02 => s.push('2'),
                0x03 => s.push('3'),
                0x04 => s.push('4'),
                0x05 => s.push('5'),
                0x06 => s.push('6'),
                0x07 => s.push('7'),
                0x08 => s.push('8'),
                0x09 => s.push('9'),
                0x0a => s.push('.'),
                0x0b => s.push('-'),
                0x0f => {
                    vec.push(s.clone());
                    s.clear();
                }
                _ => {
                    return Err(super::error::Error::InvalidOLEFile);
                }
            }
        }
        Ok(())
    };

    while count < size {
        let ch = cur.read_u8().unwrap();
        let hi = (ch & 0xF0)/16;
        let lo = ch & 0x0F;
        fx(hi, &mut tmp_str, &new_str).unwrap();
        new_str = false;
        if hi == 0x0f {
            new_str = true;
            count += 1;
        }

        fx(lo, &mut tmp_str, &new_str).unwrap();
        new_str = false;
        if lo == 0x0f {
            new_str = true;
            count += 1;
        }
    }
    Ok(vec)
}


fn read_nudge_values(cur: &mut Cursor<Vec<u8>>) -> (u16, u16){
    let b1 = cur.read_u8().unwrap();
    let b2 = cur.read_u8().unwrap();
    match b1 == 128 || b2 == 128 {
        true => (cur.read_u16::<LittleEndian>().unwrap(), cur.read_u16::<LittleEndian>().unwrap()),
        false => (b1 as u16, b2 as u16)
    }
}
