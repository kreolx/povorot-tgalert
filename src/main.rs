use teloxide::{prelude::*, utils::command::BotCommand};
use std::{env};
use std::error::Error;
use redis::{Commands,RedisError};


const REDIS_CON_STRING: &str = "REDIS_CON_STRING";
const TELEGRAM_BOT_TOKEN: &str = "TELEGRAM_BOT_TOKEN";

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "Command")]
enum Command {
    #[command(description = "add to alert notification")]
    Register,
}

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    log::info!("Starting bot");
    let token = env::var(TELEGRAM_BOT_TOKEN).expect("Not found token");
    let bot = Bot::new(token).auto_send();
    let bot_name = "povorot_alert".to_string();
    teloxide::commands_repl(bot, bot_name, answer).await;
}

async fn answer(cx: UpdateWithCx<AutoSend<Bot>, Message>, command: Command,) 
    -> Result<(), Box<dyn Error + Send + Sync>> {
    match command {
        Command::Register => {
            let author = cx.chat_id();
            let mut con = connect().unwrap();
            let _: usize = con.lpush("notification", author).unwrap();
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