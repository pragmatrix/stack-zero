use serde::Deserialize;

use crate::email;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub smtp: email::Config,
}
