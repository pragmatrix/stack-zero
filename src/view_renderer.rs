//! Template rendering using the Tera template engine, inspired by loco.

use std::path::Path;

use anyhow::{anyhow, bail, Result};
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Debug)]
pub struct ViewRenderer {
    tera: Tera,
}

impl ViewRenderer {
    pub fn from_dir(dir: &Path) -> Result<ViewRenderer> {
        let tera = Tera::new(
            dir.join("**")
                .join("*.html")
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path glob"))?,
        )?;
        Ok(Self { tera })
    }

    pub fn render(&self, key: &str, data: impl Serialize) -> Result<String> {
        let context = Context::from_serialize(data)?;

        // Expose more specific errors, see `https://github.com/Keats/tera/issues/915`
        match self.tera.render(key, &context) {
            Err(e) => bail!("Rendering `{key}` template failed: {:?}", e.kind),
            Ok(r) => Ok(r),
        }
    }
}
