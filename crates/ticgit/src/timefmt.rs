use time::OffsetDateTime;

pub fn relative_time(then: OffsetDateTime, now: OffsetDateTime) -> String {
    let seconds = (now - then).whole_seconds().max(0);
    if seconds < 60 * 60 {
        return format!("{}m", seconds / 60);
    }
    if seconds < 60 * 60 * 24 {
        return format!("{}h", seconds / (60 * 60));
    }
    if seconds < 60 * 60 * 24 * 30 {
        return format!("{}d", seconds / (60 * 60 * 24));
    }
    if seconds < 60 * 60 * 24 * 365 {
        return format!("{}mo", seconds / (60 * 60 * 24 * 30));
    }
    format!("{}y", seconds / (60 * 60 * 24 * 365))
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[test]
    fn relative_time_uses_minutes_hours_days_months_and_years() {
        let now = OffsetDateTime::UNIX_EPOCH + Duration::days(400);

        assert_eq!(relative_time(now - Duration::seconds(59), now), "0m");
        assert_eq!(relative_time(now - Duration::minutes(42), now), "42m");
        assert_eq!(relative_time(now - Duration::hours(23), now), "23h");
        assert_eq!(relative_time(now - Duration::days(29), now), "29d");
        assert_eq!(relative_time(now - Duration::days(45), now), "1mo");
        assert_eq!(relative_time(now - Duration::days(400), now), "1y");
    }
}
