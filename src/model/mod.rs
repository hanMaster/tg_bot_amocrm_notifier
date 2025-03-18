use crate::config::config;
use crate::Result;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Sqlite, SqlitePool};

pub mod deal;
pub mod sync;

mod data;

pub struct Db {
    pub db: SqlitePool,
}

impl Db {
    pub async fn new() -> Db {
        init_db().await.unwrap()
    }
}

async fn create_schema(db_url: &str) -> Result<()> {
    let pool = SqlitePool::connect(db_url).await?;
    let qry = r#"
    CREATE TABLE IF NOT EXISTS deal
    (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        deal_id         BIGINTEGER          NOT NULL,
        house           INTEGER             NOT NULL,
        object_type     TEXT                NOT NULL,
        object          INTEGER             NOT NULL,
        facing          TEXT,
        created_on      DATETIME DEFAULT    (datetime('now', 'localtime')),
        updated_on      DATETIME DEFAULT    (datetime('now', 'localtime'))
    );
    "#;
    let _ = sqlx::query(qry).execute(&pool).await?;
    pool.close().await;
    Ok(())
}

pub async fn init_db() -> Result<Db> {
    if !Sqlite::database_exists(&config().DB_URL)
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(&config().DB_URL).await?;
        match create_schema(&config().DB_URL).await {
            Ok(_) => log::info!("database created successfully"),
            Err(e) => panic!("{}", e),
        }
    }
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config().DB_URL)
        .await?;

    Ok(Db { db })
}
