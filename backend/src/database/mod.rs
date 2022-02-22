// Generic database code
pub mod postgres;

// Generic database trait; an implementation be used with Rocket for fun stuff
//
// See https://rocket.rs/v0.5-rc/guide/state/
// Async traits aren't yet stable so we can't use this :(

// pub trait Database {
//     fn get_poll_by_id(&mut self, id: &String) -> Result<Option<Poll>, ErrorKind>;
//     fn add_poll(&mut self, poll: &Poll) -> Result<(), ErrorKind>;
//     fn add_vote_to_poll(&mut self, id: &String, vote: &RankedChoiceVote) -> Result<(), ErrorKind>;
//     fn set_poll_winners(
//         &mut self,
//         id: &String,
//         winners: &[RankedCandidate<String>],
//     ) -> Result<(), ErrorKind>;
// }
