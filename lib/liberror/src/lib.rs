use std::{error::Error, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, valuable::Valuable)]
#[serde(rename_all = "camelCase")]
pub struct AnyError {
    #[serde(rename = "$type")]
    pub r#type: String,
    pub context: AnyErrorContext,
}
impl<E: Error + Sized> From<E> for AnyError {
    fn from(value: E) -> Self {
        let r#type = std::any::type_name::<E>().replace("::", ".").to_string();
        let message = format!("{value}");
        let inner_error = value.source().map(|e| Box::new(AnyError::from(e)));

        Self {
            r#type,
            context: AnyErrorContext {
                message,
                inner_error,
            },
        }
    }
}
impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.context.message)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, valuable::Valuable)]
#[serde(rename_all = "camelCase")]
pub struct AnyErrorContext {
    message: String,
    inner_error: Option<Box<AnyError>>,
}
