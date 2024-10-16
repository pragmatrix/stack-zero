//! Renders string templates with environment variables that are prefixed with `SZ_`. Inside the template,
//! the env variable is then available without the `SZ_` prefix.

use tera::Tera;

#[derive(Debug)]
pub struct EnvRenderer {
    context: tera::Context,
}

const PREFIX: &str = "SZ_";

impl EnvRenderer {
    pub fn from_env() -> Self {
        let variables = std::env::vars()
            .filter(|(key, _)| key.starts_with(PREFIX))
            .collect::<Vec<_>>();

        let mut context = tera::Context::new();
        for (key, value) in &variables {
            context.insert(&key[3..], value); // Remove "SZ_" prefix
        }

        EnvRenderer { context }
    }

    pub fn render(&self, template: &str) -> Result<String, anyhow::Error> {
        let mut tera = Tera::default();
        Ok(tera.render_str(template, &self.context)?)
    }
}
