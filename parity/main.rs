// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Ethcore client application.

#![warn(missing_docs)]
#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]
#![cfg_attr(feature="dev", allow(useless_format))]
#![cfg_attr(feature="dev", allow(match_bool))]

extern crate docopt;
extern crate num_cpus;
extern crate rustc_serialize;
extern crate ethabi;
extern crate ethcore_devtools as devtools;
extern crate ethcore;
extern crate ethsync;
extern crate env_logger;
extern crate ethcore_logger;
extern crate ctrlc;
extern crate fdlimit;
extern crate time;
extern crate number_prefix;
extern crate rpassword;
extern crate semver;
extern crate ethcore_io as io;
extern crate ethcore_ipc as ipc;
extern crate ethcore_ipc_nano as nanoipc;
extern crate serde;
extern crate serde_json;
extern crate rlp;
extern crate ethcore_hash_fetch as hash_fetch;
extern crate ethcore_light as light;

extern crate ethcore_ipc_hypervisor as hypervisor;
extern crate ethcore_rpc;

extern crate ethcore_signer;
extern crate ansi_term;

extern crate regex;
extern crate isatty;
extern crate toml;

#[macro_use]
extern crate ethcore_util as util;
#[macro_use]
extern crate log as rlog;
#[macro_use]
extern crate hyper; // for price_info.rs
#[macro_use]
extern crate lazy_static;

#[cfg(feature="stratum")]
extern crate ethcore_stratum;

#[cfg(feature = "dapps")]
extern crate ethcore_dapps;

macro_rules! dependency {
	($dep_ty:ident, $url:expr) => {
		{
			let dep = boot::dependency::<$dep_ty<_>>($url)
				.unwrap_or_else(|e| panic!("Fatal: error connecting service ({:?})", e));
			dep.handshake()
				.unwrap_or_else(|e| panic!("Fatal: error in connected service ({:?})", e));
			dep
		}
	}
}

mod cache;
mod upgrade;
mod rpc;
mod dapps;
mod informant;
mod cli;
mod configuration;
mod migration;
mod signer;
mod rpc_apis;
mod url;
mod helpers;
mod params;
mod deprecated;
mod dir;
mod modules;
mod account;
mod blockchain;
mod presale;
mod snapshot;
mod run;
#[cfg(feature="ipc")]
mod sync;
#[cfg(feature="ipc")]
mod boot;
mod user_defaults;
mod updater;
mod operations;

#[cfg(feature="stratum")]
mod stratum;

use std::{process, env};
use std::collections::HashMap;
use std::io::{self as stdio, BufReader, Write};
use std::fs::File;
use std::path::PathBuf;
use util::sha3::sha3;
use cli::Args;
use configuration::{Cmd, Execute, Configuration};
use deprecated::find_deprecated;
use ethcore_logger::setup_log;

fn print_hash_of(maybe_file: Option<String>) -> Result<String, String> {
	if let Some(file) = maybe_file {
		let mut f = BufReader::new(try!(File::open(&file).map_err(|_| "Unable to open file".to_owned())));
		let hash = try!(sha3(&mut f).map_err(|_| "Unable to read from file".to_owned()));
		Ok(hash.hex())
	} else {
		Err("Streaming from standard input not yet supported. Specify a file.".to_owned())
	}
}

enum PostExecutionAction {
	Print(String),
	Restart,
	Quit,
}

fn execute(command: Execute) -> Result<PostExecutionAction, String> {
	let logger = setup_log(&command.logger).expect("Logger is initialized only once; qed");

	match command.cmd {
		Cmd::Run(run_cmd) => {
			let restart = run::execute(run_cmd, logger)?;
			Ok(if restart { PostExecutionAction::Restart } else { PostExecutionAction::Quit })
		},
		Cmd::Version => Ok(PostExecutionAction::Print(Args::print_version())),
		Cmd::Hash(maybe_file) => print_hash_of(maybe_file).map(|s| PostExecutionAction::Print(s)),
		Cmd::Account(account_cmd) => account::execute(account_cmd).map(|s| PostExecutionAction::Print(s)),
		Cmd::ImportPresaleWallet(presale_cmd) => presale::execute(presale_cmd).map(|s| PostExecutionAction::Print(s)),
		Cmd::Blockchain(blockchain_cmd) => blockchain::execute(blockchain_cmd).map(|s| PostExecutionAction::Print(s)),
		Cmd::SignerToken(signer_cmd) => signer::execute(signer_cmd).map(|s| PostExecutionAction::Print(s)),
		Cmd::Snapshot(snapshot_cmd) => snapshot::execute(snapshot_cmd).map(|s| PostExecutionAction::Print(s)),
	}
}

fn start() -> Result<PostExecutionAction, String> {
	let args: Vec<String> = env::args().collect();
	let conf = Configuration::parse(&args).unwrap_or_else(|e| e.exit());

	let deprecated = find_deprecated(&conf.args);
	for d in deprecated {
		println!("{}", d);
	}

	let cmd = try!(conf.into_command());
	execute(cmd)
}

#[cfg(not(feature="stratum"))]
fn stratum_main(_: &mut HashMap<String, fn()>) {}

#[cfg(feature="stratum")]
fn stratum_main(alt_mains: &mut HashMap<String, fn()>) {
	alt_mains.insert("stratum".to_owned(), stratum::main);
}

#[cfg(not(feature="ipc"))]
fn sync_main(_: &mut HashMap<String, fn()>) {}

#[cfg(feature="ipc")]
fn sync_main(alt_mains: &mut HashMap<String, fn()>) {
	alt_mains.insert("sync".to_owned(), sync::main);
}

// TODO: merge with version in Updater.
fn updates_latest() -> PathBuf {
	let mut dest = PathBuf::from(env::home_dir().unwrap().to_str().expect("env filesystem paths really should be valid; qed"));
	dest.push(".parity-updates");
	dest.push("parity");
	dest
}

// Starts ~/.parity-updates/parity and returns the code it exits with.
fn run_parity() -> Option<i32> {
	let exe = updates_latest();
	process::Command::new(exe)
		.args(&env::args_os().collect::<Vec<_>>())
		.status()
		.map(|es| es.code().unwrap_or(128))
		.ok()
}

const PLEASE_RESTART_EXIT_CODE: i32 = 69;

// Run our version of parity.
// Returns the exit error code. 
fn main_direct() -> i32 {
	let mut alt_mains = HashMap::new();
	sync_main(&mut alt_mains);
	stratum_main(&mut alt_mains);
	if let Some(f) = std::env::args().nth(1).and_then(|arg| alt_mains.get(&arg.to_string())) {
		f();
		0
	} else {
		match start() {
			Ok(result) => match result {
				PostExecutionAction::Print(s) => { info!("{}", s); 0 },
				PostExecutionAction::Restart => PLEASE_RESTART_EXIT_CODE,
				PostExecutionAction::Quit => 0,
			},
			Err(err) => {
				writeln!(&mut stdio::stderr(), "{}", err).expect("StdErr available; qed");
				1
			},
		}
	}
}

fn println_trace_main(s: String) {
	if env::var("RUST_LOG").ok().and_then(|s| s.find("main=trace")).is_some() {
		println!("{}", s);
	}
}

#[macro_export]
macro_rules! trace_main {
	($arg:expr) => (println_trace_main($arg.into()));
	($($arg:tt)*) => (println_trace_main(format!("{}", format_args!($($arg)*))));
}

fn main() {
	// Always print backtrace on panic.
	env::set_var("RUST_BACKTRACE", "1");

	// assuming the user is not running with `--force-direct`, then:
	// if argv[0] == "parity" and this executable != ~/.parity-updates/parity, run that instead.
	let force_direct = std::env::args().any(|arg| arg == "--force-direct");
	let exe = std::env::current_exe().ok();
	let development = exe.as_ref().and_then(|p| p.parent().and_then(|p| p.parent()).and_then(|p| p.file_name()).map(|n| n == "target")).unwrap_or(false);
	let same_name = exe.as_ref().and_then(|p| p.file_stem().map(|s| s == "parity")).unwrap_or(false);
	let have_update = updates_latest().exists();
	let is_non_updated_current = exe.map_or(false, |p| p.canonicalize().ok() != updates_latest().canonicalize().ok());
	trace_main!("Starting up {} (force-direct: {}, development: {}, same-name: {}, have-update: {}, non-updated-current: {})", std::env::current_exe().map(|x| format!("{}", x.display())).unwrap_or("<unknown>".to_owned()), force_direct, development, same_name, have_update, is_non_updated_current);
	if !force_direct && !development && same_name && have_update && is_non_updated_current {
		// looks like we're not running ~/.parity-updates/parity when the user is expecting otherwise.
		// Everything run inside a loop, so we'll be able to restart from the child into a new version seamlessly. 
		loop {
			// If we fail to run the updated parity then fallback to local version. 
			trace_main!("Attempting to run latest update...");
			let exit_code = run_parity().unwrap_or_else(|| { trace_main!("Falling back to local..."); main_direct() });
			trace_main!("Latest exited with {}", exit_code);
			if exit_code != PLEASE_RESTART_EXIT_CODE {
				trace_main!("Quitting...");
				process::exit(exit_code);
			}
			trace_main!("Rerunning...");
		}
	} else {
		trace_main!("Running direct");
		// Otherwise, we're presumably running the version we want. Just run and fall-through.
		process::exit(main_direct());
	}
}
