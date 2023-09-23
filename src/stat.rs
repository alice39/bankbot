use sqlx::Connection;
use sqlx::SqliteConnection;
use std::collections::HashSet;

use crate::currency::{Currency, CurrencyInfo};
use crate::operation::{Transfer, TransferRow, BANK_ID};

// All Balances. Average. Median. GINI.

pub async fn get_money_supply(currency: Currency) -> anyhow::Result<i64> {
	let currency_info = CurrencyInfo::from(currency);

	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await?;
	let row = sqlx::query_as::<_, TransferRow>(
		r#"SELECT * FROM Transfer
		WHERE (from_account = 0 OR to_account = 0) AND currency=?
		ORDER BY id DESC
		LIMIT 1"#,
	)
	.bind(currency_info.code)
	.fetch_optional(&mut conn)
	.await?;

	Ok(match row {
		Some(row) => -Transfer::try_from((BANK_ID, row))?.balance,
		None => 0,
	})
}

pub async fn get_all_transfers(currency: Currency) -> anyhow::Result<i64> {
	let currency_info = CurrencyInfo::from(currency);
	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await?;

	Ok(sqlx::query_as::<_, (i64,)>(
		r#"SELECT SUM(value) FROM Transfer
		WHERE from_account != 0 AND currency=?"#,
	)
	.bind(currency_info.code)
	.fetch_one(&mut conn)
	.await?
	.0)
}

pub async fn get_all_balances(currency: Currency) -> anyhow::Result<Vec<i64>> {
	let currency_info = CurrencyInfo::from(currency);
	let mut conn = SqliteConnection::connect("sqlite://bank_database.db").await?;

	let rows = sqlx::query_as::<_, TransferRow>(
		r#"SELECT * FROM Transfer
		WHERE currency=?
		ORDER BY id DESC"#,
	)
	.bind(currency_info.code)
	.fetch_all(&mut conn)
	.await?;

	let mut processed = HashSet::new();

	let mut result = rows
		.into_iter()
		.fold(vec![], move |mut balances, transfer_row| {
			let from_account = transfer_row.from_account;
			let from_balance = transfer_row.from_balance;

			let to_account = transfer_row.to_account;
			let to_balance = transfer_row.to_balance;

			if from_account != BANK_ID && !processed.contains(&from_account) {
				balances.push(from_balance);
				processed.insert(from_account);
			}

			if to_account != BANK_ID && !processed.contains(&to_account) {
				balances.push(to_balance);
				processed.insert(to_account);
			}

			balances
		});

	result.sort();
	Ok(result)
}

pub struct Tendency {
	pub average: f64,
	pub median: f64,
	pub gini: f64,
}

pub fn calc_balances(balances: &Vec<i64>) -> Tendency {
	// Empty balances case.
	let size = balances.len();
	if size == 0 {
		return Tendency {
			average: 0.0,
			median: 0.0,
			gini: 0.0,
		};
	}

	// Calculate median. 0 1 2 3 4
	let median: f64 = if size % 2 == 0 {
		(balances[size / 2] as f64 + balances[size / 2 - 1] as f64) / 2.0
	} else {
		balances[size / 2] as f64
	};

	let mut average: f64 = 0.0;
	for balance in balances.iter() {
		average += (*balance as f64) / (balances.len() as f64);
	}

	let mut diff_sum = 0.0;
	for i in 0..balances.len() {
		for j in 0..balances.len() {
			let xi = balances[i] as f64;
			let xj = balances[j] as f64;
			diff_sum += f64::abs(xi - xj);
		}
	}

	let nsq = (size * size) as f64;
	let den: f64 = 2.0 * nsq * average;
	let gini: f64 = if den >= 1e-6 { diff_sum / den } else { 0.0 };

	Tendency {
		average,
		median,
		gini,
	}
}
