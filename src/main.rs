use crate::config::config;
pub use crate::error::Result;
use crate::model::deal::{get_house_numbers, prepare_numbers_response, prepare_response};
use crate::model::sync::sync;
use dotenvy::dotenv;
use log::info;
use std::error::Error;
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dptree::{case, deps};
use teloxide::types::{KeyboardButton, KeyboardMarkup, KeyboardRemove, ReplyMarkup};
use teloxide::{prelude::*, utils::command::BotCommands};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = std::result::Result<(), Box<dyn Error + Send + Sync>>;

mod config;
mod error;
mod model;
mod worker;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ChooseProject,
    ChooseObjectType {
        project: String,
    },
    ChooseHouseNumber {
        project: String,
        object_type: String,
    },
    ChooseObjectNumber {
        project: String,
        object_type: String,
        house: i32,
    },
}

const PROJECTS: [&str; 2] = ["DNS Сити", "ЖК Формат"];
const OBJECT_TYPES: [&str; 2] = ["Квартиры", "Кладовки"];

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

    let handler = dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .branch(case![Command::Sync].endpoint(sync_handler)),
        )
        .branch(
            Update::filter_message()
                .branch(case![State::Start].endpoint(start))
                .branch(case![State::ChooseProject].endpoint(receive_project_name))
                .branch(case![State::ChooseObjectType { project }].endpoint(receive_object_type))
                .branch(
                    case![State::ChooseHouseNumber {
                        project,
                        object_type
                    }]
                    .endpoint(receive_house_number),
                )
                .branch(
                    case![State::ChooseObjectNumber {
                        project,
                        object_type,
                        house,
                    }]
                    .endpoint(receive_object_number),
                ),
        );

    Dispatcher::builder(bot, handler)
        .dependencies(deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    /// Информация по объекту
    Start,
    /// Запрос данных в AmoCRM
    Sync,
}

fn make_kbd(step: i32) -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    let labels = if step == 1 { PROJECTS } else { OBJECT_TYPES };

    for label in labels.chunks(2) {
        let row = label
            .iter()
            .map(|&item| KeyboardButton::new(item.to_owned()))
            .collect();

        keyboard.push(row);
    }

    KeyboardMarkup::new(keyboard).resize_keyboard()
}
async fn make_house_kbd(project: &str, object_type: &str) -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    let labels = get_house_numbers(project, object_type).await;

    info!("LABELS {:?}", labels);

    for label in labels.chunks(2) {
        let row = label
            .iter()
            .map(|&item| KeyboardButton::new(item.to_string()))
            .collect();

        keyboard.push(row);
    }

    KeyboardMarkup::new(keyboard).resize_keyboard()
}

async fn sync_handler(bot: Bot, msg: Message) -> HandlerResult {
    let data_result = sync().await;
    match data_result {
        Ok(data) => {
            bot.send_message(msg.chat.id, data.1).await?;
        }
        Err(e) => {
            let admin_id = config().ADMIN_ID;
            bot.send_message(ChatId(admin_id), e.to_string()).await?;
            bot.send_message(msg.chat.id, e.to_string()).await?;
        }
    }
    Ok(())
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    if let Some(text) = msg.text() {
        if text.starts_with("/start") {
            let keyboard = make_kbd(1);
            bot.send_message(msg.chat.id, "Выберите проект")
                .reply_markup(keyboard)
                .await?;
            dialogue.update(State::ChooseProject).await?;
        }
    }
    Ok(())
}

async fn receive_project_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            if PROJECTS.contains(&text) {
                let keyboard = make_kbd(2);
                bot.send_message(msg.chat.id, "Квартиры или кладовки?")
                    .reply_markup(keyboard)
                    .await?;
                dialogue
                    .update(State::ChooseObjectType {
                        project: text.into(),
                    })
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                    .await?;
            }
        }
        None => {
            bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                .await?;
        }
    }

    Ok(())
}

async fn receive_object_type(
    bot: Bot,
    dialogue: MyDialogue,
    project: String, // Available from `State::ChooseProject`.
    msg: Message,
) -> HandlerResult {
    match msg.text() {
        Some(object_type) => {
            if OBJECT_TYPES.contains(&object_type) {
                let keyboard = make_house_kbd(&project, object_type).await;
                bot.send_message(msg.chat.id, "Выберите номер дома")
                    .reply_markup(keyboard)
                    .await?;
                dialogue
                    .update(State::ChooseHouseNumber {
                        project,
                        object_type: object_type.into(),
                    })
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                    .await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                .await?;
        }
    }

    Ok(())
}

async fn receive_house_number(
    bot: Bot,
    dialogue: MyDialogue,
    (project, object_type): (String, String), // Available from `State::ChooseObject`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(house)) => {
            let houses = get_house_numbers(&project, &object_type).await;
            if houses.contains(&house) {
                let numbers = prepare_numbers_response(&project, &object_type, house).await;
                bot.send_message(msg.chat.id, numbers)
                    .reply_markup(ReplyMarkup::KeyboardRemove(KeyboardRemove::new()))
                    .await?;

                bot.send_message(msg.chat.id, "Укажите номер помещения")
                    .await?;
                dialogue
                    .update(State::ChooseObjectNumber {
                        project,
                        object_type,
                        house,
                    })
                    .await?;
            } else {
                bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                    .await?;
            }
        }
        _ => {
            bot.send_message(msg.chat.id, "Сделайте выбор кнопками")
                .await?;
        }
    }

    Ok(())
}

async fn receive_object_number(
    bot: Bot,
    dialogue: MyDialogue,
    (project, object_type, house): (String, String, i32), // Available from `State::ChooseHouseNumber`.
    msg: Message,
) -> HandlerResult {
    if let Some(text) = msg.text() {
        let payload = text.trim_start_matches('/');
        match payload.parse::<i32>() {
            Ok(number) => {
                let report = prepare_response(&project, &object_type, house, number).await;
                bot.send_message(msg.chat.id, report).await?;
                dialogue.exit().await?;
            }
            _ => {
                bot.send_message(msg.chat.id, "Шаблон: /номер помещения")
                    .await?;
            }
        }
    }

    Ok(())
}
