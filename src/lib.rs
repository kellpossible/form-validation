//! This is a library for validating data entry forms in a user
//! interface.
//!
//! Typically to use this library, you would implement
//! [Validatable](Validatable) for your form, and in the
//! implementation use a [Validator](Validator) for each field in the
//! form, and concatinating the results with
//! [concat_results()](concat_results()).

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

/// An error associated with a form field.
pub struct ValidationError<Key> {
    /// The key for the field that this validation error is associated with.
    pub key: Key,
    /// Function that produces the error message.
    message: Rc<dyn Fn(&Key) -> String>,
}

impl<Key> Clone for ValidationError<Key>
where
    Key: Clone,
{
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            message: self.message.clone(),
        }
    }
}

impl<Key> ValidationError<Key> {
    /// Create a new `ValidationError` with a generic message.
    pub fn new(key: Key) -> Self {
        Self {
            key,
            message: Rc::new(|_| "Validation error".to_string()),
        }
    }

    /// Factory method to set the message for this error.
    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        let message_string = message.into();
        self.message = Rc::new(move |_| message_string.clone());
        self
    }

    /// Factory method to set the message for this error from a
    /// function that returns a `String`.
    ///
    /// ## Example
    /// ```
    /// use form_validation::ValidationError;
    ///
    /// let value = -10;
    /// let error = ValidationError::new("field1").with_message(move |key| {
    ///     format!(
    ///         "The value of {} ({}) cannot be less than 0",
    ///          key, value)
    /// });
    ///
    /// assert_eq!("The value of field1 (-10) cannot be less than 0", error.to_string());
    /// ```
    pub fn with_message<F: Fn(&Key) -> String + 'static>(mut self, message: F) -> Self {
        self.message = Rc::new(message);
        self
    }

    /// Get the message for this error.
    fn get_message(&self) -> String {
        (self.message)(&self.key)
    }
}

impl<Key> Display for ValidationError<Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl<Key> Debug for ValidationError<Key>
where
    Key: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidationError{{ key: {0:?}, message: {1} }}",
            self.key,
            self.get_message()
        )
    }
}

impl<Key> std::error::Error for ValidationError<Key> where Key: Debug {}

/// A collection of [ValidationError](ValidationError)s as a result of
/// validating the fields of a form.
#[derive(Debug, Clone)]
pub struct ValidationErrors<Key> {
    pub errors: Vec<ValidationError<Key>>,
}

impl<Key> ValidationErrors<Key>
where
    Key: PartialEq + Clone,
{
    /// Create a new `ValidationErrors`.
    pub fn new(errors: Vec<ValidationError<Key>>) -> Self {
        Self { errors }
    }

    /// Get errors associated with the specified field key, or `None`
    /// if there are no errors for that field.
    pub fn get(&self, key: &Key) -> Option<ValidationErrors<Key>> {
        let errors: Vec<ValidationError<Key>> = self
            .errors
            .iter()
            .filter(|error| &error.key == key)
            .map(|error| (*error).clone())
            .collect();

        if !errors.is_empty() {
            Some(ValidationErrors::new(errors))
        } else {
            None
        }
    }

    /// Returns true if there are no errors in this collection.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Extend this collection of errors with the contents of another
    /// collection.
    pub fn extend(&mut self, errors: ValidationErrors<Key>) {
        self.errors.extend(errors.errors)
    }

    /// The number of errors in this collection.
    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl<Key> Default for ValidationErrors<Key> {
    fn default() -> Self {
        Self { errors: Vec::new() }
    }
}

impl<Key> Display for ValidationErrors<Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let errors: Vec<String> = self.errors.iter().map(|e| format!("{}", e)).collect();
        write!(f, "{}", errors.join(", "))
    }
}

impl<Key> std::error::Error for ValidationErrors<Key> where Key: std::fmt::Debug {}

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
    id: uuid::Uuid,
}

impl<Value, Key> ValidatorFn<Value, Key> {
    /// Create a new `ValidatorFn`.
    pub fn new<F>(function: F) -> Self
    where
        F: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static,
    {
        Self {
            function: Rc::new(function),
            id: uuid::Uuid::new_v4(),
        }
    }
}

impl<Value, Key> PartialEq for ValidatorFn<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

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

/// A function/struct/item that can perform validation on an item with
/// a given `Value` type.
pub trait Validation<Value, Key> {
    /// Validate a given form field referenced by a given `Key`, that
    /// contains a given `Value`, returns
    /// [ValidationErrors](ValidationErrors) if there are any.
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>>;
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
    pub fn validation<F: Fn(&Value, &Key) -> Result<(), ValidationError<Key>> + 'static>(
        mut self,
        function: F,
    ) -> Self {
        self.validations.push(Rc::new(ValidatorFn::new(function)));
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

/// Join validation results, concatinating any errors they may
/// contain. If any of the results are an `Err` it will return an
/// `Err` containing all the errors from all th results.
///
/// ## Example
/// ```
/// use form_validation::{concat_results, ValidationError, ValidationErrors};
/// let results = vec![
///     Ok(()),
///     Err(ValidationErrors::new(vec![ValidationError::new("field1")])),
///     Err(ValidationErrors::new(vec![ValidationError::new("field1")])),
///     Err(ValidationErrors::new(vec![ValidationError::new("field2")])),
/// ];
///
/// let result = concat_results(results);
///
/// let errors = result.unwrap_err();
///
/// assert_eq!(3, errors.len());
///
/// let field1_errors = errors.get(&"field1").unwrap();
/// assert_eq!(2, field1_errors.len());
///
/// let field2_errors = errors.get(&"field2").unwrap();
/// assert_eq!(1, field2_errors.len());
/// ```
pub fn concat_results<Key>(
    results: Vec<Result<(), ValidationErrors<Key>>>,
) -> Result<(), ValidationErrors<Key>>
where
    Key: PartialEq + Clone,
{
    let mut all_errors: ValidationErrors<Key> = ValidationErrors::default();

    for result in results {
        if let Err(errors) = result {
            all_errors.extend(errors);
        }
    }

    if !all_errors.is_empty() {
        Err(all_errors)
    } else {
        Ok(())
    }
}
