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
//! + `"async"` - enable an asynchronous version of this API, and
//!   conversion traits from synchronous to asynchronous.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod concat_results;
mod error;
mod validatable;
mod validation;
mod validator;
mod validator_fn;

pub use concat_results::concat_results;
pub use error::*;
pub use validatable::*;
pub use validation::*;
pub use validator::*;
pub use validator_fn::*;
