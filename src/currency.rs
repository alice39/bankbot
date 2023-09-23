use std::borrow::Cow;

macro_rules! generate_currency {
	($enum_name:ident { $($elements:ident),* }, $array_name:ident) => {
		#[derive(Debug, Clone, Copy)]
		pub enum $enum_name {
			$($elements),*
		}

		pub const $array_name: &[$enum_name] = &[
			$($enum_name::$elements),*
		];
	};
}

generate_currency!(
	Currency {
		Ksn,
		Usd,
		Brl,
		Cad,
		Gbp,
		Eur,
		Bdt,
		Cop
	},
	ALL_CURRENCIES
);

pub struct CurrencyInfo<'a> {
	pub code: Cow<'a, str>,
	pub prefix: Cow<'a, str>,
	pub posfix: Cow<'a, str>,
	pub name: Cow<'a, str>,
	pub picture: Cow<'a, str>,
	pub subunitexp: i32,
}

impl TryFrom<&str> for Currency {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		if value.len() != 3 {
			anyhow::bail!("No matching currency");
		}

		match &value.to_uppercase()[0..3] {
			"KSN" => Ok(Self::Ksn),
			"USD" => Ok(Self::Usd),
			"BRL" => Ok(Self::Brl),
			"CAD" => Ok(Self::Cad),
			"GBP" => Ok(Self::Gbp),
			"EUR" => Ok(Self::Eur),
			"BDT" => Ok(Self::Bdt),
			"COP" => Ok(Self::Cop),
			_ => anyhow::bail!("No matching currency"),
		}
	}
}

impl<'a> From<Currency> for CurrencyInfo<'a> {
	fn from(currency: Currency) -> Self {
		match currency {
			Currency::Ksn => CurrencyInfo {
				code: "KSN".into(),
				prefix: "K$".into(),
				posfix: "nepers".into(),
				name: "Kid Server Neper".into(), 
				picture: "https://media.discordapp.net/attachments/1153482364907962509/1153858790341492766/twitchiconpng1-tanglesheep.png".into(),
				subunitexp: -2
			},
			Currency::Usd => CurrencyInfo {
				code: "USD".into(),
				prefix: "U$".into(),
				posfix: "dollars".into(),
				name: "American Dollar".into(),
				picture : "https://media.discordapp.net/attachments/1153482364907962509/1153858888895041599/555526.png".into(),
				subunitexp: -2
			},
			Currency::Brl => CurrencyInfo {
				code: "BRL".into(),
				prefix: "R$".into(),
				posfix: "reals".into(),
				name: "Brazilian Real".into(),
				picture : "https://media.discordapp.net/attachments/1153482364907962509/1153858933551808605/206597.png".into(),
				subunitexp: -2
			},
			Currency::Cad => CurrencyInfo {
				code: "CAD".into(),
				prefix: "C$".into(),
				posfix: "dollars".into(),
				name: "Canadian Dollar".into(),
				picture : "https://cdn.discordapp.com/attachments/1153482364907962509/1153861273793081414/555473.png".into(),
				subunitexp: -2
			},
			Currency::Gbp => CurrencyInfo {
				code: "GBP".into(),
				prefix: "£".into(),
				posfix: "pounds".into(),
				name: "British Pound".into(),
				picture : "https://media.discordapp.net/attachments/1153482364907962509/1153861461182001212/555417.png".into(),
				subunitexp: -2
			},
			Currency::Eur => CurrencyInfo {
				code: "EUR".into(),
				prefix: "€".into(),
				posfix: "euros".into(),
				name: "European Euro".into(),
				picture : "https://media.discordapp.net/attachments/1153482364907962509/1153861704141262858/330426.png".into(),
				subunitexp: -2
			},
			Currency::Bdt => CurrencyInfo {
				code: "BDT".into(),
				prefix: "৳".into(),
				posfix: "takas".into(),
				name: "Bangladeshi Taka".into(),
				picture : "https://cdn.discordapp.com/attachments/1153482364907962509/1153862592121548800/5327225.png".into(),
				subunitexp: -2
			},
			Currency::Cop => CurrencyInfo {
				code: "COP".into(),
				prefix: "$".into(),
				posfix: "pesos".into(),
				name: "Colombian Peso".into(),
				picture : "https://media.discordapp.net/attachments/1153482364907962509/1154949125960372235/330508.png".into(),
				subunitexp: -2
			},
		}
	}
}
