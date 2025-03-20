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
    pub project: String,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub facing: String,
    pub created_on: NaiveDateTime,
    pub updated_on: String,
}
#[derive(FromRow, Debug)]
pub struct HouseNumbers {
    pub house: i32,
}
#[derive(FromRow, Debug)]
pub struct ObjectNumbers {
    pub object: i32,
}

#[derive(Debug, Clone)]
pub struct DealForAdd {
    pub deal_id: u64,
    pub project: String,
    pub house: i32,
    pub object_type: String,
    pub object: i32,
    pub facing: String,
    pub created_on: NaiveDateTime,
}

impl Db {
    pub async fn list_house_numbers(&self, project: &str, object_type: &str) -> Result<Vec<i32>> {
        let records: Vec<HouseNumbers> = sqlx::query_as(
            "SELECT DISTINCT house FROM deal WHERE project = $1 AND object_type = $2 ORDER BY house ",
        )
        .bind(project)
        .bind(object_type)
        .fetch_all(&self.db)
        .await?;
        debug!("[list_house_numbers] {:#?}", records);
        let res = records.iter().map(|r| r.house).collect();
        Ok(res)
    }

    pub async fn list_numbers(
        &self,
        project: &str,
        object_type: &str,
        house: i32,
    ) -> Result<Vec<i32>> {
        let records: Vec<ObjectNumbers> = sqlx::query_as(
            "SELECT object FROM deal WHERE project = $1 AND object_type = $2 AND house = $3 ORDER BY object ",
        )
        .bind(project)
        .bind(object_type)
            .bind(house)
        .fetch_all(&self.db)
        .await?;
        let res = records.iter().map(|r| r.object).collect();
        Ok(res)
    }

    pub async fn create_deal(&self, d: &DealForAdd) -> Result<()> {
        debug!("create deal with data: {:?}", &d);
        let (id,): (i64,) = sqlx::query_as(
            r#"
                INSERT INTO deal (deal_id, project, house, object_type, object, facing, created_on)
                VALUES($1, $2, $3, $4, $5, $6,$7) returning id"#,
        )
        .bind(d.deal_id as i32)
        .bind(&d.project)
        .bind(d.house)
        .bind(&d.object_type)
        .bind(d.object)
        .bind(&d.facing)
        .bind(d.created_on)
        .fetch_one(&self.db)
        .await?;
        debug!("Created row with id: {}", id);
        Ok(())
    }

    pub async fn read_deal_ids(&self) -> Result<Vec<u64>> {
        let records: Vec<HouseData> = sqlx::query_as("SELECT * FROM deal")
            .fetch_all(&self.db)
            .await?;
        let res = records.iter().map(|r| r.deal_id).collect();
        Ok(res)
    }

    async fn get_deal(
        &self,
        project: &str,
        object_type: &str,
        house: i32,
        number: i32,
    ) -> Result<HouseData> {
        let rows = sqlx::query_as(
            r#"
            SELECT * FROM deal WHERE project = $1 AND object_type = $2 AND house = $3 AND object = $4 "#,
        )
        .bind(project)
        .bind(object_type)
        .bind(house)
        .bind(number)
        .fetch_one(&self.db)
        .await?;
        Ok(rows)
    }
}

pub async fn prepare_numbers_response(project: &str, object_type: &str, house: i32) -> String {
    let db = Db::new().await;
    let result = db.list_numbers(project, object_type, house).await;
    match result {
        Ok(numbers) => {
            if numbers.is_empty() {
                "Объектов не найдено".to_string()
            } else {
                let res = numbers.iter().fold(
                    "Найдены объекты с номерами:\n".to_string(),
                    |mut output, b| {
                        let _ = write!(output, "/{}, ", b);
                        output
                    },
                );
                res
            }
        }
        Err(err) => {
            error!("{:?}", err);
            "Ошибка при получении объектов".to_string()
        }
    }
}

pub async fn get_house_numbers(project: &str, object_type: &str) -> Vec<i32> {
    let db = Db::new().await;
    let res = db.list_house_numbers(project, object_type).await;
    res.unwrap_or_else(|e| {
        error!("[get_house_numbers] {:?}", e);
        vec![]
    })
}

pub async fn get_object_numbers(project: &str, object_type: &str, house: i32) -> Vec<i32> {
    let db = Db::new().await;
    let res = db.list_numbers(project, object_type, house).await;
    res.unwrap_or_else(|e| {
        error!("[get_object_numbers] {:?}", e);
        vec![]
    })
}

pub async fn prepare_response(project: &str, object_type: &str, house: i32, number: i32) -> String {
    let db = Db::new().await;
    let result = db.get_deal(project, object_type, house, number).await;

    match result {
        Ok(b) => {
            let facing = if b.object_type.eq("Квартиры") {
                format!("Тип отделки: {}\n", b.facing)
            } else {
                "".to_string()
            };
            let res = format!(
                "Проект: {}\nДом № {}\nТип объекта: {}\n№ {}\n{}Дата регистрации: {}\nПередать объект до: {}\n",
                b.project,
                b.house,
                b.object_type,
                b.object,
                facing,
                b.created_on.format("%d.%m.%Y"),
                b.created_on
                    .add(Duration::from_secs(2592000)) // 30 days
                    .format("%d.%m.%Y")
            );

            if res.is_empty() {
                "Нет данных".to_string()
            } else {
                res
            }
        }

        Err(e) => {
            error!("Prepare response error: {}", e);
            "Ошибка чтения данных".to_string()
        }
    }
}
