use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod currency;
use currency::{Currency, ALL_CURRENCY, CurrencyInfo};

mod operation;
use operation::get_balance;



struct Handler;

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		if msg.content == "!ping" {
			if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
				println!("Error sending message: {:?}", why);
			}
		}

		if msg.content.starts_with("!balance") {
			let author_id : i64 = *msg.author.id.as_u64() as i64;
			let mut found = false;
			for currency in ALL_CURRENCY.iter() {
				let balance = operation::get_balance(author_id, currency).await;
				if balance != 0 {
					let info = CurrencyInfo::new(currency);
					let balance = balance as f32 * f32::powf(10.0, info.subunitexp as f32);
					let formatted = format!("{:.02}", balance);
					let response = info.code + " " + &formatted;
					msg.channel_id.say(&ctx.http, response).await.unwrap();
					found = true;
					break;
				}
			}

			if found == false {
				msg.channel_id.say(&ctx.http, "KSN 0.00").await.unwrap();
			}
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	/*for (key, value) in env::vars() {
		println!("{key}: {value}");
	}*/

	// let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment.");
	let token = "OTYzMDk2MzE3NjM3Mjk2MjY5.YlRHVw.YkAx89wl14CfTmu906NCgIJMP6M";


	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}
