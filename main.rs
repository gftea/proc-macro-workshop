// One downside of the way we have implemented enum support so far is that it
// makes it impossible to see from the definition of a bitfield struct how the
// bits are being laid out. In something like the following bitfield, all we
// know from this code is that the total size is a multiple of 8 bits. Maybe
// trigger_mode is 11 bits and delivery_mode is 1 bit. This may tend to make
// maintenance problematic when the types involved are defined across different
// modules or even different crates.
//
//     #[bitfield]
//     pub struct RedirectionTableEntry {
//         trigger_mode: TriggerMode,
//         delivery_mode: DeliveryMode,
//         reserved: B4,
//     }
//
// Introduce an optional #[bits = N] attribute to serve as compile-time checked
// documentation of field size. Ensure that this attribute is entirely optional,
// meaning that the code behaves the same whether or not you write it, but if
// the user does provide the attribute then the program must not compile if
// their value is wrong.

fn main() {
    let exp: [(); 9] = [(); 1];
}
