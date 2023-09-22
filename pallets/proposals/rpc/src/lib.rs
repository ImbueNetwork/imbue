use codec::Codec;

use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
pub use pallet_proposals_rpc_runtime_api::ProposalsApi as ProposalsRuntimeApi;

use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use sp_std::vec::Vec;

use std::fmt::Display;
use std::sync::Arc;

// Runtime api return type.

#[rpc(client, server)]
pub trait ProposalsApi<BlockHash, AccountId: Ord, Balance>
where
    AccountId: Ord,
{
    #[method(name = "proposals_getProjectKitty")]
    fn project_account_id(&self, project_id: u32) -> RpcResult<AccountId>;
    #[method(name = "proposals_getProjectIndividualVotes")]
    fn project_individuals_votes(&self, project_id: u32) -> RpcResult<Vec<u8>>;
}

pub struct Proposals<C, B> {
    /// Shared reference to the client.
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

impl<C, P> Proposals<C, P> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
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

impl<C, B, AccountId, Balance> ProposalsApiServer<<B as BlockT>::Hash, AccountId, Balance>
    for Proposals<C, B>
where
    C: sp_api::ProvideRuntimeApi<B>,
    C: HeaderBackend<B>,
    C: Send + Sync + 'static,
    C::Api: ProposalsRuntimeApi<B, AccountId, Balance>,
    B: BlockT,
    AccountId: Clone + Display + Codec + Send + 'static + Ord,
{
    fn project_account_id(&self, project_id: u32) -> RpcResult<AccountId> {
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;

        api.get_project_account_by_id(at, project_id)
            .map_err(runtime_error_into_rpc_err)
    }
    fn project_individuals_votes(&self, project_id: u32) -> RpcResult<Vec<u8>> {
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;

        api.get_project_individuals_votes(at, project_id)
            .map_err(runtime_error_into_rpc_err)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
    CallError::Custom(ErrorObject::owned(
        Error::RuntimeError.into(),
        "Could not generate the account_id for the given project_id",
        Some(format!("{err:?}")),
    ))
    .into()
}
