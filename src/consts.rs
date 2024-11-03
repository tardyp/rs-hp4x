pub const DOBINT: u32 = 0x2911;
pub const DOINT: u32 = 0x02614;       // Precision Integer (HP49G)
pub const DOLNGREAL: u32 = 0x0263A;   // Precision Real (HP49G)
pub const DOLNGCMP: u32 = 0x02660;    // Precision Complex (HP49G)
pub const DOMATRIX: u32 = 0x02686;    // Symbolic matrix (HP49G)
pub const DOFLASHP: u32 = 0x026AC;    // Flash PTR (HP49G)
pub const DOAPLET: u32 = 0x026D5;     // Aplet (HP49G)
pub const DOMINIFONT: u32 = 0x026FE;  // Mini Font (HP49G)
pub const DOREAL: u32 = 0x02933;      // Real
pub const DOEREAL: u32 = 0x02955;     // Long Real
pub const DOCMP: u32 = 0x02977;       // Complex
pub const DOECMP: u32 = 0x0299D;      // Long Complex
pub const DOCHAR: u32 = 0x029BF;      // Character
pub const DOARRY: u32 = 0x029E8;      // Array
pub const DOLNKARRY: u32 = 0x02A0A;   // Linked Array
pub const DOCSTR: u32 = 0x02A2C;      // String
pub const DOHSTR: u32 = 0x02A4E;      // Binary Integer
pub const DOLIST: u32 = 0x02A74;      // List
pub const DORRP: u32 = 0x02A96;       // Directory
pub const DOSYMB: u32 = 0x02AB8;      // Algebraic
pub const DOTAG: u32 = 0x02AFC;       // Tagged
pub const DOEXT1: u32 = 0x02BAA;      // Extended Pointer
pub const DOEXT: u32 = 0x02ADA;       // Unit
pub const DOGROB: u32 = 0x02B1E;      // Graphic
pub const DOLIB: u32 = 0x02B40;       // Library
pub const DOBAK: u32 = 0x02B62;       // Backup
pub const DOEXT0: u32 = 0x02B88;      // Library Data
pub const DOEXT2: u32 = 0x02BCC;      // Reserved 1, Font (HP49G)
pub const DOEXT3: u32 = 0x02BEE;      // Reserved 2
pub const DOEXT4: u32 = 0x02C10;      // Reserved 3
pub const DOCOL: u32 = 0x02D9D;       // Program
pub const DOCODE: u32 = 0x02DCC;      // Code
pub const DOIDNT: u32 = 0x02E48;      // Global Name
pub const DOLAM: u32 = 0x02E6D;       // Local Name
pub const DOROMP: u32 = 0x02E92;      // XLIB Name
pub const SEMI: u32 = 0x0312B;        // ;

pub const GARBAGECOL: u32 = 0x0613E;  // =GARBAGECOL entry for HP48S/G/GII and HP49G(+)

#[allow(dead_code)]
pub fn prolog_to_string(prolog: u32) -> &'static str {
    match prolog {
        DOBINT => "Binary Integer",
        DOINT => "Precision Integer (HP49G)",
        DOLNGREAL => "Precision Real (HP49G)",
        DOLNGCMP => "Precision Complex (HP49G)",
        DOMATRIX => "Symbolic matrix (HP49G)",
        DOFLASHP => "Flash PTR (HP49G)",
        DOAPLET => "Aplet (HP49G)",
        DOMINIFONT => "Mini Font (HP49G)",
        DOREAL => "Real",
        DOEREAL => "Long Real",
        DOCMP => "Complex",
        DOECMP => "Long Complex",
        DOCHAR => "Character",
        DOARRY => "Array",
        DOLNKARRY => "Linked Array",
        DOCSTR => "String",
        DOHSTR => "Binary Integer",
        DOLIST => "List",
        DORRP => "Directory",
        DOSYMB => "Algebraic",
        DOTAG => "Tagged",
        DOEXT1 => "Extended Pointer",
        DOEXT => "Unit",
        DOGROB => "Graphic",
        DOLIB => "Library",
        DOBAK => "Backup",
        DOEXT0 => "Library Data",
        DOEXT2 => "Reserved 1, Font (HP49G)",
        DOEXT3 => "Reserved 2",
        DOEXT4 => "Reserved 3",
        DOCOL => "Program",
        DOCODE => "Code",
        DOIDNT => "Global Name",
        DOLAM => "Local Name",
        DOROMP => "XLIB Name",
        SEMI => ";",
        GARBAGECOL => "=GARBAGECOL entry for HP48S/G/GII and HP49G(+)",
        _ => "Unknown prolog",
    }
}
pub fn prolog_to_id(prolog: u32) -> &'static str {
    match prolog {
        DOBINT => "DOBINT",
        DOINT => "DOINT",
        DOLNGREAL => "DOLNGREAL",
        DOLNGCMP => "DOLNGCMP",
        DOMATRIX => "DOMATRIX",
        DOFLASHP => "DOFLASHP",
        DOAPLET => "DOAPLET",
        DOMINIFONT => "DOMINIFONT",
        DOREAL => "DOREAL",
        DOEREAL => "DOEREAL",
        DOCMP => "DOCMP",
        DOECMP => "DOECMP",
        DOCHAR => "DOCHAR",
        DOARRY => "DOARRY",
        DOLNKARRY => "DOLNKARRY",
        DOCSTR => "DOCSTR",
        DOHSTR => "DOHSTR",
        DOLIST => "DOLIST",
        DORRP => "DORRP",
        DOSYMB => "DOSYMB",
        DOTAG => "DOTAG",
        DOEXT1 => "DOEXT1",
        DOEXT => "DOEXT",
        DOGROB => "DOGROB",
        DOLIB => "DOLIB",
        DOBAK => "DOBAK",
        DOEXT0 => "DOEXT0",
        DOEXT2 => "DOEXT2",
        DOEXT3 => "DOEXT3",
        DOEXT4 => "DOEXT4",
        DOCOL => "DOCOL",
        DOCODE => "DOCODE",
        DOIDNT => "DOIDNT",
        DOLAM => "DOLAM",
        DOROMP => "DOROMP",
        SEMI => "SEMI",
        GARBAGECOL => "GARBAGECOL",
        _ => "Unknown prolog",
    }
}