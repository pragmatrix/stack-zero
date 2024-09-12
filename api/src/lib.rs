use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Validate, Default, Serialize, Deserialize)]
pub struct SignUp {
    #[validate(email(message = "email"))]
    pub email: String,
}

#[derive(Debug, Validate, Default, Serialize, Deserialize)]
pub struct SignUpAuthenticated {
    /// The token from the Sign up email.
    pub token: String,
    #[validate(length(min = 3, message = "username"))]
    pub name: String,
    /// Manually validated
    pub password: String,
}
