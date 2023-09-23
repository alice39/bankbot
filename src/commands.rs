use serenity::model::channel::Message;
use serenity::prelude::*;
use std::borrow::Cow;

use crate::currency::{Currency, CurrencyInfo, ALL_CURRENCIES};
use crate::operation::{force_transfer, get_balance, get_statement, send_transfer, TransferStatus};
use crate::stat;

async fn send_simple_message(response: &str, ctx: &Context, msg: &Message) {
	msg.channel_id
		.send_message(&ctx.http, |m| m.embed(|e| e.description(response)))
		.await
		.ok();
}

pub async fn get_stat_command(ctx: &Context, msg: &Message) {
	let currency = msg
		.content
		.split_whitespace()
		.find_map(|word| Currency::try_from(word).ok());

	let currency = match currency {
		Some(currency) => currency,
		None => {
			send_simple_message("Please specify a currency.", ctx, msg).await;
			return;
		}
	};

	let supply = stat::get_money_supply(currency).await.unwrap() as f64;
	let transfers = stat::get_all_transfers(currency).await.unwrap() as f64;
	let tendency = stat::calc_balances(&stat::get_all_balances(currency).await.unwrap());
	let info = CurrencyInfo::from(currency);
	let factor = f64::powf(10.0, info.subunitexp as f64);

	let response = format!(
		"Money Supply: `{} {:.02}`
		GDP: `{} {:.02}`
		GINI: `{:.05}`
		Median: `{} {:.02}`
		Average: `{} {:.02}`",
		info.code,
		supply * factor,
		info.code,
		transfers * factor,
		tendency.gini,
		info.code,
		tendency.median * factor,
		info.code,
		tendency.average * factor
	);

	send_simple_message(&response, ctx, msg).await;
}

pub async fn get_balance_command(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
	let author_id: i64 = *msg.author.id.as_u64() as i64;
	let image = "https://cdn.discordapp.com/attachments/1153482364907962509/1153482411871584267/currency_dollar_blue.png";

	let (currencies, inject_all) = match msg.content.find(' ') {
		Some(argument_index) => (
			msg.content[argument_index..]
				.split_whitespace()
				.map(Currency::try_from)
				.filter(Result::is_ok)
				.map(Result::unwrap)
				.collect(),
			false,
		),
		None => (ALL_CURRENCIES.to_vec(), true),
	};

	let response: Cow<'_, str> = if currencies.is_empty() {
		"No currencies matched".into()
	} else {
		let mut result = String::new();

		for currency in currencies {
			let balance = get_balance(author_id, currency).await?;
			if inject_all && balance == 0 {
				continue;
			}

			let info = CurrencyInfo::from(currency);

			let real_balance = balance as f32 * f32::powi(10.0, info.subunitexp);
			result.push_str(&format!("`{} {:.02}`\n", info.code, real_balance));
		}

		if result.is_empty() {
			"`KSN 0.00`".into()
		} else {
			result.into()
		}
	};

	msg.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| e.title("Balance").description(response).thumbnail(image))
		})
		.await
		.ok();

	Ok(())
}

pub async fn get_statement_command(ctx: &Context, msg: &Message) -> anyhow::Result<()> {
	let currency = msg
		.content
		.split_whitespace()
		.find_map(|word| Currency::try_from(word).ok());

	let currency = match currency {
		Some(currency) => currency,
		None => {
			send_simple_message("Please specify a currency.", ctx, msg).await;
			return Ok(());
		}
	};

	let info = CurrencyInfo::from(currency);
	let account = *msg.author.id.as_u64() as i64;
	let statement = get_statement(account, currency).await?;
	let mut overall_balance: f32 = 0.00;

	let mut counter = 0;
	let mut response: String = String::from("");
	for transfer in statement {
		let balance: f32 = (transfer.balance as f32) * f32::powf(10.0, info.subunitexp as f32);
		let value: f32 = (transfer.value as f32) * f32::powf(10.0, info.subunitexp as f32);
		if counter == 0 {
			overall_balance = balance;
		}

		response.push_str(&format!("Date: `{}`\n", transfer.date));
		response.push_str(&format!(
			"Balance: `{} {:.2}`. Operation: `{} {:.2}`\n",
			info.code, balance, info.code, value
		));
		if transfer.from_account == account {
			response.push_str(&format!("Transfer sent to <@!{}>\n", transfer.to_account));
		} else if transfer.from_account != 0 {
			response.push_str(&format!(
				"Transfer received from <@!{}>\n",
				transfer.from_account
			));
		} else {
			response.push_str("Deposit received.\n");
		}

		response.push('\n');
		counter += 1;
		if counter > 10 {
			break;
		}
	}

	if counter == 0 {
		response = String::from("There are no transactions to report.");
	}

	msg.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| {
				e.title(format!(
					"{}: {} {:.2}",
					info.name, info.code, overall_balance
				))
				.description(response)
				.thumbnail(info.picture)
			})
		})
		.await
		.ok();

	Ok(())
}

pub async fn transfer_command(ctx: &Context, msg: &Message) {
	let mentions_vector = &msg.mentions;

	if mentions_vector.is_empty() {
		send_simple_message("Please ping the individual you want to transfer.", ctx, msg).await;
		return;
	}

	if mentions_vector.len() > 1 {
		send_simple_message(
			"You cannot transfer multiple times. Only a single transfer is allowed.",
			ctx,
			msg,
		)
		.await;
		return;
	}

	if mentions_vector[0].bot {
		send_simple_message("You cannot transfer to bots.", ctx, msg).await;
		return;
	}

	let to_account = *mentions_vector[0].id.as_u64() as i64;
	let from_account = *msg.author.id.as_u64() as i64;
	if to_account == from_account {
		send_simple_message("You cannot transfer to yourself.", ctx, msg).await;
		return;
	}

	let split_iterator = msg.content.split_whitespace();
	let mut currency: Option<Currency> = None;
	let mut value: Option<f64> = None;

	for word in split_iterator {
		if currency.is_none() {
			currency = Currency::try_from(word).ok();
		}
		if value.is_none() {
			value = word.parse::<f64>().ok();
		}

		if currency.is_some() && value.is_some() {
			break;
		}
	}

	let currency = match currency {
		Some(currency) => currency,
		None => {
			send_simple_message(
				"Currency has not been detected. Specify a currency.",
				ctx,
				msg,
			)
			.await;
			return;
		}
	};

	let value = match value {
		Some(value) => value,
		None => {
			send_simple_message(
				"A value has not been detected. Specify which value to transfer.",
				ctx,
				msg,
			)
			.await;
			return;
		}
	};

	// Treat value.
	let exp = CurrencyInfo::from(currency).subunitexp as f64;
	let inter_value = f64::floor(value * f64::powf(10.0, -exp));
	let integer_value: i64 = inter_value as i64;

	// Check for positiveness.
	if integer_value <= 0 {
		send_simple_message("Please insert a positive value.", ctx, msg).await;
		return;
	}

	// Make the transfer!
	let transfer_status = send_transfer(from_account, to_account, currency, integer_value).await;

	let status_response = match transfer_status {
		TransferStatus::Authorized => "Transfer authorized.",
		TransferStatus::InsuficientBalance => "Insuficient balance for this transfer.",
		TransferStatus::BadValue => "Inserted value is bad.",
		TransferStatus::Failed => "The transfer was not authorized, and blocked.",
	};

	send_simple_message(status_response, ctx, msg).await;
}

pub async fn create_deposit_command(ctx: &Context, msg: &Message) {
	let mentions_vector = &msg.mentions;

	if mentions_vector.len() >= 2 {
		send_simple_message(
			"You cannot transfer multiple times. Only a single transfer is allowed.",
			ctx,
			msg,
		)
		.await;
		return;
	}

	// Find target.
	let target_id: i64 = if mentions_vector.is_empty() {
		*msg.author.id.as_u64() as i64
	} else {
		if mentions_vector[0].bot {
			send_simple_message("You cannot transfer to bots.", ctx, msg).await;
			return;
		}

		*mentions_vector[0].id.as_u64() as i64
	};

	// Find currency and value.
	let split_iterator = msg.content.split_whitespace();

	let mut currency: Option<Currency> = None;
	let mut value: Option<f64> = None;

	for word in split_iterator {
		if currency.is_none() {
			currency = Currency::try_from(word).ok();
		}
		if value.is_none() {
			value = word.parse().ok();
		}

		if value.is_some() && currency.is_some() {
			break;
		}
	}

	let currency = match currency {
		Some(currency) => currency,
		None => {
			send_simple_message(
				"Currency has not been detected. Specify a currency.",
				ctx,
				msg,
			)
			.await;
			return;
		}
	};

	let value = match value {
		Some(value) => value,
		None => {
			send_simple_message(
				"A value has not been detected. Specify which value to transfer.",
				ctx,
				msg,
			)
			.await;
			return;
		}
	};

	// Treat value.
	let exp = CurrencyInfo::from(currency).subunitexp as f64;
	let inter_value = f64::floor(value * f64::powf(10.0, -exp));
	let integer_value: i64 = inter_value as i64;

	// Check if it is zero.
	if integer_value == 0 {
		send_simple_message("Please select a non-zero value.", ctx, msg).await;
		return;
	}

	// Create deposit.
	match force_transfer(0, target_id, currency, integer_value).await {
		Ok(_) => send_simple_message("**Central:** Operation Authorized.", ctx, msg).await,
		Err(_) => send_simple_message("**Central:** Operation failed", ctx, msg).await,
	};
}
