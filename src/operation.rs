use sqlx::{Connection, Executor, SqliteConnection};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::currency::{Currency, CurrencyInfo};

#[derive(Debug)]
pub struct Transfer {
	pub currency: Currency,
	pub from_account: i64,
	pub to_account: i64,
	pub balance: i64,
	pub value: i64,
	pub date: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct LedgerRow {
	pub id: u32,
	pub currency: String,
	pub from_account: UserId,
	pub to_account: UserId,
	pub from_balance: i64,
	pub to_balance: i64,
	pub value: i64,
	pub transfer_date: i64,
	pub description: String,
}

pub type UserId = i64;

pub const BANK_ID: UserId = 0;

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
	let ledger_row = sqlx::query_as::<_, LedgerRow>(
		r#"SELECT * FROM Transfer
		WHERE currency=? AND (from_account=? OR to_account=?)
		ORDER BY id DESC"#,
	)
	.bind(currency_info.code)
	.bind(account)
	.bind(account)
	.fetch_optional(&mut conn)
	.await?;

	Ok(match ledger_row {
		Some(row) => Transfer::try_from((account, row))?.balance,
		None => 0,
	})
}

pub async fn get_statement(account: i64, currency: Currency) -> anyhow::Result<Vec<Transfer>> {
	let currency_info = CurrencyInfo::from(currency);

	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await?;
	let ledger_rows = sqlx::query_as::<_, LedgerRow>(
		r#"SELECT * FROM Transfer
		WHERE currency=? AND (from_account=? OR to_account=?)
		ORDER BY id DESC"#,
	)
	.bind(currency_info.code)
	.bind(account)
	.bind(account)
	.fetch_all(&mut conn)
	.await?;

	Ok(ledger_rows
		.into_iter()
		.filter_map(|row| Transfer::try_from((account, row)).ok())
		.collect())
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

	let query = sqlx::query(
		r#"INSERT INTO Transfer
		(transfer_date, from_account, to_account, from_balance, to_balance, currency, value)
		VALUES (?, ?, ?, ?, ?, ?, ?)"#,
	)
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

	let query = sqlx::query(
		r#"INSERT INTO Transfer
		(transfer_date, from_account, to_account, from_balance, to_balance, currency, value)
		VALUES (?, ?, ?, ?, ?, ?, ?)"#,
	)
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

impl TryFrom<(UserId, LedgerRow)> for Transfer {
	type Error = anyhow::Error;

	fn try_from((id, row): (UserId, LedgerRow)) -> Result<Self, Self::Error> {
		Ok(Self {
			currency: Currency::try_from(row.currency.as_str())?,
			from_account: row.from_account,
			to_account: row.to_account,
			balance: if row.from_account == id {
				row.from_balance
			} else if row.to_account == id {
				row.to_balance
			} else {
				0
			},
			value: row.value,
			date: row.transfer_date,
		})
	}
}

impl TryFrom<(LedgerRow, UserId)> for Transfer {
	type Error = anyhow::Error;

	fn try_from((row, id): (LedgerRow, UserId)) -> Result<Self, Self::Error> {
		(id, row).try_into()
	}
}
