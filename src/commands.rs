use crate::currency::{Currency, ALL_CURRENCY, CurrencyInfo};
use crate::operation::get_balance;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

async fn send_bare_message(response : String, ctx: &Context) {
	msg.channel_id.send_message(&ctx.http, |m|
	{
		m.embed(|e| {
			e.description(response)
		})
	}).await;
}

pub async fn get_balance_command(ctx: &Context, msg: &Message) {
	let author_id : i64 = *msg.author.id.as_u64() as i64;
	let mut found = false;
	let mut response : String = String::from("");
	let image = "https://cdn.discordapp.com/attachments/1153482364907962509/1153482411871584267/currency_dollar_blue.png";
	for currency in ALL_CURRENCY.iter() {
		let balance = get_balance(author_id, currency).await;
		if balance != 0 {
			let info = CurrencyInfo::new(currency);
			let balance = balance as f32 * f32::powf(10.0, info.subunitexp as f32);
			response = format!("`{} {:.02}`", &info.code, balance);
			found = true;
			break;
		}
	}

	if found == false {
		response = String::from("`KSN 0.00`");
	}

	msg.channel_id.send_message(&ctx.http, |m|
	{
		m.embed(|e| {
			e.title("Balance")
				.description(response)
				.thumbnail(image)
		})
	}).await;
}
