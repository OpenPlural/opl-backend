use serde::{Deserialize, Serialize};

pub mod auth;
pub mod folder;
pub mod member;
pub mod friend;
pub mod user;
pub mod front;
pub mod session;
pub mod fields;
pub mod sync;
pub mod privacy;
pub mod deletion;
pub mod apikey;

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default)]
    pub page: u32
}

#[derive(Debug, Serialize)]
pub struct IdResponse<T: Serialize> {
    pub id: T,
}

pub(in crate::model) fn validate_string_length(model_name: &str, field_name: &str, str: &str, min_length: Option<usize>, max_length: Option<usize>, nullable: bool) -> Result<(), String> {
    if let Some(min) = min_length && str.len() < min {
        if nullable && str.is_empty() {
            return Err(format!("{model_name}.{field_name} must be at least {min} characters long. If you want to leave this field empty, please set it to NULL instead."));
        }
        return Err(format!("{model_name}.{field_name} must be at least {min} characters long, was {}", str.len()));
    }
    if let Some(max) = max_length && str.len() > max {
        return Err(format!("{model_name}.{field_name} must be at most {max} characters long, was {}", str.len()));
    }
    Ok(())
}

pub(in crate::model) fn validate_number_range(model_name: &str, field_name: &str, num: isize, min: isize, max: isize) -> Result<(), String> {
    if num < min {
        return Err(format!("{model_name}.{field_name} must not be less than {min}, was {num}"));
    }
    if num > max {
        return Err(format!("{model_name}.{field_name} must not be greater than {max}, was {num}"));
    }
    Ok(())
}

pub(in crate::model) fn validate_color_range(model_name: &str, field_name: &str, color: u32) -> Result<(), String> {
    if color > 16777215 {
        Err(format!("{model_name}.{field_name} must be between 0 and 16777215, was {color}"))
    } else {
        Ok(())
    }
}