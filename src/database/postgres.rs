//! Postgres database code
//!
//! TODO: can we prepare statements? investigate how this'd work with Rocket

use std::time::{SystemTime, Duration};

use tallystick::RankedCandidate;

use crate::{
    error::{ErrorKind, InternalError, VisibleError},
    poll::*,
};

#[cfg_attr(not(test), database("production_db"))]
#[cfg_attr(test, database("test_db"))]
pub struct PostgresConnection(pub postgres::Client);

impl PostgresConnection {
    pub async fn get_poll_by_id(&mut self, id: String) -> Result<Option<Poll>, ErrorKind> {
        let cloned_id = id.clone();
        let poll_row = match self
            .run(move |c| c.query("SELECT * FROM polls WHERE id = $1 LIMIT 1", &[&cloned_id]))
            .await?
            .pop()
        {
            Some(row) => row,
            None => return Ok(None), // No poll by that ID
        };

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

        let cloned_id = id.clone();
        let winners_rows = self
            .run(move |c| c.query("SELECT * FROM winners WHERE poll_id = $1", &[&cloned_id]))
            .await?;
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

        let cloned_id = id.clone();
        let votes_rows = self
            .run(move |c| {
                c.query(
                    "SELECT preferences, voter_ip FROM votes WHERE poll_id = $1",
                    &[&cloned_id],
                )
            })
            .await?;
        let mut votes = Vec::with_capacity(votes_rows.len());
        for row in votes_rows {
            let ranked_choices = row.try_get("preferences")?;
            let voter_ip = row.try_get("voter_ip")?;
            votes.push(RankedChoiceVote {
                ranked_choices,
                voter_ip,
            });
        }

        let creation_systime: SystemTime = poll_row.try_get("created_at")?;
        let creation_time = match creation_systime.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => return Err(ErrorKind::Internal(InternalError::CouldNotConvertDBTimeToUNIX(e, id))),
        };

        let end_systime: SystemTime = poll_row.try_get("expires_at")?;
        let end_time = match end_systime.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => return Err(ErrorKind::Internal(InternalError::CouldNotConvertDBTimeToUNIX(e, id))),
        };

        let mut poll = Poll {
            id,
            title: poll_row.try_get("title")?,
            candidates: poll_row.try_get("candidates")?,
            creation_time,
            end_time,
            prohibit_double_vote_by_ip: poll_row.try_get("prohibit_double_vote_by_ip")?,
            num_winners,
            winners,
            votes,
            method,
        };
        if poll.end_time < std::time::SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("can't find out how long it was since the UNIX epoch").as_secs() {
            poll.finish()?;
        }

        Ok(Some(poll))
    }

    pub async fn add_poll(&mut self, poll: Poll) -> Result<(), ErrorKind> {
        let method_discrim: i32 = match poll.method {
            VotingMethod::Schulze => 0,
        };
        let id = poll.id.clone();

        let creation_time = match std::time::SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(poll.creation_time)) {
            Some(creation_time) => creation_time,
            None => return Err(ErrorKind::Internal(InternalError::InvalidCreationTime(
                poll.id,
                poll.creation_time,
            ))),
        };
        let end_time = match std::time::SystemTime::UNIX_EPOCH.checked_add(Duration::from_secs(poll.end_time)) {
            Some(end_time) => end_time,
            None => return Err(ErrorKind::Internal(InternalError::InvalidEndTime(
                poll.id,
                poll.end_time,
            ))),
        };

        self.run(move |c| {
            c.query(
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
                    &creation_time,
                    &poll.prohibit_double_vote_by_ip,
                    &end_time,
                    &(poll.num_winners as i32),
                    &method_discrim,
                ],
            )
        })
        .await?;

        for vote in poll.votes {
            Self::add_vote_to_poll(self, id.clone(), vote).await?;
        }

        if let Some(winners) = poll.winners {
            Self::set_poll_winners(self, id, winners).await?;
        }

        Ok(())
    }

    pub async fn add_vote_to_poll(
        &mut self,
        id: String,
        vote: RankedChoiceVote,
    ) -> Result<(), ErrorKind> {
        self.run(move |c| {
            c.query(
                "INSERT INTO votes (poll_id, voter_ip, preferences) VALUES ($1, $2, $3)",
                &[&id, &vote.voter_ip, &vote.ranked_choices],
            )
        })
        .await?;
        Ok(())
    }

    pub async fn set_poll_winners(
        &mut self,
        id: String,
        winners: Vec<RankedCandidate<String>>,
    ) -> Result<(), ErrorKind> {
        for winner in winners {
            let id = id.clone();
            self.run(move |c| {
                c.query(
                    "INSERT INTO winners (poll_id, candidate, rank) VALUES ($1, $2, $3)",
                    &[&id, &winner.candidate, &(winner.rank as i32)],
                )
            })
            .await?;
        }
        Ok(())
    }
}

// Unfortunately Rocket seems to make it really tough to test DB stuff :(
// For now I'll just test the API routes directly; database unit tests can be a future project.
// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use super::*;
//     use crate::poll::Poll;
//     use postgres::NoTls;

//     fn get_conn() -> Postgres {
//         dotenv::dotenv().ok();
//         let url = std::env::var("TEST_DATABASE").expect(
//             "The TEST_DATABASE environment variable must be set when testing database features",
//         );
//         let db_config = rocket_sync_db_pools::Config {
//             url,
//             pool_size: 5,
//             timeout: 5,
//         };
//         let r = rocket::custom(db_config);
//         rocket::local::asynchronous::Client::tracked(r);
//     }

//     #[test]
//     #[cfg_attr(not(feature = "db-test"), ignore)]
//     fn save_load() -> Result<(), ErrorKind> {
//         let new_poll = Poll::new(
//             None,
//             "Test Poll 1".to_owned(),
//             vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             Duration::from_secs(24 * 60 * 60),
//             2,
//         );
//         let mut in_progress_poll = Poll::new(
//             None,
//             "Test Poll 2".to_owned(),
//             vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             Duration::from_secs(24 * 60 * 60),
//             2,
//         );
//         in_progress_poll.votes.push(RankedChoiceVote {
//             ranked_choices: vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 3".to_owned(),
//                 "Candidate 2".to_owned(),
//             ],
//             voter_ip: "127.0.0.1".parse().unwrap(),
//         });
//         in_progress_poll.votes.push(RankedChoiceVote {
//             ranked_choices: vec![
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//                 "Candidate 1".to_owned(),
//             ],
//             voter_ip: "127.0.0.2".parse().unwrap(),
//         });
//         in_progress_poll.votes.push(RankedChoiceVote {
//             ranked_choices: vec![
//                 "Candidate 2".to_owned(),
//                 "Candidate 1".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             voter_ip: "127.0.0.3".parse().unwrap(),
//         });

//         let mut finished_poll = in_progress_poll.clone();
//         finished_poll.finish()?;

//         let self = &mut get_self()?;
//         for poll in [new_poll, in_progress_poll] {
//             // before a poll is saved, a poll with that ID shouldn't exist
//             assert!(
//                 Postgres::get_poll_by_id(self, &poll.id)?.is_none(),
//                 "A poll with ID {} already exists",
//                 poll.id
//             );

//             Postgres::add_poll(self, &poll)?;

//             // after a poll is saved, getting the poll should contain the same data
//             assert_eq!(Postgres::get_poll_by_id(self, &poll.id)?, Some(poll));
//         }

//         Ok(())
//     }

//     #[test]
//     #[cfg_attr(not(feature = "db-test"), ignore)]
//     fn add_vote() -> Result<(), ErrorKind> {
//         let poll = Poll::new(
//             None,
//             "Test Poll".to_owned(),
//             vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             Duration::from_secs(24 * 60 * 60),
//             1,
//         );
//         let self = &mut get_self()?;
//         Postgres::add_poll(self, &poll)?;

//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().votes,
//             vec![],
//             "should start with no votes"
//         );

//         let vote1 = RankedChoiceVote {
//             ranked_choices: vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             voter_ip: "127.0.0.1".parse().unwrap(),
//         };
//         let vote2 = RankedChoiceVote {
//             ranked_choices: vec![
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//                 "Candidate 1".to_owned(),
//             ],
//             voter_ip: "127.0.0.2".parse().unwrap(),
//         };

//         Postgres::add_vote_to_poll(self, &poll.id, &vote1)?;
//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().votes,
//             vec![vote1.clone()]
//         );

//         Postgres::add_vote_to_poll(self, &poll.id, &vote2)?;
//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().votes,
//             vec![vote1, vote2]
//         );

//         Ok(())
//     }

//     #[test]
//     #[cfg_attr(not(feature = "db-test"), ignore)]
//     fn add_winners() -> Result<(), ErrorKind> {
//         let poll = Poll::new(
//             None,
//             "Test Poll".to_owned(),
//             vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             Duration::from_secs(24 * 60 * 60),
//             2,
//         );
//         let self = &mut get_self()?;
//         Postgres::add_poll(self, &poll)?;

//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().winners,
//             None,
//             "should start without winners"
//         );

//         let winners = vec![
//             RankedCandidate {
//                 candidate: "Candidate 1".to_owned(),
//                 rank: 1,
//             },
//             RankedCandidate {
//                 candidate: "Candidate 2".to_owned(),
//                 rank: 2,
//             },
//         ];

//         Postgres::set_poll_winners(self, &poll.id, &winners)?;
//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().winners,
//             Some(winners)
//         );

//         Ok(())
//     }

//     #[test]
//     #[cfg_attr(not(feature = "db-test"), ignore)]
//     fn determine_winners_on_expiry() -> Result<(), ErrorKind> {
//         let length = Duration::from_secs(1);
//         let poll = Poll::new(
//             None,
//             "Test Poll".to_owned(),
//             vec![
//                 "Candidate 1".to_owned(),
//                 "Candidate 2".to_owned(),
//                 "Candidate 3".to_owned(),
//             ],
//             length,
//             1,
//         );
//         let self = &mut get_self()?;

//         Postgres::add_poll(self, &poll)?;
//         Postgres::add_vote_to_poll(
//             self,
//             &poll.id,
//             &RankedChoiceVote {
//                 ranked_choices: vec![
//                     "Candidate 1".to_owned(),
//                     "Candidate 2".to_owned(),
//                     "Candidate 3".to_owned(),
//                 ],
//                 voter_ip: "127.0.0.1".parse().unwrap(),
//             },
//         )?;
//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().winners,
//             None
//         );

//         std::thread::sleep(length);
//         assert_eq!(
//             Postgres::get_poll_by_id(self, &poll.id)?.unwrap().winners,
//             Some(vec![RankedCandidate {
//                 candidate: "Candidate 1".to_owned(),
//                 rank: 0,
//             },])
//         );

//         Ok(())
//     }
// }
