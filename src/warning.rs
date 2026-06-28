//! Warning dialogs and paper-backup reminders.
//!
//! Displays prominent warnings before the user closes the application.

use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

/// Create the exit warning message
pub fn exit_warning() -> Text<'static> {
    Text::from(vec![
        Line::from(vec![
            Span::styled("WARNING", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("This encryption is ", Style::default().fg(Color::White)),
            Span::styled("PERMANENT", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(".", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from("Without your password, the data on this USB drive is"),
        Line::from(vec![
            Span::styled(" GONE FOREVER.", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("There is no recovery. No backdoor. No reset button."),
        Line::from("The only way to restore the drive is to"),
        Line::from(vec![
            Span::styled(" format/wipe it completely.", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(">>> ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("WRITE YOUR PASSWORD DOWN ON PAPER", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" <<<", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("Store the paper somewhere safe. A safe, a locked drawer,"),
        Line::from("or with someone you trust. Digital copies can be stolen."),
        Line::from("Paper cannot be hacked."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC to go back and write it down, or ", Style::default().fg(Color::DarkGray)),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to exit anyway.", Style::default().fg(Color::DarkGray)),
        ]),
    ])
}

/// Create the initial encryption warning
pub fn encrypt_warning() -> Text<'static> {
    Text::from(vec![
        Line::from(vec![
            Span::styled("ENCRYPT USB DRIVE", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("This will encrypt the entire USB drive."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Requirements:", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  * You must be running as root (sudo)"),
        Line::from("  * The drive must be unmounted"),
        Line::from("  * You will need a strong password"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Enter to continue, ESC to cancel.", Style::default().fg(Color::White)),
        ]),
    ])
}

/// Create the initial decrypt warning
pub fn decrypt_warning() -> Text<'static> {
    Text::from(vec![
        Line::from(vec![
            Span::styled("DECRYPT USB DRIVE", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("This will decrypt the drive in-place."),
        Line::from("You need the original password."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press Enter to continue, ESC to cancel.", Style::default().fg(Color::White)),
        ]),
    ])
}

/// Create a styled paragraph for the exit warning dialog
pub fn exit_warning_paragraph() -> Paragraph<'static> {
    Paragraph::new(exit_warning())
        .block(
            Block::default()
                .title(" EXIT WARNING ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

/// Paper backup reminder (shown after encryption completes)
pub fn paper_backup_reminder() -> Text<'static> {
    Text::from(vec![
        Line::from(vec![
            Span::styled("IMPORTANT", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK)),
        ]),
        Line::from(""),
        Line::from("Before closing this program, write your password on paper."),
        Line::from(""),
        Line::from("If you lose the password, your data cannot be recovered."),
        Line::from(vec![
            Span::styled("There is no recovery option.", Style::default().fg(Color::Red)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ESC to acknowledge and exit.", Style::default().fg(Color::Green)),
        ]),
    ])
}

/// Create a paragraph for the paper backup reminder
pub fn paper_backup_paragraph() -> Paragraph<'static> {
    Paragraph::new(paper_backup_reminder())
        .block(
            Block::default()
                .title(" BACKUP YOUR PASSWORD ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}
