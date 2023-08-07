# Pallet DIspute

## Objectives
- Provide a system whereby a jury of  trusted accounts can decide the outcome of a dispute over services.
- Incentivise the jury allowing a simple form of income.
- Allow for the fair dispute of briefs with a single owner.

## Abstract
Now we have the fellowship pallet nearly operationaly we can use it to form a Jury to handle disputes. The creation of this pallet is to allow us some more flexibility for each project, given each project should have a configuration defining the specifics on how a dispute is handled we can abstract away from FundingType and instead handle a project on a more individual basis. Soft deadline is 1st week of september as may be required for sub0. This could act as a form of income for people out fo work, it also allows the network to be fully autonomous and farily decentralised (assuming that the pool of freelancer and jury is large enough).

## Basic Requirements
1. Deduct the fee out of funds raised potentially supplimented by a one off fee.
2. Pick the jury randomly, if someone decidedes not to participate, pick another.
3. Must be a trait that other pallets can use to raise a dispute.
4. A Reason for the dispute must be given describing why it was raised.
5. A Jury type for a dispute must be given describing who within the dispute pallet will handle it.
6. (Milestone submission should be paused while this goes on)?
7. {ProjectConfig?, DisputeConfig?} struct must be used to decide i.e contains the Jury (see Refactors Required). 

## Calculations Required
When it comes to incentivising the freelancers, there requires some experimentation as it will be a balance between security and incentive. Where 1000 fellows voting on a dispute is more secure, they each get a smaller amount and vice versa. Less fellows, less security, more incentive. A graph and discussion will be required. (Although it will surely be linear).


## Refactors Required
In this PR we also need to depricate the use of funding type in favor of a new more flexibly alternative. For more details see: https://github.com/ImbueNetwork/imbue/issues/193.

## Game Theory Required
For requirements 1, 2 a small paragraph describing the game is required before implementation for discussion.

## Possible Implementation


```pub trait DisputeRaiser {}```   - Used to raise a dispute in another pallet implemented my pallet-dispute.

```pub trait JurySelector{}``` - used to select a random set of Jury or a single person, implemented by pallet-fellowship.

```
/// For deciding who will be the jury
pub enum Jury {
    Fellowship,
    Contributors(Vec<AccountId>),
    Canonical(AccountId),
}
```
## Future Ideas
- If either party is not happy with the outcome, they can pay more for it to go to a larger jury or higher ranking officials.
- Roles specifically for a judge or something.



