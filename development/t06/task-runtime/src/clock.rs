//! The fixed task's one origin and conservative elapsed-cost conversions.
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Clock,
    Deadline,
    Budget,
}

#[derive(Clone, Copy, Debug)]
pub struct TaskClock {
    pub(crate) origin: Instant,
    pub(crate) unix_ms: u64,
    pub(crate) candidate_deadline: Instant,
    pub(crate) verification_deadline: Instant,
    pub(crate) deadline: Instant,
}

impl TaskClock {
    /// Exact original monotonic origin, shared with cancellation observations.
    #[must_use]
    pub const fn origin(self) -> Instant {
        self.origin
    }

    #[must_use]
    pub const fn deadline(self) -> Instant {
        self.deadline
    }

    /// # Errors
    /// Refuses unavailable wall time or monotonic deadline overflow.
    pub fn start() -> Result<Self, Error> {
        // These are adjacent observations, not an assertion of atomic wall/monotonic sampling.
        let unix_ms = unix_ms()?;
        let origin = Instant::now();
        Ok(Self {
            origin,
            unix_ms,
            candidate_deadline: origin
                .checked_add(Duration::from_mins(15))
                .ok_or(Error::Clock)?,
            verification_deadline: origin
                .checked_add(Duration::from_secs(1190))
                .ok_or(Error::Clock)?,
            deadline: origin
                .checked_add(Duration::from_mins(20))
                .ok_or(Error::Clock)?,
        })
    }

    #[must_use]
    pub fn elapsed(self) -> Duration {
        Instant::now().saturating_duration_since(self.origin)
    }

    /// # Errors
    /// Refuses candidate dispatch inside the final verification reserve.
    pub fn dispatch(self) -> Result<(), Error> {
        if Instant::now() >= self.candidate_deadline {
            Err(Error::Deadline)
        } else {
            Ok(())
        }
    }

    /// # Errors
    /// Refuses an exhausted allocation, overflow or expired candidate horizon.
    pub fn work_deadline(self, remaining_ms: u64) -> Result<Instant, Error> {
        let now = Instant::now();
        if remaining_ms == 0 {
            return Err(Error::Budget);
        }
        let limit = now
            .checked_add(Duration::from_millis(remaining_ms))
            .ok_or(Error::Clock)?;
        let deadline = limit.min(self.candidate_deadline);
        if now >= deadline {
            Err(Error::Deadline)
        } else {
            Ok(deadline)
        }
    }
}

/// # Errors
/// Refuses pre-epoch wall time or an unrepresentable millisecond value.
pub fn unix_ms() -> Result<u64, Error> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::Clock)?
            .as_millis(),
    )
    .map_err(|_| Error::Clock)
}

/// # Errors
/// Refuses reversed monotonic observations or unrepresentable conservative cost.
pub fn cost_ms(start: Instant, end: Instant) -> Result<u64, Error> {
    let elapsed = end.checked_duration_since(start).ok_or(Error::Clock)?;
    u64::try_from(elapsed.as_nanos().div_ceil(1_000_000)).map_err(|_| Error::Clock)
}

// A Store lease uses remaining whole milliseconds at the request boundary.
// The task's original Instant deadline remains the execution/cleanup authority.
pub(crate) fn lease_ms(now: Instant, deadline: Instant) -> Result<u64, Error> {
    let remaining = deadline
        .checked_duration_since(now)
        .ok_or(Error::Deadline)?;
    let ms = u64::try_from(remaining.as_millis()).map_err(|_| Error::Clock)?;
    if ms == 0 {
        Err(Error::Deadline)
    } else {
        Ok(ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_consumes_work_and_does_not_extend_lease() {
        let origin = Instant::now();
        let deadline = origin + Duration::from_mins(20);
        let begin = origin + Duration::from_millis(45_001);
        let finish = begin + Duration::from_millis(125_002);
        assert_eq!(cost_ms(origin, finish), Ok(170_003));
        assert_eq!(lease_ms(begin, deadline), Ok(1_154_999));
        assert_eq!(
            lease_ms(
                deadline.checked_sub(Duration::from_nanos(999_999)).unwrap(),
                deadline
            ),
            Err(Error::Deadline)
        );
        assert_eq!(lease_ms(deadline, deadline), Err(Error::Deadline));
        assert_eq!(
            lease_ms(deadline + Duration::from_nanos(1), deadline),
            Err(Error::Deadline)
        );
        assert_eq!(
            lease_ms(
                deadline
                    .checked_sub(Duration::from_nanos(1_999_999))
                    .unwrap(),
                deadline
            ),
            Ok(1)
        );
    }
    #[test]
    fn fractional_work_is_charged_and_reversed_observations_refuse() {
        let origin = Instant::now();
        assert_eq!(cost_ms(origin, origin), Ok(0));
        assert_eq!(cost_ms(origin, origin + Duration::from_nanos(1)), Ok(1));
        assert_eq!(cost_ms(origin, origin + Duration::from_millis(1)), Ok(1));
        assert_eq!(
            cost_ms(origin, origin + Duration::from_nanos(1_000_001)),
            Ok(2)
        );
        assert_eq!(
            cost_ms(origin + Duration::from_nanos(1), origin),
            Err(Error::Clock)
        );
    }
}
