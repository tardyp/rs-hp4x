mod consts;
use crate::consts::*;
use thiserror::Error;
mod nibbles;
mod basic;
mod dir;
mod extable;
mod library;
pub mod decompile;
use nibbles::*;
use basic::*;
pub use dir::*;
use library::*;
pub use extable::*;

use winnow::combinator::{opt, repeat};
use winnow::error::{ErrorKind, ParserError, StrContext};
use winnow::token::take;
use winnow::{PResult, Parser};
use std::fmt::Debug;
use std::path::Path;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Illegal Prolog: {0}")]
    IllegalProlog(u32),
    #[error("Bad header: {0:?} not HPHP48 or HPHP49")]
    BadHeader(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse Error: {0}")]
    ParseError(String),
}
type Result<T> = std::result::Result<T, Error>;

pub(crate) fn next_semi_terminated(nibs: &mut Nibbles) -> PResult<Vec<Obj>> {
    _next_semi_terminated.context(StrContext::Label("semi-terminated")).parse_next(nibs)
}
fn _next_semi_terminated(nibs: &mut Nibbles) -> PResult<Vec<Obj>> {
    let mut objs = Vec::new();
    loop {
        let obj = next_obj(nibs)?;
        match obj {
            Obj::Semi() => {
                break;
            }
            _ => {}
        }
        objs.push(obj);
    }
    Ok(objs)
}


#[derive(Debug)]
pub struct Array {
    pub obj_type: u32,
    pub num_dims: usize,
    pub dims: Vec<usize>,
    pub objects: Vec<Obj>,
}


pub(crate) fn next_array(input: &mut Nibbles) -> PResult<Array> {
    _next_array.context(StrContext::Label("array")).parse_next(input)
}
fn _next_array(input: &mut Nibbles) -> PResult<Array> {
    let _size = integer5(input)?;
    let obj_type = integer5(input)?;
    let num_dims = integer5usize(input)?;
    let dims: Vec<usize> = repeat(num_dims, integer5usize).parse_next(input)?;
    let num_objs = dims.iter().fold(1, |acc, x| acc * x);
    let mut objects = Vec::new();
    for _ in 0..num_objs {
        objects.push(next_obj_with_prolog(input, obj_type)?);
    }
    Ok(Array{obj_type, num_dims, dims, objects})
}

pub fn next_integer(input: &mut Nibbles) -> PResult<i32> {
    _next_integer.context(StrContext::Label("integer")).parse_next(input)
}
fn _next_integer(input: &mut Nibbles) -> PResult<i32> {
    let mut nib = next_lv(input)?;
    let digits = decode_bcd(&mut nib)?;
    Ok(digits as i32)
}
pub struct Blob(Vec<u8>);
impl Debug for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Blob({} bytes)", self.0.len())
    }
}

pub struct StringBlob(String);
impl Debug for StringBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.contains("\0") {
            let self_first_10_chars: String = self.0.chars().take(10).collect::<String>(); 
            write!(f, "StringBlob({} bytes, \"{}...\")", self.0.len(), self_first_10_chars)
        } else {
            write!(f, "StringBlob(\"{}\")", self.0)
        }
    }
}
#[derive(Debug)]
pub enum Obj {
    Dir(Dir),
    Real(Real),
    Int(i32),
    CStr(StringBlob),
    Prg(Vec<Obj>),
    List(Vec<Obj>),
    Symb(Vec<Obj>),
    Unit(Vec<Obj>),
    Complex(Complex),
    Array(Array),
    Integer(i128), 
    Ext(u32),
    ExtObj(u32, Blob, String),
    FixedObj(u32, Blob, String),
    Code(Blob),
    GlobalName(String),
    LocalName(String),
    Tagged(String),
    Semi(),
    Library(Library),
}

pub(crate) fn next_obj(nibs: &mut Nibbles) -> PResult<Obj> {
    let prolog = integer5(nibs)?;
    let prolog_str = prolog_to_string(prolog);
    let inner = move |nibs: &mut Nibbles| next_obj_with_prolog(nibs, prolog);
    inner.context(StrContext::Label(prolog_str)).parse_next(nibs)
}

pub(crate) fn next_obj_with_prolog(nibs: &mut Nibbles, prolog: u32 ) -> PResult<Obj> {
    match prolog {
        DORRP => {
            //Dir
            let d = next_dir(nibs)?;
            return Ok(Obj::Dir(d));
        }
        DOREAL => next_real.map(Obj::Real).parse_next(nibs),
        DOCMP => next_complex.map(Obj::Complex).parse_next(nibs),

        DOFLASHP | DOBINT  | DOEREAL | DOECMP | DOCHAR | DOROMP => {
            let size: usize = match prolog {
                DOFLASHP => 7,
                DOBINT => 5,
                DOREAL => 16,
                DOEREAL => 21,
                DOCMP => 32,
                DOECMP => 42,
                DOCHAR => 2,
                DOROMP => 6,
                _ => unreachable!(),
            };
            let data = take(size).parse_next(nibs)?;
            return Ok(Obj::FixedObj(prolog, Blob(data.to_vec()), prolog_to_id(prolog).to_owned()));
        }
        DOINT => next_integer.map(Obj::Int).parse_next(nibs),
        DOCSTR => {
            let sz = integer5usize(nibs)?;
            let sz = sz - 5;
            let cstr = take(sz).parse_next(nibs)?;
            let bytes = nibbles_to_bytes(&Nibbles::new(cstr))?;
            // TODO: create an actual codec for hp4x charset. 
            return Ok(Obj::CStr(StringBlob(String::from_utf8_lossy(&bytes).to_string())));
        }
        DOCOL | DOLIST | DOSYMB | DOEXT => {
            let objs = next_semi_terminated(nibs)?;
            let obj = match prolog {
                DOCOL => Obj::Prg(objs),
                DOLIST => Obj::List(objs),
                DOSYMB => Obj::Symb(objs),
                DOEXT => Obj::Unit(objs),
                _ => unreachable!(),
            };
            return Ok(obj);
        }
        DOCODE => {
            let sz = integer5usize(nibs)?;
            let sz = sz - 5;
            let code = take(sz).parse_next(nibs)?;
            return Ok(Obj::Code(Blob(code.to_vec())));
        }
        SEMI => {
            return Ok(Obj::Semi());
        }
        DOEXT1 | DOEXT2 | DOEXT3 | DOEXT4 | DOGROB | DOARRY | DOLNKARRY | DOHSTR | DOLIB
        | DOBAK | DOEXT0 => {
            let mut data = next_lv(nibs)?;
            match prolog {
                DOEXT1 | DOEXT2 | DOEXT3 | DOEXT4 | DOGROB | DOARRY | DOLNKARRY | DOHSTR
                | DOBAK | DOEXT0 => {
                    return Ok(Obj::ExtObj(prolog, Blob(data.to_vec()), prolog_to_id(prolog).to_owned()));
                }
                DOLIB => {
                    let lib = next_library(&mut data)?;
                    return Ok(Obj::Library(lib));
                
                }
                _ => unreachable!(),
            }
        }
        DOIDNT | DOLAM | DOTAG => {
            let data = pascal_string(nibs)?;
            return Ok(match prolog {
                DOIDNT => Obj::GlobalName(data),
                DOLAM => Obj::LocalName(data),
                DOTAG => Obj::Tagged(data),
                _ => unreachable!(),
            });
        }
        0..0x1000 => {
            return Err(winnow::error::ErrMode::Cut(ParserError::from_error_kind(nibs, ErrorKind::Verify)));
        }
        _ => {
            return Ok(Obj::Ext(prolog));
        }
    }
}

// due to byte encoding, some objects end with a random one nibble padding
fn next_obj_maybe_1_padding(nibs: &mut Nibbles) -> PResult<Obj> {
    let obj = next_obj(nibs)?;
    if nibs.len() > 0 {
        _ = take(nibs.len()).parse_next(nibs)?;
    }
    Ok(obj)
}
pub fn parse_hp4x(path: &Path) -> Result<Obj> {
    // read the file
    let file_contents = std::fs::read(path)?;
    let romrev_header = &file_contents[0..6];
    if romrev_header != b"HPHP48" && romrev_header != b"HPHP49" {
        return Err(Error::BadHeader(String::from_utf8_lossy(romrev_header).to_string()));
    }
    let nibble_array = extract_nibbles(&file_contents[8..]);
    next_obj_maybe_1_padding.parse(Nibbles::new(&nibble_array[..])).map_err(|e| Error::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    //read all files inside the src/fixtures directory relative to cargo env, and try to parse them using parse_hp4x
    #[test]
    fn test_with_fixtures() {
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures");
        for entry in std::fs::read_dir(fixtures).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            println!("testing {:?}", path);
            match parse_hp4x(&path) {
                Ok(v) => {
                    println!("{:#?}", v);
                }
                Err(e) => {
                    panic!("error: {:?}", e);
                }
            }
        }
    }
    // BABL49
    #[test]
    fn test_babal49() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/BABL49");
        match parse_hp4x(&path) {
            Ok(v) => {
                println!("{:#?}", v);
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }
    }
    // Cyclo.49
    #[test]
    fn test_cyclo49() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/Cyclo.49");
        match parse_hp4x(&path) {
            Ok(v) => {
                println!("{:#?}", v);
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }
    }
    // Dir.1
    #[test]
    fn test_dir1() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/DIR.1");
        match parse_hp4x(&path) {
            Ok(v) => {
                println!("{:#?}", v);
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }
    }   
    // EXTABLE.HP
    #[test]
    fn test_extable() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/extable.HP");
        match parse_hp4x(&path) {
            Ok(v) => {
                println!("{:#?}", v);
            }
            Err(e) => {
                panic!("error: {:?}", e);
            }
        }
    }

}