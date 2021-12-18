// Poll code
use chrono::prelude::*;
use chrono::Duration;
use rand::Rng;
use tallystick::RankedCandidate;
use tallystick::TallyError;
use tallystick::irv::Tally;
use tallystick::schulze::SchulzeTally;
use tallystick::schulze::Variant;

/// idx 0 is 1st choice, etc
pub type RankedChoiceVote = Vec<String>;

pub enum Poll {
    InProgress(PollInProgress),
    Completed(CompletedPoll),
}

pub struct PollInProgress {
    pub id: String,
    title: String,
    candidates: Vec<String>,
    creation_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    votes: Vec<RankedChoiceVote>,
    num_winners: usize,
}

impl PollInProgress {
    pub fn new(
        id: Option<String>,
        title: String,
        candidates: Vec<String>,
        length: Duration,
        num_winners: usize,
    ) -> Self {
        let id = id.unwrap_or_else(|| {
            let mut rng = rand::thread_rng();
            format!("{:16X}{:16X}", rng.gen::<u64>(), rng.gen::<u64>())
        });

        PollInProgress {
            id,
            title,
            candidates,
            creation_time: Utc::now(),
            end_time: Utc::now() + length,
            votes: Vec::new(),
            num_winners,
        }
    }

    /// Finds the winners
    pub fn find_winners(&self) -> Result<Vec<RankedCandidate<String>>, TallyError> {
        let mut tally = SchulzeTally::<String, u64>::new(self.num_winners, Variant::Winning);

        for vote in &self.votes {
            tally.add(vote)?;
        }

        Ok(tally.winners().into_vec())
    }

    /// Consumes the PollInProgress and returns a CompletedPoll
    pub fn finish(self) -> Result<CompletedPoll, TallyError> {
        let winners = self.find_winners()?;
        Ok(CompletedPoll {
            id: self.id,
            title: self.title,
            candidates: self.candidates,
            creation_time: self.creation_time,
            end_time: self.end_time,
            winners,
        })
    }
}

pub struct CompletedPoll {
    pub id: String,
    title: String,
    winners: Vec<RankedCandidate<String>>,
    candidates: Vec<String>,
    creation_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
}
