//! Error types.
//!
//! TODO: consider using anyhow or other error handling library?

use std::{fmt::Display, num::TryFromIntError};

#[derive(Debug)]
pub enum ErrorKind {
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
pub enum VisibleError {
    // Reserved for future errors
}

impl Display for VisibleError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!("this should never happen");
    }
}

#[derive(Debug)]
pub enum InternalError {
    Database(postgres::Error),
    UnknownVotingMethodDiscriminant(i32),
    InvalidNumWinners(i32, TryFromIntError),
    TallyStick(tallystick::TallyError),
    CouldNotConvertDBTimeToUNIX(std::time::SystemTimeError, String),
    InvalidCreationTime(String, u64),
    InvalidEndTime(String, u64),
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
