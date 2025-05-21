use std::{
    collections::HashMap,
    env,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Context, Result};
use rand::Rng;
use seda_common::{
    msgs::data_requests::{DataRequest, RevealBody},
    types::{ToHexStr, TryHashSelf},
};
use serde_json::json;
use xshell::{cmd, Shell};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

const TASKS: &[&str] = &[
    "cov",
    "cov-ci",
    "help",
    "proto-build",
    "proto-update",
    "tally-data-req-fixture",
    "test-all",
    "test-ci",
    "test-common",
    "test-contract",
    "wasm-opt",
];

fn try_main() -> Result<()> {
    let sh = Shell::new()?;
    // Ensure our working directory is the toplevel
    {
        let toplevel_path = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .context("Invoking git rev-parse")?;
        if !toplevel_path.status.success() {
            bail!("Failed to invoke git rev-parse");
        }
        let path_string = String::from_utf8(toplevel_path.stdout)?;
        let path = PathBuf::from(path_string.trim());
        sh.change_dir(path);
    }

    let task = env::args().nth(1);
    match task.as_deref() {
        Some("cov") => cov(&sh)?,
        Some("cov-ci") => cov_ci(&sh)?,
        Some("help") => print_help()?,
        Some("proto-build") => proto_build(&sh)?,
        Some("proto-update") => {
            let git = env::args().nth(2).expect("Missing git version");

            proto_update(&sh, &git)?
        }
        Some("tally-data-req-fixture") => tally_data_req_fixture(&sh)?,
        Some("test-all") => test_all(&sh)?,
        Some("test-ci") => test_ci(&sh)?,
        Some("test-common") => test_common(&sh)?,
        Some("test-contract") => test_contract(&sh)?,

        Some("wasm-opt") => wasm_opt(&sh)?,
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
    .env("GIT_REVISION", get_git_version()?)
    .run()?;
    Ok(())
}

fn wasm_opt(sh: &Shell) -> Result<()> {
    wasm(sh)?;
    cmd!(
			sh,
			"wasm-opt -Os --signext-lowering --enable-bulk-memory target/wasm32-unknown-unknown/release/seda_contract.wasm -o target/seda_contract.wasm"
		)
    .run()?;
    Ok(())
}

fn create_data_request(
    id: [u8; 32],
    exec_program_id: [u8; 32],
    tally_program_id: [u8; 32],
    replication_factor: u16,
    tally_inputs: Vec<u8>,
    reveals: HashMap<String, RevealBody>,
) -> DataRequest {
    DataRequest {
        version: semver::Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre:   semver::Prerelease::EMPTY,
            build: semver::BuildMetadata::EMPTY,
        },
        id: id.to_hex(),
        exec_program_id: exec_program_id.to_hex(),
        exec_inputs: Default::default(),
        exec_gas_limit: 10,
        tally_program_id: tally_program_id.to_hex(),
        tally_inputs: tally_inputs.into(),
        tally_gas_limit: 20,
        memo: Default::default(),
        replication_factor,
        consensus_filter: Default::default(),
        gas_price: 10u128.into(),
        seda_payload: Default::default(),
        commits: Default::default(),
        reveals,
        payback_address: Default::default(),
        height: rand::random(),
    }
}

fn tally_test_fixture(n: usize) -> Vec<DataRequest> {
    let exec_program_id: [u8; 32] = rand::random();
    let tally_program_id: [u8; 32] = rand::random();

    (0..n)
        .map(|_| {
            let inputs = [rand::rng().random_range::<u8, _>(1..=10); 5]
                .into_iter()
                .flat_map(|i| i.to_be_bytes())
                .collect();
            let replication_factor = rand::rng().random_range(1..=3);

            let dr_id: [u8; 32] = rand::random();
            let hex_dr_id = dr_id.to_hex();
            let reveals = (0..replication_factor)
                .map(|_| {
                    let reveal = RevealBody {
                        dr_id:             hex_dr_id.clone(),
                        dr_block_height:   1,
                        exit_code:         0,
                        gas_used:          10,
                        reveal:            rand::rng().random_range(1..=100u8).to_be_bytes().into(),
                        proxy_public_keys: vec![],
                    };

                    (
                        reveal
                            .try_hash()
                            .expect("Could not hash reveal due to base64 result")
                            .to_hex(),
                        reveal,
                    )
                })
                .collect();

            create_data_request(
                dr_id,
                exec_program_id,
                tally_program_id,
                replication_factor,
                inputs,
                reveals,
            )
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

fn get_git_version() -> Result<String> {
    let git_version = Command::new("git")
        .args(["describe", "--always", "--dirty=-modified", "--tags"])
        .output()
        .context("Invoking git describe")?;
    if !git_version.status.success() {
        return Ok("unknown".to_string());
    }

    let version = String::from_utf8(git_version.stdout)?;
    Ok(version)
}

fn test_common(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "cargo nextest run --locked -p seda-common --failure-output final --success-output final"
    )
    .run()?;
    cmd!(
        sh,
        "cargo nextest run --locked -p seda-common --failure-output final --success-output final --features cosmwasm"
    )
    .run()?;
    Ok(())
}

fn test_contract(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "cargo nextest run --locked -p seda-contract --failure-output final --success-output final"
    )
    .run()?;
    Ok(())
}

fn test_all(sh: &Shell) -> Result<()> {
    test_common(sh)?;
    test_contract(sh)?;
    Ok(())
}

fn test_ci(sh: &Shell) -> Result<()> {
    cmd!(sh, "cargo nextest run --locked -p seda-common -P ci").run()?;
    cmd!(
        sh,
        "cargo nextest run --locked -p seda-common -P ci --features cosmwasm"
    )
    .run()?;
    cmd!(sh, "cargo nextest run --locked -p seda-contract -P ci").run()?;
    Ok(())
}

fn cov(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "cargo llvm-cov -p seda-common -p seda-contract --locked --ignore-filename-regex contract/src/bin/* nextest -P ci"
    )
    .run()?;
    Ok(())
}

fn cov_ci(sh: &Shell) -> Result<()> {
    cmd!(
        sh,
        "cargo llvm-cov -p seda-common -p seda-contract --cobertura --output-path cobertura.xml --locked --ignore-filename-regex contract/src/bin/* nextest -P ci"
    )
    .run()?;
    Ok(())
}

fn proto_build(sh: &Shell) -> Result<()> {
    sh.change_dir("proto-common");
    let proto_folder = sh.current_dir();
    cmd!(sh, "echo {proto_folder}").run()?;
    cmd!(sh, "echo Generating Rust proto code").run()?;
    cmd!(sh, "buf generate --template buf.gen.rust.yaml").run()?;
    let mut cmd = sh
        .cmd("find")
        .arg("src/gen")
        .arg("-type")
        .arg("f")
        .arg("-name")
        .arg("*.rs")
        .arg("-exec")
        .arg("sed")
        .arg("-i");

    #[cfg(target_os = "macos")]
    {
        cmd = cmd.arg("");
    }

    cmd = cmd.arg("s/super::super::/super::/g; s/::v1::/::/g").arg("{}").arg("+");

    cmd.run()?;

    Ok(())
}

fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(src_path, dst_path)?;
        } else {
            fs::copy(src_path, dst_path)?;
        }
    }
    Ok(())
}

fn proto_update(sh: &Shell, git: &str) -> Result<()> {
    let archive_url = format!("https://codeload.github.com/sedaprotocol/seda-chain/tar.gz/{git}");

    let tmp_dir = sh.create_temp_dir()?;
    let tmp_path = tmp_dir.path();
    let archive_path = tmp_path.join("seda-chain.tar.gz");

    cmd!(sh, "curl -L {archive_url} -o {archive_path}").run()?;
    if !archive_path.exists() {
        bail!("Failed to download archive");
    }
    cmd!(sh, "tar -xzf {archive_path} --strip-components=1 -C {tmp_path}").run()?;

    let proto_dir = tmp_path.join("proto");
    if !proto_dir.exists() {
        bail!("Failed to extract repo, was git version correct?");
    }

    let dest_proto = sh.current_dir().join("proto-common").join("proto");

    if dest_proto.exists() {
        sh.remove_path(&dest_proto)?;
    }
    copy_dir_recursive(proto_dir, &dest_proto)?;

    let go_proto_gen_yaml = dest_proto.join("buf.gen.gogo.yaml");
    if go_proto_gen_yaml.exists() {
        sh.remove_path(&go_proto_gen_yaml)?;
    }

    let lib_rs_path = sh.current_dir().join("proto-common").join("src").join("lib.rs");
    let lib_rs_content = format!(
        r#"include!("gen/mod.rs");

pub const SEDA_CHAIN_VERSION: &str = "{}";
"#,
        git
    );
    sh.write_file(lib_rs_path, lib_rs_content)?;
    proto_build(sh)?;

    Ok(())
}
