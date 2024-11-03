use winnow::PResult;
use crate::{Obj, next_obj};
use crate::nibbles::*;

#[derive(Debug)]
pub struct DirEntity {
    pub name: String,
    pub obj: Obj,
}
#[derive(Debug)]
pub struct Dir {
    pub entities: Vec<DirEntity>,
}
pub(crate) fn next_dir_entity(nibs: &mut Nibbles) -> PResult<DirEntity> {
    // An entity consists of an ASCIX name followed by the contents of
    // the object. We need to read the ASCIX name, then the object
    // contents.
    // dump_nibbles(nibs);
    let name = pascal_string(nibs)?;
    let name_len_back = integer2usize(nibs)?;
    debug_assert_eq!(name.len(), name_len_back);
    let obj = next_obj(nibs)?;
    Ok(
        DirEntity {
            name: name,
            obj: obj,
        },
    )
}
// Prologue (5 nibbles): 02A96 – This identifies the object type as a subdirectory.

// Number of Attached Libraries (3 nibbles): This field indicates how many libraries are currently attached to this specific subdirectory. Only one library can be directly associated with a subdirectory; HOME has more flexible library attachment. A value of 7FFh (hex) signifies that no libraries are attached.

// Offset to Last Object (5 nibbles): This is a crucial pointer. It represents the memory offset (in nibbles) to the last object within the subdirectory. It helps the calculator quickly navigate to the end of the directory's contents when searching.

// Five Null Nibbles (5 nibbles): These are always present and probably act as padding or a marker to separate the header information from the actual object entries.

// Object Entries (variable): This is the main part. Each object entry consists of the following:

// Number of Characters in Object Name (2 nibbles): Length of the object's name (ASCII).
// Object Name (variable): The object's name. ASCII encoding is used; each character takes two nibbles.
// Object Data (variable): This is the object itself. It can be any type of HP 48/49 object. The size and type of this object vary.
// Size of Object Data (5 nibbles): The size of the full object data (including name and lenght) (in nibbles) follows.

pub(crate) fn next_dir(nibs: &mut Nibbles) -> PResult<Dir> {

    let attached_libs = integer3(nibs)?;
    let offset = integer5(nibs)?;
    let _zeros = integer5(nibs)?;
    let offset = offset - 10;
    println!("attached_libs: {:x}, offset: {:x}", attached_libs, offset);
    let mut entities = Vec::new();
    // offset is the offset of the last object of the directory
    let last_obj_slice = &nibs[offset as usize..];
    loop {
        let entity = next_dir_entity(nibs)?;
        entities.push(entity);
        if nibs.as_ptr() >= last_obj_slice.as_ptr() {
            break;
        }
        // 5 nibbles after each object but the last one (so that the dir can be parsed backwards)
        let _ = integer5(nibs)?;
    }
    Ok(Dir { entities: entities })
}
