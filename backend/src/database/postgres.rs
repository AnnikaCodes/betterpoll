//! Postgres database code
//!
//! TODO: can we prepare statements? investigate how this'd work with Rocket

use std::time::{Duration, SystemTime};

use crate::{
    error::{ErrorKind, InternalError},
    poll::*,
};

#[cfg_attr(test, database("test_db"))]
#[cfg_attr(not(test), database("production_db"))]
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
            Err(e) => {
                return Err(ErrorKind::Internal(
                    InternalError::CouldNotConvertDBTimeToUNIX(e, id),
                ))
            }
        };

        let end_systime: SystemTime = poll_row.try_get("expires_at")?;
        let end_time = match end_systime.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(e) => {
                return Err(ErrorKind::Internal(
                    InternalError::CouldNotConvertDBTimeToUNIX(e, id),
                ))
            }
        };

        let mut poll = Poll {
            id,
            title: poll_row.try_get("title")?,
            description: poll_row.try_get("description")?,
            candidates: poll_row.try_get("candidates")?,
            creation_time,
            end_time,
            prohibit_double_vote_by_ip: poll_row.try_get("prohibit_double_vote_by_ip")?,
            num_winners,
            winners: None,
            votes,
            method,
        };
        if poll.end_time
            < std::time::SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("can't find out how long it was since the UNIX epoch")
                .as_secs()
        {
            poll.finish()?;
        }

        Ok(Some(poll))
    }

    pub async fn add_poll(&mut self, poll: Poll) -> Result<(), ErrorKind> {
        let method_discrim: i32 = match poll.method {
            VotingMethod::Schulze => 0,
        };
        let id = poll.id.clone();

        let creation_time = match std::time::SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(poll.creation_time))
        {
            Some(creation_time) => creation_time,
            None => {
                return Err(ErrorKind::Internal(InternalError::InvalidCreationTime(
                    poll.id,
                    poll.creation_time,
                )))
            }
        };
        let end_time = match std::time::SystemTime::UNIX_EPOCH
            .checked_add(Duration::from_secs(poll.end_time))
        {
            Some(end_time) => end_time,
            None => {
                return Err(ErrorKind::Internal(InternalError::InvalidEndTime(
                    poll.id,
                    poll.end_time,
                )))
            }
        };

        self.run(move |c| {
            c.query(
                "INSERT INTO polls (
                id,
                title,
                description,
                candidates,
                created_at,
                prohibit_double_vote_by_ip,
                expires_at,
                num_winners,
                method
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &poll.id,
                    &poll.title,
                    &poll.description,
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

    pub async fn get_total_polls(
        &mut self,
    ) -> Result<i64, ErrorKind> {
        let mut rows = self.run(move |c| c.query("SELECT COUNT(*) AS count FROM polls", &[])).await?;
        let row = rows.pop().expect("there should be exactly one row");
        let count: i64 = row.try_get("count")?;
        Ok(count)
    }

    pub async fn get_active_polls(
        &mut self,
    ) -> Result<i64, ErrorKind> {
        let now = SystemTime::now();

        let mut rows = self.run(move |c| {
            c.query("SELECT COUNT(*) AS count FROM polls WHERE expires_at > $1", &[&now])
        }).await?;
        let row = rows.pop().expect("there should be exactly one row");
        let count: i64 = row.try_get("count")?;
        Ok(count)
    }
}
