//! Postgres database code
//!
//! TODO: can we prepare statements? investigate how this'd work with Rocket



use tallystick::RankedCandidate;

use super::Database;
use crate::{
    error::{ErrorKind, InternalError},
    poll::*,
};

pub struct Postgres;

impl Database<&mut postgres::Client> for Postgres {
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

        let mut poll = Poll {
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
        };
        if poll.end_time < std::time::SystemTime::now() {
            poll.finish()?;
        }

        Ok(Some(poll))
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use postgres::NoTls;
    use super::*;
    use crate::poll::Poll;

    fn get_conn() -> Result<postgres::Client, postgres::Error> {
        dotenv::dotenv().ok();
        let db_url = std::env::var("TEST_DATABASE").expect("The TEST_DATABASE environment variable must be set when testing database features");
        postgres::Client::connect(&db_url, NoTls)
    }

    #[test]
    #[cfg_attr(not(feature = "db-test"), ignore)]
    fn save_load() -> Result<(), ErrorKind> {
        let new_poll = Poll::new(
            None,
            "Test Poll 1".to_owned(),
            vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            Duration::from_secs(24 * 60 * 60),
            2,
        );
        let mut in_progress_poll = Poll::new(
            None,
            "Test Poll 2".to_owned(),
            vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            Duration::from_secs(24 * 60 * 60),
            2,
        );
        in_progress_poll.votes.push(RankedChoiceVote {
            ranked_choices: vec!["Candidate 1".to_owned(), "Candidate 3".to_owned(), "Candidate 2".to_owned()],
            voter_ip: "127.0.0.1".parse().unwrap(),
        });
        in_progress_poll.votes.push(RankedChoiceVote {
            ranked_choices: vec!["Candidate 2".to_owned(), "Candidate 3".to_owned(), "Candidate 1".to_owned()],
            voter_ip: "127.0.0.2".parse().unwrap(),
        });
        in_progress_poll.votes.push(RankedChoiceVote {
            ranked_choices: vec!["Candidate 2".to_owned(), "Candidate 1".to_owned(), "Candidate 3".to_owned()],
            voter_ip: "127.0.0.3".parse().unwrap(),
        });

        let mut finished_poll = in_progress_poll.clone();
        finished_poll.finish()?;

        let conn = &mut get_conn()?;
        for poll in [new_poll, in_progress_poll] {
            // before a poll is saved, a poll with that ID shouldn't exist
            assert!(
                Postgres::get_poll_by_id(conn, &poll.id)?.is_none(),
                "A poll with ID {} already exists", poll.id
            );

            Postgres::add_poll(conn, &poll)?;

            // after a poll is saved, getting the poll should contain the same data
            assert_eq!(
                Postgres::get_poll_by_id(conn, &poll.id)?,
                Some(poll)
            );
        }

        Ok(())
    }

    #[test]
    #[cfg_attr(not(feature = "db-test"), ignore)]
    fn add_vote() -> Result<(), ErrorKind> {
        let poll = Poll::new(
            None,
            "Test Poll".to_owned(),
            vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            Duration::from_secs(24 * 60 * 60),
            1,
        );
        let conn = &mut get_conn()?;
        Postgres::add_poll(conn, &poll)?;

        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().votes, vec![], "should start with no votes");

        let vote1 = RankedChoiceVote {
            ranked_choices: vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            voter_ip: "127.0.0.1".parse().unwrap(),
        };
        let vote2 = RankedChoiceVote {
            ranked_choices: vec!["Candidate 2".to_owned(), "Candidate 3".to_owned(), "Candidate 1".to_owned()],
            voter_ip: "127.0.0.2".parse().unwrap(),
        };

        Postgres::add_vote_to_poll(conn, &poll.id, &vote1)?;
        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().votes, vec![vote1.clone()]);

        Postgres::add_vote_to_poll(conn, &poll.id, &vote2)?;
        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().votes, vec![vote1, vote2]);

        Ok(())
    }

    #[test]
    #[cfg_attr(not(feature = "db-test"), ignore)]
    fn add_winners() -> Result<(), ErrorKind> {
        let poll = Poll::new(
            None,
            "Test Poll".to_owned(),
            vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            Duration::from_secs(24 * 60 * 60),
            2,
        );
        let conn = &mut get_conn()?;
        Postgres::add_poll(conn, &poll)?;

        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().winners, None, "should start without winners");

        let winners = vec![
            RankedCandidate {
                candidate: "Candidate 1".to_owned(),
                rank: 1,
            },
            RankedCandidate {
                candidate: "Candidate 2".to_owned(),
                rank: 2,
            },
        ];

        Postgres::set_poll_winners(conn, &poll.id, &winners)?;
        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().winners, Some(winners));

        Ok(())
    }

    #[test]
    #[cfg_attr(not(feature = "db-test"), ignore)]
    fn determine_winners_on_expiry() -> Result<(), ErrorKind> {
        let length = Duration::from_secs(1);
        let poll = Poll::new(
            None,
            "Test Poll".to_owned(),
            vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            length,
            1,
        );
        let conn = &mut get_conn()?;

        Postgres::add_poll(conn, &poll)?;
        Postgres::add_vote_to_poll(conn, &poll.id, &RankedChoiceVote {
            ranked_choices: vec!["Candidate 1".to_owned(), "Candidate 2".to_owned(), "Candidate 3".to_owned()],
            voter_ip: "127.0.0.1".parse().unwrap(),
        })?;
        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().winners, None);


        std::thread::sleep(length);
        assert_eq!(Postgres::get_poll_by_id(conn, &poll.id)?.unwrap().winners, Some(vec![
            RankedCandidate {
                candidate: "Candidate 1".to_owned(),
                rank: 0,
            },
        ]));

        Ok(())
    }
}