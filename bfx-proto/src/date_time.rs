#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use crate::DateTime;
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use tonic::Status;

impl<Tz: TimeZone> From<chrono::DateTime<Tz>> for DateTime {
    fn from(value: chrono::DateTime<Tz>) -> Self {
        let utc = value.to_utc();

        let year = utc.year();
        let ordinal = utc.ordinal();
        let second = utc.num_seconds_from_midnight();
        let nanos = utc.nanosecond();

        Self {
            year,
            ordinal,
            second,
            nanos,
        }
    }
}

impl TryFrom<DateTime> for chrono::DateTime<Utc> {
    type Error = Status;

    fn try_from(value: DateTime) -> Result<Self, Self::Error> {
        let date = NaiveDate::from_yo_opt(value.year, value.ordinal)
            .ok_or_else(|| Status::invalid_argument("invalid year or ordinal"))?;
        let time = NaiveTime::from_num_seconds_from_midnight_opt(value.second, value.nanos)
            .ok_or_else(|| Status::invalid_argument("invalid second or nanos"))?;

        Ok(date.and_time(time).and_utc())
    }
}
