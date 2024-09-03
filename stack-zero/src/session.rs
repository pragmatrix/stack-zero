use anyhow::{Context, Result};
use axum::Router;
use tokio::task::JoinHandle;
use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, SessionManagerLayer};
use tower_sessions_redis_store::{fred::prelude::*, RedisStore};

use crate::Environment;

#[derive(Debug)]
pub enum SessionStore {
    Memory(MemoryStore),
    Redis {
        pool: RedisPool,
        connection: JoinHandle<Result<(), RedisError>>,
    },
}

impl SessionStore {
    pub async fn from_env(environment: Environment) -> Result<Self> {
        match environment {
            Environment::Development => Ok(Self::Memory(MemoryStore::default())),
            Environment::Production => {
                let redis_config = RedisConfig::default();

                let pool = RedisPool::new(RedisConfig::default(), None, None, None, 6)?;

                let redis_conn = pool.connect();
                pool.wait_for_connect()
                    .await
                    .with_context(|| format!("Connecting Redis pool: {:?}", redis_config.server))?;

                Ok(Self::Redis {
                    pool,
                    connection: redis_conn,
                })
            }
        }
    }

    pub fn add_layer<S: Clone + Send + Sync + 'static>(
        &self,
        environment: Environment,
        router: Router<S>,
    ) -> Router<S> {
        match self {
            SessionStore::Memory(store) => {
                let session_layer = SessionManagerLayer::new(store.clone())
                    .with_secure(environment.use_secure_cookies())
                    // TODO: configure?
                    .with_expiry(Expiry::OnInactivity(Duration::seconds(20)));
                router.layer(session_layer)
            }
            SessionStore::Redis { pool, .. } => {
                let session_store = RedisStore::new(pool.clone());

                let session_layer = SessionManagerLayer::new(session_store)
                    .with_secure(environment.use_secure_cookies())
                    // TODO: configure?
                    .with_expiry(Expiry::OnInactivity(Duration::seconds(20)));
                router.layer(session_layer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use anyhow::Result;
    use rstest::rstest;

    use crate::test_helper::redis_container;

    #[rstest]
    #[tokio::test]
    #[ignore = "manually only"]
    async fn start_redis_container(
        redis_container: impl Future<Output = Result<String>>,
    ) -> Result<()> {
        let _c = redis_container.await?;
        Ok(())
    }
}
