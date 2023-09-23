use dotenv::dotenv;
use std::env;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

mod commands;
mod currency;
mod operation;

struct Handler {
	director_id: u64,
}

#[async_trait]
impl EventHandler for Handler {
	async fn message(&self, ctx: Context, msg: Message) {
		if msg.content == "!ping" {
			if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
				println!("Error sending message: {:?}", why);
			}
		}

		if msg.content.starts_with("!balance") {
			commands::get_balance_command(&ctx, &msg).await;
		}

		if msg.content.starts_with("!transfer") {
			commands::transfer_command(&ctx, &msg).await;
		}

		if msg.content.starts_with("!statement") {
			commands::get_statement_command(&ctx, &msg).await;
		}

		if msg.content.starts_with("!create") {
			if *msg.author.id.as_u64() == self.director_id {
				commands::create_deposit_command(&ctx, &msg).await;
			}
		}
	}

	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();

	let token = env::var("BANK_DISCORD_TOKEN").expect("Expected a token in the environment.");
	let director_id = env::var("DIRECTOR_ID").expect("Expected an admin ID.");
	let director_id = director_id
		.parse::<u64>()
		.expect("Expected an integer admin ID.");

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;

	let mut client = Client::builder(&token, intents)
		.event_handler(Handler { director_id })
		.await
		.expect("Err creating client");

	if let Err(why) = client.start().await {
		println!("Client error: {:?}", why);
	}
}
