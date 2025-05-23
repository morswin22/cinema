use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
    #[validate(must_match = "password")]
    pub password_confirmation: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct LoginForm {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}
