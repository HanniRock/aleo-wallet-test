use std::fmt;
use std::fmt::Formatter;
use std::io::Write;
/**
 * @Author IronC <apehuang123@gmail.com>
 * @create 2023/4/13 11:03
 * @Project aleo-wallet-test
 *
 * This file is part of aleo-wallet-test.
 */

use std::sync::Arc;
use anyhow::bail;
use parking_lot::RwLock;
use snarkvm_console_network::Network;
use snarkvm_console_network::prelude::{bech32, IoResult, ToBase32};
use snarkvm_console_network_environment::Environment;
use snarkvm_console_program::ProgramID;
use snarkvm_synthesizer::{Block, BlockStore, ConsensusStorage, ConsensusStore, Process, Program, ProgramStore, ProvingKey, TransactionStore, TransitionStore, VerifyingKey};


use crate::transfer::{MY_CREDITS_PROVING_KEYS, MY_CREDITS_VERIFYING_KEYS};

#[derive(Clone)]
pub struct MyVm<N: Network, C: ConsensusStorage<N>> {
    /// The process.
    process: Arc<RwLock<Process<N>>>,
    /// The VM store.
    store: ConsensusStore<N, C>,
}

impl<N: Network, C: ConsensusStorage<N>> MyVm<N, C> {
    // Initializes the VM from storage.
    // #[inline]
    // pub fn from(store: ConsensusStore<N, C>) -> anyhow::Result<Self> {

        // let program = Program::<N>::credits()?;
        //
        //
        // let mut cache = Default::default();
        //
        // for k in program.functions().keys() {
        //     let pk = MY_CREDITS_PROVING_KEYS.get(&k.to_string()).unwrap().clone();
        //     let vk = MY_CREDITS_VERIFYING_KEYS.get(&k.to_string()).unwrap().clone();
        //     ProvingKey::
        // }
        //
        // // Initialize a new process.
        // let mut process = Process::load_with_cache(cache)?;
        //
        // // Retrieve the transaction store.
        // let transaction_store = store.transaction_store();
        // // Load the deployments from the store.
        // for transaction_id in transaction_store.deployment_transaction_ids() {
        //     // Retrieve the deployment.
        //     match transaction_store.get_deployment(&transaction_id)? {
        //         // Load the deployment.
        //         Some(deployment) => process.load_deployment(&deployment)?,
        //         None => bail!("Deployment transaction '{transaction_id}' is not found in storage."),
        //     };
        // }
        //
        // // Return the new VM.
        // Ok(Self { process: Arc::new(RwLock::new(process)), store })
    // }

    // /// Returns `true` if a program with the given program ID exists.
    // #[inline]
    // pub fn contains_program(&self, program_id: &ProgramID<N>) -> bool {
    //     self.process.read().contains_program(program_id)
    // }
    //
    // /// Adds the given block into the VM.
    // #[inline]
    // pub fn add_next_block(&self, block: &Block<N>) -> anyhow::Result<()> {
    //     // First, insert the block.
    //     self.block_store().insert(block)?;
    //     // Next, finalize the transactions.
    //     match self.finalize(block.transactions()) {
    //         Ok(_) => Ok(()),
    //         Err(error) => {
    //             // Rollback the block.
    //             self.block_store().remove_last_n(1)?;
    //             // Return the error.
    //             Err(error)
    //         }
    //     }
    // }
    //
    // /// Returns the process.
    // #[inline]
    // pub fn process(&self) -> Arc<RwLock<Process<N>>> {
    //     self.process.clone()
    // }
    //
    // /// Returns the program store.
    // #[inline]
    // pub fn program_store(&self) -> &ProgramStore<N, C::ProgramStorage> {
    //     self.store.program_store()
    // }
    //
    // /// Returns the block store.
    // #[inline]
    // pub fn block_store(&self) -> &BlockStore<N, C::BlockStorage> {
    //     self.store.block_store()
    // }
    //
    // /// Returns the transaction store.
    // #[inline]
    // pub fn transaction_store(&self) -> &TransactionStore<N, C::TransactionStorage> {
    //     self.store.transaction_store()
    // }
    //
    // /// Returns the transition store.
    // #[inline]
    // pub fn transition_store(&self) -> &TransitionStore<N, C::TransitionStorage> {
    //     self.store.transition_store()
    // }
    //
    // /// Starts an atomic batch write operation.
    // pub fn start_atomic(&self) {
    //     self.store.start_atomic();
    // }
    //
    // /// Checks if an atomic batch is in progress.
    // pub fn is_atomic_in_progress(&self) -> bool {
    //     self.store.is_atomic_in_progress()
    // }
    //
    // /// Aborts an atomic batch write operation.
    // pub fn abort_atomic(&self) {
    //     self.store.abort_atomic();
    // }
    //
    // /// Finishes an atomic batch write operation.
    // pub fn finish_atomic(&self) -> anyhow::Result<()> {
    //     self.store.finish_atomic()
    // }
}

#[test]
fn test_serialize_proving_key() {
    use snarkvm_algorithms::snark::marlin;
    use crate::CurrentNetwork;
    use serde::{Serialize, Serializer};
    use snarkvm_utilities::{ToBytes, ToBytesSerializer};
    use std::collections::HashMap;
    use std::fmt::Display;

    // #[derive(Serialize)]
    struct Model {
        proving_key: Arc<marlin::CircuitProvingKey<<CurrentNetwork as Environment>::PairingCurve, marlin::MarlinHidingMode>>,
    }

    impl Serialize for Model {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match serializer.is_human_readable() {
                true => serializer.collect_str(self),
                false => ToBytesSerializer::serialize_with_size_encoding(self, serializer),
            }
        }
    }

    impl Display for Model {
        /// Writes the proving key as a bech32m string.
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            // Convert the proving key to bytes.
            let bytes = self.to_bytes_le().map_err(|_| fmt::Error)?;
            // Encode the bytes into bech32m.
            let string =
                bech32::encode("prover", bytes.to_base32(), bech32::Variant::Bech32m).map_err(|_| fmt::Error)?;
            // Output the string.
            Display::fmt(&string, f)
        }
    }

    impl ToBytes for Model {
        /// Writes the proving key to a buffer.
        fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
            // Write the version.
            0u16.write_le(&mut writer)?;
            // Write the bytes.
            self.proving_key.write_le(&mut writer)
        }
    }

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for function_name in program.functions().keys() {
        let pk = MY_CREDITS_PROVING_KEYS.get(&function_name.to_string()).unwrap().clone();
        let pk_s = Model{proving_key: pk};

        let middle = serde_json::to_string(&pk_s).unwrap();
        let res = serde_json::from_str::<ProvingKey<CurrentNetwork>>(&middle).unwrap();
        let res_st = serde_json::to_string(&res).unwrap();
        assert_eq!(middle, res_st)
    }
}


#[test]
fn test_serialize_verifying_key() {
    use snarkvm_algorithms::snark::marlin;
    use crate::CurrentNetwork;
    use serde::{Serialize, Serializer};
    use snarkvm_utilities::{ToBytes, ToBytesSerializer};
    use std::collections::HashMap;
    use std::fmt::Display;

    // #[derive(Serialize)]
    struct Model {
        verifying_key: Arc<marlin::CircuitVerifyingKey<<CurrentNetwork as Environment>::PairingCurve, marlin::MarlinHidingMode>>,
    }

    impl Serialize for Model {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match serializer.is_human_readable() {
                true => serializer.collect_str(self),
                false => ToBytesSerializer::serialize_with_size_encoding(self, serializer),
            }
        }
    }

    impl Display for Model {
        /// Writes the proving key as a bech32m string.
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            // Convert the proving key to bytes.
            let bytes = self.to_bytes_le().map_err(|_| fmt::Error)?;
            // Encode the bytes into bech32m.
            let string =
                bech32::encode("verifier", bytes.to_base32(), bech32::Variant::Bech32m).map_err(|_| fmt::Error)?;
            // Output the string.
            Display::fmt(&string, f)
        }
    }

    impl ToBytes for Model {
        /// Writes the proving key to a buffer.
        fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
            // Write the version.
            0u16.write_le(&mut writer)?;
            // Write the bytes.
            self.verifying_key.write_le(&mut writer)
        }
    }

    let program = Program::<CurrentNetwork>::credits().unwrap();
    for function_name in program.functions().keys() {
        let vk = MY_CREDITS_VERIFYING_KEYS.get(&function_name.to_string()).unwrap().clone();
        let vk_s = Model{verifying_key: vk };

        let middle = serde_json::to_string(&vk_s).unwrap();
        let res = serde_json::from_str::<VerifyingKey<CurrentNetwork>>(&middle).unwrap();
        let res_st = serde_json::to_string(&res).unwrap();
        assert_eq!(middle, res_st)
    }
}

