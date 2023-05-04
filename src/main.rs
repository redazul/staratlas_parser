use reqwest::header::CONTENT_TYPE;
use serde_json::{json, Value};
use regex::Regex;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Write};
use std::fs::File;
use serde::{Deserialize, Serialize};
use clap::{App, Arg};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Ship {
    ship: String,
    block_spawn: String,
    positions: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FeePayer {
    fee_payer: String,
    ships: Vec<Ship>,
}

async fn get_block(slot: u64,rpc: String) -> Result<String, Box<dyn std::error::Error>> {

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBlock",
        "params": [
            slot,
            {
                "encoding": "json",
                "maxSupportedTransactionVersion": 0,
                "transactionDetails": "full",
                "rewards": false
            }
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(rpc)
        .header(CONTENT_TYPE, "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let response_text = response.text().await?;
    //let json_response: Value = serde_json::from_str(&response_text)?;

    Ok(response_text)
    //Ok(json_response)
}

async fn get_block_time(slot: u64,rpc: String) -> Result<String, Box<dyn std::error::Error>> {

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBlockTime",
        "params": [
            slot
        ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(rpc)
        .header(CONTENT_TYPE, "application/json")
        .body(payload.to_string())
        .send()
        .await?;

    let response_text = response.text().await?;
    //let json_response: Value = serde_json::from_str(&response_text)?;

    Ok(response_text)
    //Ok(json_response)
}

fn find_transactions_with_account_key(json_response: &Value, account_key: &str) -> Vec<Value> {
    let transactions = json_response["result"]["transactions"].as_array().unwrap();

    let mut matching_transactions = Vec::new();

    for transaction in transactions {
        let account_keys = transaction["transaction"]["message"]["accountKeys"].as_array().unwrap();
        if account_keys.iter().any(|key| key == account_key) {
            matching_transactions.push(transaction.clone());
        }
    }

    matching_transactions
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    let matches = App::new("client")
        .version("0.1.0")
        .author("https://twitter.com/azuldev5")
        .about("StarAtlas Escape Velocity Parser")
        .arg(
            Arg::with_name("slot")
                .short('s')
                .long("slot")
                .value_name("SLOT")
                .help("Sets the slot number")
                .takes_value(true)
                .required(true), // This line makes the 'slot' argument required
        )
        .arg(
            Arg::with_name("rpc")
                .short('r')
                .long("rpc")
                .value_name("RPC")
                .help("Sets the RPC URL")
                .takes_value(true)
                .default_value("https://api.mainnet-beta.solana.com"),
        )
        .get_matches();

    let slot = matches
        .value_of("slot")
        .unwrap()
        .parse::<u64>()
        .expect("Failed to parse slot number");

    let rpc = matches.value_of("rpc").unwrap().to_string();

    // Attempt to read the existing JSON file or create a new empty file if it doesn't exist.
    let file = match OpenOptions::new().read(true).write(true).create(true).open("data.json") {
        Ok(f) => f,
        Err(e) => panic!("Error opening or creating file: {}", e),
    };

    let mut contents = String::new();
    let mut reader = BufReader::new(file);
    reader.read_to_string(&mut contents).expect("Error reading file.");

    // If the file is empty, initialize it with an empty array.
    if contents.is_empty() {
        contents = "[]".to_string();
    }
    
    let mut fee_payers: Vec<FeePayer> = serde_json::from_str(&contents).unwrap_or_default();

    //println!("Fee Payers {:?}", fee_payers);

    let str_response = get_block(slot,rpc.clone()).await?;
    ////println!("{:#?}", str_response);

    //touse later
    let _block_time = get_block_time(slot,rpc).await?;
    //println!("{:#?}", block_time);

    let json_response: Value = serde_json::from_str(&str_response).unwrap();

    let account_key = "TESTWCwvEv2idx6eZVQrFFdvEJqGHfVA1soApk2NFKQ";
    let matching_transactions = find_transactions_with_account_key(&json_response, account_key);

    //println!("Matching transactions:");
    for transaction in matching_transactions {
        ////println!("{}", serde_json::to_string_pretty(&transaction).unwrap());
        
        // Extract accountKeys
        let account_keys = transaction["transaction"]["message"]["accountKeys"].as_array().unwrap();
        ////println!("Account Keys: {:?}", account_keys);
        //println!("Account Keys FeePayer: {:?}", account_keys[0]);
        //println!("Account Keys Ship: {:?}", account_keys[1]);

        // Extract Program log
        let log_messages = transaction["meta"]["logMessages"].as_array().unwrap();
        ////println!("log_messages: {:?}", log_messages); 
        ////println!("log_messages: {:?}", log_messages[2]);

        let to_regex = Regex::new(r"to: \[(-?\d+),\s+(-?\d+)\]").unwrap();

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut values_updated = false;
        
        if let Some(log_message) = log_messages[2].as_str() {
            if let Some(captures) = to_regex.captures(log_message) {
                x = captures[1].parse().unwrap();
                y = captures[2].parse().unwrap();
        
                //println!("to: [{}, {}]", x, y);
                values_updated = true;
            }
        }
        
        if values_updated {
            let position = Position { x: x, y: y };
        
            // The rest of the code that checks for the FeePayer and Ship, and writes to the file.
            // Extracted information from the previous Rust code.
            let fee_payer_key =  &account_keys[0];
            let ship_key = &account_keys[1];

            // Check if the FeePayer exists, and if not, add it.
            let fee_payer = fee_payers.iter_mut().find(|fp| fp.fee_payer == fee_payer_key.as_str().unwrap().to_string());
            let fee_payer = match fee_payer {
                Some(fp) => fp,
                None => {
                    let new_fee_payer = FeePayer {
                        fee_payer: fee_payer_key.to_string().trim_matches('\"').to_string(),
                        ships: Vec::new(),
                    };
                    fee_payers.push(new_fee_payer);
                    fee_payers.last_mut().unwrap()
                }
            };

            //println!("Fee Payer Object  {:?}", fee_payer);

            // Check if the Ship exists, and if not, add it.
            let ship = fee_payer.ships.iter_mut().find(|s| s.ship == ship_key.as_str().unwrap().to_string());
            let ship = match ship {
                Some(s) => s,
                None => {
                    let new_ship = Ship {
                        ship: ship_key.to_string().trim_matches('\"').to_string(),
                        block_spawn: "165151651".to_string(),
                        positions: Vec::new(),
                    };
                    fee_payer.ships.push(new_ship);
                    fee_payer.ships.last_mut().unwrap()
                }
            };

            // Check if the last Position is not the same as the extracted one and append it if needed.
            let last_position = ship.positions.last();
            if last_position.is_none() || *last_position.unwrap() != position {
                ship.positions.push(position);
            }

            // Write the updated data structure back to the JSON file.
            let updated_contents = serde_json::to_string_pretty(&fee_payers).expect("Error serializing data.");
            let mut file = File::create("data.json").expect("Error creating file.");
            file.write_all(updated_contents.as_bytes()).expect("Error writing file.");
        }



    }

    println!("Processed Slot: {}",slot);

    Ok(())
}
