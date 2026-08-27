use chrono::{DateTime, Duration, Utc};
use sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SessionError {
    Expired,
    Revoked,
}

impl Session {
    pub fn validate(&self, now: DateTime<Utc>) -> Result<(), SessionError> {
        if self.revoked_at.is_some() {
            return Err(SessionError::Revoked);
        }
        if self.expires_at <= now {
            return Err(SessionError::Expired);
        }
        Ok(())
    }
}

const LAST_SEEN_STALE_AFTER: Duration = Duration::minutes(30);

/// Returns true when a user.last_seen_at is missing(this is the first sign in attemp) or is older than the 15 minutes.
pub fn is_last_seen_stale(last_seen_at: Option<DateTime<Utc>>) -> bool {
    match last_seen_at {
        None => true,
        Some(last_seen_at) => Utc::now() - last_seen_at >= LAST_SEEN_STALE_AFTER,
    }
}

#[cfg(test)]
mod is_last_seen_stale_tests {
    use super::*;

    #[test]
    fn none_is_stale() {
        assert!(is_last_seen_stale(None));
    }

    #[test]
    fn recent_is_not_stale() {
        assert!(!is_last_seen_stale(Some(Utc::now() - Duration::minutes(5))));
    }

    #[test]
    fn older_than_threshold_is_stale() {
        assert!(is_last_seen_stale(Some(Utc::now() - Duration::minutes(35))));
    }
}

#[cfg(test)]
mod session_validate_tests {
    use super::*;

    fn sample(expires_at: DateTime<Utc>, revoked_at: Option<DateTime<Utc>>) -> Session {
        Session {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            created_at: Utc::now(),
            expires_at,
            revoked_at,
        }
    }

    #[test]
    fn session_is_valid() {
        let now = Utc::now();
        // Expires a day from now, it's not revoked
        assert!(sample(now + Duration::days(1), None).validate(now).is_ok());
    }

    #[test]
    fn expired_session_is_not_valid() {
        let now = Utc::now();
        assert_eq!(
            sample(now - Duration::seconds(1), None).validate(now),
            Err(SessionError::Expired)
        );
    }

    #[test]
    fn revoked_session_is_not_valid() {
        let now = Utc::now();
        // not expired, but it's revoked
        assert_eq!(
            sample(now + Duration::days(1), Some(now)).validate(now),
            Err(SessionError::Revoked)
        );
    }
}
