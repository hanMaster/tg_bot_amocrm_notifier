use crate::model::sync::sync;
use cron::Schedule;
use log::debug;
use sqlx::types::chrono::Local;
use std::env;
use std::str::FromStr;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::time::sleep;

pub fn do_work(bot: Bot) {
    tokio::spawn(async move {
        // Каждый день в 9:00
        let expression = env::var("SCHEDULE").expect("Setup schedule failed in worker");

        let chat_id = env::var("TG_GROUP_ID")
            .expect("TG_GROUP_ID is not set")
            .parse::<i64>()
            .unwrap();

        let schedule = Schedule::from_str(&expression).expect("Schedule is not valid");
        debug!("Upcoming fire times:");
        for datetime in schedule.upcoming(Local).take(5) {
            debug!("-> {}", datetime);
        }

        loop {
            let now = Local::now();
            if let Some(next) = schedule.upcoming(Local).next() {
                let duration = (next - now).to_std().expect("duration cannot be negative");
                sleep(duration).await;

                debug!("Задача выполняется в: {}", Local::now());
                let (have_data, data) = sync().await;
                if have_data {
                    bot.send_message(ChatId(chat_id), data)
                        .await
                        .expect("TODO: panic message");
                }
            }
        }
    });
}
