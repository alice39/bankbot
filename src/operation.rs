use crate::Currency;
use crate::currency::CurrencyInfo;
use sqlx::Connection;
use sqlx::SqliteConnection;
use sqlx::Executor;
use sqlx::Row;
use std::time::{SystemTime, UNIX_EPOCH};


use futures::TryStreamExt;
// use futures_util::stream::try_stream::TryStreamExt;

pub struct Transfer {
	pub date : i64,
	pub from_account : i64,
	pub to_account : i64,	
	pub currency : Currency,
	pub value : i64,
}

fn get_current_time() -> i64 {
	let start = SystemTime::now();
	let since_the_epoch = start
		.duration_since(UNIX_EPOCH)
		.expect("Time went backwards");

	let timestamp = since_the_epoch.as_secs() as i64* 1000 +
		since_the_epoch.subsec_nanos() as i64 / 1_000_000;

	return timestamp;
}

pub async fn get_balance(account: i64, currency: &Currency) -> i64 {
	let currency_info = CurrencyInfo::new(&currency);

	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await.unwrap();
	let mut rows = sqlx::query(
		"SELECT * FROM Transfer WHERE currency=? AND (from_account=? OR to_account=?) ORDER BY id DESC"
	)
		.bind(currency_info.code)
		.bind(account)
		.bind(account)
		.fetch(&mut conn);

	while let Some(row) = rows.try_next().await.unwrap() {
		let from_account : i64 = row.try_get("from_account").unwrap();
		let to_account : i64 = row.try_get("to_account").unwrap();

		if account == from_account {
			let balance : i64 = row.try_get("from_balance").unwrap();
			return balance;
		}

		else if account == to_account {
			let balance : i64 = row.try_get("to_balance").unwrap();
			return balance;
		}

		else {
			let balance = 0;
			return balance;
		}
	}

	let balance = 0;
	return balance;
}

fn get_statement(from_account: i64, currency: Currency, entries: i64) {

}

#[derive(Debug)]
pub enum TransferStatus {
	Authorized, InsuficientBalance, BadValue, Unauthorized,
}

pub async fn send_transfer(from_account: i64, to_account: i64, currency: &Currency, value: i64) -> TransferStatus {
	let currency_info = CurrencyInfo::new(&currency);
	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await.unwrap();

	let before_from_balance = get_balance(from_account, &currency).await;
	let before_to_balance = get_balance(to_account, &currency).await;

	if value <= 0 {
		return TransferStatus::BadValue;
	}

	if value > before_from_balance && from_account != 0 {
		return TransferStatus::InsuficientBalance;
	}

	let after_from_balance = before_from_balance - value;
	let after_to_balance = before_to_balance + value;
	let timestamp_now = get_current_time();

	let query = sqlx::query("INSERT INTO Transfer (transfer_date, from_account, to_account, from_balance, to_balance, currency, value) VALUES (?, ?, ?, ?, ?, ?, ?)")
		.bind(timestamp_now)
		.bind(from_account)
		.bind(to_account)
		.bind(after_from_balance)
		.bind(after_to_balance)
		.bind(currency_info.code)
		.bind(value);

	conn.execute(query).await.unwrap();
	return TransferStatus::Authorized;
}
