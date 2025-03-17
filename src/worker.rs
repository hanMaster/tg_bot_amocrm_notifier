use crate::config::config;
use crate::model::sync::sync;
use cron::Schedule;
use log::debug;
use sqlx::types::chrono::Local;
use std::str::FromStr;
use teloxide::prelude::Requester;
use teloxide::types::ChatId;
use teloxide::Bot;
use tokio::time::sleep;

pub fn do_work(bot: Bot) {
    tokio::spawn(async move {
        let schedule = Schedule::from_str(&config().SCHEDULE).expect("Schedule is not valid");
        debug!("Upcoming fire times:");
        for datetime in schedule.upcoming(Local).take(5) {
            debug!("-> {}", datetime);
        }

        loop {
            let now = Local::now();
            if let Some(next) = schedule.upcoming(Local).next() {
                let duration = (next - now).to_std().expect("duration cannot be negative");
                sleep(duration).await;
                let info = format!("Задача выполняется в: {}", Local::now());
                debug!("{}", info);
                bot.send_message(ChatId(config().ADMIN_ID), info)
                    .await
                    .expect("TODO: panic message");
                let sync_result = sync().await;
                match sync_result {
                    Ok((have_data, data)) => {
                        if have_data {
                            bot.send_message(ChatId(config().TG_GROUP_ID), data)
                                .await
                                .expect("TODO: panic message");
                        }
                    }
                    Err(e) => {
                        bot.send_message(ChatId(config().ADMIN_ID), e.to_string())
                            .await
                            .expect("TODO: panic message");
                    }
                }
            }
        }
    });
}
