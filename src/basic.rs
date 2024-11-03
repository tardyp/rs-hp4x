use winnow::PResult;

use crate::{decompile::Decompiled, nibbles::*, Extable};

// Data structures
#[derive(Debug)]
pub struct Real {
    pub exponent: u16,
    pub mantissa: u64,
    pub sign: u8,
}
impl Decompiled for Real {
    fn decompile(&self, _extable: &Extable) -> String {
        format!(
            "{}", self.as_float()
        )
    }
}
impl Real {
    /// fixme: this is probably not the correct way to convert the hp48 real to a float
    pub fn as_float(&self) -> f64 {
        let mut f = self.mantissa as f64;
        f /= (1 << 12) as f64;
        f *= 2f64.powi(self.exponent as i32);
        if self.sign == 1 {
            f = -f;
        }
        f
    }
}
// Parsers for various object types
pub(crate) fn next_real(input: &mut Nibbles) -> PResult<Real> {
    let exp = integer3(input)?;
    let mantissa = integer12(input)?;
    let sign = integer1(input)?;
    Ok(
        Real {
            exponent: exp as u16,
            mantissa,
            sign: sign as u8,
        },
    )
}

#[derive(Debug)]
pub struct Complex {
    pub real: Real,
    pub imag: Real,
}
impl Decompiled for Complex {
    fn decompile(&self, _extable: &Extable) -> String {
        format!(
            "{} + {}i", self.real.as_float(), self.imag.as_float()
        )
    }
}
// *** Enhanced Object Parsers ***
pub(crate) fn next_complex(input: &mut Nibbles) -> PResult<Complex> {
    let real = next_real(input)?;
    let imag = next_real(input)?;
    Ok(Complex { real, imag })
}