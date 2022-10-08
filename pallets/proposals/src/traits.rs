
// a state flow diagram would be useful.

// what is the flow between roudn types. for example voting before contributing.


pub enum RoundType {
    ContributionRound,
    VotingRound,
    VoteOfNoConfidence,
}

// or perhaps

// in the new round method you pass round type
// and there is a match arm handling each type

//this way we can clearly specify the implementation of all the rounds






