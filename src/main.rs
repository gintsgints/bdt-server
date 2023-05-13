use salvo::{__private::once_cell::sync::OnceCell, prelude::*};
use anyhow::{Result};
use serde::Serialize;
use sqlx::sqlite::SqlitePool;
use sqlx::{Row, Column};

use std::{env, collections::HashMap};

static SQLITE: OnceCell<SqlitePool> = OnceCell::new();

#[inline]
pub fn get_sqlite() -> &'static SqlitePool {
    unsafe { SQLITE.get_unchecked() }
}

#[handler]
async fn hello_world(req: &mut Request, res: &mut Response) -> Result<()> {

    let table = req.query::<String>("table").unwrap();

    let sql_string = sql_query_builder::Select::new()
        .select("config_type")
        .from(&table)
        .as_string();

    let recs = sqlx::query(&sql_string)
        .fetch_all(get_sqlite())
        .await?;
    let mut rows:Vec<BdtRow> = vec![];
    for rec in recs {
        let mut values: HashMap<String, String> = HashMap::new();
        for col in rec.columns() {
            let name = col.name();
            values.insert(name.to_string(), rec.get(name));        
        }
        let row = BdtRow { values };
        rows.push(row);
    }

    res.render(Json(rows));
    Ok(())
}

#[derive(Serialize)]
pub struct BdtRow {
    pub values: HashMap<String, String>
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
