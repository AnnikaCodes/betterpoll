use std::net::IpAddr;
use std::time::SystemTime;

// Poll code

use std::time::Duration;

use tallystick::schulze::SchulzeTally;
use tallystick::schulze::Variant;
use tallystick::RankedCandidate;

use crate::error::ErrorKind;

#[derive(Debug, PartialEq, Clone)]
pub struct RankedChoiceVote {
    /// idx 0 is 1st choice, etc
    pub ranked_choices: Vec<String>,
    pub voter_ip: IpAddr,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum VotingMethod {
    Schulze,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Poll {
    pub id: String,
    pub title: String,
    pub candidates: Vec<String>,
    pub creation_time: SystemTime,
    pub end_time: SystemTime,
    pub votes: Vec<RankedChoiceVote>,
    pub num_winners: usize,
    pub winners: Option<Vec<RankedCandidate<String>>>,
    pub method: VotingMethod,
    pub prohibit_double_vote_by_ip: bool,
}

impl Poll {
    pub fn new(
        id: Option<String>,
        title: String,
        candidates: Vec<String>,
        length: Duration,
        num_winners: usize,
    ) -> Self {
        let id = id.unwrap_or_else(|| format!("{:16x}", rand::random::<u64>()));
        let creation_time = SystemTime::now();
        let end_time = creation_time.checked_add(length).unwrap_or_else(|| {
            eprintln!("WARNING: Duration for the poll with ID '{}' is too long! ({} seconds)", &id, length.as_secs());
            eprintln!("This should have been caught already - defaulting to the current time (poll ends immediately)");
            creation_time
        });

        Self {
            id,
            title,
            candidates,
            creation_time,
            end_time,
            votes: Vec::new(),
            num_winners,
            winners: None,
            method: VotingMethod::Schulze,
            prohibit_double_vote_by_ip: true,
        }
    }

    /// Finds the winners
    pub fn find_winners(&self) -> Result<Vec<RankedCandidate<String>>, ErrorKind> {
        let winners = match self.method {
            VotingMethod::Schulze => {
                let mut tally =
                    SchulzeTally::<String, u64>::new(self.num_winners, Variant::Winning);
                for candidate in &self.candidates {
                    tally.add_candidate(candidate.clone());
                }

                for vote in &self.votes {
                    tally.add(&vote.ranked_choices)?;
                }

                tally.winners().into_vec()
            }
        };

        Ok(winners)
    }

    pub fn finish(&mut self) -> Result<(), ErrorKind> {
        let winners = self.find_winners()?;
        self.winners = Some(winners);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn autogen_random_id() {
        let poll1 = Poll::new(None, "".to_string(), vec![], Duration::from_secs(1), 1);
        let poll2 = Poll::new(None, "".to_string(), vec![], Duration::from_secs(1), 1);

        // Has a 1/(2^64 - 1) chance of failing when there is no bug.
        // This is effectively negligible.
        assert_ne!(poll1.id, poll2.id);
    }

    #[test]
    fn support_custom_id() {
        let id = "custom_id".to_string();
        let poll = Poll::new(
            Some(id.clone()),
            "".to_string(),
            vec![],
            Duration::from_secs(1),
            1,
        );

        assert_eq!(poll.id, id);
    }

    #[test]
    fn winners() {
        let a = String::from("a");
        let b = String::from("b");
        let c = String::from("c");

        let mut poll = Poll::new(
            None,
            "".to_string(),
            vec![a.clone(), b.clone(), c.clone()],
            Duration::from_secs(1),
            1,
        );
        poll.votes.push(RankedChoiceVote {
            ranked_choices: vec![c.clone(), a.clone(), b.clone()],
            voter_ip: "127.0.0.1".parse().unwrap(),
        });
        poll.votes.push(RankedChoiceVote {
            ranked_choices: vec![a.clone(), c.clone(), b.clone()],
            voter_ip: "127.0.0.2".parse().unwrap(),
        });
        poll.votes.push(RankedChoiceVote {
            ranked_choices: vec![a.clone(), c.clone()],
            voter_ip: "127.0.0.3".parse().unwrap(),
        });
        poll.votes.push(RankedChoiceVote {
            ranked_choices: vec![b.clone(), c.clone()],
            voter_ip: "127.0.0.3".parse().unwrap(),
        });
        poll.votes.push(RankedChoiceVote {
            ranked_choices: vec![b.clone(), c.clone()],
            voter_ip: "127.0.0.3".parse().unwrap(),
        });

        poll.finish().unwrap();

        assert_eq!(poll.winners.unwrap()[0].candidate, c);
    }
}
