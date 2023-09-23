use crate::currency::{Currency, CurrencyInfo, ALL_CURRENCIES};
use crate::operation::{force_transfer, get_balance, get_statement, send_transfer, TransferStatus};
use crate::stat;
use crate::stat::Tendency;

use serenity::model::channel::Message;
use serenity::prelude::*;

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
			send_simple_message("Please specify a currency.", &ctx, &msg).await;
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
		GINI: `{:.02}`
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
	let mut found = false;
	let mut response: String = String::from("");
	let image = "https://cdn.discordapp.com/attachments/1153482364907962509/1153482411871584267/currency_dollar_blue.png";

	for &currency in ALL_CURRENCIES {
		let balance = get_balance(author_id, currency).await?;

		if balance != 0 {
			let info = CurrencyInfo::from(currency);
			let balance = balance as f32 * f32::powf(10.0, info.subunitexp as f32);
			response = format!("`{} {:.02}`", &info.code, balance);
			found = true;
			break;
		}
	}

	if found == false {
		response = String::from("`KSN 0.00`");
	}

	msg.channel_id
		.send_message(&ctx.http, |m| {
			m.embed(|e| e.title("Balance").description(response).thumbnail(image))
		})
		.await
		.ok();

	Ok(())
}

pub async fn get_statement_command(ctx: &Context, msg: &Message) {
	let currency = msg
		.content
		.split_whitespace()
		.find_map(|word| Currency::try_from(word).ok());

	let currency = match currency {
		Some(currency) => currency,
		None => {
			send_simple_message("Please specify a currency.", &ctx, &msg).await;
			return;
		}
	};

	let info = CurrencyInfo::from(currency);
	let account = *msg.author.id.as_u64() as i64;
	let statement = get_statement(account, currency).await;
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

		response.push_str("\n");
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
}

pub async fn transfer_command(ctx: &Context, msg: &Message) {
	let mentions_vector = &msg.mentions;

	if mentions_vector.len() == 0 {
		send_simple_message(
			"Please ping the individual you want to transfer.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	if mentions_vector.len() != 1 {
		send_simple_message(
			"You cannot transfer multiple times. Only a single transfer is allowed.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	if mentions_vector[0].bot == true {
		send_simple_message("You cannot transfer to bots.", &ctx, &msg).await;
		return;
	}

	let to_account = *mentions_vector[0].id.as_u64() as i64;
	let from_account = *msg.author.id.as_u64() as i64;
	if to_account == from_account {
		send_simple_message("You cannot transfer to yourself.", &ctx, &msg).await;
		return;
	}

	let split_iterator = msg.content.split(" ");
	let mut currency: Currency = Currency::Ksn;
	let mut got_currency = false;

	let mut value: f64 = 0.0;
	let mut got_value = false;

	for word in split_iterator {
		if got_value == false {
			let try_parse = word.parse::<f64>();
			match try_parse {
				Ok(v) => {
					value = v;
					got_value = true;
				}
				Err(_) => got_value = false,
			}
		}

		if got_currency == false {
			if let Ok(c) = Currency::try_from(word) {
				got_currency = true;
				currency = c;
			}
		}

		if got_value == true && got_currency == true {
			break;
		}
	}

	if got_currency == false {
		send_simple_message(
			"Currency has not been detected. Specify a currency.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	if got_value == false {
		send_simple_message(
			"A value has not been detected. Specify which value to transfer.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	// Treat value.
	let exp = CurrencyInfo::from(currency).subunitexp as f64;
	let inter_value = f64::floor(value * f64::powf(10.0, -exp));
	let integer_value: i64 = inter_value as i64;

	// Check for positiveness.
	if integer_value <= 0 {
		send_simple_message("Please insert a positive value.", &ctx, &msg).await;
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

	send_simple_message(status_response, &ctx, &msg).await;
}

pub async fn create_deposit_command(ctx: &Context, msg: &Message) {
	let mentions_vector = &msg.mentions;

	if mentions_vector.len() >= 2 {
		send_simple_message(
			"You cannot transfer multiple times. Only a single transfer is allowed.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	// Find target.
	let target_id: i64 = if mentions_vector.len() == 0 {
		*msg.author.id.as_u64() as i64
	} else {
		if mentions_vector[0].bot == true {
			send_simple_message("You cannot transfer to bots.", &ctx, &msg).await;
			return;
		}

		*mentions_vector[0].id.as_u64() as i64
	};

	// Find currency and value.
	let split_iterator = msg.content.split(" ");
	let mut currency: Currency = Currency::Ksn;
	let mut got_currency = false;

	let mut value: f64 = 0.0;
	let mut got_value = false;

	for word in split_iterator {
		if got_value == false {
			let try_parse = word.parse::<f64>();
			match try_parse {
				Ok(v) => {
					value = v;
					got_value = true;
				}
				Err(_) => got_value = false,
			}
		}

		if got_currency == false {
			if let Ok(c) = Currency::try_from(word) {
				got_currency = true;
				currency = c;
			}
		}

		if got_value == true && got_currency == true {
			break;
		}
	}

	if got_currency == false {
		send_simple_message(
			"Currency has not been detected. Specify a currency.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	if got_value == false {
		send_simple_message(
			"A value has not been detected. Specify which value to transfer.",
			&ctx,
			&msg,
		)
		.await;
		return;
	}

	// Treat value.
	let exp = CurrencyInfo::from(currency).subunitexp as f64;
	let inter_value = f64::floor(value * f64::powf(10.0, -exp));
	let integer_value: i64 = inter_value as i64;

	// Check if it is zero.
	if integer_value == 0 {
		send_simple_message("Please select a non-zero value.", &ctx, &msg).await;
		return;
	}

	// Create deposit.
	match force_transfer(0, target_id, currency, integer_value).await {
		Ok(_) => send_simple_message("**Central:** Operation Authorized.", &ctx, &msg).await,
		Err(_) => send_simple_message("**Central:** Operation failed", &ctx, &msg).await,
	};
}
