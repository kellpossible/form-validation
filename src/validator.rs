use crate::{ValidationError, Validation, ValidationErrors};
use uuid::Uuid;
use std::{fmt::Debug, rc::Rc};

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
    function: Rc<ValidatorFnTraitObject<Value, Key>>,
    id: Uuid,
}

impl<Value, Key> ValidatorFn<Value, Key> {
    /// Create a new `ValidatorFn`.
    pub fn new<C>(closure: C) -> Self
    where
        C: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static,
    {
        Self {
            function: Rc::new(closure),
            id: Uuid::new_v4(),
        }
    }
}

impl<Value, Key> PartialEq for ValidatorFn<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Value, Key> Validation<Value, Key> for ValidatorFn<Value, Key>
where
    Key: Clone + PartialEq,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        (self.function)(value, key).map_err(|err| ValidationErrors::new(vec![err]))
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
    pub fn validation<C: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static>(
        mut self,
        closure: C,
    ) -> Self {
        self.validations.push(Rc::new(ValidatorFn::new(closure)));
        self
    }

    /// A factory method to add a [ValidatorFn] to this validator.
    pub fn validator_fn(mut self, validator_fn: ValidatorFn<Value, Key>) -> Self {
        self.validations.push(Rc::new(validator_fn));
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