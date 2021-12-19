use std::time::SystemTime;

// Poll code
use rand::Rng;
use std::time::Duration;

use tallystick::schulze::SchulzeTally;
use tallystick::schulze::Variant;
use tallystick::RankedCandidate;
use tallystick::TallyError;

/// idx 0 is 1st choice, etc
pub struct RankedChoiceVote(Vec<String>);
impl RankedChoiceVote {
    pub fn from_vec(candidates: Vec<String>) -> Self {
        RankedChoiceVote(candidates)
    }
}

pub enum VotingMethod {
    Schulze,
}

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
}

impl Poll {
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
        let creation_time = SystemTime::now();
        let end_time = creation_time.checked_add(length).unwrap_or_else(|| {
            eprintln!("Duration for poll {} is too long! ({:?})", &id, length);
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
        }
    }

    /// Finds the winners
    pub fn find_winners(&self) -> Result<Vec<RankedCandidate<String>>, TallyError> {
        let winners = match self.method {
            VotingMethod::Schulze => {
                let mut tally =
                    SchulzeTally::<String, u64>::new(self.num_winners, Variant::Winning);

                for vote in &self.votes {
                    tally.add(&vote.0)?;
                }

                tally.winners().into_vec()
            }
        };

        Ok(winners)
    }

    pub fn finish(&mut self) -> Result<(), TallyError> {
        let winners = self.find_winners()?;
        self.winners = Some(winners);
        Ok(())
    }
}
