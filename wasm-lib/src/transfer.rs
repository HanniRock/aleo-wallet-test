/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/12 10:34
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */
use crate::utils::post_request;
use crate::CurrentNetwork;
use anyhow::{bail, ensure};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use serde_json::from_str;
use snarkvm_console_account::address::Address;
use snarkvm_console_account::PrivateKey;
use snarkvm_console_network::Network;
use snarkvm_console_network::environment::Console;
use snarkvm_console_program::{Identifier, Locator, Plaintext, ProgramID, Record, Request, U64, Value};
use snarkvm_synthesizer::{Authorization, CallStack, ConsensusMemory, ConsensusStore, Process, Program, ProvingKey, Query, Transaction, VerifyingKey, VM};
use snarkvm_utilities::ToBytes;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use wasm_bindgen_futures::JsFuture;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct TransferRequest<N: Network> {
    request: Request<N>,
    fee_request: Option<Request<N>>,
    fee_record: Option<String>,
    fee: Option<u64>,
}

pub(crate) async fn transfer_internal<N: Network>(
    private_key: String,
    record: String,
    fee_record: Option<String>,
    amount: u64,
    fee: Option<u64>,
    recipient: String,
    broadcast: String,
) -> anyhow::Result<String> {
    // Initialize an RNG.
    let rng = &mut rand::thread_rng();

    let record = Record::<N, Plaintext<N>>::from_str(&record)?;
    let recipient = Address::<N>::from_str(&recipient)?;

    let inputs = vec![
        Value::Record(record.clone()),
        Value::from_str(&format!("{}", recipient))?,
        Value::from_str(&format!("{}u64", amount))?,
    ];

    let program = Program::<N>::credits()?;

    // Initialize the 'credits.aleo' program.
    let function_name = Identifier::<N>::from_str("transfer")?;
    // Retrieve the private key.
    let private_key = PrivateKey::<N>::from_str(&private_key)?;
    let input_types = program.get_function(&function_name)?.input_types();

    let request = Request::<N>::sign(&private_key, *program.id(), function_name, &mut inputs.into_iter(), &input_types, rng)?;

    let mut transfer_request = TransferRequest {
        request,
        fee_request: None,
        fee_record: None,
        fee: None,
    };

    if let Some(fee_record) = fee_record {
        let fee_record_raw = Record::<N, Plaintext<N>>::from_str(&fee_record)?;
        let fee_inputs = [Value::Record(fee_record_raw), Value::from_str(&format!("{}", U64::<N>::new(fee.unwrap_or_default())))?];
        let fee_function_name = Identifier::<N>::from_str("fee")?;
        let fee_input_types = program.get_function(&fee_function_name)?.input_types();
        let fee_request = Request::<N>::sign(&private_key, *program.id(), fee_function_name, &mut fee_inputs.into_iter(), &fee_input_types, rng)?;
        transfer_request.fee = Some(fee.unwrap_or_default());
        transfer_request.fee_record = Some(fee_record);
        transfer_request.fee_request = Some(fee_request);
    }

    // send to vm server
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:17777/execute_function";
    let body = serde_json::to_string(&transfer_request).map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let response = client.post(url).body(body).send().await.map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let response_body = response.text().await.map_err(|e| anyhow::Error::msg(e.to_string()))?;

    // broadcast
    let transaction = Transaction::<CurrentNetwork>::from_str(&response_body)?;
    match client.post(broadcast).body(response_body).send().await {
        Ok(response) => {
            let id = response.text().await.map_err(|e| anyhow::Error::new(e))?;
            if id.eq(&transaction.id().to_string()) {
                Ok(format!("transaction_id: {}", id))
            } else {
                Err(anyhow::Error::msg("failed to broadcast: transaction id not met"))
            }
        }
        Err(e) => {
            Err(anyhow::Error::msg(format!("failed to broadcast: {}", e)))
        }
    }
}

// wasm-pack test --chrome
// #[cfg(target_arch = "wasm32")]
mod tests {
    use std::str::FromStr;
    use circuit::AleoV0;
    use rand::Rng;
    use reqwest::{Error, Response};
    use snarkvm_console_account::{Address, PrivateKey};
    use snarkvm_console_network::prelude::CryptoRng;
    use snarkvm_console_program::{Identifier, Plaintext, Record, Request, U64, Value};
    use snarkvm_synthesizer::{Authorization, CallStack, ConsensusMemory, ConsensusStore, Program, Transaction, VM};
    use wasm_bindgen_test::{console_log, wasm_bindgen_test, wasm_bindgen_test_configure};
    use crate::CurrentNetwork;
    use crate::utils::post_request;
    use serde::{Serialize, Deserialize};
    wasm_bindgen_test_configure!(run_in_browser);

    const TRANSFER_CONF_DATA: &[u8] = include_bytes!("transfer_conf");

    #[wasm_bindgen_test]
    async fn test_transfer_internal() {
        use crate::transfer::transfer_internal;
        use crate::CurrentNetwork;
        use std::str::FromStr;

        let file_contents = std::str::from_utf8(TRANSFER_CONF_DATA).unwrap();
        let conf = file_contents
            .split('\n')
            .map(|c| c.to_string())
            .collect::<Vec<String>>();

        console_log!("{}", conf[3].clone());

        let msg = transfer_internal::<CurrentNetwork>(
            conf[0].clone(),
            conf[3].clone(),
            Some(conf[6].clone()),
            u64::from_str(&conf[4]).unwrap(),
            Some(u64::from_str(&conf[7]).unwrap()),
            conf[5].clone(),
            conf[2].clone(),
        )
            .await.unwrap();
        console_log!("{}", msg)
    }

    #[test]
    fn test_transfer_conf_data() {
        let file_contents = std::str::from_utf8(TRANSFER_CONF_DATA).unwrap();
        let split = file_contents
            .split('\n')
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();
        println!("{:?}", split);
    }


    // #[tokio::test]
    // async fn test_transfer() {
    //     #[derive(Clone, Debug, Serialize, Deserialize)]
    //     pub struct MyRequest {
    //         request: Request<CurrentNetwork>,
    //         fee_request: Option<Request<CurrentNetwork>>,
    //         fee_record: Option<String>,
    //         fee: Option<u64>,
    //     }
    //     let file_contents = std::str::from_utf8(TRANSFER_CONF_DATA).unwrap();
    //     let conf = file_contents
    //         .split('\n')
    //         .map(|c| c.to_string())
    //         .collect::<Vec<String>>();
    //
    //     // Initialize an RNG.
    //     let rng = &mut rand::thread_rng();
    //
    //     let record = Record::<CurrentNetwork, Plaintext<CurrentNetwork>>::from_str(&conf[3].clone()).unwrap();
    //     let recipient = Address::<CurrentNetwork>::from_str(&conf[5].clone()).unwrap();
    //
    //     let inputs = vec![
    //         Value::Record(record.clone()),
    //         Value::from_str(&format!("{}", recipient)).unwrap(),
    //         Value::from_str(&format!("{}u64", u64::from_str(&conf[4]).unwrap())).unwrap(),
    //     ];
    //
    //     let program = Program::<CurrentNetwork>::credits().unwrap();
    //
    //     // Initialize the 'credits.aleo' program.
    //     let function_name = Identifier::<CurrentNetwork>::from_str("transfer").unwrap();
    //     // Retrieve the private key.
    //     let private_key = PrivateKey::<CurrentNetwork>::from_str(&conf[0].clone()).unwrap();
    //     let input_types = program.get_function(&function_name).unwrap().input_types();
    //
    //     let request = Request::<CurrentNetwork>::sign(&private_key, *program.id(), function_name, &mut inputs.into_iter(), &input_types, rng).unwrap();
    //
    //     //TODO test fee
    //     let fee_record = Record::<CurrentNetwork, Plaintext<CurrentNetwork>>::from_str(&conf[6].clone()).unwrap();
    //     let fee_inputs = [Value::Record(fee_record.clone()), Value::from_str(&format!("{}", U64::<CurrentNetwork>::new(200))).unwrap()];
    //     let fee_function_name = Identifier::<CurrentNetwork>::from_str("fee").unwrap();
    //     let fee_input_types = program.get_function(&fee_function_name).unwrap().input_types();
    //     let fee_request = Request::<CurrentNetwork>::sign(&private_key, *program.id(), fee_function_name, &mut fee_inputs.into_iter(), &fee_input_types, rng).unwrap();
    //
    //
    //     let req = MyRequest {
    //         request,
    //         fee_request: Some(fee_request),
    //         fee_record: Some(fee_record.to_string()),
    //         fee: Some(200),
    //     };
    //
    //     let client = reqwest::Client::new();
    //     let url = "http://127.0.0.1:17777/execute_function";
    //     let body = serde_json::to_string(&req).unwrap();
    //     let response = client.post(url).body(body).send().await.unwrap();
    //     let response_body = response.text().await.unwrap();
    //
    //     println!("response_body {}", response_body);
    //
    //     // println!("from vm server: {:?}", response_body);
    //     // broadcast
    //     let transaction = Transaction::<CurrentNetwork>::from_str(&response_body).unwrap();
    //     let broadcast_url = conf[2].clone();
    //     match client.post(broadcast_url).body(response_body).send().await {
    //         Ok(response) => {
    //             let id = response.text().await.unwrap();
    //             println!("transaction_id: {}", id);
    //             assert_eq!(id, transaction.id().to_string())
    //         }
    //         Err(e) => {
    //             println!("faile to broadcast {}", e);
    //         }
    //     }
    // }
}
