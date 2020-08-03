use crate::ValidationErrors;

/// An item that can be validated.
pub trait Validatable<Key> {
    /// Validate this item. Returns `Ok(())` if no errors were
    /// encountered, and returns `Err(ValidationErrors)` if any errors
    /// were encountered.
    fn validate(&self) -> Result<(), ValidationErrors<Key>>;
    /// Validate this item. Returns an empty
    /// [ValidationErrors](ValidationErrors) if no errors were
    /// encountered during validation.
    fn validate_or_empty(&self) -> ValidationErrors<Key> {
        match self.validate() {
            Ok(()) => ValidationErrors::default(),
            Err(errors) => errors,
        }
    }
}