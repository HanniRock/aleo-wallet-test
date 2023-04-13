/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/13 15:05
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */
use snarkvm_algorithms::snark::marlin;
use snarkvm_console_network::prelude::{bech32, IoResult, ToBase32};
use snarkvm_console_network::Network;
use snarkvm_console_network::environment::Environment;
use snarkvm_synthesizer::{Program, ProvingKey};
use snarkvm_utilities::{FromBytes, ToBytes, ToBytesSerializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::sync::Arc;
use crate::transfer::CREDITS_PROVING_KEYS_T;
use crate::utils::MarlinProvingKey;
use crate::CurrentNetwork;
use indexmap::IndexMap;
use serde::{Serialize, Serializer};

pub(crate) struct ProvingKeyModel<N: Network> {
    proving_key: Arc<marlin::CircuitProvingKey<N::PairingCurve, marlin::MarlinHidingMode>>,
}

impl<N: Network> ProvingKeyModel<N> {
    pub(crate) fn setup(program: &Program<N>) -> IndexMap<String, ProvingKey<N>> {
        let credits_verifying_keys =
            Self::get_credits_proving_keys(CREDITS_PROVING_KEYS_T).unwrap();
        let mut cache = IndexMap::new();
        for function_name in program.functions().keys() {
            let pk = credits_verifying_keys
                .get(&function_name.to_string())
                .unwrap()
                .clone();
            let pk_s = ProvingKeyModel::<N> { proving_key: pk };

            let middle = serde_json::to_string(&pk_s).unwrap();
            let res = serde_json::from_str::<ProvingKey<N>>(&middle).unwrap();
            cache.insert(function_name.to_string(), res);
        }
        cache
    }

    fn get_credits_proving_keys(
        data: &[u8],
    ) -> anyhow::Result<IndexMap<String, Arc<MarlinProvingKey<N>>>> {
        let credits_proving_keys_raw: IndexMap<String, Vec<u8>> = bincode::deserialize(data)
            .map_err(|err| anyhow::Error::msg(format!("failed to deserialize data: {}", err)))?;
        let mut credits_proving_keys = IndexMap::new();
        for (k, v) in credits_proving_keys_raw.iter() {
            let le: Arc<MarlinProvingKey<N>> =
                Arc::new(MarlinProvingKey::<N>::read_le(v.as_slice()).map_err(|err| {
                    anyhow::Error::msg(format!("failed to read_le data: {}", err))
                })?);
            credits_proving_keys.insert(k.clone(), le);
        }
        Ok(credits_proving_keys)
    }
}

impl<N: Network> Serialize for ProvingKeyModel<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> anyhow::Result<S::Ok, S::Error> {
        match serializer.is_human_readable() {
            true => serializer.collect_str(self),
            false => ToBytesSerializer::serialize_with_size_encoding(self, serializer),
        }
    }
}

impl<N: Network> Display for ProvingKeyModel<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Convert the proving key to bytes.
        let bytes = self.to_bytes_le().map_err(|_| fmt::Error)?;
        // Encode the bytes into bech32m.
        let string = bech32::encode("prover", bytes.to_base32(), bech32::Variant::Bech32m)
            .map_err(|_| fmt::Error)?;
        // Output the string.
        Display::fmt(&string, f)
    }
}

impl<N: Network> ToBytes for ProvingKeyModel<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Write the version.
        0u16.write_le(&mut writer)?;
        // Write the bytes.
        self.proving_key.write_le(&mut writer)
    }
}

#[test]
fn test_serialize_proving_key() {
    use super::*;
    use snarkvm_synthesizer::Program;
    use snarkvm_synthesizer::ProvingKey;

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for function_name in program.functions().keys() {
        let pk =
            ProvingKeyModel::<CurrentNetwork>::get_credits_proving_keys(CREDITS_PROVING_KEYS_T)
                .unwrap()
                .get(&function_name.to_string())
                .unwrap()
                .clone();
        let pk_s = ProvingKeyModel::<CurrentNetwork> { proving_key: pk };

        let middle = serde_json::to_string(&pk_s).unwrap();
        let res = serde_json::from_str::<ProvingKey<CurrentNetwork>>(&middle).unwrap();
        let res_st = serde_json::to_string(&res).unwrap();
        assert_eq!(middle, res_st)
    }
}

#[test]
fn test_setup_proving_keys() {
    let program = Program::<CurrentNetwork>::credits().unwrap();
    let map = ProvingKeyModel::<CurrentNetwork>::setup(&program);
    assert_eq!(map.len(), program.functions().len())
}

#[test]
fn test_credits_proving_keys() {
    use crate::CurrentNetwork;
    use indexmap::IndexMap;
    use snarkvm_console_network::CREDITS_PROVING_KEYS;
    use snarkvm_console_network::environment::Console;
    use snarkvm_synthesizer::Program;
    use snarkvm_utilities::ToBytes;
    use std::fs::File;
    use std::io::{Read, Write};

    // type MarlinProvingKey<N> =
    //     CircuitProvingKey<<N as Environment>::PairingCurve, MarlinHidingMode>;

    let mut new_credits_proving_keys = IndexMap::new();

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for k in program.functions().keys() {
        if let Some(v) = CREDITS_PROVING_KEYS.get(&k.to_string()) {
            new_credits_proving_keys.insert(k.to_string(), v.clone());
        }
    }
    println!("{:?}", new_credits_proving_keys.keys());
    assert_eq!(
        new_credits_proving_keys.len(),
        program.functions().keys().len()
    );

    let mut credits_proving_keys_1 = IndexMap::new();
    for (k, v) in new_credits_proving_keys.iter() {
        credits_proving_keys_1.insert(k.clone(), v.clone().to_bytes_le().unwrap());
    }

    let serialized_data = bincode::serialize(&credits_proving_keys_1).unwrap();
    let mut file = File::create("credits_proving_keys_test").unwrap();
    file.write_all(&serialized_data).unwrap();

    let mut file = File::open("credits_proving_keys_test").unwrap();
    let mut content = Vec::new();
    let _ = file.read_to_end(&mut content).unwrap();

    let credits_proving_keys_2: IndexMap<String, Vec<u8>> = bincode::deserialize(&content).unwrap();

    assert_eq!(credits_proving_keys_2, credits_proving_keys_1);

    // let mut credits_proving_keys_3 = IndexMap::new();
    // for (k, v) in credits_proving_keys_2.iter() {
    //     let le: Arc<MarlinProvingKey<Console>> =
    //         Arc::new(MarlinProvingKey::<Console>::read_le(v.as_slice()).unwrap());
    //     credits_proving_keys_3.insert(k.clone(), le);
    // }
    let credits_proving_keys_3 =
        ProvingKeyModel::<CurrentNetwork>::get_credits_proving_keys(&content).unwrap();
    assert_eq!(new_credits_proving_keys, credits_proving_keys_3)
}
