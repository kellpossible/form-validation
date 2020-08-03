//! This is a library for validating data entry forms in a user
//! interface.
//!
//! Typically to use this library, you would implement
//! [Validatable](Validatable) for your form, and in the
//! implementation use a [Validator](Validator) for each field in the
//! form, and concatinating the results with
//! [concat_results()](concat_results()).
//!
//! ## Optional Features
//!
//! + `"stdweb-support"` - enable support for
//!   [stdweb](https://crates.io/crates/stdweb) on the
//!   `wasm32-unknown-unknown` platform.
//! + `"wasm-bindgen-support"` - enable for
//!   [wasm-bindgen](https://crates.io/crates/wasm-bindgen) on the
//!   `wasm32-unknown-unknown` platform.

mod error;
mod validator;
mod validation;
mod validatable;
mod concat_results;

pub use error::*;
pub use validator::*;
pub use validation::*;
pub use validatable::*;
pub use concat_results::concat_results;