use crate::ValidationErrors;
use futures::Future;
use std::pin::Pin;

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

pub trait AsyncValidatable<Key>
where
    Key: 'static,
{
    fn validate_future(&self) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>>;
    fn validate_future_or_empty(&self) -> Pin<Box<dyn Future<Output = ValidationErrors<Key>>>> {
        let future = self.validate_future();
        Box::pin(async move {
            let result: Result<(), ValidationErrors<Key>> = future.await;
            match result {
                Ok(()) => ValidationErrors::default(),
                Err(errors) => errors,
            }
        })
    }
}
