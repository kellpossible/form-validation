use crate::{AsyncValidatorFn, Validation, ValidationErrors, ValidatorFn};
use std::fmt::Debug;

#[cfg(feature = "async")]
use futures::future::join_all;

/// Validates a particular type of value, can contain many validation
/// functions. Generally used with a single key for all contained
/// validation functions.
///
/// ## Example
/// ```
/// use form_validation::{Validation, ValidationError, Validator};
///
/// let v: Validator<i32, String> = Validator::new()
/// .validation(|value: &i32, key: &String| {
///     if value < &0 {
///         let value_clone = *value;
///         Err(ValidationError::new(key.clone()).with_message(move |key| {
///             format!(
///                 "The value of {} ({}) cannot be less than 0",
///                 key, value_clone
///             )
///         }).into()) // convert into ValidationErrors
///     } else {
///         Ok(())
///     }
/// })
/// .validation(|value: &i32, key: &String| {
///     if value > &10 {
///         let value_clone = *value;
///         Err(ValidationError::new(key.clone()).with_message(move |key| {
///             format!(
///                 "The value of {} ({}) cannot be greater than 10",
///                 key, value_clone
///             )
///         }).into())
///     } else {
///         Ok(())
///     }
/// });
///
/// let key = "field1".to_string();
/// assert!(v.validate_value(&11, &key).is_err());
/// assert!(v.validate_value(&5, &key).is_ok());
/// assert!(v.validate_value(&-1, &key).is_err());
/// ```
#[derive(Clone, Debug)]
pub struct Validator<Value, Key> {
    pub validations: Vec<ValidatorFn<Value, Key>>,
}

impl<Value, Key> PartialEq for Validator<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        if self.validations.len() == other.validations.len() {
            let mut all_validations_same = true;

            for (i, this_validation) in self.validations.iter().enumerate() {
                let other_validation = other.validations.get(i).unwrap();

                all_validations_same &= this_validation == other_validation;
            }

            all_validations_same
        } else {
            false
        }
    }
}

impl<Value, Key> Validator<Value, Key> {
    /// Create a new `Validator`.
    pub fn new() -> Self {
        Self {
            validations: Vec::new(),
        }
    }

    /// A factory method to add a validation function to this validator.
    pub fn validation<F: Into<ValidatorFn<Value, Key>> + 'static>(
        mut self,
        validator_fn: F,
    ) -> Self {
        self.validations.push(validator_fn.into());
        self
    }
}

impl<Value, Key> Validation<Value, Key> for Validator<Value, Key>
where
    Key: PartialEq + Clone,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        let mut errors = ValidationErrors::default();

        for validation in &self.validations {
            if let Err(new_errors) = validation.validate_value(value, key) {
                errors.extend(new_errors)
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

impl<Value, Key> Default for Validator<Value, Key> {
    fn default() -> Self {
        Validator::new()
    }
}

/// Validates a particular type of value asynchronously, can contain
/// many validation functions. Generally used with a single key for
/// all contained validation functions.
///
/// See [Validator] for the synchronous version.
///
/// ```
/// use form_validation::{AsyncValidator, ValidationError, AsyncValidatorFn, ValidatorFn};
/// use futures::executor::block_on;
///
/// let v: AsyncValidator<i32, String> = AsyncValidator::new()
///     .validation(AsyncValidatorFn::new(|value: &i32, key: &String| {
///         let value = *value;
///         let key = key.clone();
///         Box::pin(async move {
///             if value < 0 {
///                 Err(ValidationError::new(key.clone())
///                     .with_message(move |key| {
///                         format!("The value of {} ({}) cannot be less than 0", key, value)
///                     })
///                     .into()) // convert into ValidationErrors
///             } else {
///                 Ok(())
///             }
///         })
///     }))
///     // also supports compatibility with the synchronous ValidatorFn
///     .validation(ValidatorFn::new(|value: &i32, key: &String| {
///         if value > &10 {
///             let value_clone = *value;
///             Err(ValidationError::new(key.clone())
///                 .with_message(move |key| {
///                     format!(
///                         "The value of {} ({}) cannot be greater than 10",
///                         key, value_clone
///                     )
///                 })
///                 .into()) // convert into ValidationErrors
///         } else {
///             Ok(())
///         }
///     }));
/// let key = "field1".to_string();
/// assert!(block_on(v.validate_value(&11, &key)).is_err());
/// assert!(block_on(v.validate_value(&5, &key)).is_ok());
/// assert!(block_on(v.validate_value(&-1, &key)).is_err());
/// ```
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[derive(Clone, PartialEq, Debug)]
pub struct AsyncValidator<Value, Key> {
    pub validations: Vec<AsyncValidatorFn<Value, Key>>,
}

#[cfg(feature = "async")]
impl<Value, Key> AsyncValidator<Value, Key>
where
    Key: Clone + PartialEq,
    Value: Clone + PartialEq,
{
    /// Create a new `Validator`.
    pub fn new() -> Self {
        Self {
            validations: Vec::new(),
        }
    }

    /// A factory method to add a validation function to this validator.
    pub fn validation<F: Into<AsyncValidatorFn<Value, Key>> + 'static>(
        mut self,
        async_validator_fn: F,
    ) -> Self {
        self.validations.push(async_validator_fn.into());
        self
    }

    pub async fn validate_value(
        &self,
        value: &Value,
        key: &Key,
    ) -> Result<(), ValidationErrors<Key>> {
        let mut errors = ValidationErrors::default();

        let futures = self
            .validations
            .iter()
            .map(|async_validator_fn| async_validator_fn.validate_value(value, key))
            .collect::<Vec<_>>();

        // Execute all the futures concurrently
        let results: Vec<Result<(), ValidationErrors<Key>>> = join_all(futures).await;

        for result in results {
            if let Err(new_errors) = result {
                errors.extend(new_errors)
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "async")]
impl<Value, Key> Default for AsyncValidator<Value, Key>
where
    Key: Clone + PartialEq,
    Value: Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "async")]
impl<Value, Key> From<Validator<Value, Key>> for AsyncValidator<Value, Key>
where
    Value: Clone + PartialEq + 'static,
    Key: Clone + PartialEq + 'static,
{
    fn from(validator: Validator<Value, Key>) -> Self {
        let mut async_validator: AsyncValidator<Value, Key> = AsyncValidator::new();

        for validator_fn in validator.validations {
            async_validator = async_validator.validation(validator_fn);
        }

        async_validator
    }
}

#[cfg(test)]
mod test {
    #[cfg(feature = "async")]
    mod async_tests {
        use super::super::{AsyncValidator, Validator};
        use crate::ValidationError;
        use futures::executor::block_on;

        /// Unit test for the `From<Validator> for AsyncValidator` implmentation
        #[test]
        fn async_validator_from_validator() {
            let v: Validator<i32, String> = Validator::new()
                .validation(|value: &i32, key: &String| {
                    if value < &0 {
                        let value_clone = *value;
                        Err(ValidationError::new(key.clone())
                            .with_message(move |key| {
                                format!(
                                    "The value of {} ({}) cannot be less than 0",
                                    key, value_clone
                                )
                            })
                            .into())
                    } else {
                        Ok(())
                    }
                })
                .validation(|value: &i32, key: &String| {
                    if value > &10 {
                        let value_clone = *value;
                        Err(ValidationError::new(key.clone())
                            .with_message(move |key| {
                                format!(
                                    "The value of {} ({}) cannot be greater than 10",
                                    key, value_clone
                                )
                            })
                            .into())
                    } else {
                        Ok(())
                    }
                });

            // perform the conversion
            let av: AsyncValidator<i32, String> = v.into();

            let key = "field1".to_string();
            assert!(block_on(av.validate_value(&11, &key)).is_err());
            assert!(block_on(av.validate_value(&5, &key)).is_ok());
            assert!(block_on(av.validate_value(&-1, &key)).is_err());
        }
    }
}
