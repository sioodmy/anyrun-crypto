use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct Coin {
    symbol: String,
    price: String,
}

#[derive(Deserialize)]
struct Config {
    prefix: String,
    coins: Vec<String>,
    max_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        let list = vec![
            "BTCUSDT", "ETHUSDT", "BNBUSDT", "XRPUSDT", "ADAUSDT", "XMRUSDT", "SOLUSDT",
        ];
        Self {
            prefix: "$".to_string(),
            coins: list.iter().map(|c| c.to_string()).collect(),
            max_entries: 10,
        }
    }
}

struct State {
    config: Config,
    coins: Vec<Coin>,
}

fn fetch_coins(coins: Vec<String>) -> Vec<Coin> {
    let mut vec = Vec::new();
    for coin in coins {
        match reqwest::blocking::get(format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}",
            coin
        )) {
            Ok(response) => match response.json::<Coin>() {
                Ok(data) => vec.push(data),
                Err(why) => println!("Error parsing currency json: {}", why),
            },
            Err(why) => println!("Error fetching up-to-date currency conversions: {}", why),
        }
    }
    vec
}
#[init]
fn init(config_dir: RString) -> State {
    let config = match fs::read_to_string(format!("{}/translate.ron", config_dir)) {
        Ok(content) => ron::from_str(&content).unwrap_or_default(),
        Err(_) => Config::default(),
    };
    let coins = config.coins.clone();

    State {
        config,
        coins: fetch_coins(coins),
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Crypto".into(),
        icon: "help-about".into(), // Icon from the icon theme
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    if !input.starts_with(&state.config.prefix) {
        return RVec::new();
    }
    let mut vec = RVec::new();
    let mut i: usize = 0;
    for coin in &state.coins {
        i += 1;
        let price = (coin.price.clone().parse::<f64>().unwrap() * 100.).round() / 100.;
        vec.push(Match {
            title: coin.symbol.clone().into(),
            icon: ROption::RNone,
            use_pango: false,
            description: ROption::RSome(format!("{:.2}$", price).into()),
            id: ROption::RNone,
        });
        if i >= state.config.max_entries {
            break;
        }
    }

    vec
}

#[handler]
fn handler(selection: Match) -> HandleResult {
    if let ROption::RSome(desc) = selection.description {
        HandleResult::Copy(desc.into_bytes())
    } else {
        HandleResult::Close
    }
}
