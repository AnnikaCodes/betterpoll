//! Postgres database code

use tallystick::RankedCandidate;

use super::Database;
use crate::{
    error::{ErrorKind, InternalError},
    poll::{*},
};

pub struct PostgresDatabase;

impl Database<&mut postgres::Client> for PostgresDatabase {
    fn get_poll_by_id(conn: &mut postgres::Client, id: String) -> Result<Option<Poll>, ErrorKind> {
        let poll_row = match conn
            .query("SELECT * FROM polls WHERE id = $1 LIMIT 1", &[&id])?
            .pop()
        {
            Some(row) => row,
            None => return Ok(None), // No poll by that ID
        };

        let id = poll_row.try_get("id")?;
        let method_discrim: i32 = poll_row.try_get("method")?;
        let method = match method_discrim {
            0 => VotingMethod::Schulze,
            _ => {
                return Err(ErrorKind::Internal(
                    InternalError::UnknownVotingMethodDiscriminant(method_discrim),
                ))
            }
        };
        let num_winners: i32 = poll_row.try_get("num_winners")?;

        let winners_rows = conn.query("SELECT * FROM winners WHERE poll_id = $1", &[&id])?;
        let mut winners_vec = Vec::with_capacity(winners_rows.len());
        for row in winners_rows {
            let candidate: String = row.try_get("candidate")?;
            let rank: i32 = row.try_get("rank")?;
            winners_vec.push(RankedCandidate {
                candidate,
                rank: match rank.try_into() {
                    Ok(rank) => rank,
                    Err(err) => {
                        return Err(ErrorKind::Internal(InternalError::InvalidWinnerRank(
                            rank, err,
                        )))
                    }
                },
            });
        }
        let winners = if winners_vec.is_empty() {
            None
        } else {
            Some(winners_vec)
        };

        let num_winners = match num_winners.try_into() {
            Ok(num_winners) => num_winners,
            Err(e) => {
                return Err(ErrorKind::Internal(InternalError::InvalidNumWinners(
                    num_winners,
                    e,
                )))
            }
        };

        let votes_rows = conn.query("SELECT preferences FROM votes WHERE poll_id = $1", &[&id])?;
        let mut votes = Vec::with_capacity(votes_rows.len());
        for row in votes_rows {
            let preferences = row.try_get("preferences")?;
            votes.push(RankedChoiceVote::from_vec(preferences));
        }

        Ok(Some(Poll {
            id,
            title: poll_row.try_get("title")?,
            candidates: poll_row.try_get("candidates")?,
            creation_time: poll_row.try_get("created_at")?,
            end_time: poll_row.try_get("expires_at")?,
            num_winners,
            winners,
            votes,
            method,
        }))
    }

    fn add_poll(_conn: &mut postgres::Client, _poll: Poll) -> Result<(), ErrorKind> {
        unimplemented!();
    }

    fn add_vote_to_poll(
        _conn: &mut postgres::Client,
        _id: String,
        _vote: RankedChoiceVote,
    ) -> Result<(), ErrorKind> {
        unimplemented!();
    }

    fn set_poll_winners(
        _conn: &mut postgres::Client,
        _id: String,
        _winners: Vec<String>,
    ) -> Result<(), ErrorKind> {
        unimplemented!();
    }
}
