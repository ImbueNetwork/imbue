

pub trait DemocracyHandle<AccountId> {
    fn initiate_shortlist_vote() -> ();
    fn cancel_shortlist_vote() -> ();
}

pub trait FellowshipHandle<AccountId> {
    fn add_to_fellowship() -> ();

}