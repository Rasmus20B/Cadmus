
use anyhow::Result;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    let req_login = hc.do_post(
        "/api/login",
        json!({
            "username": "demo",
            "pwd":  "welcome"
        }),
    );

    req_login.await?.print().await?;

    hc.do_get("/hello").await?.print().await?;

    Ok(())
}
