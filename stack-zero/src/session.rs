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
