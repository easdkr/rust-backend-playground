use validator::ValidationError;

pub fn non_empty_trimmed(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        let mut err = ValidationError::new("non_empty_trimmed");
        err.message = Some("must not be empty".into());
        return Err(err);
    }
    Ok(())
}
