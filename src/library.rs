// documentation by Gilbert Fernandes and Eric Rechlin.
// do extracted from asmtut.doc, then converted by pandoc into markdown
// for better readability by LLMs
// "Voyage au centre" documents the library structure in a more detailed way
// but its license is very clear about redistribution.
// ## Library
// Prologue: 02B40 Epilogue: none
// Size: varies Type: 16
// In memory:
//   ------------------------------------------ ---- -----------------------
//   Prologue (02B40)                                5 nibbles
//   Size (CRC not included)                         5 nibbles
//   Library name's size                             2 nibbles
//   Library name's first character                  5 nibbles
//   ...
//   Library name's last character                   2 nibbles
//   Library name's size                             2 nibbles
//   Library number                                  3 nibbles
//   Hash table offset                               5 nibbles
//   Message table offset                            5 nibbles
//   Link table offset                               5 nibbles
//   Config object offset                            5 nibbles
//   Message table array                             varies
//   Link table                                      varies
//   XLIB 1 (kind)                                   1 or 3 nibbles
//   Library number                                  3 nibbles
//   Command number                                  3 nibbles
//   XLIB 1 objects                                  varies
//   XLIB 2 (kind)                                   1 or 3 nibbles
//   Library number                                  3 nibbles
//   Command number                                  3 nibbles
//   XLIB 2 objects                                  varies
//   ...
//   XLIB *n* (kind)                                 1 or 3 nibbles
//   Library number                                  3 nibbles
//   Command number                                  3 nibbles
//   XLIB *n* objects                                varies
//   Masked object 1                                 varies
//   ...
//   Last masked object                              varies
//   Config object                                   varies
//   CRC                                             4 nibbles
//   ------------------------------------------ ---- -----------------------
// Not only is the library structure somewhat complex, but it varies too.
// Four objects inside the library object have a very specific structure:
// the hash table, the message table, the link table and the config object.
// As you have seen, there are four offsets inside the library with this
// very purpose.
// If the library has no name, its first length field will be zero, and
// there will **not** be a second length field. If there is a name, we find
// its length, then the characters, then again the length. A library
// without a name will not appear when you will do \[Right-shift\]\[2\].
// The hash table is used to increase the access speed of library commands.
// The message table contains all custom error messages of the library and
// the link table references all objects of the library. Offsets are used
// inside the library, and if an offset is #00000h then the object we are
// looking for does not exist.
// We will discuss the four object types below. But first there is
// something else about the library that's important: there are two types
// of objects, those that are visible, called XLIB's, and those which are
// masked. For each *visible* object of the library, you will find:
// -   its kind (encoded using 1 to 3 nibbles)
// -   the library number (3 nibbles)
// -   the command number (3 nibbles)
// The CRC is calculated using a hardware CRC circuit inside the HP. The
// CRC of an object is calculated from the size field (the prologue is not
// included) to the end of the object. The CRC is *not* a part of the
// object that is CRC'd. Then, the CRC is added to the library. The
// calculated size of the library contains neither the prologue nibbles nor
// the CRC nibbles.
// ### Hash table
// The purpose of this table is to speed up access to library commands.
// It's very simple, so the gain is not monstrous, but it's a good idea.
// Commands are distributed according to their name's length, from 1 to 16.
// A command can, however, have more than 16 characters, but then it will
// be grouped with the 16-character long commands.
// Here is the memory scheme of the hash table:
// Prologue: 02A4E Epilogue: none
// Size: varies Type: none
// Category 1: commands whose names are 1 character long
// ...
// Category *n*: commands whose name are *n* characters long
// In memory:
//   -------------------------------------- ----- ---------------------------
//   Prologue (02A4E)                             5 nibbles
//   Size of hash table                           5 nibbles
//   First category offset                        5 nibbles
//   ...
//   Last category offset                         5 nibbles
//   Size of the names list                       2 nibbles
//   Size of one-character command names          2 nibbles
//   First one-character command                  2 nibbles
//   Number of command                            2 nibbles
//   Second one-character command                 2 nibbles
//   Number of command                            2 nibbles
//   ...
//   *n*th one-character command                  2 nibbles
//   Size of 2 character command names            2 nibbles
//   First two-character command                  4 nibbles
//   Number of command                            2 nibbles
//   ...
//   Offset to command 1                          5 nibbles
//   ...
//   Offset to last command                       5 nibbles
//   -------------------------------------- ----- ---------------------------
// Here, a command that has no visible name is not listed in the hash
// table. Each library command has a name and number inside the library.
// This information is ordered in the hash table. The command number
// follows the names: just after the first one-character command we find
// its number, and so on. First, we order by length of name, and after each
// name we put the command number as inside the library, but only for
// visible commands. Next, we find the hash table offsets, of which there
// are two kinds: one to find a command according to its length and another
// to find the command using the offsets found at the end of the hash
// table.
// ### Message table
// The message table can have two forms: it can either be a
// single-dimensional array or an *indexed* single-dimensional array.
// If we find an array (prologue 029E8) we will find the number of messages
// inside the library, followed by all messages, each one starting with its
// length (Pascal string encoding), and then each ASCII byte of each
// character.
// If we find an indexed array (prologue 02A0A) we will find the number of
// messages and then a list of offsets to each message. Its length (still
// Pascal string encoding) precedes each message and is followed with one
// ASCII byte for each character of the message.
// ### Link table
// The link table is a simple binary integer. It encodes the addresses of
// the library's objects inside. You can divide this binary integer into
// chunks of five nibbles. The first one is the prologue (02A4E) and is
// followed by the length, which, as usual, does not include the prologue's
// length. After that, every five nibbles is an address to an object. Here
// we just have a table of addresses, one for each object inside the
// library.

use std::collections::HashMap;

use crate::next_array;
use crate::next_obj;
use crate::nibbles::*;
use crate::Obj;
use winnow::combinator::repeat;
use winnow::error::StrContext;
use winnow::stream::Location;
use winnow::token::take;
use winnow::PResult;
use winnow::Parser;

#[derive(Debug)]
pub struct Xlib {
    pub kind: u16,
    pub library_number: u16,
    pub command_number: u16,
    pub object: Box<Obj>,
}
#[derive(Debug)]
pub struct Library {
    pub name: String,
    pub number: u16,

    pub message_table: Vec<String>,
    pub hash_table: HashTable,
    pub xlib: Vec<Xlib>,
    pub hidden_objects: Vec<Obj>,
    pub config_object: Option<Box<Obj>>,
    pub extra_objects: Vec<Obj>,
    pub crc: u16,
}
#[derive(Debug, Default)]
pub struct HashTable {
    // Hashtable references object names to command number
    pub name_to_cmd: HashMap<String, u16>,
    pub cmd_to_name: HashMap<u16, String>,
}

fn next_message_table(nib: &mut Nibbles) -> PResult<Vec<String>> {
    _next_message_table.context(StrContext::Label("message table")).parse_next(nib)
}
fn _next_message_table(nib: &mut Nibbles) -> PResult<Vec<String>> {
    let prolog = integer5(nib)?;
    assert_eq!(prolog, 0x029E8);
    let array = next_array(nib)?;
    let messages = array
        .objects
        .iter()
        .map(|x| match x {
            Obj::CStr(s) => s.0.clone(),
            _ => panic!("expected string in message table"),
        })
        .collect();
    Ok(messages)
}
fn next_link_table<'a>(nib: &mut Nibbles<'a>) -> PResult<Vec<Nibbles<'a>>> {
    _next_link_table.context(StrContext::Label("link table")).parse_next(nib)
}
fn _next_link_table<'a>(nib: &mut Nibbles<'a>) -> PResult<Vec<Nibbles<'a>>> {
    let prolog = integer5(nib)?;
    assert_eq!(prolog, 0x02A4E);
    let size = integer5usize(nib)?;
    let num_links = (size / 5) - 1;
    let links = repeat(num_links, next_mandatory_offset).parse_next(nib)?;
    Ok(links)
}
fn next_hash_table(nib: &mut Nibbles) -> PResult<HashTable> {
    _next_hash_table.context(StrContext::Label("hash table")).parse_next(nib)
}
fn _next_hash_table(nib: &mut Nibbles) -> PResult<HashTable> {
    let (prolog, mut blob) = next_tlv(nib)?;
    let nib = &mut blob;
    assert_eq!(prolog, 0x02A4E);
    let mut name_to_cmd = HashMap::new();
    let mut cmd_to_name = HashMap::new();

    // first we take 16 offsets of name lists
    let name_list_buffers: Vec<Option<Nibbles>> = repeat(16, next_offset).parse_next(nib)?;
    let reverse_table = next_mandatory_offset(nib)?;
    //filter out the offset that are zero
    let name_list_buffers: Vec<Nibbles> = name_list_buffers.into_iter().flatten().collect();

    for i in 0..name_list_buffers.len() {
        let buffer = name_list_buffers[i];
        let last_position = if i == name_list_buffers.len() - 1 {
            reverse_table.location()
        } else {
            name_list_buffers[i + 1].location()
        };
        let mut nib = buffer;
        while nib.location() < last_position {
            let name = pascal_string(&mut nib)?;
            if name.len() == 0 {
                break;
            }

            let cmd_number = integer3(&mut nib)?;
            name_to_cmd.insert(name.clone(), cmd_number);
            cmd_to_name.insert(cmd_number, name);
        }
    }
    Ok(HashTable {
        name_to_cmd,
        cmd_to_name,
    })
}
// we expect xlibs are objects which have library number 3 nibbles before the pointer
// in the link table

// @-7: (or @-9-7)kind (explained in voyage au centre, but not in asmtut.doc)
// @-6-4: library number
// @-3-1: command number
fn find_xlib_header(xlib_object: &Nibbles) -> PResult<(u16, u16, u16)> {
    let mut prev_nibbles = previous_nibbles(xlib_object, 9)?;
    let kind = prev_nibbles[2];
    let kind = if kind & 0x8 == 0 {
        // in this case, we fetch 2 nibbles too many
        let _ = integer2(&mut prev_nibbles)?;
        integer1(&mut prev_nibbles)? as u16
    } else {
        integer3(&mut prev_nibbles)?
    };

    let obj_library_number = integer3(&mut prev_nibbles)?;
    let command_number = integer3(&mut prev_nibbles)?;
    Ok((kind, obj_library_number, command_number))
}

/// Decode a library object (without header and size)
pub(crate) fn next_library(nib: &mut Nibbles) -> PResult<Library> {
    let lib = nib.clone();
    let name = pascal_string(nib)?;
    let name_len_back = integer2(nib)?;
    assert_eq!(name_len_back, name.len() as u8);
    let number = integer3(nib)?;

    // those tables are in arbitrary order
    // we must use the offset in order to parse them
    let hash_table_nibs = next_offset(nib)?;
    let message_table_nibs = next_offset(nib)?;
    let link_table_nibs = next_offset(nib)?;
    let config_object_nibs = next_offset(nib)?;

    let mut extra_objects = Vec::new();
    let hash_table = if let Some(nib) = hash_table_nibs {
        let mut nib = nib;
        next_hash_table(&mut nib)?
    } else {
        HashTable::default()
    };

    let link_table = if let Some(nib) = link_table_nibs {
        let mut nib = nib;
        next_link_table(&mut nib)?
    } else {
        Vec::new()
    };
    // take message table
    let message_table = if let Some(nib) = message_table_nibs {
        let mut nib = nib;
        next_message_table(&mut nib)?
    } else {
        Vec::new()
    };
    let mut hidden_objects = Vec::new();
    let mut xlib = Vec::new();
    let mut last_obj_location = 0;
    // extract the objects from the link table
    for offset in link_table.clone() {
        if last_obj_location != 0 && last_obj_location + 10 < offset.location() {
            // there are extra objects in between objects from the link table
            // this trick is used for extable, maybe for other objects too
            let mut nibs = Nibbles::new(&lib[last_obj_location as usize..offset.location()]);
            // on extable, there are 16 nibble of headers (HPHP48..)
            let _ = take(16usize).parse_next(&mut nibs)?;
            if let Ok(obj) =  next_obj(&mut nibs) {
                extra_objects.push(obj);
        }
        }
        // skip if config object
        if offset.location() == config_object_nibs.map(|x| x.location()).unwrap_or(0) {
            continue;
        }
        let (kind, library_number, command_number) = find_xlib_header(&offset)?;
        let mut nib = offset;
        let obj = next_obj(&mut nib)?;
        last_obj_location = nib.location();

        // we always parse the xlib header, but if the library number does not match,
        // this means this is a hidden object
        if library_number == number {
            xlib.push(Xlib {
                kind,
                library_number,
                command_number,
                object: Box::new(obj),
            });
        } else {
            hidden_objects.push(obj);
        }
    }
    let config_object = if let Some(nib) = config_object_nibs {
        let mut nib = nib;
        Some(Box::new(next_obj(&mut nib)?))
    } else {
        None
    };
    let crc = integer4(nib)?;
    Ok(Library {
        name,
        number,
        message_table,
        hash_table,
        xlib,
        hidden_objects,
        config_object,
        extra_objects,
        crc,
    })
}
