// Generic database code

use crate::poll::{Poll, PollInProgress, RankedChoiceVote};

pub enum DatabaseError {}

/// Generic database trait; an implementation be used with Rocket for fun stuff
///
/// See https://rocket.rs/v0.5-rc/guide/state/
pub trait Database<T> {
    fn get_poll_by_id(connection: T, id: &str) -> Option<Poll>;
    fn add_poll(connection: T, poll: PollInProgress) -> Result<(), DatabaseError>;
    fn add_vote_to_poll(connection: T, id: &str, vote: RankedChoiceVote) -> Result<(), DatabaseError>;
    fn set_poll_winners(connection: T, id: &str, winners: Vec<String>) -> Result<(), DatabaseError>;
}
