use salvo::{__private::once_cell::sync::OnceCell, prelude::*};
use serde::{Serialize, Deserialize};
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, Column, query};

use std::{env, collections::HashMap};

static SQLITE: OnceCell<SqlitePool> = OnceCell::new();

#[inline]
pub fn get_sqlite() -> &'static SqlitePool {
    unsafe { SQLITE.get_unchecked() }
}

#[derive(Deserialize)]
struct BdtFilter {
    column: String,
    operator: String,
    value: String,
}

#[derive(Deserialize, Default)]
struct BdtRequest {
    table: String,
    columns: Vec<String>,
    filters: Vec<BdtFilter>
}

#[handler]
async fn hello_world(req: &mut Request, res: &mut Response) -> anyhow::Result<()> {

    let request = req.parse_json::<BdtRequest>().await.unwrap_or_default();

    let query_str = {
        let select_str = request.columns.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",");
        let mut constraints: Vec<String> = vec![];

        for (i, filter) in request.filters.iter().enumerate() {
            let constraint = format!("{} {} ?{}", filter.column, filter.operator, i + 1);
            constraints.push(constraint);
        };

        let mut select = sql_query_builder::Select::new()
            .select(&select_str)
            .from(request.table.as_str());

        for constraint in &constraints {
            select = select.where_clause(&constraint);
        }
        select.as_string()
    };

    let mut my_query = query(&query_str);
    for parameter in request.filters {
        my_query = my_query.bind(parameter.value);
    }

    let recs: Vec<SqliteRow> = my_query.fetch_all(get_sqlite()).await.unwrap();

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
