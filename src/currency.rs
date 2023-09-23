#[derive(Copy, Clone, Debug)]
pub enum Currency {
	KSN,
	USD,
	BRL,
	CAD,
	GBP,
	EUR,
	BDT,
	COP,
}

pub static ALL_CURRENCY : [Currency; 8] = [
	Currency::KSN,
	Currency::USD,
	Currency::BRL,
	Currency::CAD,
	Currency::GBP,
	Currency::EUR,
	Currency::BDT,
	Currency::COP,
];

pub struct CurrencyInfo {
	pub code : String,
	pub prefix : String,
	pub posfix: String,
	pub name : String,
	pub picture : String,
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
				picture: String::from("https://media.discordapp.net/attachments/1153482364907962509/1153858790341492766/twitchiconpng1-tanglesheep.png"),
				subunitexp: -2
			},
		
			Currency::USD => CurrencyInfo {
				code: String::from("USD"),
				prefix: String::from("U$"),
				posfix: String::from("dollars"),
				name: String::from("American Dollar"),
				picture : String::from("https://media.discordapp.net/attachments/1153482364907962509/1153858888895041599/555526.png"),
				subunitexp: -2
			},

			Currency::BRL => CurrencyInfo {
				code: String::from("BRL"),
				prefix: String::from("R$"),
				posfix: String::from("reals"),
				name: String::from("Brazilian Real"),
				picture : String::from("https://media.discordapp.net/attachments/1153482364907962509/1153858933551808605/206597.png"),
				subunitexp: -2
			},

			Currency::CAD => CurrencyInfo {
				code: String::from("CAD"),
				prefix: String::from("C$"),
				posfix: String::from("dollars"),
				name: String::from("Canadian Dollar"),
				picture : String::from("https://cdn.discordapp.com/attachments/1153482364907962509/1153861273793081414/555473.png"),
				subunitexp: -2
			},

			Currency::GBP => CurrencyInfo {
				code: String::from("GBP"),
				prefix: String::from("£"),
				posfix: String::from("pounds"),
				name: String::from("British Pound"),
				picture : String::from("https://media.discordapp.net/attachments/1153482364907962509/1153861461182001212/555417.png"),
				subunitexp: -2
			},

			Currency::EUR => CurrencyInfo {
				code: String::from("EUR"),
				prefix: String::from("€"),
				posfix: String::from("euros"),
				name: String::from("European Euro"),
				picture : String::from("https://media.discordapp.net/attachments/1153482364907962509/1153861704141262858/330426.png"),
				subunitexp: -2
			},

			Currency::BDT => CurrencyInfo {
				code: String::from("BDT"),
				prefix: String::from("৳"),
				posfix: String::from("takas"),
				name: String::from("Bangladeshi Taka"),
				picture : String::from("https://cdn.discordapp.com/attachments/1153482364907962509/1153862592121548800/5327225.png"),
				subunitexp: -2
			},

			Currency::COP => CurrencyInfo {
				code: String::from("COP"),
				prefix: String::from("$"),
				posfix: String::from("pesos"),
				name: String::from("Colombian Peso"),
				picture : String::from("https://media.discordapp.net/attachments/1153482364907962509/1154949125960372235/330508.png"),
				subunitexp: -2
			},
		}
	}
}
