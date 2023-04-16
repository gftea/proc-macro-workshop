// Crates that have the "proc-macro" crate type are only allowed to export
// procedural macros. So we cannot have one crate that defines procedural macros
// alongside other types of public APIs like traits and structs.
//
// For this project we are going to need a #[bitfield] macro but also a trait
// and some structs. We solve this by defining the trait and structs in this
// crate, defining the attribute macro in a separate bitfield-impl crate, and
// then re-exporting the macro from this crate so that users only have one crate
// that they need to import.
//
// From the perspective of a user of this crate, they get all the necessary APIs
// (macro, trait, struct) through the one bitfield crate.

pub use bitfield_impl::*;

// TODO other things
#[macro_use]
mod checks;
// only export the MultipleOf8Bits type alias so that it can be referenced in user code
// the require_multiple_of_eight! macro is inserted into the generated code, which emit the codes referencing the type alias
pub use checks::DiscriminantInRangeCheck;
pub use checks::MultipleOf8Bits;
pub trait Specifier {
    const BITS: usize;
    type InnerType;

    fn to_u64(value: Self::InnerType) -> u64;
    fn from_u64(value: u64) -> Self::InnerType;
}

bitspec!(B1, 1, u8);
bitspec!(B2, 2, u8);
bitspec!(B3, 3, u8);
bitspec!(B4, 4, u8);
bitspec!(B5, 5, u8);
bitspec!(B6, 6, u8);
bitspec!(B7, 7, u8);
bitspec!(B8, 8, u8);
bitspec!(B9, 9, u16);
bitspec!(B10, 10, u16);
bitspec!(B11, 11, u16);
bitspec!(B12, 12, u16);
bitspec!(B13, 13, u16);
bitspec!(B14, 14, u16);
bitspec!(B15, 15, u16);
bitspec!(B16, 16, u16);
bitspec!(B17, 17, u32);
bitspec!(B18, 18, u32);
bitspec!(B19, 19, u32);
bitspec!(B20, 20, u32);
bitspec!(B21, 21, u32);
bitspec!(B22, 22, u32);
bitspec!(B23, 23, u32);
bitspec!(B24, 24, u32);
bitspec!(B25, 25, u32);
bitspec!(B26, 26, u32);
bitspec!(B27, 27, u32);
bitspec!(B28, 28, u32);
bitspec!(B29, 29, u32);
bitspec!(B30, 30, u32);
bitspec!(B31, 31, u32);
bitspec!(B32, 32, u32);
bitspec!(B33, 33, u64);
bitspec!(B34, 34, u64);
bitspec!(B35, 35, u64);
bitspec!(B36, 36, u64);
bitspec!(B37, 37, u64);
bitspec!(B38, 38, u64);
bitspec!(B39, 39, u64);
bitspec!(B40, 40, u64);
bitspec!(B41, 41, u64);
bitspec!(B42, 42, u64);
bitspec!(B43, 43, u64);
bitspec!(B44, 44, u64);
bitspec!(B45, 45, u64);
bitspec!(B46, 46, u64);
bitspec!(B47, 47, u64);
bitspec!(B48, 48, u64);
bitspec!(B49, 49, u64);
bitspec!(B50, 50, u64);
bitspec!(B51, 51, u64);
bitspec!(B52, 52, u64);
bitspec!(B53, 53, u64);
bitspec!(B54, 54, u64);
bitspec!(B55, 55, u64);
bitspec!(B56, 56, u64);
bitspec!(B57, 57, u64);
bitspec!(B58, 58, u64);
bitspec!(B59, 59, u64);
bitspec!(B60, 60, u64);
bitspec!(B61, 61, u64);
bitspec!(B62, 62, u64);
bitspec!(B63, 63, u64);
bitspec!(B64, 64, u64);

impl Specifier for bool {
    const BITS: usize = 1;
    type InnerType = bool;

    fn to_u64(value: Self::InnerType) -> u64 {
        match value {
            false => 0,
            true => 1,
        }
    }

    fn from_u64(value: u64) -> Self::InnerType {
        match value {
            0 => false,
            1 => true,
            _ => panic!("bool can only be 0 or 1"),
        }
    }
}
