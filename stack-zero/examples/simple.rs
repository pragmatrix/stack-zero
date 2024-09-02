use std::{net::SocketAddr, sync::Arc};

use anyhow::Result;
use axum::{extract::FromRef, Router};
use dotenv::dotenv;
use stack_zero::{Config, StackZero};
use tokio::net::TcpListener;

#[derive(Clone)]
struct MyState {
    sz: Arc<StackZero>,
}

impl FromRef<MyState> for Arc<StackZero> {
    fn from_ref(input: &MyState) -> Self {
        input.sz.clone()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv()?;

    let stack_zero = StackZero::new(Config::default()).await?;

    let app = Router::new();
    let app = StackZero::install_routes(app).with_state(MyState {
        sz: Arc::new(stack_zero),
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
