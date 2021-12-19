//! Postgres database code
//!
//! TODO: can we prepare statements? investigate how this'd work with Rocket



use tallystick::RankedCandidate;

use super::Database;
use crate::{
    error::{ErrorKind, InternalError},
    poll::*,
};

pub struct PostgresDatabase;

impl Database<&mut postgres::Client> for PostgresDatabase {
    fn get_poll_by_id(conn: &mut postgres::Client, id: &String) -> Result<Option<Poll>, ErrorKind> {
        let poll_row = match conn
            .query("SELECT * FROM polls WHERE id = $1 LIMIT 1", &[id])?
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

        let votes_rows = conn.query(
            "SELECT preferences, voter_ip FROM votes WHERE poll_id = $1",
            &[&id],
        )?;
        let mut votes = Vec::with_capacity(votes_rows.len());
        for row in votes_rows {
            let ranked_choices = row.try_get("preferences")?;
            let voter_ip = row.try_get("voter_ip")?;
            votes.push(RankedChoiceVote {
                ranked_choices,
                voter_ip,
            });
        }

        Ok(Some(Poll {
            id,
            title: poll_row.try_get("title")?,
            candidates: poll_row.try_get("candidates")?,
            creation_time: poll_row.try_get("created_at")?,
            end_time: poll_row.try_get("expires_at")?,
            prohibit_double_vote_by_ip: poll_row.try_get("prohibit_double_vote_by_ip")?,
            num_winners,
            winners,
            votes,
            method,
        }))
    }

    fn add_poll(conn: &mut postgres::Client, poll: &Poll) -> Result<(), ErrorKind> {
        let method_discrim: i32 = match poll.method {
            VotingMethod::Schulze => 0,
        };
        conn.query(
            "INSERT INTO polls (
                id,
                title,
                candidates,
                created_at,
                prohibit_double_vote_by_ip,
                expires_at,
                num_winners,
                method
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            &[
                &poll.id,
                &poll.title,
                &poll.candidates,
                &poll.creation_time,
                &poll.prohibit_double_vote_by_ip,
                &poll.end_time,
                &(poll.num_winners as i32),
                &method_discrim,
            ],
        )?;

        for vote in &poll.votes {
            Self::add_vote_to_poll(conn, &poll.id, vote)?;
        }

        if let Some(winners) = &poll.winners {
            Self::set_poll_winners(conn, &poll.id, winners)?;
        }

        Ok(())
    }

    fn add_vote_to_poll(
        conn: &mut postgres::Client,
        id: &String,
        vote: &RankedChoiceVote,
    ) -> Result<(), ErrorKind> {
        conn.query(
            "INSERT INTO votes (poll_id, voter_ip, preferences) VALUES ($1, $2, $3)",
            &[id, &vote.voter_ip, &vote.ranked_choices],
        )?;
        Ok(())
    }

    fn set_poll_winners(
        conn: &mut postgres::Client,
        id: &String,
        winners: &Vec<RankedCandidate<String>>,
    ) -> Result<(), ErrorKind> {
        let winner_insertion =
            &conn.prepare("INSERT INTO winners (poll_id, candidate, rank) VALUES ($1, $2, $3)")?;
        for winner in winners {
            conn.query(
                winner_insertion,
                &[id, &winner.candidate, &(winner.rank as i32)],
            )?;
        }
        Ok(())
    }
}
