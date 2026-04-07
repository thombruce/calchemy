use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Stylize,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

#[derive(Debug, Clone, PartialEq)]
pub enum DialogChoice {
    None,
    Confirm,
    Option1,
    Option2,
    Option3,
    Cancel,
}

pub struct ConfirmDialog<'a> {
    title: &'a str,
    message: &'a str,
    option1_label: Option<&'a str>,
    option2_label: Option<&'a str>,
    option3_label: Option<&'a str>,
}

impl<'a> ConfirmDialog<'a> {
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            title,
            message,
            option1_label: None,
            option2_label: None,
            option3_label: None,
        }
    }

    pub fn with_option1(mut self, label: &'a str) -> Self {
        self.option1_label = Some(label);
        self
    }

    pub fn with_option2(mut self, label: &'a str) -> Self {
        self.option2_label = Some(label);
        self
    }

    pub fn with_option3(mut self, label: &'a str) -> Self {
        self.option3_label = Some(label);
        self
    }

    pub fn get_option_label(&self, idx: usize) -> Option<&'a str> {
        match idx {
            0 => self.option1_label,
            1 => self.option2_label,
            2 => self.option3_label,
            _ => None,
        }
    }

    pub fn option_count(&self) -> usize {
        let mut count = 0;
        if self.option1_label.is_some() {
            count += 1;
        }
        if self.option2_label.is_some() {
            count += 1;
        }
        if self.option3_label.is_some() {
            count += 1;
        }
        count
    }
}

impl<'a> Widget for ConfirmDialog<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 40 || area.height < 10 {
            return;
        }

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .bg(Color::Black);

        block.clone().render(area, buf);

        let inner = block.inner(area);

        // Render message
        let msg_y = inner.y + 1;
        let msg_width = inner.width as usize;
        let words: Vec<&str> = self.message.split_whitespace().collect();
        let mut line = String::new();
        let mut y = msg_y;

        for word in words {
            if line.len() + word.len() + 1 > msg_width - 2 {
                buf.set_string(inner.x + 1, y, &line, Style::default().fg(Color::White));
                y += 1;
                line = word.to_string();
            } else {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
        }
        if !line.is_empty() {
            buf.set_string(inner.x + 1, y, &line, Style::default().fg(Color::White));
            y += 1;
        }

        // Render options
        y += 1;
        let options_y = y;

        if let Some(label) = self.option1_label {
            let opt_str = format!("[1] {}", label);
            let style = Style::default().fg(Color::Yellow);
            buf.set_string(inner.x + 1, options_y, &opt_str, style);
        }

        if let Some(label) = self.option2_label {
            let opt_str = format!("[2] {}", label);
            let style = Style::default().fg(Color::Yellow);
            buf.set_string(inner.x + 1, options_y + 1, &opt_str, style);
        }

        if let Some(label) = self.option3_label {
            let opt_str = format!("[3] {}", label);
            let style = Style::default().fg(Color::Yellow);
            buf.set_string(inner.x + 1, options_y + 2, &opt_str, style);
        }

        // Render cancel hint
        let hint_y = options_y + self.option_count() as u16 + 1;
        let hint = "Press 1/2/3 to select, Esc to cancel";
        let hint_x = inner.x + 1;
        buf.set_string(hint_x, hint_y, hint, Style::default().fg(Color::DarkGray));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_dialog_sets_title_and_message() {
        let dialog = ConfirmDialog::new("Test Title", "Test Message");
        assert_eq!(dialog.title, "Test Title");
        assert_eq!(dialog.message, "Test Message");
    }

    #[test]
    fn test_with_options_chaining() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_option1("Option 1")
            .with_option2("Option 2")
            .with_option3("Option 3");

        assert_eq!(dialog.option1_label, Some("Option 1"));
        assert_eq!(dialog.option2_label, Some("Option 2"));
        assert_eq!(dialog.option3_label, Some("Option 3"));
    }

    #[test]
    fn test_with_option1_sets_option1() {
        let dialog = ConfirmDialog::new("Title", "Message").with_option1("First");
        assert_eq!(dialog.option1_label, Some("First"));
        assert_eq!(dialog.option2_label, None);
        assert_eq!(dialog.option3_label, None);
    }

    #[test]
    fn test_with_option2_sets_option2() {
        let dialog = ConfirmDialog::new("Title", "Message").with_option2("Second");
        assert_eq!(dialog.option1_label, None);
        assert_eq!(dialog.option2_label, Some("Second"));
        assert_eq!(dialog.option3_label, None);
    }

    #[test]
    fn test_with_option3_sets_option3() {
        let dialog = ConfirmDialog::new("Title", "Message").with_option3("Third");
        assert_eq!(dialog.option1_label, None);
        assert_eq!(dialog.option2_label, None);
        assert_eq!(dialog.option3_label, Some("Third"));
    }

    #[test]
    fn test_get_option_label_returns_correct_labels() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_option1("First")
            .with_option2("Second")
            .with_option3("Third");

        assert_eq!(dialog.get_option_label(0), Some("First"));
        assert_eq!(dialog.get_option_label(1), Some("Second"));
        assert_eq!(dialog.get_option_label(2), Some("Third"));
        assert_eq!(dialog.get_option_label(3), None);
    }

    #[test]
    fn test_option_count_zero_when_no_options() {
        let dialog = ConfirmDialog::new("Title", "Message");
        assert_eq!(dialog.option_count(), 0);
    }

    #[test]
    fn test_option_count_one_with_single_option() {
        let dialog = ConfirmDialog::new("Title", "Message").with_option1("Option");
        assert_eq!(dialog.option_count(), 1);
    }

    #[test]
    fn test_option_count_two_with_two_options() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_option1("Option 1")
            .with_option2("Option 2");
        assert_eq!(dialog.option_count(), 2);
    }

    #[test]
    fn test_option_count_three_with_all_options() {
        let dialog = ConfirmDialog::new("Title", "Message")
            .with_option1("Option 1")
            .with_option2("Option 2")
            .with_option3("Option 3");
        assert_eq!(dialog.option_count(), 3);
    }

    #[test]
    fn test_dialog_choice_variants() {
        assert_eq!(DialogChoice::None, DialogChoice::None);
        assert_eq!(DialogChoice::Confirm, DialogChoice::Confirm);
        assert_eq!(DialogChoice::Option1, DialogChoice::Option1);
        assert_eq!(DialogChoice::Option2, DialogChoice::Option2);
        assert_eq!(DialogChoice::Option3, DialogChoice::Option3);
        assert_eq!(DialogChoice::Cancel, DialogChoice::Cancel);
    }
}
