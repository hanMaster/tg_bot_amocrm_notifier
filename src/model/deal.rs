use crate::model::Db;
use crate::Result;
use log::debug;
use sqlx::types::chrono::{Local, NaiveDateTime};
use sqlx::FromRow;
use std::fmt::Write;

#[allow(dead_code)]
#[derive(FromRow)]
pub struct HouseData {
    pub id: i32,
    pub deal_id: String,
    pub house: String,
    pub object: String,
    pub created_on: String,
    pub updated_on: String,
}

#[derive(Debug, Clone)]
pub struct DealForAdd {
    pub deal_id: String,
    pub house: String,
    pub object_type: String,
    pub object: String,
    pub created_on: NaiveDateTime,
}

impl Db {
    async fn list(&self, object_type: &str) -> Result<Vec<HouseData>> {
        debug!("get apartments_list");
        let rows = sqlx::query_as("SELECT * FROM deal WHERE object_type = $1")
            .bind(object_type)
            .fetch_all(&self.db)
            .await?;
        Ok(rows)
    }

    pub async fn create_deal(&self, d: DealForAdd) -> Result<()> {
        debug!("create deal with data: {:?}", &d);
        let (id,): (i64,) =
            sqlx::query_as("INSERT INTO deal (deal_id, house, object_type, object, created_on) VALUES($1, $2, $3, $4, $5) returning id")
                .bind(d.deal_id)
                .bind(d.house)
                .bind(d.object_type)
                .bind(d.object)
                .bind(d.created_on)
                .fetch_one(&self.db)
                .await?;
        debug!("Created row with id: {}", id);
        Ok(())
    }

    pub async fn read_deal(&self, deal_id: &str) -> Option<HouseData> {
        sqlx::query_as("SELECT * FROM deal WHERE deal.deal_id=$1")
            .bind(deal_id)
            .fetch_one(&self.db)
            .await
            .ok()
    }

    pub async fn update_log(&self, row_count: usize) -> Result<()> {
        debug!("update log");
        let dt = Local::now();

        let (id,): (i64,) = sqlx::query_as(
            "INSERT INTO log (last_checked_date, row_count) VALUES($1, $2) returning id",
        )
        .bind(dt.timestamp())
        .bind(row_count as i32)
        .fetch_one(&self.db)
        .await?;
        debug!("Created row with id: {}", id);
        Ok(())
    }
}

pub async fn apartments() -> String {
    debug!("get apartments");
    let db = Db::new().await;
    let result = db.list("квартира").await;

    if let Ok(rows) = result {
        let res = rows.iter().fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "Дом: {} Квартира: {}", b.house, b.object);
            output
        });
        if res.is_empty() {
            "Нет данных".to_string()
        } else {
            res
        }
    } else {
        "Failed to get data".to_string()
    }
}
pub async fn storage_rooms() -> String {
    debug!("get storage_rooms");
    let db = Db::new().await;
    let result = db.list("кладовка").await;

    if let Ok(rows) = result {
        let res = rows.iter().fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "Дом: {} Кладовка: {}", b.house, b.object);
            output
        });
        if res.is_empty() {
            "Нет данных".to_string()
        } else {
            res
        }
    } else {
        "Failed to get data".to_string()
    }
}
