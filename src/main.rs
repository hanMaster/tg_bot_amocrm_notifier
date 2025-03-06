pub use crate::error::Result;
use crate::model::deal::{apartments, storage_rooms};
use crate::model::sync::sync;
use dotenvy::dotenv;
use teloxide::{prelude::*, utils::command::BotCommands};

mod error;
mod model;
mod worker;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect("dotenv init failed");

    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot = Bot::from_env();
    bot.set_my_commands(Command::bot_commands())
        .await
        .expect("Failed to set bot commands");

    let cloned_bot = bot.clone();

    worker::do_work(cloned_bot);

    Command::repl(bot, answer).await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "Здесь можно увидеть номера квартир и кладовок, проданных по ДКП:"
)]
enum Command {
    #[command(description = "Вывод спсика квартир")]
    Apartments,
    #[command(description = "Вывод списка кладовок")]
    StorageRooms,
    #[command(description = "Запросить данные из AmoCRM")]
    Sync,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Apartments => {
            let data = apartments().await;
            bot.send_message(msg.chat.id, data).await?
        }
        Command::StorageRooms => {
            let data = storage_rooms().await;
            bot.send_message(msg.chat.id, data).await?
        }
        Command::Sync => {
            let data = sync().await;
            bot.send_message(msg.chat.id, data.1).await?
        }
    };

    Ok(())
}
