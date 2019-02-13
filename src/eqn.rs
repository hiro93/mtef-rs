use std::io::{Cursor, Read};

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::BufRead;
use encoding::{DecoderTrap, decode};
use encoding::all::UTF_8;

#[derive(Debug)]
pub struct MTEquation {
    m_mtef_ver: u8,
    m_platform: u8,
    m_product: u8,
    m_version: u8,
    m_version_sub: u8,
    m_application: String,
    m_inline: u8,

    records: Vec<MTRecords>,
}

#[derive(Debug)]
enum MTCharacter {
    /// The MTCode value defines the character independent of its font.
    /// MTCode is a superset of Unicode and is described in MTCode Encoding Tables.
    MTCode { value: u16 },
    /// The 8-bit and 16-bit font positions are mutually exclusive but may both be absent.
    /// This is the position of the character within its font.
    /// Some of the common font encodings are given in Font Encoding Tables.
    FontPosition { value: u16 },
}



#[derive(Debug)]
enum MTRecords {
    END,
    /// LINE record (1):
    /// Consists of:
    /// - record type (1)
    /// - options
    /// - [nudge] if mtefOPT_NUDGE is set
    /// - [line spacing] if mtefOPT_LINE_LSPACE is set (16-bit integer)
    /// - [RULER record] if mtefOPT_LP_RULER is set
    /// - object list contents of line (a single pile, characters and templates, or nothing)
    LINE {
        record_type: u8,
        options: u8,
        nudge: u8,
        line_spacing: u16,
//        ruler_record: vec![],
//        object_list: vec![],
    },

    /// CHAR record (2):
    /// Consists of:
    /// - record type (2)
    /// - options
    /// - [nudge] if mtefOPT_NUDGE is set
    /// - [typeface] typeface value (signed integer; see FONT below)
    /// - [character] character value (see below)
    /// - [embellishment list] if mtefOPT_CHAR_EMBELL is set (embellishments)
    CHAR {
        record_type: u8,
        options: u8,
        nudge: u8,
        typeface: i32,
        character: MTCharacter
    },

    ENCODING_DEF { name: String },
    FUTURE
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
        println!("{:?}", cur);
        let mut eqn = MTEquation {
            m_mtef_ver: cur.read_u8().unwrap(),
            m_platform: cur.read_u8().unwrap(),
            m_product: cur.read_u8().unwrap(),
            m_version: cur.read_u8().unwrap(),
            m_version_sub: cur.read_u8().unwrap(),
            m_application: "".to_string(),
            m_inline: 0,
            records: vec![],
        };
        let mut m_application = vec![];
        cur.read_until(b'\0', &mut m_application).unwrap();
        eqn.m_application = String::from_utf8(m_application).unwrap();
        eqn.m_inline = cur.read_u8().unwrap();
        loop {
            match cur.read_u8() {
                Ok(END) => eqn.records.push(MTRecords::END),
                Ok(LINE) => { println!("LINE") }
                Ok(CHAR) => { println!("CHAR") }
                Ok(TMPL) => { println!("TMPL") }
                Ok(PILE) => { println!("PILE") }
                Ok(EMBELL) => { println!("EMBELL") }
                Ok(MATRIX) => { println!("MATRIX") }
                Ok(RULER) => { println!("RULER") }
                Ok(FONT_STYLE_DEF) => { println!("FONT_STYLE_DEF") }
                Ok(SIZE) => { println!("SIZE") }
                Ok(FULL) => { println!("FULL") }
                Ok(SUB) => { println!("SUB") }
                Ok(SUB2) => { println!("SUB2") }
                Ok(SYM) => { print!("SYM"); }
                Ok(SUBSYM) => { println!("SUBSYM") }
                Ok(COLOR) => { println!("COLOR") }
                Ok(COLOR_DEF) => { println!("COLOR_DEF") }
                Ok(FONT_DEF) => {
                    let enc_def_index = cur.read_u8().unwrap();
                    let mut name = vec![];
                    cur.read_until(b'\0', &mut name);
//                    let name = String::from_utf8(name).unwrap_or_default();
                    println!("FONT_DEF: {}, {:?}", enc_def_index, name);
                }
                Ok(EQN_PREFS) => {
                    let options = cur.read_u8().unwrap();
                    let size = cur.read_u8().unwrap();
                    println!("EQN_PREFS: options={}, size={}", options, size);
                }
                Ok(ENCODING_DEF) => {
                    let mut name = vec![];
                    cur.read_until(b'\0', &mut name);
                    let name = String::from_utf8(name).unwrap();
//                    let name = read_null_terminated_string(&mut cur);
//                    println!("ENCODING_DEF: {:?}", name);
                    let record = MTRecords::ENCODING_DEF { name };
                    println!("{:?}", record);
//                    t.records.push(MTRecords::ENCODING_DEF { name });
                }
                Ok(FUTURE) => eqn.records.push(MTRecords::FUTURE),
                Ok(_) => eqn.records.push(MTRecords::FUTURE),
                Err(_e) => break
            }
        }
        Ok(eqn)
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


fn read_null_terminated_string(cur: &mut Cursor<&Vec<u8>>) -> String {

    let mut buf = vec![];
    cur.read_until(b'\0', &mut buf).unwrap();
    decode(buf.as_slice(), DecoderTrap::Strict, UTF_8)
}