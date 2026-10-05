use super::*;

impl LeveragedTrendHoldStrategy {
    #[cfg(test)]
    pub(super) fn session_minutes(
        _is_overseas: bool,
        _toss_us_session: &str,
    ) -> Option<(i64, i64)> {
        Some((60, 10_000))
    }

    #[cfg(not(test))]
    pub(super) fn session_minutes(is_overseas: bool, toss_us_session: &str) -> Option<(i64, i64)> {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        let contains = |open: i64, close: i64| -> Option<(i64, i64)> {
            if open <= close {
                (mins >= open && mins < close).then_some((mins - open, close - mins))
            } else if mins >= open {
                Some((mins - open, (24 * 60 - mins) + close))
            } else if mins < close {
                Some(((24 * 60 - open) + mins, close - mins))
            } else {
                None
            }
        };
        if is_overseas {
            let day = (9 * 60, 16 * 60 + 50);
            let pre = (17 * 60, 22 * 60 + 30);
            let regular = (22 * 60 + 30, 5 * 60);
            let after = (5 * 60, 7 * 60);
            match crate::market_hours::UsTradingSessionPolicy::parse(Some(toss_us_session)) {
                crate::market_hours::UsTradingSessionPolicy::Auto => [day, pre, regular, after]
                    .into_iter()
                    .find_map(|(open, close)| contains(open, close)),
                crate::market_hours::UsTradingSessionPolicy::Day => contains(day.0, day.1),
                crate::market_hours::UsTradingSessionPolicy::Pre => contains(pre.0, pre.1),
                crate::market_hours::UsTradingSessionPolicy::Regular => {
                    contains(regular.0, regular.1)
                }
                crate::market_hours::UsTradingSessionPolicy::After => contains(after.0, after.1),
            }
        } else {
            let open = 9 * 60;
            let close = 15 * 60 + 30;
            contains(open, close)
        }
    }

    pub(super) fn in_blackout_window(windows: &[String]) -> bool {
        use chrono::Timelike;
        let now = chrono::Local::now();
        let mins = now.hour() as i64 * 60 + now.minute() as i64;
        Self::is_blackout_minute(mins, windows)
    }

    pub(super) fn is_blackout_minute(mins: i64, windows: &[String]) -> bool {
        windows.iter().any(|w| {
            let Some((start, end)) = w.split_once('-') else {
                return false;
            };
            let Some(s) = parse_hhmm(start) else {
                return false;
            };
            let Some(e) = parse_hhmm(end) else {
                return false;
            };
            if s <= e {
                mins >= s && mins <= e
            } else {
                mins >= s || mins <= e
            }
        })
    }

    pub(super) fn preview_session_minutes(
        minute_of_day: i64,
        is_overseas: bool,
        toss_us_session: &str,
    ) -> Option<(i64, i64)> {
        let contains = |open: i64, close: i64| -> Option<(i64, i64)> {
            if open <= close {
                (minute_of_day >= open && minute_of_day < close)
                    .then_some((minute_of_day - open, close - minute_of_day))
            } else if minute_of_day >= open {
                Some((minute_of_day - open, (24 * 60 - minute_of_day) + close))
            } else if minute_of_day < close {
                Some(((24 * 60 - open) + minute_of_day, close - minute_of_day))
            } else {
                None
            }
        };
        if is_overseas {
            let day = (9 * 60, 16 * 60 + 50);
            let pre = (17 * 60, 22 * 60 + 30);
            let regular = (22 * 60 + 30, 5 * 60);
            let after = (5 * 60, 7 * 60);
            match crate::market_hours::UsTradingSessionPolicy::parse(Some(toss_us_session)) {
                crate::market_hours::UsTradingSessionPolicy::Auto => [day, pre, regular, after]
                    .into_iter()
                    .find_map(|(open, close)| contains(open, close)),
                crate::market_hours::UsTradingSessionPolicy::Day => contains(day.0, day.1),
                crate::market_hours::UsTradingSessionPolicy::Pre => contains(pre.0, pre.1),
                crate::market_hours::UsTradingSessionPolicy::Regular => {
                    contains(regular.0, regular.1)
                }
                crate::market_hours::UsTradingSessionPolicy::After => contains(after.0, after.1),
            }
        } else {
            contains(9 * 60, 15 * 60 + 30)
        }
    }

    pub(super) fn minute_of_day_from_time(value: &str) -> Option<i64> {
        if let Some(idx) = value.find('T') {
            let time = &value[idx + 1..];
            let mut parts = time.split(':');
            let h = parts.next()?.parse::<i64>().ok()?;
            let m = parts.next()?.parse::<i64>().ok()?;
            if (0..24).contains(&h) && (0..60).contains(&m) {
                return Some(h * 60 + m);
            }
        }
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 12 {
            let h = digits[8..10].parse::<i64>().ok()?;
            let m = digits[10..12].parse::<i64>().ok()?;
            if (0..24).contains(&h) && (0..60).contains(&m) {
                return Some(h * 60 + m);
            }
        }
        None
    }
}

pub(super) fn parse_hhmm(value: &str) -> Option<i64> {
    let (h, m) = value.trim().split_once(':')?;
    let h = h.parse::<i64>().ok()?;
    let m = m.parse::<i64>().ok()?;
    if (0..24).contains(&h) && (0..60).contains(&m) {
        Some(h * 60 + m)
    } else {
        None
    }
}
