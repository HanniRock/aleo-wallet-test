//
// @Author IronC <apehuang123@gmail.com>
// @create 2023/4/14 17:20
// @Project aleo-wallet-test
//
// This file is part of aleo-wallet-test.
//

use std::mem::zeroed;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use aleo_std::prelude::{finish, lap, timer};
use anyhow::ensure;
use circuit::AleoV0;
use parking_lot::RwLock;
use snarkvm_console_account::PrivateKey;
use snarkvm_console_network::prelude::{CryptoRng, Rng};
use snarkvm_console_network::{Network, Testnet3};
use snarkvm_console_program::{Identifier, Plaintext, ProgramID, Record, Request, Response};
use snarkvm_synthesizer::{Authorization, BlockMemory, CallMetrics, CallStack, cast_ref, ConsensusMemory, ConsensusStorage, ConsensusStore, Execution, Fee, Inclusion, InclusionAssignment, Query, Stack, Transaction, Transition, VM};
use tracing::debug;
use warp::{Filter, reject, Rejection, Reply};
use warp::http::HeaderName;
use anyhow::anyhow;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static!(
    static ref VM_INSTANCE: VM<Testnet3, ConsensusMemory<Testnet3>> = {
    let store = ConsensusStore::<Testnet3, ConsensusMemory<Testnet3>>::open(None).unwrap();
    VM::from(store).unwrap()
};
);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyRequest {
    request: Request<Testnet3>,
    fee_request: Option<Request<Testnet3>>,
    fee_record: Option<String>,
    fee: Option<u64>,
}

const TRANSFER_CONF_DATA: &[u8] = include_bytes!("../../wasm-lib/src/transfer_conf");

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header(HeaderName::from_static("content-type"))
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    // Initialize the routes.
    let routes = routes();

    // Add custom logging for each request.
    let custom_log = warp::log::custom(|info| match info.remote_addr() {
        Some(addr) => debug!("Received '{} {}' from '{addr}' ({})", info.method(), info.path(), info.status()),
        None => debug!("Received '{} {}' ({})", info.method(), info.path(), info.status()),
    });

    // Start the server.
    let addr = SocketAddr::from_str("0.0.0.0:17777").unwrap();
    warp::serve(routes.with(cors).with(custom_log)).run(addr).await
}

fn routes() -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    // POST /execute_function
    let execute_function = warp::post()
        .and(warp::path!("execute_function"))
        .and(warp::body::content_length_limit(16 * 1024 * 1024))
        .and(warp::body::json())
        .and_then(execute_function);
    execute_function
}

async fn execute_function(request: MyRequest) -> anyhow::Result<impl Reply, Rejection> {
    let stack = VM_INSTANCE.process().read().get_stack(request.request.program_id()).or_reject()?.clone();
    let authorization = Authorization::new(&[request.request.clone()]);
    let private_key = PrivateKey::<Testnet3>::from_str("APrivateKey1zkp8CZNn3yeCseEtxuVPbDCwSyhGW6yZKUYKfgXmcpoGPWH").unwrap();
    // Construct the call stack.
    let call_stack = CallStack::Authorize(vec![request.request], private_key, authorization.clone());
    // Initialize an RNG.
    let rng = &mut rand::thread_rng();
    // Construct the authorization from the function.
    let _response = stack.execute_function::<AleoV0, _>(call_stack, rng).or_reject()?;

    let mut fee = None;
    // Prepare the fees.
    if let Some(_) = request.fee_request {
        fee = match request.fee_record {
            Some(record) => {
                let record = Record::<Testnet3, Plaintext<Testnet3>>::from_str(&record).map_err(|e| anyhow!(e)).or_reject()?;
                let fee_amount = request.fee.unwrap_or(0);

                Some((record, fee_amount))
            }
            None => None,
        };
    }

    let file_contents = std::str::from_utf8(TRANSFER_CONF_DATA).unwrap();
    let conf = file_contents
        .split('\n')
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    println!("{}", conf[1]);
    let query = Some(Query::<Testnet3, BlockMemory<Testnet3>>::from(conf[1].clone()));

    let result = execute_authorization_with_additional_fee(&VM_INSTANCE, &private_key, &request.fee_request, authorization, fee, query, rng, &stack).or_reject()?;
    println!("execute_function ok");
    Ok(result.to_string())
}

fn execute_authorization_with_additional_fee<C: ConsensusStorage<Testnet3>, R: Rng + CryptoRng>(
    _vm: &VM<Testnet3, C>,
    private_key: &PrivateKey<Testnet3>,
    fee_request: &Option<Request<Testnet3>>,
    authorization: Authorization<Testnet3>,
    additional_fee: Option<(Record<Testnet3, Plaintext<Testnet3>>, u64)>,
    query: Option<Query<Testnet3, BlockMemory<Testnet3>>>,
    rng: &mut R,
    stack: &Stack<Testnet3>,
) -> anyhow::Result<Transaction<Testnet3>> {
    // Compute the execution.
    let (_response, execution, _metrics) = VM_INSTANCE.execute(authorization, query.clone(), rng)?;

    let mut additional_fee_f = None;
    // Compute the additional fee, if it is present.
    if let Some(fee_request) = fee_request {
        additional_fee_f = match additional_fee {
            Some((_credits, _additional_fee_in_gates)) => {
                Some(execute_fee(&VM_INSTANCE, private_key, fee_request, query, rng, stack)?.1)
            }
            None => None,
        };
    }

    Transaction::from_execution(execution, additional_fee_f)
}

fn execute_fee<R: Rng + CryptoRng, C: ConsensusStorage<Testnet3>>(
    _vm: &VM<Testnet3, C>,
    private_key: &PrivateKey<Testnet3>,
    fee_request: &Request<Testnet3>,
    query: Option<Query<Testnet3, BlockMemory<Testnet3>>>,
    rng: &mut R,
    stack: &Stack<Testnet3>,
) -> anyhow::Result<(Response<Testnet3>, Fee<Testnet3>, Vec<CallMetrics<Testnet3>>)> {
    println!("VM::execute_fee");

    // Prepare the query.
    let query = match query {
        Some(query) => query,
        None => Query::VM(VM_INSTANCE.block_store().clone()),
    };

    println!("Process::execute_fee");

    // Ensure the fee has the correct program ID.
    let program_id = ProgramID::<Testnet3>::from_str("credits.aleo")?;
    // Ensure the fee has the correct function.
    let function_name = Identifier::<Testnet3>::from_str("fee")?;
    // Initialize the authorization.
    let authorization = Authorization::new(&[fee_request.clone()]);
    println!("Initialize the authorization");
    // Construct the call stack.
    let call_stack = CallStack::Authorize(vec![fee_request.clone()], *private_key, authorization.clone());
    // Construct the authorization from the function.
    let _response = stack.execute_function::<AleoV0, R>(call_stack, rng)?;
    println!("Construct the authorization from the function");

    // Retrieve the main request (without popping it).
    let request = authorization.peek_next()?;
    // Prepare the stack.
    let binding = VM_INSTANCE.process();
    let stack = binding.read().get_stack(request.program_id())?;
    // Initialize the execution.
    let execution = Arc::new(RwLock::new(Execution::new()));
    // Initialize the inclusion.
    let inclusion = Arc::new(RwLock::new(Inclusion::new()));
    // Initialize the metrics.
    let metrics = Arc::new(RwLock::new(Vec::new()));
    // Initialize the call stack.
    let call_stack = CallStack::execute(authorization, execution.clone(), inclusion.clone(), metrics.clone())?;
    // Execute the circuit.
    let binding = binding.read();
    let stack = binding.get_stack(request.program_id())?;
    let response = stack.execute_function::<AleoV0, R>(call_stack, rng)?;
    println!("Execute the circuit");

    // Extract the execution.
    let execution = Arc::try_unwrap(execution).unwrap().into_inner();
    // Ensure the execution contains 1 transition.
    ensure!(execution.len() == 1, "Execution of '{}/{}' does not contain 1 transition", program_id, function_name);
    // Extract the inclusion.
    let inclusion = Arc::try_unwrap(inclusion).unwrap().into_inner();
    // Extract the metrics.
    let metrics = Arc::try_unwrap(metrics).unwrap().into_inner();

    let (response, fee_transition, inclusion, metrics) = (response, execution.peek()?.clone(), inclusion, metrics);

    // Prepare the assignments.
    let assignments = {
        let fee_transition = cast_ref!(fee_transition as Transition<Testnet3>);
        let inclusion = cast_ref!(inclusion as Inclusion<Testnet3>);
        inclusion.prepare_fee(fee_transition, query)?
    };
    let assignments = cast_ref!(assignments as Vec<InclusionAssignment<Testnet3>>);
    println!("Prepare the assignments");

    // Compute the inclusion proof and construct the fee.
    let fee = inclusion.prove_fee::<AleoV0, R>(fee_transition, assignments, rng)?;
    println!("Compute the inclusion proof and construct the fee");

    // Prepare the return.
    let response = cast_ref!(response as Response<Testnet3>).clone();
    let fee = cast_ref!(fee as Fee<Testnet3>).clone();
    let metrics = cast_ref!(metrics as Vec<CallMetrics<Testnet3>>).clone();
    println!("Prepare the response, fee, and metrics");

    println!("execute_fee finished");

    // Return the response, fee, metrics.
    Ok((response, fee, metrics))
}

/// A trait to unwrap a `Result` or `Reject`.
pub trait OrReject<T> {
    /// Returns the result if it is successful, otherwise returns a rejection.
    fn or_reject(self) -> Result<T, Rejection>;
}

impl<T> OrReject<T> for anyhow::Result<T> {
    /// Returns the result if it is successful, otherwise returns a rejection.
    fn or_reject(self) -> Result<T, Rejection> {
        self.map_err(|e| reject::custom(RestError::Request(e.to_string())))
    }
}

/// An enum of error handlers for the REST API server.
#[derive(Debug)]
pub enum RestError {
    Request(String),
}

impl warp::reject::Reject for RestError {}
