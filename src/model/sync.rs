use crate::model::data::StringOrInt::Str;
use crate::model::data::{CustomField, Record, Value};
use crate::model::deal::DealForAdd;
use crate::model::Db;
use crate::Result;
use log::debug;
use reqwest::Client;
use sqlx::types::chrono::{DateTime, Local};
use sqlx::FromRow;
use std::env;
use std::fmt::Write;

#[allow(dead_code)]
#[derive(Debug, FromRow)]
pub struct LogData {
    pub id: i32,
    pub last_checked_date: i32,
    pub row_count: i32,
    pub created_on: String,
    pub updated_on: String,
}

impl Db {
    pub async fn get_last_sync_date(&self) -> Result<String> {
        let log_record: Option<LogData> =
            sqlx::query_as("SELECT * FROM log ORDER BY created_on DESC LIMIT 1")
                .fetch_optional(&self.db)
                .await?;
        debug!("log_record: {:?}", log_record);
        match log_record {
            None => Ok("1600437670".to_owned()),
            Some(r) => Ok(r.last_checked_date.to_string()),
        }
    }
}

pub async fn sync() -> (bool, String) {
    fetch()
        .await
        .unwrap_or((false, "Failed to fetch AmoCRM".to_string()))
}

async fn fetch() -> Result<(bool, String)> {
    let db = Db::new().await;
    let url = env::var("AMO_URL").expect("AMO_URL must be set");
    let token = env::var("AMO_TOKEN").expect("AMO_TOKEN must be set");
    let from_date = db.get_last_sync_date().await?;

    debug!("From Date: {:?}", from_date);

    let client = Client::new()
        .get(format!("{url}&filter[created_at][from]={from_date}"))
        .header("Authorization", format!("Bearer {}", token));

    let result = client.send().await?;

    if result.status() == reqwest::StatusCode::NO_CONTENT {
        db.update_log(0).await?;
        return Ok((false, "Новых сделок не найдено".to_string()));
    }

    let mut data = result.json::<Record>().await?;

    let mut next = data._links.next.take();
    debug!("next: {:?}", next);

    let mut leads = extract_data(data);

    while next.is_some() {
        let client = Client::new()
            .get(format!(
                "{url}&filter[created_at][from]={}",
                next.as_ref().unwrap().href
            ))
            .header("Authorization", format!("Bearer {}", token));
        let mut data = client.send().await?.json::<Record>().await?;

        next = data._links.next.take();
        debug!("next in while: {:?}", next);
        let leads_in_while = extract_data(data);

        leads.extend(leads_in_while);
    }

    let response = if !leads.is_empty() {
        db.update_log(leads.len()).await?;
        let leads_cloned = leads.clone();
        for lead in leads {
            let saved = db.read_deal(&lead.deal_id).await.is_some();
            if !saved {
                db.create_deal(lead).await?;
            }
        }
        let res = leads_cloned.iter().fold(String::new(), |mut output, b| {
            let _ = writeln!(output, "Дом: {} {} {}", b.house, b.object_type, b.object);
            output
        });
        (true, res)
    } else {
        (false, "Синхронизация выполнена".to_string())
    };

    db.db.close().await;
    Ok(response)
}

fn extract_data(record: Record) -> Vec<DealForAdd> {
    let leads = record
        ._embedded
        .leads
        .iter()
        .filter(|l| {
            l.custom_fields_values.contains(&CustomField {
                field_id: 1631153,
                field_name: "Тип договора".to_string(),
                values: vec![Value {
                    value: Str("ДКП".to_string()),
                    enum_id: Some(4661181),
                }],
            })
        })
        .map(|l| {
            let id = l.id;
            let object = l.name.clone();
            let date_utc = DateTime::from_timestamp(l.created_at, 0).unwrap();
            let created_on = DateTime::<Local>::from(date_utc).naive_local();

            DealForAdd {
                deal_id: id.to_string(),
                house: "дом".to_string(),
                object_type: "кладовка".to_string(),
                object,
                created_on,
            }
        })
        .collect::<Vec<_>>();

    debug!("extractor leads {:?}", leads);
    leads
}
