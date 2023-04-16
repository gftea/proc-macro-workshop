#[macro_export]
macro_rules! require_multiple_of_eight {
    ($e: expr) => {
        const _: MultipleOf8Bits<[(); $e % 8]> = ();
    };
}

pub type MultipleOf8Bits<T> = <<T as ArrayOfMod>::ModType as TotalSizeIsMultipleOfEightBits>::Check;

pub enum ZeroMod8 {}
pub enum OneMod8 {}
pub enum TwoMod8 {}
pub enum ThreeMod8 {}
pub enum FourMod8 {}
pub enum FiveMod8 {}
pub enum SixMod8 {}
pub enum SevenMod8 {}
// Common trait for all the above enums
// It makes all enums generic over the trait, but emit different asscociated type.
pub trait ArrayOfMod {
    type ModType;
}
impl ArrayOfMod for [(); 0] {
    type ModType = ZeroMod8;
}
impl ArrayOfMod for [(); 1] {
    type ModType = OneMod8;
}
impl ArrayOfMod for [(); 2] {
    type ModType = TwoMod8;
}
impl ArrayOfMod for [(); 3] {
    type ModType = ThreeMod8;
}
impl ArrayOfMod for [(); 4] {
    type ModType = FourMod8;
}
impl ArrayOfMod for [(); 5] {
    type ModType = FiveMod8;
}
impl ArrayOfMod for [(); 6] {
    type ModType = SixMod8;
}
impl ArrayOfMod for [(); 7] {
    type ModType = SevenMod8;
}

// Speicial trait for type representing multiple of 8 bits
pub trait TotalSizeIsMultipleOfEightBits {
    type Check;
}
impl TotalSizeIsMultipleOfEightBits for ZeroMod8 {
    type Check = ();
}

#[macro_export]
macro_rules! require_discriminant_in_range {
    ($discriminant: expr, $max: expr) => {
        const _: DiscriminantInRangeCheck<[(); ($discriminant < $max) as usize]> = ();
    };
}

pub type DiscriminantInRangeCheck<T> =
    <<T as EnumDiscriminant>::Marker as DiscriminantInRange>::Check;

pub trait EnumDiscriminant {
    type Marker;
}
pub trait DiscriminantInRange {
    type Check;
}
pub enum True {}
pub enum False {}

impl EnumDiscriminant for [(); 1] {
    type Marker = True;
}
impl EnumDiscriminant for [(); 0] {
    type Marker = False;
}
impl DiscriminantInRange for True {
    type Check = ();
}
