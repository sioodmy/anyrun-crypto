use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Coin {
    symbol: String,
    price: String,
}

#[derive(Deserialize, Clone)]
struct Config {
    prefix: String,
    coins: Vec<String>,
    max_cache_age: u64,
}

impl Default for Config {
    fn default() -> Self {
        let list = vec![
            "BTCUSDT", "ETHUSDT", "BNBUSDT", "XRPUSDT", "ADAUSDT", "XMRUSDT", "SOLUSDT",
        ];
        Self {
            prefix: "$".to_string(),
            coins: list.iter().map(|c| c.to_string()).collect(),
            max_cache_age: 900,
        }
    }
}

struct State {
    config: Config,
    coins: Vec<Coin>,
}

#[derive(Serialize, Deserialize)]
struct Cache {
    age: u64,
    coins: Vec<Coin>,
}

/// Fetches coins and handles cache
fn fetch_coins(coins: Vec<String>, max_age: u64) -> Vec<Coin> {
    let mut vec = Vec::new();

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let cache_path = format!("{}/cryptorun.json", env!("XDG_CACHE_HOME"));
    let cache = match fs::read_to_string(&cache_path) {
        Ok(content) => {
            Some(serde_json::from_str::<Cache>(&content).expect("Failed to parse cache file"))
        }
        Err(_) => None,
    };

    if let Some(cache) = cache {
        if time - cache.age < max_age {
            return cache.coins;
        }
    }

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

    let new_cache = Cache {
        coins: vec.clone(),
        age: time,
    };
    let mut cache_file = File::create(&cache_path).expect("Error while writing cache file");
    cache_file
        .write_all(serde_json::to_string(&new_cache).unwrap().as_bytes())
        .unwrap();

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
        config: config.clone(),
        coins: fetch_coins(coins, config.max_cache_age),
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
    for coin in &state.coins {
        let price = (coin.price.clone().parse::<f64>().unwrap() * 100.).round() / 100.;
        vec.push(Match {
            title: coin.symbol.clone().into(),
            icon: ROption::RNone,
            use_pango: false,
            description: ROption::RSome(format!("{:.2}$", price).into()),
            id: ROption::RNone,
        });
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
