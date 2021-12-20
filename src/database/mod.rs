// Generic database code
pub mod postgres;

use tallystick::RankedCandidate;

use crate::error::ErrorKind;
use crate::poll::{Poll, RankedChoiceVote};

/// Generic database trait; an implementation be used with Rocket for fun stuff
///
/// See https://rocket.rs/v0.5-rc/guide/state/
pub trait Database<T> {
    fn get_poll_by_id(conn: T, id: &String) -> Result<Option<Poll>, ErrorKind>;
    fn add_poll(conn: T, poll: &Poll) -> Result<(), ErrorKind>;
    fn add_vote_to_poll(conn: T, id: &String, vote: &RankedChoiceVote) -> Result<(), ErrorKind>;
    fn set_poll_winners(
        conn: T,
        id: &String,
        winners: &[RankedCandidate<String>],
    ) -> Result<(), ErrorKind>;
}
