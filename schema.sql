CREATE TABLE Transfer (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	transfer_date INT NOT NULL,
	from_account INT NOT NULL,
	to_account INT NOT NULL,
	from_balance INT NOT NULL,
	to_balance INT NOT NULL,
	currency CHAR(3) NOT NULL,
	value INT NOT NULL,
	description VARCHAR(255)
);
