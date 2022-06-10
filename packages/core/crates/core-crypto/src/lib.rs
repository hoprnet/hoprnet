mod utils;

pub mod commitment;
pub mod parameters;
pub mod keys;
pub mod primitive;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[macro_use]
extern crate static_assertions;

// Static assertions on cryptographic parameters


const_assert!(parameters::SECRET_KEY_LENGTH >= 32);
//const_assert_eq!(constants::SECRET_KEY_LENGTH, keys::KeyBytes::)

