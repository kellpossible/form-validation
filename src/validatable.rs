use crate::ValidationErrors;

#[cfg(feature = "async")]
use futures::Future;

#[cfg(feature = "async")]
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

/// An item that can be validated asynchronously.
///
/// See [Validatable] for the synchronous version.
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub trait AsyncValidatable<Key>
where
    Key: 'static,
{
    /// Creates a future that will validate this item. The future
    /// returns `Ok(())` if no errors were encountered, and returns
    /// `Err(ValidationErrors)` if any errors were encountered.
    fn validate_future(&self) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>>;
    /// Creates a future that will validate this item. The future
    /// returns an empty [ValidationErrors](ValidationErrors) if no
    /// errors were encountered during validation.
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
