use std::env;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use sqlx::sqlite::SqlitePoolOptions;
use crate::Result;

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
        deal_id         TEXT                NOT NULL,
        house           INTEGER             NOT NULL,
        object_type     TEXT                NOT NULL,
        object          INTEGER             NOT NULL,
        created_on      DATETIME DEFAULT    (datetime('now', 'localtime')),
        updated_on      DATETIME DEFAULT    (datetime('now', 'localtime'))
    );

    CREATE TABLE IF NOT EXISTS log
    (
        id                  INTEGER PRIMARY KEY AUTOINCREMENT,
        last_checked_date   TIMESTAMP               NOT NULL,
        row_count           INT                     NOT NULL,
        created_on          DATETIME DEFAULT    (datetime('now', 'localtime')),
        updated_on          DATETIME DEFAULT    (datetime('now', 'localtime'))
    );
    "#;
    let _ = sqlx::query(qry).execute(&pool).await?;
    pool.close().await;
    Ok(())
}

pub async fn init_db()->Result<Db> {
    let db_url = env::var("DB_URL")?;
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url).await?;
        match create_schema(&db_url).await {
            Ok(_) => log::info!("database created successfully"),
            Err(e) => panic!("{}", e),
        }
    }
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url).await?;
    // let qry = "INSERT INTO log (last_checked_date) VALUES($1)";
    // let result = sqlx::query(&qry).bind("testing").execute(&db).await?;

    Ok(Db { db })
}
