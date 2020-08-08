use crate::ValidationErrors;

/// Join validation results, concatinating any errors they may
/// contain. If any of the results are an `Err` it will return an
/// `Err` containing all the errors from all the results.
///
/// ## Example
/// ```
/// use form_validation::{concat_results, ValidationError, ValidationErrors};
/// let results = vec![
///     Ok(()),
///     Err(ValidationErrors::new(vec![ValidationError::new("field1", "TEST_ERROR1")])),
///     Err(ValidationErrors::new(vec![ValidationError::new("field1", "TEST_ERROR2")])),
///     Err(ValidationErrors::new(vec![ValidationError::new("field2", "TEST_ERROR1")])),
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
