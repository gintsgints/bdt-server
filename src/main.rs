use salvo::{__private::once_cell::sync::OnceCell, prelude::*};
use anyhow::{Result};
use sqlx::sqlite::SqlitePool;
use std::env;

static SQLITE: OnceCell<SqlitePool> = OnceCell::new();

#[inline]
pub fn get_sqlite() -> &'static SqlitePool {
    unsafe { SQLITE.get_unchecked() }
}

#[handler]
async fn hello_world(res: &mut Response) -> Result<()> {
    let recs = sqlx::query(
        r#"
SELECT config_type
FROM TT_CONFIG
        "#
    )
    .fetch_all(get_sqlite())
    .await?;

    res.render("hello world!");
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    _ = &env::set_var("DATABASE_URL", "sqlite:./data/db.db");
    let pool = SqlitePool::connect(&env::var("DATABASE_URL")?).await?;
    SQLITE.set(pool).unwrap();

    let router = Router::new().get(hello_world);
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    Server::new(acceptor)
        .serve(router)
        .await;
    Ok(())
}
