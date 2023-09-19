use crate::currency;
use crate::currency::{Currency, ALL_CURRENCY, CurrencyInfo};
use crate::operation::{TransferStatus, get_balance, send_transfer};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

async fn send_simple_message(response : String, ctx: &Context, msg: &Message) {
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


pub async fn transfer_command(ctx: &Context, msg: &Message) {
	let mentions_vector = &msg.mentions;
	let mut response : String = String::from("Not implemented.");

	if mentions_vector.len() == 0 {
		response = String::from("Please ping the individual you want to transfer.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}

	if mentions_vector.len() != 1 {
		response = String::from("You cannot transfer multiple times. Only a single transfer is allowed.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}

	if mentions_vector[0].bot == true {
		response = String::from("You cannot transfer to bots.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}

	let to_account = *mentions_vector[0].id.as_u64() as i64;
	let from_account = *msg.author.id.as_u64() as i64;
	if to_account == from_account {
		response = String::from("You cannot transfer to yourself.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}

	let split_iterator = msg.content.split(" ");
	let mut currency : Currency = Currency::KSN;
	let mut got_currency = false;

	let mut value : f64 = 0.0;
	let mut got_value = false;

	for word in split_iterator {
		if got_value == false {
			let try_parse = word.parse::<f64>();
			match try_parse {
				Ok(v) => {value = v; got_value = true;}
				Err(_) => got_value = false,
			}
		}

		if got_currency == false {
			for c in ALL_CURRENCY.iter() {
				let info = CurrencyInfo::new(&c);
				if word == info.code {
					currency = *c;
					got_currency = true;
				}
			}
		}

		if got_value == true  &&  got_currency == true {
			break;
		}
	}

	if got_currency == false {
		response = String::from("Currency has not been detected. Specify a currency.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}

	if got_value == false {
		response = String::from("A value has not been detected. Specify which value to transfer.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}


	// Treat value.
	let exp = CurrencyInfo::new(&currency).subunitexp as f64;
	let inter_value = f64::floor(value * f64::powf(10.0, -exp));
	let integer_value : i64 = inter_value as i64;


	// Check for positiveness.
	if integer_value <= 0 {
		response = String::from("Please insert a positive value.");
		send_simple_message(response, &ctx, &msg).await;
		return;
	}

	// Make the transfer!
	let transfer_status = send_transfer(from_account, to_account, &currency, integer_value).await;
	
	match transfer_status {
		TransferStatus::Authorized => response = String::from("Transfer authorized."),
		TransferStatus::InsuficientBalance => response = String::from("Insuficient balance for this transfer."),
		TransferStatus::BadValue => response = String::from("Inserted value is bad."),
		TransferStatus::Unauthorized => response = String::from("The transfer was not authorized, and blocked."),
	}
	
	send_simple_message(response, &ctx, &msg).await;
}
