/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/13 15:05
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */
use crate::transfer::CREDITS_VERIFYING_KEYS_T;
use crate::utils::MarlinVerifyingKey;
use crate::CurrentNetwork;
use indexmap::IndexMap;
use serde::{Serialize, Serializer};
use snarkvm_algorithms::snark::marlin;
use snarkvm_console_network::prelude::{bech32, IoResult, ToBase32};
use snarkvm_console_network::Network;
use snarkvm_console_network::environment::{Console, Environment};
use snarkvm_synthesizer::{Program, VerifyingKey};
use snarkvm_utilities::{FromBytes, ToBytes, ToBytesSerializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::sync::Arc;

pub(crate) struct VerifyingKeyModel<N: Network> {
    verifying_key: Arc<marlin::CircuitVerifyingKey<N::PairingCurve, marlin::MarlinHidingMode>>,
}

impl<N: Network> VerifyingKeyModel<N> {
    pub(crate) fn setup(program: &Program<N>) -> IndexMap<String, VerifyingKey<N>> {
        let credits_verifying_keys =
            Self::get_credits_verifying_keys(CREDITS_VERIFYING_KEYS_T).unwrap();
        let mut cache = IndexMap::new();
        for function_name in program.functions().keys() {
            let vk = credits_verifying_keys
                .get(&function_name.to_string())
                .unwrap()
                .clone();
            let vk_s = VerifyingKeyModel::<N> { verifying_key: vk };

            let middle = serde_json::to_string(&vk_s).unwrap();
            let res = serde_json::from_str::<VerifyingKey<N>>(&middle).unwrap();
            cache.insert(function_name.to_string(), res);
        }
        cache
    }

    fn get_credits_verifying_keys(
        data: &[u8],
    ) -> anyhow::Result<IndexMap<String, Arc<MarlinVerifyingKey<N>>>> {
        let credits_verifying_keys_raw: IndexMap<String, Vec<u8>> = bincode::deserialize(data)
            .map_err(|err| anyhow::Error::msg(format!("failed to deserialize data: {}", err)))?;
        let mut credits_verifying_keys = IndexMap::new();
        for (k, v) in credits_verifying_keys_raw.iter() {
            let le: Arc<MarlinVerifyingKey<N>> = Arc::new(
                MarlinVerifyingKey::<N>::read_le(v.as_slice()).map_err(|err| {
                    anyhow::Error::msg(format!("failed to read_le data: {}", err))
                })?,
            );
            credits_verifying_keys.insert(k.clone(), le);
        }
        Ok(credits_verifying_keys)
    }
}

impl<N: Network> Serialize for VerifyingKeyModel<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match serializer.is_human_readable() {
            true => serializer.collect_str(self),
            false => ToBytesSerializer::serialize_with_size_encoding(self, serializer),
        }
    }
}

impl<N: Network> Display for VerifyingKeyModel<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Convert the verifying key to bytes.
        let bytes = self.to_bytes_le().map_err(|_| fmt::Error)?;
        // Encode the bytes into bech32m.
        let string = bech32::encode("verifier", bytes.to_base32(), bech32::Variant::Bech32m)
            .map_err(|_| fmt::Error)?;
        // Output the string.
        Display::fmt(&string, f)
    }
}

impl<N: Network> ToBytes for VerifyingKeyModel<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Write the version.
        0u16.write_le(&mut writer)?;
        // Write the bytes.
        self.verifying_key.write_le(&mut writer)
    }
}

#[test]
fn test_serialize_verifying_key() {
    use super::*;
    use snarkvm_synthesizer::Program;
    use snarkvm_synthesizer::VerifyingKey;

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for function_name in program.functions().keys() {
        let vk = VerifyingKeyModel::<CurrentNetwork>::get_credits_verifying_keys(
            CREDITS_VERIFYING_KEYS_T,
        )
        .unwrap()
        .get(&function_name.to_string())
        .unwrap()
        .clone();
        let vk_s = VerifyingKeyModel::<CurrentNetwork> { verifying_key: vk };

        let middle = serde_json::to_string(&vk_s).unwrap();
        let res = serde_json::from_str::<VerifyingKey<CurrentNetwork>>(&middle).unwrap();
        let res_st = serde_json::to_string(&res).unwrap();
        assert_eq!(middle, res_st)
    }
}

#[test]
fn test_setup_verifying_keys() {
    let program = Program::<CurrentNetwork>::credits().unwrap();
    let map = VerifyingKeyModel::<CurrentNetwork>::setup(&program);
    assert_eq!(map.len(), program.functions().len())
}

#[test]
fn test_credits_verifying_keys() {
    use crate::CurrentNetwork;
    use indexmap::IndexMap;
    use snarkvm_console_network::CREDITS_VERIFYING_KEYS;
    use snarkvm_console_network::environment::Console;
    use snarkvm_synthesizer::Program;
    use snarkvm_utilities::ToBytes;
    use std::fs::File;
    use std::io::{Read, Write};

    let mut new_credits_verifying_keys = IndexMap::new();

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for k in program.functions().keys() {
        if let Some(v) = CREDITS_VERIFYING_KEYS.get(&k.to_string()) {
            new_credits_verifying_keys.insert(k.to_string(), v.clone());
        }
    }
    println!("{:?}", new_credits_verifying_keys.keys());
    assert_eq!(
        new_credits_verifying_keys.len(),
        program.functions().keys().len()
    );

    let mut credits_verifying_keys_1 = IndexMap::new();
    for (k, v) in new_credits_verifying_keys.iter() {
        credits_verifying_keys_1.insert(k.clone(), v.clone().to_bytes_le().unwrap());
    }

    let serialized_data = bincode::serialize(&credits_verifying_keys_1).unwrap();
    let mut file = File::create("credits_verifying_keys_test").unwrap();
    file.write_all(&serialized_data).unwrap();

    let mut file = File::open("credits_verifying_keys_test").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();

    let credits_verifying_keys_2: IndexMap<String, Vec<u8>> =
        bincode::deserialize(&content).unwrap();

    assert_eq!(credits_verifying_keys_2, credits_verifying_keys_1);

    let credits_verifying_keys_3 =
        VerifyingKeyModel::<CurrentNetwork>::get_credits_verifying_keys(&content).unwrap();
    assert_eq!(new_credits_verifying_keys, credits_verifying_keys_3)
}
