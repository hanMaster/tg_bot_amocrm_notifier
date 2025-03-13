use crate::error::Error;
use crate::model::data::FlexibleType::Str;
use crate::model::data::{CustomField, ProfitRecord, Record, Val};
use crate::model::deal::DealForAdd;
use crate::model::Db;
use crate::Result;
use log::{debug, error, info};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
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

    let mut leads = extract_deal_ids(data);

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
        let leads_in_while = extract_deal_ids(data);

        leads.extend(leads_in_while);
    }

    let full_data = get_profit_data(leads).await?;

    let response = if !full_data.is_empty() {
        db.update_log(full_data.len()).await?;
        let leads_cloned = full_data.clone();
        for lead in full_data {
            let saved = db.read_deal(&lead.deal_id).await.is_some();
            if !saved {
                db.create_deal(lead).await?;
            }
        }
        let res = leads_cloned.iter().fold(String::new(), |mut output, b| {
            let _ = writeln!(
                output,
                "Дом № {} - {} №{}",
                b.house, b.object_type, b.object
            );
            output
        });
        (true, res)
    } else {
        (false, "Синхронизация выполнена".to_string())
    };

    db.db.close().await;
    Ok(response)
}

fn extract_deal_ids(record: Record) -> Vec<u64> {
    let leads = record
        ._embedded
        .leads
        .iter()
        .filter(|l| {
            l.custom_fields_values.contains(&CustomField {
                field_id: 1631153,
                field_name: "Тип договора".to_string(),
                values: vec![Val {
                    value: Str("ДКП".to_string()),
                    enum_id: Some(4661181),
                }],
            })
        })
        .map(|l| l.id)
        .collect::<Vec<_>>();

    info!("extractor leads {:?}", leads);
    leads
}

async fn get_profit_data(ids: Vec<u64>) -> Result<Vec<DealForAdd>> {
    let base_url = env::var("PROFIT_URL").expect("PROFIT_URL must be set");

    let token = get_profit_token(&base_url).await?;

    let mut res: Vec<DealForAdd> = Vec::with_capacity(ids.len());

    for id in ids {
        let url = format!("{}/property/deal/{}?access_token={}", base_url, id, token);
        info!("fetching {}", url);
        let response = Client::new()
            .get(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        if response.status() == reqwest::StatusCode::OK {
            info!("JSON parse");

            let data = response.json::<ProfitRecord>().await;
            // let date_utc = DateTime::from_timestamp(l.created_at, 0).unwrap();
            // let created_on = DateTime::<Local>::from(date_utc).naive_local();

            match data {
                Ok(d) => {
                    info!("received: {:?}", d);
                    if d.status == "success" {
                        let p = d.data.iter().next().unwrap();
                        let object_type = if p.house_name.contains("Кладовк") {
                            "кладовка".to_string()
                        } else {
                            "Квартира".to_string()
                        };

                        let house = p.house_name.split('№').collect::<Vec<_>>()[1];
                        let house = house.parse::<i32>()?;

                        let rec = DealForAdd {
                            deal_id: id.to_string(),
                            house,
                            object_type,
                            object: p.number.parse::<i32>()?,
                        };
                        res.push(rec);
                    }
                }
                Err(e) => {
                    error!("PARSE ERROR: {:?}", e);
                }
            }
        }
    }

    Ok(res)
}

#[derive(Deserialize)]
struct AuthResponse {
    pub access_token: String,
}
async fn get_profit_token(url: &str) -> Result<String> {
    let key = env::var("PROFIT_API_KEY").expect("PROFIT_API_KEY must be set");

    let payload = json!({
      "type": "api-app",
      "credentials": {
        "pb_api_key": key
      }
    });
    let client = Client::new()
        .post(format!("{url}/authentication"))
        .json(&payload);

    let result = client.send().await?;

    if result.status() == reqwest::StatusCode::OK {
        let token = result.json::<AuthResponse>().await?.access_token;
        debug!("Profitbase Token: {:?}", token);
        return Ok(token);
    }

    Err(Error::ProfitAuthFailed)
}
