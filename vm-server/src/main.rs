//
// @Author IronC <apehuang123@gmail.com>
// @create 2023/4/14 17:20
// @Project aleo-wallet-test
//
// This file is part of aleo-wallet-test.
//

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::ensure;
use circuit::{Aleo, AleoV0, Environment};
use parking_lot::RwLock;
use snarkvm_console_account::PrivateKey;
use snarkvm_console_network::prelude::{CryptoRng, Rng};
use snarkvm_console_network::Network;
use snarkvm_console_program::{Identifier, Plaintext, ProgramID, Record, Request, Response};
use snarkvm_synthesizer::{Authorization, CallMetrics, CallStack, cast_ref, ConsensusMemory, ConsensusStorage, ConsensusStore, Execution, Fee, Inclusion, InclusionAssignment, Query, Stack, Transaction, Transition, VM};
use tracing::debug;
use warp::{Filter, reject, Rejection, Reply};
use warp::http::HeaderName;
use anyhow::anyhow;
use rand::prelude::ThreadRng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct MyRequest<N: Network> {
    request: Request<N>,
    fee_request: Option<Request<N>>,
    fee_record: Option<String>,
    fee: Option<u64>,
}

const TRANSFER_CONF_DATA: &[u8] = include_bytes!("../../wasm-lib/src/transfer_conf");

type CurrentNetwork = <AleoV0 as Environment>::Network;

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header(HeaderName::from_static("content-type"))
        .allow_methods(vec!["GET", "POST", "OPTIONS"]);

    let store = ConsensusStore::<CurrentNetwork, ConsensusMemory<CurrentNetwork>>::open(None).unwrap();
    let vm = VM::from(store).unwrap();

    // Initialize the routes.
    let routes = routes::<CurrentNetwork, AleoV0, ConsensusMemory<CurrentNetwork>>(vm);

    // Add custom logging for each request.
    let custom_log = warp::log::custom(|info| match info.remote_addr() {
        Some(addr) => debug!("Received '{} {}' from '{addr}' ({})", info.method(), info.path(), info.status()),
        None => debug!("Received '{} {}' ({})", info.method(), info.path(), info.status()),
    });

    // Start the server.
    let addr = SocketAddr::from_str("0.0.0.0:17777").unwrap();
    warp::serve(routes.with(cors).with(custom_log)).run(addr).await
}

fn routes<N: Network, A: Aleo<Network=N>, C: ConsensusStorage<N>>(vm: VM<N, C>) -> impl Filter<Extract=(impl Reply, ), Error=Rejection> + Clone {
    // POST /execute_function
    warp::post()
        .and(warp::path!("execute_function"))
        .and(warp::body::content_length_limit(16 * 1024 * 1024))
        .and(warp::body::json())
        .and(warp::any().map(move || vm.clone()))
        .and_then(execute_function::<N, A, C>)
}

async fn execute_function<N: Network, A: Aleo<Network=N>, C: ConsensusStorage<N>>(request: MyRequest<N>, vm: VM<N, C>) -> anyhow::Result<impl Reply, Rejection> {
    let stack = vm.process().read().get_stack(request.request.program_id()).or_reject()?.clone();
    let authorization = Authorization::new(&[request.request.clone()]);
    let private_key = PrivateKey::<N>::from_str("APrivateKey1zkp5EYonCQEWFuTA3mDDgdun3dQhp4pMXZs9wuSZAKzcHAr").unwrap();
    // Construct the call stack.
    let call_stack = CallStack::Authorize(vec![request.request], private_key, authorization.clone());
    // Initialize an RNG.
    let rng = &mut rand::thread_rng();
    // Construct the authorization from the function.
    let _response = stack.execute_function::<A, _>(call_stack, rng).or_reject()?;

    let mut fee = None;
    // Prepare the fees.
    if request.fee_request.is_some() {
        fee = match request.fee_record {
            Some(record) => {
                let record = Record::<N, Plaintext<N>>::from_str(&record).map_err(|e| anyhow!(e)).or_reject()?;
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
    let query = Some(Query::<N, C::BlockStorage>::from(conf[1].clone()));

    let result = execute_authorization_with_additional_fee::<N, A, C, ThreadRng>(&vm, &private_key, &request.fee_request, authorization, fee, query, rng, &stack).or_reject()?;
    println!("execute_function ok");
    Ok(result.to_string())
}

#[allow(clippy::too_many_arguments)]
fn execute_authorization_with_additional_fee<N: Network, A: Aleo<Network=N>, C: ConsensusStorage<N>, R: Rng + CryptoRng>(
    vm: &VM<N, C>,
    private_key: &PrivateKey<N>,
    fee_request: &Option<Request<N>>,
    authorization: Authorization<N>,
    additional_fee: Option<(Record<N, Plaintext<N>>, u64)>,
    query: Option<Query<N, C::BlockStorage>>,
    rng: &mut R,
    stack: &Stack<N>,
) -> anyhow::Result<Transaction<N>> {
    // Compute the execution.
    let (_response, execution, _metrics) = vm.execute(authorization, query.clone(), rng)?;

    let mut additional_fee_f = None;
    // Compute the additional fee, if it is present.
    if let Some(fee_request) = fee_request {
        additional_fee_f = match additional_fee {
            Some((_credits, _additional_fee_in_gates)) => {
                Some(execute_fee::<N, A, R, C>(vm, private_key, fee_request, query, rng, stack)?.1)
            }
            None => None,
        };
    }

    Transaction::from_execution(execution, additional_fee_f)
}

#[allow(clippy::type_complexity)]
fn execute_fee<N: Network, A: Aleo<Network=N>, R: Rng + CryptoRng, C: ConsensusStorage<N>>(
    vm: &VM<N, C>,
    private_key: &PrivateKey<N>,
    fee_request: &Request<N>,
    query: Option<Query<N, C::BlockStorage>>,
    rng: &mut R,
    stack: &Stack<N>,
) -> anyhow::Result<(Response<N>, Fee<N>, Vec<CallMetrics<N>>)> {
    println!("VM::execute_fee");

    // Prepare the query.
    let query = match query {
        Some(query) => query,
        None => Query::VM(vm.block_store().clone()),
    };

    println!("Process::execute_fee");

    // Ensure the fee has the correct program ID.
    let program_id = ProgramID::<N>::from_str("credits.aleo")?;
    // Ensure the fee has the correct function.
    let function_name = Identifier::<N>::from_str("fee")?;
    // Initialize the authorization.
    let authorization = Authorization::new(&[fee_request.clone()]);
    println!("Initialize the authorization");
    // Construct the call stack.
    let call_stack = CallStack::Authorize(vec![fee_request.clone()], *private_key, authorization.clone());
    // Construct the authorization from the function.
    let _response = stack.execute_function::<A, R>(call_stack, rng)?;
    println!("Construct the authorization from the function");

    // Retrieve the main request (without popping it).
    let request = authorization.peek_next()?;
    // Initialize the execution.
    let execution = Arc::new(RwLock::new(Execution::new()));
    // Initialize the inclusion.
    let inclusion = Arc::new(RwLock::new(Inclusion::new()));
    // Initialize the metrics.
    let metrics = Arc::new(RwLock::new(Vec::new()));
    // Initialize the call stack.
    let call_stack = CallStack::execute(authorization, execution.clone(), inclusion.clone(), metrics.clone())?;
    // Prepare the stack.
    let binding = vm.process();
    // Execute the circuit.
    let binding = binding.read();
    let stack = binding.get_stack(request.program_id())?;
    let response = stack.execute_function::<A, R>(call_stack, rng)?;
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
        let fee_transition = cast_ref!(fee_transition as Transition<N>);
        let inclusion = cast_ref!(inclusion as Inclusion<N>);
        inclusion.prepare_fee(fee_transition, query)?
    };
    let assignments = cast_ref!(assignments as Vec<InclusionAssignment<N>>);
    println!("Prepare the assignments");

    // Compute the inclusion proof and construct the fee.
    let fee = inclusion.prove_fee::<A, R>(fee_transition, assignments, rng)?;
    println!("Compute the inclusion proof and construct the fee");

    // Prepare the return.
    let response = cast_ref!(response as Response<N>).clone();
    let fee = cast_ref!(fee as Fee<N>).clone();
    let metrics = cast_ref!(metrics as Vec<CallMetrics<N>>).clone();
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
