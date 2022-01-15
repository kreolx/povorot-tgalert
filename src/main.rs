use teloxide::{prelude::*, utils::command::BotCommand};
use std::{env, str};
use std::error::Error;
use redis::{Commands,RedisError};
use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use tokio::runtime::Runtime;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Duration};


const REDIS_CON_STRING: &str = "REDIS_CON_STRING";
const TELEGRAM_BOT_TOKEN: &str = "TELEGRAM_BOT_TOKEN";
const REDIS_NOTIFY_CUSTOMERS: &str = "notifycustomers";
const RABBIT_CON_STRING: &str = "RABBIT_CON_STRING";

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "Command")]
enum Command {
    #[command(description = "add to alert notification")]
    Register,
}

#[tokio::main]
async fn main() {
    let (_, _) = tokio::join!(create_bot(), consume());
}

async fn create_bot() {
    teloxide::enable_logging!();
    log::info!("Starting bot");
    let token = env::var(TELEGRAM_BOT_TOKEN).expect("Not found token");
    let bot = Bot::new(token).auto_send();
    let bot_name = "povorot_alert".to_string();
    teloxide::commands_repl(bot, bot_name, answer).await;
}

async fn consume() {
    log::info!("Starting consumer");
        let rabbit_con_str =
            env::var(RABBIT_CON_STRING).unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".into());
        let conn = Connection::connect(
            &rabbit_con_str,
            ConnectionProperties::default().with_default_executor(8),
        )
        .await
        .unwrap();
        let channel = conn.create_channel().await.unwrap();
        let _queue_notify = channel
            .queue_declare("request-notification", QueueDeclareOptions::default(), FieldTable::default());
        let mut consumer_notify = channel
        .basic_consume(
            "request-notification",
            "notify_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("consume notification request faild");

        let rt = Runtime::new().unwrap();
        let _ = rt.spawn(async move{
            log::info!("Will be consume");
            let token = env::var(TELEGRAM_BOT_TOKEN).expect("Not found token");
            let bot1 = Bot::new(token).auto_send();
            while let Some(delivery) = consumer_notify.next().await {
                let (_, delivery) = delivery.expect("error in consumer");
                delivery.ack(BasicAckOptions::default())
                    .await
                    .expect("Ack");

                let text = str::from_utf8(&delivery.data).unwrap();
                let request: RequestSignupDto = serde_json::from_str(text).unwrap();
                let mut con = connect().unwrap();
                let users: Vec<String> = con.lrange(REDIS_NOTIFY_CUSTOMERS, 0, -1).unwrap();
                let date = DateTime::parse_from_rfc3339(&request.date).unwrap();
                let date = date + Duration::hours(10);
                for user in users {
                    let _ = bot1.send_message(user, format!{"Новая запись в автосервис:\nдата: {}\nмашина: {}\nтелефон: {}\nчто делать: {}", date.format("%d.%m.%Y %H:%M:%S"), request.car, request.phone, request.description}).await;
                }
            }
        }).await;
}

async fn answer(cx: UpdateWithCx<AutoSend<Bot>, Message>, command: Command,) 
    -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Register => {
            let author = cx.chat_id();
            let mut con = connect().unwrap();
            let _: usize = con.lpush(REDIS_NOTIFY_CUSTOMERS, author).unwrap();
            cx.answer(format!("Your id {} was added to notification", author)).await?
        }
    };
    Ok(())
}

fn connect() -> Result<redis::Connection, RedisError> {
    let con_str = env::var(REDIS_CON_STRING).unwrap_or_else(|_| "redis://127.0.0.1/".into());
    let client = redis::Client::open(con_str)?;
    Ok(client.get_connection()?)
}

#[derive(Debug, Deserialize, Serialize)]
struct  RequestSignupDto {
    date: String,
    phone: String,
    car: String,
    description: String,
}