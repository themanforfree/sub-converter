use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("parse error: {detail}")]
    ParseError { detail: String },

    #[error("unsupported: {what}")]
    Unsupported { what: String },

    #[error("emit error: {detail}")]
    EmitError { detail: String },

    #[error("template error: {field}")]
    TemplateError { field: String },

    #[error("validation error: {reason}")]
    ValidationError { reason: String },
}

pub type Result<T> = std::result::Result<T, Error>;
