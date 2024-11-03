
// base nibble parser utilities, built with winnow
// basically a re-design of nibblers.rs
// but using winnow instead of nom
use winnow::error::{ErrMode, ErrorKind, Needed, ParserError, StrContext};
use winnow::prelude::*;
use winnow::stream::{Stream, Located, Location};

pub type Nibbles<'a> = Located<&'a[u8]>;

pub fn hexdump_nibbles(nibs: Nibbles, limit: Option<usize>) -> String {
    let l = std::cmp::min(nibs.len(), limit.unwrap_or(200));
    let mut s = String::new();
    let offset = nibs.location();
    for i in (0..l).step_by(16) {
        let mut line = String::new();
        s.push_str(&format!("{:04x}  ", offset + i));
        for j in i..i + 16 {
            if j < nibs.len() {
                line.push_str(&format!("{:01x} ", nibs[j]));
            } else {
                line.push_str("  ");
            }
        }
        line.push_str("  ");
        for shift in 0..2 {
            for j in 0..8 {
                if i + j * 2 + shift + 1 < nibs.len() {
                    let mut b = nibs[i + j * 2 + shift + 1];
                    b <<= 4;
                    b |= nibs[i + j * 2 + shift];
                    let c = b as char;
                    if c.is_ascii() && c >= 0x1f as char {
                        line.push(c);
                    } else {
                        line.push('.');
                    }
                } else {
                    line.push(' ');
                }
            }   
            if shift == 0 {
                line.push_str("  ");
            }
        }
        s.push_str(&line);
        s.push('\n');
    }
    s
}
pub fn print_nibbles(nibbles: Nibbles) {
    println!("{}", hexdump_nibbles(nibbles, None));
}
/// extract nibbles from a buffer of u8
pub fn extract_nibbles(input: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::new();
    for byte in input {
        nibbles.push(byte & 0x0f);
        nibbles.push(byte >> 4);
    }
    nibbles
}

macro_rules! integer {
    ($name:ident, $count:expr, $output:ty) => {
        #[allow(dead_code)]
        pub fn $name(input: &mut Nibbles) -> PResult<$output> {
            fn inner(input: &mut Nibbles) -> PResult<$output> {
                if input.len() < $count {
                    return Err(ErrMode::Cut(ParserError::from_error_kind(input, ErrorKind::Eof)));
                }
                let input = input.next_slice($count);
                let mut value: $output = 0;
                for i in (0..$count).rev() {
                    value <<= 4;
                    value |= input[i] as $output;
                }
                Ok(value)
            }
            inner.context(StrContext::Label(stringify!($name))).parse_next(input)
        }
    };
}
pub fn decode_bcd(input: &mut Nibbles) -> PResult<u64> {
    let mut value: u64 = 0;
    for i in 0..input.len() {
        value = (value * 10) + (input[i] as u64);
    }
    Ok(value)
}
integer!(integer12, 12usize, u64);
integer!(integer5usize, 5usize, usize);
integer!(integer5, 5usize, u32);
integer!(integer4, 4usize, u16);
integer!(integer3, 3usize, u16);
integer!(integer2usize, 2usize, usize);
integer!(integer2, 2usize, u8);
integer!(integer1, 1usize, u8);

// generator for bcd encoded integers
macro_rules! bcd {
    ($name:ident, $count:expr, $output:ty) => {
        #[allow(dead_code)]
        pub fn $name(input: &mut Nibbles) -> PResult<$output> {
            let input = input.next_slice($count);
            let mut value: $output = 0;
            for i in 0..$count {
                value = (value * 10) + (input[i] as $output);
            }
            Ok(value)
        }
    };
}
bcd!(bcd12, 12usize, u64);
bcd!(bcd5, 5usize, u32);
bcd!(bcd4, 4usize, u16);
bcd!(bcd3, 3usize, u16);
bcd!(bcd2, 2usize, u8);
bcd!(bcd1, 1usize, u8);

/// take an offset(5) from the input
/// if the offset is 0, return None
/// otherwise, return Some(slice pointing to the offset)
/// the input is only advanced for the offset, not for the slice
pub fn next_offset<'a>(input: &mut Nibbles<'a>) -> PResult<Option<Nibbles<'a>>> {
    _next_offset.context(StrContext::Label("offset")).parse_next(input)
}

fn _next_offset<'a>(input: &mut Nibbles<'a>) -> PResult<Option<Nibbles<'a>>> {
    let offset = integer5usize(input)?;
    if offset > 0 {
        if offset - 5 >= input.len() {
            return Err(ErrMode::Cut(ParserError::from_error_kind(input, ErrorKind::Eof)));
        }
        let mut input = input.clone();
        let _ = input.next_slice(offset - 5);
        Ok(Some(input))
    } else {
        Ok(None)
    }
}
/// take an offset(5) from the input
/// if the offset is 0, return an error
pub fn next_mandatory_offset<'a>(input: &mut Nibbles<'a>) -> PResult<Nibbles<'a>> {
    next_offset(input)?.ok_or_else(|| {
        ErrMode::Cut(ParserError::from_error_kind(input, ErrorKind::Assert))
    })
}
/// return a slice of the previous nibbles from the input
pub fn previous_nibbles<'a>(input: &Nibbles<'a>, count: usize) -> PResult<Nibbles<'a>> {
    let mut input = input.clone();
    let location = input.location();
    input.reset_to_start();
    if location < count {
        return Err(ErrMode::Cut(ParserError::from_error_kind(&input, ErrorKind::Assert)));
    }
    input.next_slice(location - count);
    Ok(input)
}
/// Convert the whole input to u8
pub fn nibbles_to_bytes(input: &Nibbles) -> PResult<Vec<u8>> {
    let len = input.len() / 2;
    let mut bytes = Vec::new();
    for i in 0..len {
        let mut b = input[i * 2 + 1];
        b <<= 4;
        b |= input[i * 2];
        bytes.push(b);
    }
    Ok(bytes)
}
/// parse a pascal string
pub fn pascal_string(input: &mut Nibbles) -> PResult<String> {
    _pascal_string.context(StrContext::Label("pascal string")).parse_next(input)
}
fn _pascal_string(input: &mut Nibbles) -> PResult<String> {
    let len: usize = integer2usize(input)? * 2;
    if len > input.len() {
        return Err(ErrMode::Cut(ParserError::from_error_kind(input, ErrorKind::Eof)));
    }
    let str = input.next_slice(len);
    let bytes = nibbles_to_bytes(&Nibbles::new(str))?;
    // TODO: create an actual codec for hp4x charset. 
    //would be useful to convert e.g for the -> char
    let s = String::from_utf8_lossy(&bytes).to_string();
    Ok(s)
}
pub fn next_tlv<'a>(input: &mut Nibbles<'a>) -> PResult<(u32, Nibbles<'a>)> {
    _next_tlv.context(StrContext::Label("tlv")).parse_next(input)
}
fn _next_tlv<'a>(input: &mut Nibbles<'a>) -> PResult<(u32, Nibbles<'a>)> {
    let tag = integer5(input)?;
    let length = integer5usize(input)?;
    if length > input.len() {
        return Err(ErrMode::Cut(ParserError::from_error_kind(input, ErrorKind::Eof)));
    }
    let value: &[u8] = input.next_slice(length);
    // todo the new located should be relative to the original input
    // but there is missing api in winnow to do that
    Ok((tag, Located::new(value)))
}

pub fn next_lv<'a>(input: &mut Nibbles<'a>) -> PResult<Nibbles<'a>> {
    _next_lv.context(StrContext::Label("lv")).parse_next(input)
}
fn _next_lv<'a>(input: &mut Nibbles<'a>) -> PResult<Nibbles<'a>> {
    let length = integer5usize(input)? - 5;
    if length > input.len() {
        return Err(ErrMode::Cut(ParserError::from_error_kind(input, ErrorKind::Eof)));
    }
    let value: &[u8] = input.next_slice(length);
    // todo the new located should be relative to the original input
    // but there is missing api in winnow to do that
    Ok(Located::new(value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_nibbles() {
        let input = vec![0x12, 0x34, 0x56, 0x78];
        let expected = vec![0x2, 0x1, 0x4, 0x3, 0x6, 0x5, 0x8, 0x7];
        let result = extract_nibbles(&input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_integer5() {
        let input = vec![0x1, 0x2, 0x3, 0x4, 0x5];
        let expected = 0x54321;
        let slice = Nibbles::new(&input[..]);
        assert_eq!(integer5.parse(slice).unwrap(), expected);
    }
    // test usage of winnow tuple combinator
    #[test]
    fn test_integer3_then_integer2() {
        let input = vec![0x1, 0x2, 0x3, 0x4, 0x5];
        let slice = Nibbles::new(&input[..]);
        let expecteda = 0x21;
        let expectedb = 0x543;
        let (a, b) = (integer2, integer3).parse(slice).unwrap();
        assert_eq!(a, expecteda);
        assert_eq!(b, expectedb);
    }
    #[test]
    fn test_next_offset() {
        let input = vec![0x8, 0, 0, 0, 0, 0, 0, 0 ,0x1, 0x2, 0x3, 0x4, 0x5];
        let mut nibbles: Nibbles = Nibbles::new(&input);
        let offset = next_offset(&mut nibbles).unwrap().unwrap();
        assert_eq!(*offset, &input[8..]);
    }
    // error cases
    #[test]
    fn test_next_offset_incomplete() {
        let input = vec![0x10, 0, 0, 0, 0, 0x2, 0x3, 0x4, 0x5];
        let nibbles: Nibbles = Nibbles::new(&input);
        let err = next_offset.parse(nibbles).unwrap_err();
        assert_eq!(err.to_string(), "\u{10}\0\0\0\0\u{2}\u{3}\u{4}\u{5}\n     ^\ninvalid offset");
    }
    #[test]
    fn test_pascal_string() {
        let input = vec![0x3, 0x0, 0x1, 0x6, 0x2, 0x6, 0x3, 0x6];
        let mut nibbles: Nibbles = Nibbles::new(&input);
        let s = pascal_string(&mut nibbles).unwrap();
        assert_eq!(s, "abc");
    }
    // error cases
    #[test]
    fn test_pascal_string_incomplete() {
        let input = vec![0x03, 0x00, 0x01, 0x06, 0x02, 0x06, 0x03];
        let nibbles: Nibbles = Nibbles::new(&input);
        let err = pascal_string.parse(nibbles).unwrap_err();
        assert_eq!(format!("{:}", err), "\u{3}\0\u{1}\u{6}\u{2}\u{6}\u{3}\n  ^\ninvalid pascal string");
    }

    #[test]
    fn test_hexdump_nibbles() {
        let input = vec![0x2, 0x6, 0xf, 0x6, 0xe, 0x6, 0xa, 0x6, 0xf, 0x6, 0x5, 0x7, 0x2, 0x7, 0x1,0x2];
        let mut nibbles:Nibbles = Nibbles::new(&input);
        let s = hexdump_nibbles(nibbles, None);
        println!("{}", s);
        assert_eq!(s, "0000  2 6 f 6 e 6 a 6 f 6 5 7 2 7 1 2   bonjour!  ....V'. \n");
        nibbles.next_slice(3);
        // shifted with odd number, we see the ascii in the right
        let s = hexdump_nibbles(nibbles, None);
        assert_eq!(s, "0003  6 e 6 a 6 f 6 5 7 2 7 1 2         ...V'.    njour!  \n");
    }
}
