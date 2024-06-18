use std::{collections::HashMap, env, process::Command};

use anyhow::{bail, Context, Result};
use rand::Rng;
use seda_common::{
    msgs::data_requests::{DataRequest, RevealBody},
    types::{HashSelf, ToHexStr},
};
use serde_json::json;
use xshell::{cmd, Shell};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

const TASKS: &[&str] = &["help", "wasm", "wasm-opt", "tally-data-req-fixture"];

fn try_main() -> Result<()> {
    // Ensure our working directory is the toplevel
    {
        let toplevel_path = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Invoking git rev-parse")?;
        if !toplevel_path.status.success() {
            bail!("Failed to invoke git rev-parse");
        }
        let path = String::from_utf8(toplevel_path.stdout)?;
        std::env::set_current_dir(path.trim()).context("Changing to toplevel")?;
    }

    let task = env::args().nth(1);
    let sh = Shell::new()?;
    match task.as_deref() {
        Some("help") => print_help()?,
        Some("wasm") => wasm(&sh)?,
        Some("wasm-opt") => wasm_opt(&sh)?,
        Some("tally-data-req-fixture") => tally_data_req_fixture(&sh)?,
        _ => print_help()?,
    }

    Ok(())
}

fn print_help() -> Result<()> {
    println!("Tasks:");
    for name in TASKS {
        println!("  - {name}");
    }
    Ok(())
}

fn wasm(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "cargo build -p seda-contract --release --lib --target wasm32-unknown-unknown --locked"
    )
    .env("RUSTFLAGS", "-C link-arg=-s")
    .run()?;
    Ok(())
}

fn wasm_opt(sh: &Shell) -> Result<()> {
    wasm(sh)?;
    cmd!(
			sh,
			"wasm-opt -Os --signext-lowering target/wasm32-unknown-unknown/release/seda_contract.wasm -o target/seda_contract.wasm"
		).run()?;
    Ok(())
}

fn create_data_request(
    dr_binary_id: [u8; 32],
    tally_binary_id: [u8; 32],
    replication_factor: u16,
    tally_inputs: Vec<u8>,
    reveals: HashMap<String, RevealBody>,
) -> DataRequest {
    let id: [u8; 32] = rand::random();
    DataRequest {
        version: semver::Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre:   semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        },
        id: id.to_hex(),
        dr_binary_id: dr_binary_id.to_hex(),
        tally_binary_id: tally_binary_id.to_hex(),
        dr_inputs: Default::default(),
        tally_inputs: tally_inputs.into(),
        memo: Default::default(),
        replication_factor,
        gas_price: 10u128.into(),
        gas_limit: 20u128.into(),
        seda_payload: Default::default(),
        commits: Default::default(),
        reveals,
        payback_address: Default::default(),
        height: rand::random(),
    }
}

fn tally_test_fixture(n: usize) -> Vec<DataRequest> {
    let dr_binary_id: [u8; 32] = rand::random();
    let tally_binary_id: [u8; 32] = rand::random();

    (0..n)
        .map(|_| {
            let inputs = [rand::thread_rng().gen_range::<u8, _>(1..=10); 5]
                .into_iter()
                .flat_map(|i| i.to_be_bytes())
                .collect();
            let replication_factor = rand::thread_rng().gen_range(1..=3);

            let salt: [u8; 32] = rand::random();
            let reveals = (0..replication_factor)
                .map(|_| {
                    let reveal = RevealBody {
                        salt:      salt.to_hex(),
                        exit_code: 0,
                        gas_used:  10u128.into(),
                        reveal:    rand::thread_rng().gen_range(1..=100u8).to_be_bytes().into(),
                    };

                    (reveal.hash().to_hex(), reveal)
                })
                .collect();

            create_data_request(dr_binary_id, tally_binary_id, replication_factor, inputs, reveals)
        })
        .collect()
}

fn tally_data_req_fixture(_sh: &Shell) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open("tally_data_request_fixture.json")?;

    serde_json::to_writer(
        file,
        &json!({
            "test_one_dr_ready_to_tally": tally_test_fixture(1),
            "test_two_dr_ready_to_tally": tally_test_fixture(2),
        }),
    )?;

    Ok(())
}
