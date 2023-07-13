
use pallet_proposals_runtime_api::ProposalsApi as ProposalsRuntimeApi; 
use codec::{Codec, Decode};
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_rpc::number::NumberOrHex;
use sp_runtime::traits::{Block as BlockT};
use frame_system::Config;
use std::sync::Arc;

#[rpc(client, server)]
pub trait ProposalsApi<AccountId> {
	#[method(name = "proposals_getProjectKitty")]
	fn get_project_account_by_id(project_id: u32) -> RpcResult<AccountId>;
}

pub struct Proposals<C, B> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, P> Proposals<C, P> {
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i32 {
	fn from(e: Error) -> i32 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

#[async_trait]
impl<C, B, AccountId>
	ProposalsApiServer<AccountId> for Proposals<C, B>
where 
	C: sp_api::ProvideRuntimeApi<B>,
	C: HeaderBackend<B>,
	C: Send + Sync + 'static,
	C::Api: ProposalsRuntimeApi<AccountId>,
	Block: BlockT,
	AccountId: Clone + Display + Codec + Send + 'static,
{
	fn get_project_account_by_id(&self, project_id: u32) -> RpcResult<AccountId> {
		let api = self.client.runtime_api();
		api.get_project_account_by_id(project_id).map_err(runtime_error_into_rpc_err)
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	CallError::Custom(ErrorObject::owned(
		Error::RuntimeError,
		"Could not generate the account_id for the given project_id",
		Some(format!("{:?}", err)),
	))
	.into()
}