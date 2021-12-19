//! Error types.
//!
//! TODO: consider using anyhow or other error handling library?

use std::num::TryFromIntError;

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

pub enum VisibleError {}

pub enum InternalError {
    DatabaseError(postgres::Error),
    UnknownVotingMethodDiscriminant(i32),
    InvalidNumWinners(i32, TryFromIntError),
    InvalidWinnerRank(i32, TryFromIntError),
}

impl From<postgres::Error> for InternalError {
    fn from(err: postgres::Error) -> Self {
        InternalError::DatabaseError(err)
    }
}
