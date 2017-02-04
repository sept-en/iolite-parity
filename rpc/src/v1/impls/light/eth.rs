// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

//! Eth RPC interface for the light client.

use std::sync::Arc;

use jsonrpc_core::Error;
use jsonrpc_macros::Trailing;

use light::client::Client as LightClient;
use light::cht;
use light::on_demand::{request, OnDemand};

use ethcore::account_provider::{AccountProvider, DappId};
use ethcore::basic_account::BasicAccount;
use ethcore::encoded;
use ethcore::ids::BlockId;
use ethsync::LightSync;

use futures::{future, Future, BoxFuture};

use v1::helpers::{CallRequest as CRequest, errors, limit_logs};
use v1::helpers::dispatch::{dispatch_transaction, default_gas_price};
use v1::helpers::block_import::is_major_importing;
use v1::traits::Eth;
use v1::types::{
	RichBlock, Block, BlockTransactions, BlockNumber, Bytes, SyncStatus, SyncInfo,
	Transaction, CallRequest, Index, Filter, Log, Receipt, Work,
	H64 as RpcH64, H256 as RpcH256, H160 as RpcH160, U256 as RpcU256,
};
use v1::metadata::Metadata;

use util::Address;

/// Light client `ETH` RPC.
pub struct EthClient {
	sync: Arc<LightSync>,
	client: Arc<LightClient>,
	on_demand: Arc<OnDemand>,
	accounts: Arc<AccountProvider>,
}

// helper for a specific kind of internal error.
fn err_no_context() -> Error {
	errors::internal("network service detached", "")
}

impl EthClient {
	/// Create a new `EthClient` with a handle to the light sync instance, client,
	/// and on-demand request service, which is assumed to be attached as a handler.
	pub fn new(
		sync: Arc<LightSync>,
		client: Arc<LightClient>,
		on_demand: Arc<OnDemand>,
		accounts: Arc<AccountProvider>,
	) -> Self {
		EthClient {
			sync: sync,
			client: client,
			on_demand: on_demand,
			accounts: accounts,
		}
	}

	/// Get a block header from the on demand service or client, or error.
	fn header(&self, id: BlockId) -> BoxFuture<encoded::Header, Error> {
		if let Some(h) = self.client.get_header(id) {
			return future::ok(h).boxed()
		}

		let maybe_future = match id {
			BlockId::Number(n) => {
				let cht_root = cht::block_to_cht_number(n).and_then(|cn| self.client.cht_root(cn as usize));
				match cht_root {
					None => return future::err(errors::unknown_block()).boxed(),
					Some(root) => {
						let req = request::HeaderByNumber {
							num: n,
							cht_root: root,
						};

						self.sync.with_context(|ctx|
							self.on_demand.header_by_number(ctx, req)
								.map(|(h, _)| h)
								.map_err(errors::from_on_demand_error)
								.boxed()
						)
					}
				}
			}
			BlockId::Hash(h) => {
				self.sync.with_context(|ctx|
					self.on_demand.header_by_hash(ctx, request::HeaderByHash(h))
						.map_err(errors::from_on_demand_error)
						.boxed()
				)
			}
			_ => None, // latest, earliest, and pending will have all already returned.
		};

		// todo: cache returned values (header, TD)
		match maybe_future {
			Some(recv) => recv,
			None => future::err(err_no_context()).boxed()
		}
	}

	// helper for getting account info.
	fn account(&self, address: Address, id: BlockId) -> BoxFuture<BasicAccount, Error> {
		let (sync, on_demand) = (self.sync.clone(), self.on_demand.clone());

		self.header(id).and_then(move |header| {
			sync.with_context(|ctx| on_demand.account(ctx, request::Account {
				header: header,
				address: address,
			}))
				.map(|x| x.map_err(errors::from_on_demand_error).boxed())
				.unwrap_or_else(|| future::err(err_no_context()).boxed())
		}).boxed()
	}
}

impl Eth for EthClient {
	type Metadata = Metadata;

	fn protocol_version(&self) -> Result<String, Error> {
		Ok(format!("{}", ::light::net::MAX_PROTOCOL_VERSION))
	}

	fn syncing(&self) -> Result<SyncStatus, Error> {
		rpc_unimplemented!()
	}

	fn author(&self, _meta: Self::Metadata) -> BoxFuture<RpcH160, Error> {
		future::ok(Default::default()).boxed()
	}

	fn is_mining(&self) -> Result<bool, Error> {
		Ok(false)
	}

	fn hashrate(&self) -> Result<RpcU256, Error> {
		Ok(Default::default())
	}

	fn gas_price(&self) -> Result<RpcU256, Error> {
		Ok(Default::default())
	}

	fn accounts(&self, meta: Metadata) -> BoxFuture<Vec<RpcH160>, Error> {
		let dapp: DappId = meta.dapp_id.unwrap_or_default().into();

		let accounts = self.accounts
			.note_dapp_used(dapp.clone())
			.and_then(|_| self.accounts.dapps_addresses(dapp))
			.map_err(|e| errors::internal("Could not fetch accounts.", e))
			.map(|accs| accs.into_iter().map(Into::<RpcH160>::into).collect());

		future::done(accounts).boxed()
	}

	fn block_number(&self) -> Result<RpcU256, Error> {
		Ok(self.client.chain_info().best_block_number.into())
	}

	fn balance(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		self.account(address.into(), num.0.into()).map(|acc| acc.balance.into()).boxed()
	}

	fn storage_at(&self, address: RpcH160, key: RpcU256, num: Trailing<BlockNumber>) -> BoxFuture<RpcH256, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_by_hash(&self, hash: RpcH256, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_by_number(&self, num: BlockNumber, include_txs: bool) -> BoxFuture<Option<RichBlock>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn transaction_count(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<RpcU256, Error> {
		self.account(address.into(), num.0.into()).map(|acc| acc.nonce.into()).boxed()
	}

	fn block_transaction_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_transaction_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_uncles_count_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcU256>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn block_uncles_count_by_number(&self, num: BlockNumber) -> BoxFuture<Option<RpcU256>, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn code_at(&self, address: RpcH160, num: Trailing<BlockNumber>) -> BoxFuture<Bytes, Error> {
		future::err(errors::unimplemented(None)).boxed()
	}

	fn send_raw_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		Err(errors::unimplemented(None))
	}

	fn submit_transaction(&self, raw: Bytes) -> Result<RpcH256, Error> {
		Err(errors::unimplemented(None))
	}

	fn call(&self, req: CallRequest, num: Trailing<BlockNumber>) -> Result<Bytes, Error> {
		Err(errors::unimplemented(None))
	}

	fn estimate_gas(&self, req: CallRequest, num: Trailing<BlockNumber>) -> Result<RpcU256, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_hash(&self, hash: RpcH256) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_block_hash_and_index(&self, hash: RpcH256, idx: Index) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> Result<Option<Transaction>, Error> {
		Err(errors::unimplemented(None))
	}

	fn transaction_receipt(&self, hash: RpcH256) -> Result<Option<Receipt>, Error> {
		Err(errors::unimplemented(None))
	}

	fn uncle_by_block_hash_and_index(&self, hash: RpcH256, idx: Index) -> Result<Option<RichBlock>, Error> {
		Err(errors::unimplemented(None))
	}

	fn uncle_by_block_number_and_index(&self, num: BlockNumber, idx: Index) -> Result<Option<RichBlock>, Error> {
		Err(errors::unimplemented(None))
	}

	fn compilers(&self) -> Result<Vec<String>, Error> {
		Err(errors::unimplemented(None))
	}

	fn compile_lll(&self, _code: String) -> Result<Bytes, Error> {
		Err(errors::unimplemented(None))
	}

	fn compile_solidity(&self, _code: String) -> Result<Bytes, Error> {
		Err(errors::unimplemented(None))
	}

	fn compile_serpent(&self, _code: String) -> Result<Bytes, Error> {
		Err(errors::unimplemented(None))
	}

	fn logs(&self, _filter: Filter) -> Result<Vec<Log>, Error> {
		Err(errors::unimplemented(None))
	}

	fn work(&self, _timeout: Trailing<u64>) -> Result<Work, Error> {
		Err(errors::unimplemented(None))
	}

	fn submit_work(&self, _nonce: RpcH64, _pow_hash: RpcH256, _mix_hash: RpcH256) -> Result<bool, Error> {
		Err(errors::unimplemented(None))
	}

	fn submit_hashrate(&self, _rate: RpcU256, _id: RpcH256) -> Result<bool, Error> {
		Err(errors::unimplemented(None))
	}
}
