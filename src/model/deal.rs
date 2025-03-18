use crate::model::Db;
use crate::Result;
use log::{debug, error};
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;
use std::fmt::Write;
use std::ops::Add;
use std::time::Duration;

#[allow(dead_code)]
#[derive(FromRow)]
pub struct HouseData {
    pub id: i32,
    pub deal_id: u64,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub created_on: NaiveDateTime,
    pub updated_on: String,
}

#[derive(Debug, Clone)]
pub struct DealForAdd {
    pub deal_id: u64,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub created_on: NaiveDateTime,
}

impl Db {
    async fn list(&self, object_type: &str) -> Result<Vec<HouseData>> {
        debug!("get apartments_list");
        let rows = sqlx::query_as("SELECT * FROM deal WHERE object_type = $1 ORDER BY object ASC ")
            .bind(object_type)
            .fetch_all(&self.db)
            .await?;
        Ok(rows)
    }

    pub async fn create_deal(&self, d: &DealForAdd) -> Result<()> {
        debug!("create deal with data: {:?}", &d);
        let (id, ): (i64,) =
            sqlx::query_as("INSERT INTO deal (deal_id, house, object_type, object, created_on) VALUES($1, $2, $3, $4, $5) returning id")
                .bind(d.deal_id as i32)
                .bind(d.house)
                .bind(&d.object_type)
                .bind(d.object)
                .bind(d.created_on)
                .fetch_one(&self.db)
                .await?;
        debug!("Created row with id: {}", id);
        Ok(())
    }

    pub async fn read_deals(&self) -> Result<Vec<u64>> {
        let records: Vec<HouseData> = sqlx::query_as("SELECT * FROM deal")
            .fetch_all(&self.db)
            .await?;
        let res = records.iter().map(|r| r.deal_id).collect();
        Ok(res)
    }
}

pub async fn apartments() -> String {
    debug!("get apartments");
    prepare_response("Квартира").await
}
pub async fn storage_rooms() -> String {
    debug!("get storage_rooms");
    prepare_response("кладовка").await
}

async fn prepare_response(object_type: &str) -> String {
    let db = Db::new().await;
    let result = db.list(object_type).await;

    match result {
        Ok(rows) => {
            let res = rows.iter().fold(String::new(), |mut output, b| {
                let _ = writeln!(
                    output,
                    "Проект: Сити\nДом № {}\nТип объекта: {} № {:0>3}\nРегистрация: {}\nПередача: {}\n",
                    b.house,
                    b.object_type,
                    b.object,
                    b.created_on.format("%d.%m.%Y"),
                    b.created_on
                        .add(Duration::from_secs(2592000))// 30 days
                        .format("%d.%m.%Y")
                );
                output
            });
            if res.is_empty() {
                "Нет данных".to_string()
            } else {
                format!("{res}\nВсего записей: {}", rows.len())
            }
        }

        Err(e) => {
            error!("Prepare response error: {}", e);
            "Ошибка чтения данных".to_string()
        }
    }
}
