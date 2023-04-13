/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/13 15:05
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */
use snarkvm_algorithms::snark::marlin;
use crate::CurrentNetwork;
use serde::{Serialize, Serializer};
use snarkvm_utilities::{ToBytes, ToBytesSerializer};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::sync::Arc;
use snarkvm_console_network::prelude::{bech32, IoResult, ToBase32};
use snarkvm_console_network_environment::Environment;

pub(crate) struct VerifyingKeyModel {
    verifying_key: Arc<marlin::CircuitVerifyingKey<<CurrentNetwork as Environment>::PairingCurve, marlin::MarlinHidingMode>>,
}


impl Serialize for VerifyingKeyModel {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match serializer.is_human_readable() {
            true => serializer.collect_str(self),
            false => ToBytesSerializer::serialize_with_size_encoding(self, serializer),
        }
    }
}

impl Display for VerifyingKeyModel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Convert the verifying key to bytes.
        let bytes = self.to_bytes_le().map_err(|_| fmt::Error)?;
        // Encode the bytes into bech32m.
        let string =
            bech32::encode("verifier", bytes.to_base32(), bech32::Variant::Bech32m).map_err(|_| fmt::Error)?;
        // Output the string.
        Display::fmt(&string, f)
    }
}

impl ToBytes for VerifyingKeyModel {
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
    use crate::transfer::MY_CREDITS_VERIFYING_KEYS;
    use snarkvm_synthesizer::VerifyingKey;

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for function_name in program.functions().keys() {
        let vk = MY_CREDITS_VERIFYING_KEYS.get(&function_name.to_string()).unwrap().clone();
        let vk_s = VerifyingKeyModel { verifying_key: vk };

        let middle = serde_json::to_string(&vk_s).unwrap();
        let res = serde_json::from_str::<VerifyingKey<CurrentNetwork>>(&middle).unwrap();
        let res_st = serde_json::to_string(&res).unwrap();
        assert_eq!(middle, res_st)
    }
}
