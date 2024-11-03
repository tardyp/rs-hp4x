use crate::{hexdump_nibbles, nibbles::Nibbles, Extable, Obj};

/// module to decompile hp4x objects
/// this is a little bit like debug, but more adapted to viewing the objects
/// in a human readble format
///

pub trait Decompiled {
    fn decompile(&self, extable: &Extable) -> String;
}

impl Decompiled for Obj {
    fn decompile(&self, extable: &Extable) -> String {
        match self {
            Obj::Array(arr) => {
                let mut s = format!(
                    "Array of type {:x}, with {} dimensions: [",
                    arr.obj_type, arr.num_dims
                );
                for dim in &arr.dims {
                    s.push_str(&format!("{}, ", dim));
                }
                s.push_str("]\n");
                for obj in &arr.objects {
                    s.push_str(&obj.decompile(extable));
                }
                s
            }
            Obj::ExtObj(typ_code, blob, typ) => {
                // dump the nibbles in hexadecimal, by rows of 16
                // put the ext type in the first line
                let mut s = format!("ExtObj of type {:x} {}\n", typ_code, typ);
                let nib = Nibbles::new(&blob.0);
                s.push_str(&hexdump_nibbles(nib, Some(usize::MAX)));
                s
            }
            Obj::CStr(str) => str.0.clone(),
            Obj::Real(v) => v.decompile(extable),
            Obj::Complex(v) => v.decompile(extable),
            Obj::Prg(v) => {
                let mut s = "// Program:\n".to_string();
                for obj in v {
                    s.push_str(&obj.decompile(extable));
                    s.push(char::from(10));
                }
                s
            }
            Obj::Ext(v) => {
                if let Some(n) = extable.addr_to_name.get(v) {
                    n.clone()
                } else {
                    format!("Ext{:x}", v)
                }
            }
            Obj::Semi() => ";".to_string(),
            _ => format!("{:?}", self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::extable::Extable;
    use std::collections::HashMap;

    #[test]
    fn test_decompile_real() {
        let r = Obj::Real(crate::Real {
            exponent: 0,
            mantissa: 0x1000,
            sign: 0,
        });
        let extable = Extable {
            name_to_addr: HashMap::new(),
            addr_to_name: HashMap::new(),
        };
        assert_eq!(r.decompile(&extable), "1");
    }

    #[test]
    fn test_decompile_complex() {
        let r = Obj::Complex(crate::Complex {
            real: crate::Real {
                exponent: 0,
                mantissa: 0x1000,
                sign: 0,
            },
            imag: crate::Real {
                exponent: 0,
                mantissa: 0x1000,
                sign: 0,
            },
        });
        let extable = Extable {
            name_to_addr: HashMap::new(),
            addr_to_name: HashMap::new(),
        };
        assert_eq!(r.decompile(&extable), "1 + 1i");
    }

    #[test]
    fn test_decompile_ext() {
        let r = Obj::Ext(0x1234);
        let mut name_to_addr = HashMap::new();
        name_to_addr.insert("test".to_string(), 0x1234);
        let mut addr_to_name = HashMap::new();
        addr_to_name.insert(0x1234, "test".to_string());
        let extable = Extable {
            name_to_addr,
            addr_to_name,
        };
        assert_eq!(r.decompile(&extable), "test");
        let r = Obj::Ext(0x1235);
        assert_eq!(r.decompile(&extable), "Ext1235");
    }

    #[test]
    fn test_decompile_semi() {
        let r = Obj::Semi();
        let extable = Extable {
            name_to_addr: HashMap::new(),
            addr_to_name: HashMap::new(),
        };
        assert_eq!(r.decompile(&extable), ";");
    }
    #[test]
    fn test_decompile_prg() {
        let r = Obj::Prg(vec![
            Obj::Real(crate::Real {
                exponent: 0,
                mantissa: 0x1000,
                sign: 0,
            }),
            Obj::Semi(),
        ]);
        let extable = Extable {
            name_to_addr: HashMap::new(),
            addr_to_name: HashMap::new(),
        };
        assert_eq!(r.decompile(&extable), "// Program:\n1\n;\n");
    }
}
