use codec::Codec;
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
pub use pallet_proposals_rpc_runtime_api::ProposalsApi as ProposalsRuntimeApi;
use pallet_proposals::MilestoneKey;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;
use frame_support::{BoundedBTreeMap, pallet_prelude::Get, Serialize};
use sp_api::Decode;

use std::fmt::Display;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
type IndividualVotes<AccountId, Balance, MaxContributors, MaxMilestones> = BoundedBTreeMap<u32, BoundedBTreeMap<AccountId, (bool, Balance), MaxContributors>, MaxMilestones>

#[rpc(client, server)]
pub trait ProposalsApi<BlockHash, AccountId, Balance, MaxMilestones: Get<u32>, MaxContributors: Get<u32>> {

    #[method(name = "proposals_getProjectKitty")]
    fn project_account_id(&self, project_id: u32) -> RpcResult<AccountId>;
    #[method(name = "proposals_getProjectIndividualVotes")]
    fn project_individuals_votes(
        &self,
        project_id: u32,
    ) -> RpcResult<IndividualVotes<AccountId, Balance, MaxContributors, MaxMilestones>>;
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

impl<C, B, AccountId, Balance, MilestoneBound, ContributorBound> ProposalsApiServer<<B as BlockT>::Hash, AccountId, Balance, MilestoneBound, ContributorBound>
    for Proposals<C, B>
where
    C: sp_api::ProvideRuntimeApi<B>,
    C: HeaderBackend<B>,
    C: Send + Sync + 'static,
    C::Api: ProposalsRuntimeApi<B, AccountId, Balance, MilestoneBound, ContributorBound>,
    B: BlockT,
    AccountId: Clone + Display + Codec + Send + 'static,
    MilestoneBound: Get<u32>,
    ContributorBound: Get<u32>,
    BoundedBTreeMap<u32, BoundedBTreeMap<AccountId, (bool, Balance), ContributorBound>, MilestoneBound>: Decode + Serialize
{
    fn project_account_id(&self, project_id: u32) -> RpcResult<AccountId> {
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;

        api.get_project_account_by_id(at, project_id)
            .map_err(runtime_error_into_rpc_err)
    }
    fn project_individuals_votes(&self, project_id: u32) -> RpcResult<BoundedBTreeMap<u32, BoundedBTreeMap<AccountId, (bool, Balance), ContributorBound>, MilestoneBound>> {
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
