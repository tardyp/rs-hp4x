use std::collections::HashMap;

use winnow::{error::{ErrMode, ErrorKind, ParserError}, PResult, stream::Location};

use crate::{library::Library, nibbles::*, DOEXT3};

// extable data format
// @0 -> offset to ???
// @5 -> pointer to begining of name table

// Then hashtable of entries
// hashtable is 2 levels deep
// @A -> @285: offset to table of offsets to names
// each intermediate table ends with 0x00000 
// hash algorithm is unknown to me. There is a chance it is based on the HP48's hardwired CRC module:
// x^16 + x^12+ ×^5 + 1 (the same as the kermit CRC, aka CRC-16-CCITT)
// @285 -> intermediate table, there is a large chunk of data, probably mapping of addresses to names

// Name entry format:
// 2 nibbles: length of name
// n nibbles: name
// 5 nibbles: address
#[derive(Debug, Default)]
pub struct Extable {
    pub name_to_addr: HashMap<String, u32>,
    pub addr_to_name: HashMap<u32, String>,
}
impl From<Library> for Extable {
    fn from(lib: Library) -> Self {
        let obj = lib
            .extra_objects
            .into_iter()
            .next()
            .expect("table is in extra objects, but this lib doesn't have it");
        match obj {
            crate::Obj::ExtObj(DOEXT3, nib, _) => {
                let mut buf = Nibbles::new(&nib.0);
                parse_ext3(&mut buf).unwrap()
            }
            _ => panic!("expected ext3, got {:?}", obj),
        }
    }
}
#[derive(Debug)]
struct Entry {
    name: String,
    addr: u32,
}
fn parse_ext3(nib: &mut Nibbles) -> PResult<Extable> {
    let _address_table = next_mandatory_offset(nib)?;
    let mut name_table = next_mandatory_offset(nib)?;
    let entries = extract_entries(&mut name_table)?;
    Ok(Extable {
        name_to_addr: entries.iter().map(|e| (e.name.clone(), e.addr)).collect(),
        addr_to_name: entries.into_iter().map(|e| (e.addr, e.name)).collect(),
    })
}
fn next_entry(nib: &mut Nibbles) -> PResult<Entry> {
    let name = pascal_string(nib)?;
    // verify that the name is really ascii
    if name.contains('\0') {
        return Err(ErrMode::Cut(ParserError::from_error_kind(nib, ErrorKind::Verify)));
    }
    let addr = integer5(nib)?;
    Ok(Entry { name, addr })
}

fn extract_entries(all: &mut Nibbles) -> PResult<Vec<Entry>> {
    let _ = integer5(all)?;
    let mut entries = Vec::new();
    let mut nib = all.clone();
    while nib.len() > 0 {
        if let Ok(entry) = next_entry(&mut nib) {
            entries.push(entry);
        } else {
            break;
        }
    }
    Ok(entries)
}


#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{parse_hp4x, Obj};
    #[test]
    fn test_extable() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/fixtures/extable.HP");
        let obj = parse_hp4x(&path).expect("failed to parse extable");
        if let Obj::Library(lib) = obj {
            let extable = Extable::from(lib);
            assert_eq!(extable.name_to_addr.len(), 5307);
            assert_eq!(extable.name_to_addr["xDISP"], 0x3816b);
        } 
    }
}
