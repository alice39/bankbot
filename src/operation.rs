use crate::currency::{Currency, CurrencyInfo};
use sqlx::Connection;
use sqlx::Executor;
use sqlx::Row;
use sqlx::SqliteConnection;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::TryStreamExt;
// use futures_util::stream::try_stream::TryStreamExt;

#[derive(Debug)]
pub struct Transfer {
	pub currency: Currency,
	pub from_account: i64,
	pub to_account: i64,
	pub balance: i64,
	pub value: i64,
	pub date: i64,
}

fn get_current_time() -> i64 {
	let start = SystemTime::now();
	let since_the_epoch = start
		.duration_since(UNIX_EPOCH)
		.expect("Time went backwards");

	since_the_epoch.as_secs() as i64 * 1000 + since_the_epoch.subsec_nanos() as i64 / 1_000_000
}

pub async fn get_balance(account: i64, currency: Currency) -> anyhow::Result<i64> {
	let currency_info = CurrencyInfo::from(currency);

	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await?;
	let mut rows = sqlx::query("SELECT * FROM Transfer WHERE currency=? AND (from_account=? OR to_account=?) ORDER BY id DESC")
		.bind(currency_info.code)
		.bind(account)
		.bind(account)
		.fetch(&mut conn);

	let row = match rows.try_next().await? {
		Some(row) => row,
		None => return Ok(0),
	};

	let from_account: i64 = row.try_get("from_account")?;
	let to_account: i64 = row.try_get("to_account")?;

	Ok(if account == from_account {
		row.try_get("from_balance")?
	} else if account == to_account {
		row.try_get("to_balance")?
	} else {
		0
	})
}

pub async fn get_statement(account: i64, currency: Currency) -> Vec<Transfer> {
	let currency_info = CurrencyInfo::from(currency);
	let mut conn = SqliteConnection::connect("sqlite://bank_database.db")
		.await
		.unwrap();

	let rows = sqlx::query("SELECT * FROM Transfer WHERE currency=? AND (from_account=? OR to_account=?) ORDER BY id DESC")
		.bind(currency_info.code)
		.bind(account)
		.bind(account)
		.fetch_all(&mut conn);

	let mut result: Vec<Transfer> = vec![];
	if let Ok(rows) = rows.await {
		for row in rows.iter() {
			let date: i64 = row.try_get("transfer_date").unwrap();
			let value: i64 = row.try_get("value").unwrap();
			let from_account: i64 = row.try_get("from_account").unwrap();
			let to_account: i64 = row.try_get("to_account").unwrap();
			let balance: i64 = if from_account == account {
				row.try_get("from_balance").unwrap()
			} else {
				row.try_get("to_balance").unwrap()
			};

			result.push(Transfer {
				currency,
				from_account,
				to_account,
				balance,
				value,
				date,
			});
		}
	}

	result
}

#[derive(Debug)]
pub enum TransferStatus {
	Authorized,
	InsuficientBalance,
	BadValue,
	Failed,
}

pub async fn send_transfer(
	from_account: i64,
	to_account: i64,
	currency: Currency,
	value: i64,
) -> TransferStatus {
	let currency_info = CurrencyInfo::from(currency);
	let mut conn = match SqliteConnection::connect("sqlite://bank_database.db").await {
		Ok(conn) => conn,
		Err(_) => return TransferStatus::Failed,
	};

	let before_from_balance = match get_balance(from_account, currency).await {
		Ok(balance) => balance,
		Err(_) => return TransferStatus::Failed,
	};
	let before_to_balance = match get_balance(to_account, currency).await {
		Ok(balance) => balance,
		Err(_) => return TransferStatus::Failed,
	};

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

	match conn.execute(query).await {
		Ok(_) => TransferStatus::Authorized,
		Err(_) => TransferStatus::Failed,
	}
}

pub async fn force_transfer(
	from_account: i64,
	to_account: i64,
	currency: Currency,
	value: i64,
) -> anyhow::Result<()> {
	let currency_info = CurrencyInfo::from(currency);
	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await?;

	let before_from_balance = get_balance(from_account, currency).await?;
	let before_to_balance = get_balance(to_account, currency).await?;

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

	conn.execute(query).await?;

	Ok(())
}
