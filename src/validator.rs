use crate::{Validation, ValidationError, ValidationErrors};
use std::{cell::RefCell, fmt::Debug, future::Future, marker::PhantomData, pin::Pin, rc::Rc};
use uuid::Uuid;

// TODO: make this optional
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};

type ValidatorFnTraitObject<Value, Key> = dyn Fn(&Value, &Key) -> Result<(), ValidationErrors<Key>>;

/// Function to perform validation on a form field.
///
/// ## Example
///
/// ```
/// use form_validation::{Validation, ValidationError, ValidatorFn};
///
/// let v: ValidatorFn<i32, String> = ValidatorFn::new(|value, key: &String| {
///     if value < &0 {
///         let value_clone = *value;
///         Err(ValidationError::new(key.clone()).with_message(move |key| {
///             format!(
///                 "The value of {} ({}) cannot be less than 0",
///                 key, value_clone
///             )
///         }).into())
///     } else {
///         Ok(())
///     }
/// });
///
/// let key = "field1".to_string();
/// assert!(v.validate_value(&20, &key).is_ok());
/// assert!(v.validate_value(&-1, &key).is_err());
/// assert_eq!(
///     "The value of field1 (-1) cannot be less than 0",
///     v.validate_value(&-1, &key).unwrap_err().to_string()
/// );
/// ```
pub struct ValidatorFn<Value, Key> {
    closure: Rc<ValidatorFnTraitObject<Value, Key>>,
    id: Uuid,
}

impl<Value, Key> ValidatorFn<Value, Key> {
    /// Create a new `ValidatorFn`.
    pub fn new<C>(closure: C) -> Self
    where
        C: Fn(&Value, &Key) -> Result<(), ValidationErrors<Key>> + 'static,
    {
        Self {
            closure: Rc::new(closure),
            id: Uuid::new_v4(),
        }
    }
}

impl<Value, Key> Clone for ValidatorFn<Value, Key> {
    fn clone(&self) -> Self {
        Self {
            closure: Rc::clone(&self.closure),
            id: self.id.clone(),
        }
    }
}

impl<Value, Key> PartialEq for ValidatorFn<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<C, Value, Key> From<C> for ValidatorFn<Value, Key>
where
    C: Fn(&Value, &Key) -> Result<(), ValidationErrors<Key>> + 'static,
{
    fn from(closure: C) -> Self {
        ValidatorFn::new(closure)
    }
}

impl<Value, Key> Debug for ValidatorFn<Value, Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidatorFn(closure: {:p}, id: {})",
            self.closure, self.id
        )
    }
}

/// An function to perform validation on a field asynchonously.
///
/// For the synchronous version, see [ValidationFn].
///
/// ## Example
///
/// ```
/// use form_validation::{AsyncValidatorFn, ValidationError};
/// use futures::executor::block_on;
///
/// let v: AsyncValidatorFn<i32, String> =
///     AsyncValidatorFn::new(|value: &i32, key: &String| {
///         let key = key.clone();
///         let value = *value;
///         Box::pin(async move {
///             // perform actions here that require async
///             if value < 0 {
///                 Err(ValidationError::new(key.clone())
///                     .with_message(move |key| {
///                         format!(
///                             "The value of {} ({}) cannot be less than 0",
///                             key, value
///                         )
///                     })
///                     .into())
///             } else {
///                 Ok(())
///             }
///         })
///     });
///
/// let key = "field1".to_string();
/// assert!(block_on(v.validate_value(&20, &key)).is_ok());
/// assert!(block_on(v.validate_value(&-1, &key)).is_err());
/// assert_eq!(
///     "The value of field1 (-1) cannot be less than 0",
///     block_on(v.validate_value(&-1, &key))
///         .unwrap_err()
///         .to_string()
/// );
/// ```
pub struct AsyncValidatorFn<Value, Key> {
    future_producer: Rc<
        dyn Fn(&Value, &Key) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>>,
    >,
    id: Uuid,
    key_type: PhantomData<Key>,
    value_type: PhantomData<Value>,
}

impl<Value, Key> AsyncValidatorFn<Value, Key>
where
    Key: Clone + PartialEq,
    Value: Clone + PartialEq,
{
    /// Takes a closure that produces a `Future` that produces a [ValidatorFn] closure.
    pub fn new<C>(closure: C) -> Self
    where
        C: Fn(&Value, &Key) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>>
            + 'static,
    {
        Self {
            future_producer: Rc::new(closure),
            id: Uuid::new_v4(),
            key_type: PhantomData,
            value_type: PhantomData,
        }
    }

    /// Runs the future to produce the [ValidatorFn] closure, and then
    /// performs the validation with that.
    pub async fn validate_value(
        &self,
        value: &Value,
        key: &Key,
    ) -> Result<(), ValidationErrors<Key>> {
        let future = (self.future_producer)(value, key);
        future.await
    }
}

impl<Value, Key> From<ValidatorFn<Value, Key>> for AsyncValidatorFn<Value, Key>
where
    Key: Clone + PartialEq + 'static,
    Value: Clone + PartialEq + 'static,
{
    fn from(validator_fn: ValidatorFn<Value, Key>) -> Self {
        Self::new(move |value, key| {
            let value_clone = value.clone();
            let key_clone = key.clone();
            let new_fn = validator_fn.clone();
            Box::pin(async move { new_fn.validate_value(&value_clone, &key_clone) })
        })
    }
}

impl<Value, Key> PartialEq for AsyncValidatorFn<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Value, Key> Clone for AsyncValidatorFn<Value, Key> {
    fn clone(&self) -> Self {
        Self {
            future_producer: Rc::clone(&self.future_producer),
            id: self.id,
            key_type: PhantomData,
            value_type: PhantomData,
        }
    }
}

impl<Value, Key> Debug for AsyncValidatorFn<Value, Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AsyncValidatorFn(future_producer: {:p}, id: {})",
            self.future_producer, self.id
        )
    }
}

impl<Value, Key> Validation<Value, Key> for ValidatorFn<Value, Key>
where
    Key: Clone + PartialEq,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        (self.closure)(value, key)
    }
}

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
///         }).into())
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
///                     .into())
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
///                 .into())
///         } else {
///             Ok(())
///         }
///     }));
/// let key = "field1".to_string();
/// assert!(block_on(v.validate_value(&11, &key)).is_err());
/// assert!(block_on(v.validate_value(&5, &key)).is_ok());
/// assert!(block_on(v.validate_value(&-1, &key)).is_err());
/// ```
#[derive(Clone, PartialEq, Debug)]
pub struct AsyncValidator<Value, Key> {
    pub validations: Vec<AsyncValidatorFn<Value, Key>>,
}

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

impl<Value, Key> Default for AsyncValidator<Value, Key>
where
    Key: Clone + PartialEq,
    Value: Clone + PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

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
    use super::{ValidationError, Validator, AsyncValidator};
    use futures::executor::block_on;

    #[test]
    fn async_validator_from_validator() {
        let v: Validator<i32, String> = Validator::new()
        .validation(|value: &i32, key: &String| {
            if value < &0 {
                let value_clone = *value;
                Err(ValidationError::new(key.clone()).with_message(move |key| {
                    format!(
                        "The value of {} ({}) cannot be less than 0",
                        key, value_clone
                    )
                }).into())
            } else {
                Ok(())
            }
        })
        .validation(|value: &i32, key: &String| {
            if value > &10 {
                let value_clone = *value;
                Err(ValidationError::new(key.clone()).with_message(move |key| {
                    format!(
                        "The value of {} ({}) cannot be greater than 10",
                        key, value_clone
                    )
                }).into())
            } else {
                Ok(())
            }
        });

        let av: AsyncValidator<i32, String> = v.into();
        let key = "field1".to_string();
        assert!(block_on(av.validate_value(&11, &key)).is_err());
        assert!(block_on(av.validate_value(&5, &key)).is_ok());
        assert!(block_on(av.validate_value(&-1, &key)).is_err());
    }
}