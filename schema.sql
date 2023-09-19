CREATE TABLE Transfer (
	id INTEGER PRIMARY KEY AUTOINCREMENT,
	currency CHAR(3) NOT NULL,
	from_account INT NOT NULL,
	to_account INT NOT NULL,
	from_balance INT NOT NULL,
	to_balance INT NOT NULL,
	value INT NOT NULL,
	transfer_date INT NOT NULL,
	description VARCHAR(255)
);
