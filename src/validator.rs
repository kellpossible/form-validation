use crate::{Validation, ValidationError, ValidationErrors};
use std::{cell::RefCell, fmt::Debug, future::Future, marker::PhantomData, pin::Pin, rc::Rc};
use uuid::Uuid;

// TODO: make this optional
use futures::{
    future::join_all,
    stream::{self, StreamExt},
};

type ValidatorFnTraitObject<Value, Key> = dyn Fn(&Value, &Key) -> Result<(), ValidationError<Key>>;

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
///         }))
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
        C: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static,
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
    C: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static,
{
    fn from(closure: C) -> Self {
        ValidatorFn::new(closure)
    }
}

pub struct AsyncValidatorFn<Value, Key> {
    future_producer: Rc<dyn Fn() -> Pin<Box<dyn Future<Output = ValidatorFn<Value, Key>>>>>,
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
        C: Fn() -> Pin<Box<dyn Future<Output = ValidatorFn<Value, Key>>>> + 'static,
    {
        Self {
            future_producer: Rc::new(closure),
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
        let future = (self.future_producer)();
        let validator_fn: ValidatorFn<Value, Key> = future.await;
        validator_fn.validate_value(value, key)
    }
}

impl<Value, Key> From<ValidatorFn<Value, Key>> for AsyncValidatorFn<Value, Key>
where
    Key: Clone + PartialEq + 'static,
    Value: Clone + PartialEq + 'static,
{
    fn from(validator_fn: ValidatorFn<Value, Key>) -> Self {
        Self::new(move || {
            let new_fn = validator_fn.clone();
            Box::pin(async { new_fn })
        })
    }
}

impl<Value, Key> Validation<Value, Key> for ValidatorFn<Value, Key>
where
    Key: Clone + PartialEq,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        (self.closure)(value, key).map_err(|err| ValidationErrors::new(vec![err]))
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
/// .validation(|value, key: &String| {
///     if value < &0 {
///         let value_clone = *value;
///         Err(ValidationError::new(key.clone()).with_message(move |key| {
///             format!(
///                 "The value of {} ({}) cannot be less than 0",
///                 key, value_clone
///             )
///         }))
///     } else {
///         Ok(())
///     }
/// })
/// .validation(|value, key: &String| {
///     if value > &10 {
///         let value_clone = *value;
///         Err(ValidationError::new(key.clone()).with_message(move |key| {
///             format!(
///                 "The value of {} ({}) cannot be greater than 10",
///                 key, value_clone
///             )
///         }))
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
#[derive(Clone)]
pub struct Validator<Value, Key> {
    pub validations: Vec<Rc<ValidatorFn<Value, Key>>>,
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

impl<Value, Key> Debug for Validator<Value, Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let validation_addresses: Vec<String> = self
            .validations
            .iter()
            .map(|validation| format!("ValidatorFn: {:p}", *validation))
            .collect();

        write!(f, "Validator{{{0}}}", validation_addresses.join(", "))
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
        self.validations.push(Rc::new(validator_fn.into()));
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
