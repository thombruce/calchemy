use chrono::{Datelike, NaiveDate};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Widget},
};

use crate::Calendar;

pub struct CalendarView<'a> {
    calendar: &'a Calendar,
    current_month: NaiveDate,
    selected_day: Option<NaiveDate>,
}

impl<'a> CalendarView<'a> {
    pub fn new(
        calendar: &'a Calendar,
        current_month: NaiveDate,
        selected_day: Option<NaiveDate>,
    ) -> Self {
        Self {
            calendar,
            current_month,
            selected_day,
        }
    }

    fn get_days_with_events(&self) -> Vec<NaiveDate> {
        let mut days = Vec::new();
        let month_start = self.current_month;
        let month_end = self.get_month_end();

        for event in self.calendar.events() {
            let event_start = event.date;
            let event_end = event.end_date.unwrap_or(event.date);

            // Skip if event doesn't overlap with this month
            if event_end < month_start || event_start > month_end {
                continue;
            }

            // Add all days in the overlap
            let overlap_start = event_start.max(month_start);
            let overlap_end = event_end.min(month_end);

            let mut current = overlap_start;
            while current <= overlap_end {
                if !days.contains(&current) {
                    days.push(current);
                }
                current = current + chrono::Duration::days(1);
            }
        }
        days
    }

    fn get_month_end(&self) -> NaiveDate {
        let (year, month) = if self.current_month.month() == 12 {
            (self.current_month.year() + 1, 1)
        } else {
            (self.current_month.year(), self.current_month.month() + 1)
        };
        NaiveDate::from_ymd_opt(year, month, 1).unwrap() - chrono::Duration::days(1)
    }

    fn get_first_day_of_week(&self) -> u32 {
        self.current_month.weekday().num_days_from_monday()
    }
}

impl<'a> Widget for CalendarView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Calendar ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White));

        block.clone().render(area, buf);

        let inner = block.inner(area);
        if inner.width < 20 || inner.height < 10 {
            return;
        }

        // Days of week header
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let cell_width = (inner.width as usize) / 7;

        for (i, day) in days.iter().enumerate() {
            let x = inner.x + (i as u16) * cell_width as u16;
            buf.set_string(x, inner.y, day, Style::default().fg(Color::LightBlue));
        }

        let events = self.get_days_with_events();
        let first_day = self.get_first_day_of_week() as usize;
        let days_in_month = self.get_month_end().day() as usize;
        let today = chrono::Utc::now().date_naive();

        let row_height = (inner.height - 1) as usize / 6;

        for day in 1..=days_in_month {
            let day_date = NaiveDate::from_ymd_opt(
                self.current_month.year(),
                self.current_month.month(),
                day as u32,
            )
            .unwrap();

            let idx = first_day + day - 1;
            let row = idx / 7;
            let col = idx % 7;

            let x = inner.x + (col as u16) * cell_width as u16;
            let y = inner.y + 1 + (row as u16) * row_height as u16;

            // Build base style with today and selected day styling
            let mut style = Style::default();

            // Highlight today - bold + underline (no background)
            if day_date == today {
                style = style.bold().underlined();
            }

            // Highlight selected day (most important - keep as-is)
            if self.selected_day == Some(day_date) {
                style = style.bg(Color::LightBlue).fg(Color::Black);
            }

            // Highlight days with events - add indicator symbol after date
            let day_str = if events.contains(&day_date) {
                // Add light blue color to event days (but not override selected day bg)
                if self.selected_day != Some(day_date) {
                    style = style.fg(Color::LightBlue);
                }
                format!("{:1}•", day)
            } else {
                format!("{:1} ", day)
            };
            buf.set_string(x, y, day_str, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn make_event(date: NaiveDate, title: &str) -> crate::Event {
        crate::Event {
            date,
            start_time: None,
            end_time: None,
            end_date: None,
            title: title.to_string(),
            rrule: None,
            exceptions: Vec::new(),
            tags: Vec::new(),
            hashtags: Vec::new(),
            location: None,
            completed: false,
        }
    }

    fn make_multi_day_event(start: NaiveDate, end: NaiveDate, title: &str) -> crate::Event {
        crate::Event {
            date: start,
            start_time: None,
            end_time: None,
            end_date: Some(end),
            title: title.to_string(),
            rrule: None,
            exceptions: Vec::new(),
            tags: Vec::new(),
            hashtags: Vec::new(),
            location: None,
            completed: false,
        }
    }

    fn make_calendar_with_events(events: Vec<crate::Event>) -> Calendar {
        let mut cal = Calendar::new();
        for event in events {
            cal.add_event(event);
        }
        cal
    }

    mod date_calculations {
        use super::*;

        #[test]
        fn test_get_month_end_returns_31_for_january() {
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None);

            let end = view.get_month_end();

            assert_eq!(end.day(), 31);
            assert_eq!(end.month(), 1);
        }

        #[test]
        fn test_get_month_end_returns_29_for_february_leap_year() {
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(
                &cal,
                NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(), // 2024 is leap year
                None,
            );

            let end = view.get_month_end();

            assert_eq!(end.day(), 29);
            assert_eq!(end.month(), 2);
        }

        #[test]
        fn test_get_month_end_returns_28_for_february_non_leap_year() {
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(
                &cal,
                NaiveDate::from_ymd_opt(2023, 2, 1).unwrap(), // 2023 is not leap year
                None,
            );

            let end = view.get_month_end();

            assert_eq!(end.day(), 28);
            assert_eq!(end.month(), 2);
        }

        #[test]
        fn test_get_month_end_returns_30_for_april() {
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 4, 1).unwrap(), None);

            let end = view.get_month_end();

            assert_eq!(end.day(), 30);
            assert_eq!(end.month(), 4);
        }

        #[test]
        fn test_get_month_end_returns_31_for_december() {
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap(), None);

            let end = view.get_month_end();

            assert_eq!(end.day(), 31);
            assert_eq!(end.month(), 12);
        }

        #[test]
        fn test_get_first_day_of_week_january_2024() {
            // January 1, 2024 is a Monday
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), None);

            let first = view.get_first_day_of_week();

            assert_eq!(first, 0); // Monday = 0
        }

        #[test]
        fn test_get_first_day_of_week_january_2025() {
            // January 1, 2025 is a Wednesday
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(), None);

            let first = view.get_first_day_of_week();

            assert_eq!(first, 2); // Wednesday = 2
        }
    }

    mod event_days {
        use super::*;

        #[test]
        fn test_get_days_with_events_returns_unique_days() {
            let cal = make_calendar_with_events(vec![
                make_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "Event 1"),
                make_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "Event 2"),
                make_event(NaiveDate::from_ymd_opt(2024, 3, 16).unwrap(), "Event 3"),
            ]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None);

            let days = view.get_days_with_events();

            assert_eq!(days.len(), 2);
            assert!(days.contains(&NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
            assert!(days.contains(&NaiveDate::from_ymd_opt(2024, 3, 16).unwrap()));
        }

        #[test]
        fn test_get_days_with_events_filters_by_month() {
            let cal = make_calendar_with_events(vec![
                make_event(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(), "March Event"),
                make_event(NaiveDate::from_ymd_opt(2024, 4, 15).unwrap(), "April Event"),
            ]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None);

            let days = view.get_days_with_events();

            assert_eq!(days.len(), 1);
            assert!(days.contains(&NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
        }

        #[test]
        fn test_get_days_with_events_returns_empty_for_no_events() {
            let cal = make_calendar_with_events(vec![]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None);

            let days = view.get_days_with_events();

            assert!(days.is_empty());
        }

        #[test]
        fn test_get_days_with_events_includes_all_days_of_multi_day_event() {
            let cal = make_calendar_with_events(vec![make_multi_day_event(
                NaiveDate::from_ymd_opt(2024, 3, 15).unwrap(),
                NaiveDate::from_ymd_opt(2024, 3, 20).unwrap(),
                "Conference",
            )]);
            let view = CalendarView::new(&cal, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap(), None);

            let days = view.get_days_with_events();

            // Should include all 6 days: 15, 16, 17, 18, 19, 20
            assert_eq!(days.len(), 6);
            for day in 15..=20 {
                assert!(days.contains(&NaiveDate::from_ymd_opt(2024, 3, day).unwrap()));
            }
        }
    }
}
