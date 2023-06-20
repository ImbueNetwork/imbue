

/// The democracy handle is used to inititate the regular referenda for new applicants into the fellowship.
pub trait DemocracyHandle<AccountId> {
    fn initiate_shortlist_vote() -> ();
    fn cancel_shortlist_vote() -> ();
}

/// Used by external pallets that decide when to add and remove members from the fellowship.
/// Makes a ying/yang with the democracy handle :)
pub trait FellowshipHandle<AccountId> {
    fn add_to_fellowship() -> ();
    fn revoke_fellowship() -> ();
    fn slash_fellowship_deposit() -> ();
}