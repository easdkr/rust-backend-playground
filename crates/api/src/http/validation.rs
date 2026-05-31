use validator::ValidationErrors;

/// Formats `validator` field errors into a single client-facing message.
pub fn format_validation_errors(errors: &ValidationErrors) -> String {
    let messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, field_errors)| {
            let field = (*field).to_string();
            field_errors.iter().map(move |error| {
                let msg = error
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("validation failed on `{field}`"));
                format!("{field}: {msg}")
            })
        })
        .collect();

    if messages.is_empty() {
        errors.to_string()
    } else {
        messages.join("; ")
    }
}
