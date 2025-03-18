use crate::config::config;
use crate::error::Error;
use crate::model::data::FlexibleType::Str;
use crate::model::data::{CustomField, ProfitRecord, Record, Val};
use crate::model::deal::DealForAdd;
use crate::model::Db;
use crate::Result;
use log::{debug, info};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use sqlx::types::chrono::DateTime;
use std::fmt::Write;
use std::ops::Add;
use std::time::Duration;

pub async fn sync() -> Result<(bool, String)> {
    let db = Db::new().await;

    let client = Client::new()
        .get(format!(
            "{}&filter[created_at][from]=1600437670",
            config().AMO_URL
        ))
        .header("Authorization", format!("Bearer {}", config().AMO_TOKEN));

    let result = client.send().await?;

    if result.status() == reqwest::StatusCode::NO_CONTENT {
        return Ok((false, "Новых сделок не найдено".to_string()));
    }

    let mut data = result.json::<Record>().await?;

    let mut next = data._links.next.take();
    debug!("next: {:?}", next);

    let mut leads = extract_deal_ids(data);

    while next.is_some() {
        let client = Client::new()
            .get(format!(
                "{}&filter[created_at][from]={}",
                config().AMO_URL,
                next.as_ref().unwrap().href
            ))
            .header("Authorization", format!("Bearer {}", config().AMO_TOKEN));
        let mut data = client.send().await?.json::<Record>().await?;

        next = data._links.next.take();
        debug!("next in while: {:?}", next);
        let leads_in_while = extract_deal_ids(data);

        leads.extend(leads_in_while);
    }

    let response = if !leads.is_empty() {
        let mut new_data: Vec<DealForAdd> = vec![];
        for lead in leads {
            let saved = db.read_deal(lead).await.is_some();
            if !saved {
                let full_data = get_profit_data(lead).await?;
                db.create_deal(&full_data).await?;
                new_data.push(full_data);
            }
        }
        let res = new_data.iter().fold(String::new(), |mut output, b| {
            let _ = writeln!(
                output,
                "Проект: Сити\nДом № {}\nТип объекта: {} № {:0>3}\nРегистрация: {}\nПередача: {}\n",
                b.house,
                b.object_type,
                b.object,
                b.created_on.format("%d.%m.%Y"),
                b.created_on.add(Duration::from_secs(8400)).format("%d.%m.%Y")
            );
            output
        });

        if res.is_empty() {
            (false, "Новых сделок не найдено".to_string())
        } else {
            (true, res)
        }
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

async fn get_profit_data(deal_id: u64) -> Result<DealForAdd> {
    let token = get_profit_token(&config().PROFIT_URL).await?;

    let url = format!(
        "{}/property/deal/{}?access_token={}",
        config().PROFIT_URL,
        deal_id,
        token
    );

    debug!("fetching {}", url);
    let response = Client::new()
        .get(url)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    if response.status() == reqwest::StatusCode::OK {
        debug!("JSON parse");

        let data = response.json::<ProfitRecord>().await?;

        debug!("received: {:?}", data);
        if data.status == "success" {
            let p = data.data.first().unwrap();
            let object_type = if p.house_name.contains("Кладовк") {
                "кладовка".to_string()
            } else {
                "Квартира".to_string()
            };

            let house_parts = p.house_name.split('№').collect::<Vec<_>>();
            let house = if house_parts.len() < 2 {
                house_parts[0].to_string()
            } else {
                house_parts[1].to_string()
            };
            let house = house.parse::<i32>().unwrap_or(-1);

            // soldAt
            let created_on = DateTime::parse_from_str(
                format!("{} +0000", p.sold_at).as_str(),
                "%Y-%m-%d %H:%M %z",
            )
            .unwrap_or(Default::default())
            .naive_local();

            Ok(DealForAdd {
                deal_id,
                house,
                object_type,
                object: p.number.parse::<i32>()?,
                created_on,
            })
        } else {
            Err(Error::ProfitGetDataFailed)
        }
    } else {
        Err(Error::ProfitGetDataFailed)
    }
}

#[derive(Deserialize)]
struct AuthResponse {
    pub access_token: String,
}
async fn get_profit_token(url: &str) -> Result<String> {
    let payload = json!({
      "type": "api-app",
      "credentials": {
        "pb_api_key": config().PROFIT_API_KEY,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_date() {
        let str_date = "2025-03-12 04:38 +1000";
        let res = DateTime::parse_from_str(str_date, "%Y-%m-%d %H:%M %z");
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
