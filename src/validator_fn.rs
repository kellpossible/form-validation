use crate::{Validation, ValidationErrors};
use std::{fmt::Debug, rc::Rc};
use uuid::Uuid;

#[cfg(feature = "async")]
use std::{future::Future, marker::PhantomData, pin::Pin};

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
///         Err(ValidationError::new(key.clone(), "NOT_LESS_THAN_0")
///                 .with_message(move |key| {
///                     format!(
///                         "The value of {} ({}) cannot be less than 0",
///                         key, value_clone
///                     )
///         }).into()) // convert into ValidationErrors
///     } else {
///         Ok(())
///     }
/// });
///
/// let key = "field1".to_string();
/// assert!(v.validate_value(&20, &key).is_ok());
/// let errors = v.validate_value(&-1, &key).unwrap_err();
/// assert_eq!(1, errors.len());
/// let error = errors.errors.get(0).unwrap();
/// assert_eq!(
///     "The value of field1 (-1) cannot be less than 0",
///     error.to_string()
/// );
/// assert_eq!("NOT_LESS_THAN_0", error.type_id);
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
            id: self.id,
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

impl<Value, Key> Validation<Value, Key> for ValidatorFn<Value, Key>
where
    Key: Clone + PartialEq,
{
    fn validate_value(&self, value: &Value, key: &Key) -> Result<(), ValidationErrors<Key>> {
        (self.closure)(value, key)
    }
}

/// An function to perform validation on a field asynchonously.
///
/// For the synchronous version, see [ValidatorFn].
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
///                 Err(ValidationError::new(key.clone(), "NOT_LESS_THAN_0")
///                     .with_message(move |key| {
///                         format!(
///                             "The value of {} ({}) cannot be less than 0",
///                             key, value
///                         )
///                     })
///                     .into()) // convert into ValidationErrors
///             } else {
///                 Ok(())
///             }
///         })
///     });
///
/// let key = "field1".to_string();
/// assert!(block_on(v.validate_value(&20, &key)).is_ok());
///
/// let errors = block_on(v.validate_value(&-1, &key)).unwrap_err();
/// assert_eq!(1, errors.len());
/// let error = errors.errors.get(0).unwrap();
/// assert_eq!(
///     "The value of field1 (-1) cannot be less than 0",
///     error.to_string()
/// );
/// assert_eq!("NOT_LESS_THAN_0", error.type_id);
/// ```
#[cfg(feature = "async")]
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
pub struct AsyncValidatorFn<Value, Key> {
    future_producer: Rc<
        dyn Fn(&Value, &Key) -> Pin<Box<dyn Future<Output = Result<(), ValidationErrors<Key>>>>>,
    >,
    id: Uuid,
    key_type: PhantomData<Key>,
    value_type: PhantomData<Value>,
}

#[cfg(feature = "async")]
impl<Value, Key> AsyncValidatorFn<Value, Key>
where
    Key: Clone + PartialEq,
    Value: Clone + PartialEq,
{
    /// Takes a closure that produces a `Future` that produces a
    /// [ValidatorFn] closure.
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

#[cfg(feature = "async")]
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

#[cfg(feature = "async")]
impl<Value, Key> PartialEq for AsyncValidatorFn<Value, Key> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[cfg(feature = "async")]
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

#[cfg(feature = "async")]
impl<Value, Key> Debug for AsyncValidatorFn<Value, Key> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AsyncValidatorFn(future_producer: {:p}, id: {})",
            self.future_producer, self.id
        )
    }
}
