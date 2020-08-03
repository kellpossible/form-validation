
use crate::{ValidatorFn, ValidationErrors};

/// A function/struct/item that can perform validation on an item with
/// a given `Value` type.
pub trait Validation<Value, Key> {
    /// Validate a given form field referenced by a given `Key`, that
    /// contains a given `Value`, returns
    /// [ValidationErrors](ValidationErrors) if there are any.
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>>;
}

