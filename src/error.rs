//! Error types.
//!
//! TODO: consider using anyhow or other error handling library?

use std::num::TryFromIntError;

#[derive(Debug)]
pub enum ErrorKind {
    PubliclyVisible(VisibleError),
    Internal(InternalError),
}

impl<T> From<T> for ErrorKind
where
    InternalError: From<T>,
{
    fn from(err: T) -> Self {
        ErrorKind::Internal(err.into())
    }
}

#[derive(Debug)]
pub enum VisibleError {}

#[derive(Debug)]
pub enum InternalError {
    Database(postgres::Error),
    UnknownVotingMethodDiscriminant(i32),
    InvalidNumWinners(i32, TryFromIntError),
    InvalidWinnerRank(i32, TryFromIntError),
    TallyStick(tallystick::TallyError),
}

impl From<postgres::Error> for InternalError {
    fn from(err: postgres::Error) -> Self {
        InternalError::Database(err)
    }
}

impl From<tallystick::TallyError> for InternalError {
    fn from(err: tallystick::TallyError) -> Self {
        InternalError::TallyStick(err)
    }
}
