use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, TimeZone, Utc};
use rrule::Tz;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub title: String,
    pub rrule: Option<String>,
    pub exceptions: Vec<NaiveDate>,
    pub tags: Vec<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExpandedEvent {
    pub date: NaiveDate,
    pub start_time: Option<NaiveTime>,
    pub end_time: Option<NaiveTime>,
    pub title: String,
    pub tags: Vec<String>,
    pub location: Option<String>,
    pub is_exception: bool,
}

#[derive(Error, Debug)]
pub enum CalchemyError {
    #[error("Failed to parse date: {0}")]
    ParseDate(String),
    #[error("Failed to parse time: {0}")]
    ParseTime(String),
    #[error("Failed to parse event: {0}")]
    ParseEvent(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Recurrence error: {0}")]
    Recurrence(String),
}

pub struct Calendar {
    events: Vec<Event>,
}

impl Calendar {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn load(path: &str) -> Result<Self, CalchemyError> {
        let content = std::fs::read_to_string(path)?;
        Ok(Calendar::parse(&content))
    }

    pub fn save(&self, path: &str) -> Result<(), CalchemyError> {
        let content = self.format();
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }

    pub fn events_between(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ExpandedEvent>, CalchemyError> {
        let mut expanded = Vec::new();

        for event in &self.events {
            let occurrences = self.expand_recurrence(event, start, end)?;
            for occ in occurrences {
                let is_exception = event.exceptions.contains(&occ);
                if is_exception {
                    continue;
                }
                expanded.push(ExpandedEvent {
                    date: occ,
                    start_time: event.start_time,
                    end_time: event.end_time,
                    title: event.title.clone(),
                    tags: event.tags.clone(),
                    location: event.location.clone(),
                    is_exception: false,
                });
            }
        }

        expanded.sort_by(|a, b| {
            a.date
                .cmp(&b.date)
                .then_with(|| a.start_time.cmp(&b.start_time))
        });

        Ok(expanded)
    }

    fn expand_recurrence(
        &self,
        event: &Event,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<NaiveDate>, CalchemyError> {
        if let Some(rrule_str) = &event.rrule {
            let rrule_with_start = format!(
                "DTSTART:{}\nRRULE:{}",
                event.date.format("%Y%m%dT000000Z"),
                rrule_str
            );

            let rset: rrule::RRuleSet = rrule_with_start
                .parse()
                .map_err(|e: rrule::RRuleError| CalchemyError::Recurrence(e.to_string()))?;

            let start_dt = Utc
                .with_ymd_and_hms(start.year(), start.month(), start.day(), 0, 0, 0)
                .unwrap();
            let end_dt = Utc
                .with_ymd_and_hms(end.year(), end.month(), end.day(), 23, 59, 59)
                .unwrap();

            let occurrences: Vec<DateTime<Tz>> = (&rset)
                .into_iter()
                .skip_while(|dt| *dt < start_dt)
                .take_while(|dt| *dt <= end_dt)
                .collect();

            Ok(occurrences.into_iter().map(|dt| dt.date_naive()).collect())
        } else {
            Ok(vec![event.date])
        }
    }

    fn parse(content: &str) -> Self {
        let mut events = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(event) = parse_event_line(line) {
                events.push(event);
            }
        }

        Self { events }
    }

    fn format(&self) -> String {
        let mut lines = Vec::new();

        for event in &self.events {
            lines.push(format_event(event));
        }

        lines.join("\n")
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_event_line(line: &str) -> Option<Event> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let date = NaiveDate::parse_from_str(parts[0], "%Y-%m-%d").ok()?;

    let mut start_time = None;
    let mut end_time = None;
    let mut title_parts = Vec::new();
    let mut rrule = None;
    let mut exceptions = Vec::new();
    let mut tags = Vec::new();
    let mut location = None;

    let mut i = 1;
    while i < parts.len() {
        let part = parts[i];

        if part == "00:00"
            && (i + 1 >= parts.len() || parts[i + 1] == "+" || parts[i + 1].starts_with('+'))
        {
            i += 1;
            continue;
        }

        if part.parse::<NaiveTime>().is_ok() && start_time.is_none() {
            start_time = NaiveTime::parse_from_str(part, "%H:%M").ok();
            i += 1;

            if i < parts.len() && parts[i].parse::<NaiveTime>().is_ok() {
                end_time = NaiveTime::parse_from_str(parts[i], "%H:%M").ok();
                i += 1;
            }
            continue;
        }

        if part.starts_with('@') {
            location = Some(part[1..].to_string());
            i += 1;
            continue;
        }

        if part.starts_with('+') {
            let key_value = &part[1..];
            if key_value.starts_with("RRULE:") {
                rrule = Some(key_value[6..].to_string());
            } else if key_value.starts_with("EXDATE:") {
                if let Ok(d) = NaiveDate::parse_from_str(&key_value[7..], "%Y-%m-%d") {
                    exceptions.push(d);
                }
            } else {
                tags.push(key_value.to_string());
            }
            i += 1;
            continue;
        }

        title_parts.push(part);
        i += 1;
    }

    let title = title_parts.join(" ").trim_matches('"').to_string();

    if title.is_empty() {
        return None;
    }

    Some(Event {
        date,
        start_time,
        end_time,
        title,
        rrule,
        exceptions,
        tags,
        location,
    })
}

fn format_event(event: &Event) -> String {
    let mut parts = Vec::new();

    parts.push(event.date.format("%Y-%m-%d").to_string());

    if let Some(start) = event.start_time {
        parts.push(start.format("%H:%M").to_string());

        if let Some(end) = event.end_time {
            parts.push(end.format("%H:%M").to_string());
        }
    }

    if let Some(loc) = &event.location {
        parts.push(format!("@{}", loc));
    }

    parts.push(format!("\"{}\"", event.title));

    if let Some(rrule) = &event.rrule {
        parts.push(format!("+RRULE:{}", rrule));
    }

    for ex in &event.exceptions {
        parts.push(format!("+EXDATE:{}", ex.format("%Y-%m-%d")));
    }

    for tag in &event.tags {
        parts.push(format!("+{}", tag));
    }

    parts.join(" ")
}

pub fn export_ics(calendar: &Calendar, path: &str) -> Result<(), CalchemyError> {
    use icalendar::{Calendar, CalendarDateTime, Component, Event};

    let mut ical = Calendar::new();
    ical.append_property(icalendar::Property::new("VERSION", "2.0"));
    ical.append_property(icalendar::Property::new("PRODID", "-//Calchemy//EN"));

    for event in calendar.events() {
        let mut ics_event = Event::new();

        let start: CalendarDateTime = if let Some(time) = event.start_time {
            let ndt = event.date.and_time(time);
            CalendarDateTime::from(ndt)
        } else {
            let ndt = chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .unwrap()
                .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
            CalendarDateTime::from(ndt)
        };

        let end: CalendarDateTime = if let Some(time) = event.end_time {
            let ndt = event.date.and_time(time);
            CalendarDateTime::from(ndt)
        } else if event.start_time.is_none() {
            let ndt = event.date.and_hms_opt(23, 59, 59).unwrap();
            CalendarDateTime::from(ndt)
        } else {
            start.clone()
        };

        ics_event.summary(&event.title);
        ics_event.starts(start);
        ics_event.ends(end);

        if let Some(loc) = &event.location {
            ics_event.location(loc);
        }

        if let Some(rrule) = &event.rrule {
            ics_event.add_property("RRULE", rrule);
        }

        for tag in &event.tags {
            ics_event.add_property("CATEGORIES", tag);
        }

        for ex in &event.exceptions {
            ics_event.add_property("EXDATE", &ex.format("%Y-%m-%d").to_string());
        }

        ical.push(ics_event);
    }

    std::fs::write(path, ical.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_event() {
        let line = "2024-01-15 09:00 10:00 \"Team standup\"";
        let event = parse_event_line(line).unwrap();
        assert_eq!(event.date, NaiveDate::from_ymd_opt(2024, 1, 15).unwrap());
        assert_eq!(
            event.start_time,
            Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap())
        );
        assert_eq!(
            event.end_time,
            Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap())
        );
        assert_eq!(event.title, "Team standup");
    }

    #[test]
    fn test_parse_event_with_rrule() {
        let line = "2024-01-15 09:00 10:00 \"Team standup\" +RRULE:FREQ=WEEKLY";
        let event = parse_event_line(line).unwrap();
        assert_eq!(event.rrule, Some("FREQ=WEEKLY".to_string()));
    }

    #[test]
    fn test_parse_event_with_location() {
        let line = "2024-01-15 09:00 10:00 @office \"Team standup\"";
        let event = parse_event_line(line).unwrap();
        assert_eq!(event.location, Some("office".to_string()));
    }

    #[test]
    fn test_parse_allday_event() {
        let line = "2024-07-04 Holiday +RRULE:FREQ=YEARLY";
        let event = parse_event_line(line).unwrap();
        assert_eq!(event.start_time, None);
        assert_eq!(event.title, "Holiday");
        assert_eq!(event.rrule, Some("FREQ=YEARLY".to_string()));
    }

    #[test]
    fn test_format_event() {
        let event = Event {
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            start_time: Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
            end_time: Some(NaiveTime::from_hms_opt(10, 0, 0).unwrap()),
            title: "Team standup".to_string(),
            rrule: Some("FREQ=WEEKLY".to_string()),
            exceptions: Vec::new(),
            tags: vec!["work".to_string()],
            location: Some("office".to_string()),
        };

        let formatted = format_event(&event);
        assert!(formatted.contains("2024-01-15"));
        assert!(formatted.contains("+RRULE:FREQ=WEEKLY"));
        assert!(formatted.contains("+work"));
        assert!(formatted.contains("@office"));
    }
}
