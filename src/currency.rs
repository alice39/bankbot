pub enum Currency {
	KSN,
	USD,
	BRL,
}

pub static ALL_CURRENCY : [Currency; 3] = [
	Currency::KSN,
	Currency::USD,
	Currency::BRL,
];

pub struct CurrencyInfo {
	pub code : String,
	pub prefix : String,
	pub posfix: String,
	pub name : String,
	pub subunitexp : i32,
}

impl CurrencyInfo {
	pub fn new(currency : &Currency) -> Self {
		match currency {
			Currency::KSN => CurrencyInfo {
				code: String::from("KSN"),
				prefix: String::from("K$"),
				posfix: String::from("nepers"),
				name: String::from("Kid Server Neper"), 
				subunitexp: -2
			},
		
			Currency::USD => CurrencyInfo {
				code: String::from("USD"),
				prefix: String::from("U$"),
				posfix: String::from("dollar"),
				name: String::from("United State Dollar"),
				subunitexp: -2
			},

			Currency::BRL => CurrencyInfo {
				code: String::from("BRL"),
				prefix: String::from("R$"),
				posfix: String::from("reals"),
				name: String::from("Brazilian Real"),
				subunitexp: -2
			},
		}
	}
}
