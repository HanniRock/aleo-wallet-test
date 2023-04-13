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
use snarkvm_console_network::prelude::{bech32, IoResult, ToBase32};
use snarkvm_console_network::Network;
use snarkvm_console_network_environment::Environment;
use snarkvm_console_program::ProgramID;
use snarkvm_synthesizer::{
    Block, BlockStore, ConsensusStorage, ConsensusStore, Process, Program, ProgramStore,
    ProvingKey, TransactionStore, TransitionStore, VerifyingKey, VM,
};
use std::fmt;
use std::fmt::Formatter;
use std::io::Write;

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
