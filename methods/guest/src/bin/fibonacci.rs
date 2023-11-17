// Copyright 2023 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_main]

use revm::{
    db::{CacheDB, EmptyDB},
    primitives::{address, ExecutionResult, TransactTo, U256},
    EVM,
};
use revm_primitives::{AccountInfo, Bytecode};
use risc0_zkvm::guest::env;
use serde_json::Value;
use std::io::Read;

// TODO this is too much data to include in the binary, only include necessary data (bytecode, ident)
const FORGE_OUTPUT: &str = include_str!("../../out/fib.sol/Fibonacci.json");

risc0_zkvm::guest::entry!(main);

fn main() {
    // Read data sent from the application contract.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    // TODO unclear if this should match Eth contract address or be unique
    let contract_addr = address!("1000000000000000000000000000000000000000");

    // initialise empty in-memory-db
    // TODO possibly could fill some values
    let mut cache_db = CacheDB::new(EmptyDB::default());

    // Get contract bytecode.
    let json: Value = serde_json::from_str(&FORGE_OUTPUT).unwrap();

    // Extract the function selector to prepend the input.
    let function_selector_hex = json["methodIdentifiers"]["fib(uint256)"]
        .as_str()
        .expect("Function identifier not found");

    // Convert the hex string function selector to bytes
    let mut input = hex::decode(function_selector_hex).expect("Failed to decode hex string");

    // Append the serialized parameters to the evm input.
    input.extend(input_bytes);

    // Extract the deployed bytecode (or bytecode, if needed)
    // Note: switch to deployedBytecode if setting storage manually
    let bytecode_hex = json["deployedBytecode"]["object"]
        .as_str()
        .expect("Bytecode not found")
        .trim_start_matches("0x");
    let bytecode_raw = hex::decode(bytecode_hex).unwrap();
    let bytecode = Bytecode::new_raw(bytecode_raw.into());

    // Insert the account info to set the bytecode of the contract
    // TODO might be better to do an actual deploy, instead of dealing with contstructor issues.
    cache_db.insert_account_info(
        contract_addr,
        AccountInfo {
            balance: U256::from(0),
            nonce: 0,
            // NOTE: just defaulting to not hashing code, not worth. Can switch if needed.
            // code_hash: bytecode.hash_slow(),
            code_hash: Default::default(),
            code: Some(bytecode),
        },
    );

    // initialise an empty (default) EVM
    let mut evm = EVM::new();
    evm.database(cache_db);

    // TODO deploy funcionality has compilation error:
    //    bonacci-fa5d95f51605b7b1.fibonacci.e6316d01-cgu.9.rcgu.o:(.text._start+0x14): relocation R_RISCV_JAL out of range: 1094344 is not in [-1048576, 1048575]; references __start
    // // Deploy contract transaction.
    // evm.env.tx.caller = address!("0000000000000000000000000000000000000000");
    // evm.env.tx.transact_to = TransactTo::create(); // Indicate a contract creation
    // evm.env.tx.data = bytecode_raw.into(); // Use the creation bytecode
    // evm.env.tx.value = U256::from(0);

    // let ref_tx = evm.transact().unwrap();
    // let deploy_result = ref_tx.result;

    // let contract_addr = match deploy_result {
    //     ExecutionResult::Success { .. } => {
    //         if let ExecutionResult::Success {
    //             output: Output::Create(_bz, Some(addr)),
    //             ..
    //         } = deploy_result
    //         {
    //             addr
    //         } else {
    //             panic!("Deployment failed: {deploy_result:?}")
    //         }
    //     }
    //     result => panic!("Deployment failed: {result:?}"),
    // };

    // TODO: Probably just copy whichever env variables from Eth to here.
    evm.env.tx.caller = address!("0000000000000000000000000000000000000000");
    evm.env.tx.transact_to = TransactTo::Call(contract_addr);
    evm.env.tx.data = input.into();
    evm.env.tx.value = U256::from(0);

    // execute transaction without writing to the DB
    let ref_tx = evm.transact_ref().unwrap();
    // select ExecutionResult struct
    let result = ref_tx.result;

    // unpack output call enum into raw bytes
    let value = match result {
        ExecutionResult::Success { output, .. } => output.into_data(),
        result => panic!("Execution failed: {result:?}"),
    };

    env::commit_slice(&value);
}
