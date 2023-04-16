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
pub use checks::MultipleOf8Bits;

pub trait Specifier {
    const BITS: usize;
}

bitspec!(B1, 1);
bitspec!(B2, 2);
bitspec!(B3, 3);
bitspec!(B4, 4);
bitspec!(B5, 5);
bitspec!(B6, 6);
bitspec!(B7, 7);
bitspec!(B8, 8);
bitspec!(B9, 9);
bitspec!(B10, 10);
bitspec!(B11, 11);
bitspec!(B12, 12);
bitspec!(B13, 13);
bitspec!(B14, 14);
bitspec!(B15, 15);
bitspec!(B16, 16);
bitspec!(B17, 17);
bitspec!(B18, 18);
bitspec!(B19, 19);
bitspec!(B20, 20);
bitspec!(B21, 21);
bitspec!(B22, 22);
bitspec!(B23, 23);
bitspec!(B24, 24);
bitspec!(B25, 25);
bitspec!(B26, 26);
bitspec!(B27, 27);
bitspec!(B28, 28);
bitspec!(B29, 29);
bitspec!(B30, 30);
bitspec!(B31, 31);
bitspec!(B32, 32);
bitspec!(B33, 33);
bitspec!(B34, 34);
bitspec!(B35, 35);
bitspec!(B36, 36);
bitspec!(B37, 37);
bitspec!(B38, 38);
bitspec!(B39, 39);
bitspec!(B40, 40);
bitspec!(B41, 41);
bitspec!(B42, 42);
bitspec!(B43, 43);
bitspec!(B44, 44);
bitspec!(B45, 45);
bitspec!(B46, 46);
bitspec!(B47, 47);
bitspec!(B48, 48);
bitspec!(B49, 49);
bitspec!(B50, 50);
bitspec!(B51, 51);
bitspec!(B52, 52);
bitspec!(B53, 53);
bitspec!(B54, 54);
bitspec!(B55, 55);
bitspec!(B56, 56);
bitspec!(B57, 57);
bitspec!(B58, 58);
bitspec!(B59, 59);
bitspec!(B60, 60);
bitspec!(B61, 61);
bitspec!(B62, 62);
bitspec!(B63, 63);
bitspec!(B64, 64);
