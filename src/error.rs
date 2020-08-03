use std::{rc::Rc, fmt::{Display, Debug}};

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
    pub fn with_message<F: Fn(&Key) -> String + 'static>(mut self, message_fn: F) -> Self {
        self.message = Rc::new(message_fn);
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