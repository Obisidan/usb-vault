//! TUI frontend for USB Vault.
//!
//! Neon-dark themed terminal UI built with ratatui + crossterm.
//! Navigable with mouse or keyboard.

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseButton, MouseEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, Padding, Paragraph};
use ratatui::Frame;
use std::io;
use std::io::stdout;
use std::path::PathBuf;

use crate::usb;
use crate::warning;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    MainMenu,
    EncryptSelectDevice,
    EncryptEnterPassword,
    EncryptConfirm,
    Encrypting,
    EncryptSuccess,
    DecryptSelectDevice,
    DecryptEnterPassword,
    Decrypting,
    DecryptSuccess,
    ExitWarning,
    PaperBackup,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Menu,
    Input,
    Dialog,
}

pub struct App {
    state: AppState,
    focus: Focus,
    menu_index: usize,
    device_path: String,
    password: String,
    confirm_password: String,
    input_field: usize,
    progress: f64,
    status_message: String,
    show_exit_warning: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::MainMenu,
            focus: Focus::Menu,
            menu_index: 0,
            device_path: String::new(),
            password: String::new(),
            confirm_password: String::new(),
            input_field: 0,
            progress: 0.0,
            status_message: String::new(),
            show_exit_warning: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        loop {
            terminal.draw(|f| self.draw(f))?;

            if self.state == AppState::Quit {
                break;
            }

            if event::poll(std::time::Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key),
                    Event::Mouse(mouse) => self.handle_mouse(mouse),
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        match self.state {
            AppState::MainMenu => self.handle_menu_key(key),
            AppState::EncryptSelectDevice => self.handle_input_key(key, 0),
            AppState::EncryptEnterPassword => self.handle_input_key(key, 1),
            AppState::EncryptConfirm => self.handle_input_key(key, 2),
            AppState::DecryptSelectDevice => self.handle_input_key(key, 0),
            AppState::DecryptEnterPassword => self.handle_input_key(key, 1),
            AppState::ExitWarning => self.handle_exit_warning_key(key),
            AppState::PaperBackup => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                    self.state = AppState::Quit;
                }
            }
            AppState::Encrypting | AppState::Decrypting => {}
            AppState::EncryptSuccess | AppState::DecryptSuccess => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Enter {
                    self.state = AppState::PaperBackup;
                }
            }
            AppState::Quit => {}
        }
    }

    fn handle_menu_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.menu_index > 0 {
                    self.menu_index -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.menu_index < 2 {
                    self.menu_index += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => match self.menu_index {
                0 => {
                    self.state = AppState::EncryptSelectDevice;
                    self.focus = Focus::Input;
                    self.input_field = 0;
                    self.device_path.clear();
                    self.password.clear();
                    self.confirm_password.clear();
                }
                1 => {
                    self.state = AppState::DecryptSelectDevice;
                    self.focus = Focus::Input;
                    self.input_field = 0;
                    self.device_path.clear();
                    self.password.clear();
                }
                2 => {
                    self.state = AppState::ExitWarning;
                    self.focus = Focus::Dialog;
                }
                _ => {}
            },
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = AppState::ExitWarning;
                self.focus = Focus::Dialog;
            }
            _ => {}
        }
    }

    fn handle_input_key(&mut self, key: event::KeyEvent, field: usize) {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::MainMenu;
                self.focus = Focus::Menu;
            }
            KeyCode::BackTab => {
                if field > 0 {
                    self.input_field = field - 1;
                }
            }
            KeyCode::Tab | KeyCode::Enter => {
                if field == 0 {
                    if !self.device_path.is_empty() {
                        if self.state == AppState::EncryptSelectDevice {
                            self.state = AppState::EncryptEnterPassword;
                            self.input_field = 1;
                        } else if self.state == AppState::DecryptSelectDevice {
                            self.state = AppState::DecryptEnterPassword;
                            self.input_field = 1;
                        }
                    }
                } else if field == 1 {
                    if self.state == AppState::EncryptEnterPassword {
                        self.state = AppState::EncryptConfirm;
                        self.input_field = 2;
                    } else if self.state == AppState::DecryptEnterPassword {
                        self.start_decrypt();
                    }
                } else if field == 2 {
                    if self.password == self.confirm_password && !self.password.is_empty() {
                        self.start_encrypt();
                    } else {
                        self.status_message = "Passwords do not match!".to_string();
                    }
                }
            }
            KeyCode::Backspace => {
                if field == 0 {
                    self.device_path.pop();
                } else if field == 1 {
                    self.password.pop();
                } else if field == 2 {
                    self.confirm_password.pop();
                }
            }
            KeyCode::Char(c) => {
                if field == 0 {
                    self.device_path.push(c);
                } else if field == 1 {
                    self.password.push(c);
                } else if field == 2 {
                    self.confirm_password.push(c);
                }
            }
            _ => {}
        }
    }

    fn handle_exit_warning_key(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Enter => {
                self.state = AppState::Quit;
            }
            KeyCode::Esc => {
                self.state = AppState::MainMenu;
                self.focus = Focus::Menu;
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, _mouse: event::MouseEvent) {
        // Reserved for future clickable elements
    }

    fn start_encrypt(&mut self) {
        self.state = AppState::Encrypting;
        self.progress = 0.0;
        self.status_message = "Encrypting...".to_string();

        let path = PathBuf::from(&self.device_path);
        let password = self.password.clone();

        match usb::encrypt_device(&path, &password) {
            Ok(()) => {
                self.state = AppState::EncryptSuccess;
                self.progress = 1.0;
                self.status_message = "Encryption complete!".to_string();
            }
            Err(e) => {
                self.state = AppState::MainMenu;
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    fn start_decrypt(&mut self) {
        self.state = AppState::Decrypting;
        self.progress = 0.0;
        self.status_message = "Decrypting...".to_string();

        let path = PathBuf::from(&self.device_path);
        let password = self.password.clone();

        match usb::decrypt_device(&path, &password) {
            Ok(()) => {
                self.state = AppState::DecryptSuccess;
                self.progress = 1.0;
                self.status_message = "Decryption complete!".to_string();
            }
            Err(e) => {
                self.state = AppState::MainMenu;
                self.status_message = format!("Error: {}", e);
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.size();

        frame.render_widget(
            Block::default().style(Style::default().bg(Color::Black)),
            area,
        );

        match self.state {
            AppState::MainMenu => self.draw_main_menu(frame, area),
            AppState::EncryptSelectDevice
            | AppState::EncryptEnterPassword
            | AppState::EncryptConfirm => {
                self.draw_encrypt_form(frame, area);
            }
            AppState::DecryptSelectDevice | AppState::DecryptEnterPassword => {
                self.draw_decrypt_form(frame, area);
            }
            AppState::Encrypting | AppState::Decrypting => {
                self.draw_progress(frame, area);
            }
            AppState::EncryptSuccess | AppState::DecryptSuccess => {
                self.draw_success(frame, area);
            }
            AppState::ExitWarning => {
                self.draw_main_menu(frame, area);
                self.draw_centered_overlay(frame, area, warning::exit_warning_paragraph());
            }
            AppState::PaperBackup => {
                self.draw_centered_overlay(frame, area, warning::paper_backup_paragraph());
            }
            AppState::Quit => {}
        }
    }

    fn draw_main_menu(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(3),
                Constraint::Length(8),
                Constraint::Min(3),
                Constraint::Length(2),
            ])
            .split(area);

        // Title
        let title = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "USB",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "VAULT",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                "pure rust encryption suite",
                Style::default().fg(Color::DarkGray),
            )]),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
        frame.render_widget(title, chunks[0]);

        // Separator
        let dash = "-";
        let sep = Paragraph::new(Line::from(vec![Span::styled(
            dash.repeat(50),
            Style::default().fg(Color::DarkGray),
        )]))
        .alignment(Alignment::Center);
        frame.render_widget(sep, chunks[1]);

        // Menu
        let menu_items = [
            ("[E] Encrypt a USB drive", Color::Green),
            ("[D] Decrypt a USB drive", Color::Blue),
            ("[Q] Quit", Color::Red),
        ];

        let list_items: Vec<ListItem> = menu_items
            .iter()
            .enumerate()
            .map(|(i, (text, color))| {
                let style = if i == self.menu_index {
                    Style::default()
                        .fg(*color)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(*color)
                };
                ListItem::new(Line::from(Span::styled(*text, style)))
            })
            .collect();

        let menu = List::new(list_items).block(
            Block::default()
                .title(" MAIN MENU ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .padding(Padding::uniform(1)),
        );
        frame.render_widget(menu, chunks[2]);

        // Status message
        if !self.status_message.is_empty() {
            let status = Paragraph::new(Line::from(Span::styled(
                &self.status_message,
                Style::default().fg(Color::Yellow),
            )))
            .alignment(Alignment::Center);
            frame.render_widget(status, chunks[3]);
        }

        // Footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("j/k navigate", Style::default().fg(Color::DarkGray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter select", Style::default().fg(Color::DarkGray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("q quit", Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[4]);
    }

    fn draw_input_field(
        frame: &mut Frame,
        chunk: Rect,
        label: &str,
        value: &str,
        placeholder: &str,
        field_active: bool,
        title: &str,
    ) {
        let border_style = if field_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let display_val = if value.is_empty() {
            placeholder.to_string()
        } else {
            value.to_string()
        };
        let val_color = if value.is_empty() {
            Color::DarkGray
        } else {
            Color::Green
        };
        let input = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled(label, Style::default().fg(Color::White)),
            Span::styled(display_val, Style::default().fg(val_color)),
        ])]))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(input, chunk);
    }

    fn draw_password_field(
        frame: &mut Frame,
        chunk: Rect,
        label: &str,
        value: &str,
        placeholder: &str,
        field_active: bool,
        title: &str,
    ) {
        let border_style = if field_active {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let masked = if value.is_empty() {
            placeholder.to_string()
        } else {
            "*".repeat(value.len())
        };
        let val_color = if value.is_empty() {
            Color::DarkGray
        } else {
            Color::Green
        };
        let input = Paragraph::new(Text::from(vec![Line::from(vec![
            Span::styled(label, Style::default().fg(Color::White)),
            Span::styled(masked, Style::default().fg(val_color)),
        ])]))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        );
        frame.render_widget(input, chunk);
    }

    fn draw_encrypt_form(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        let title = Paragraph::new(Line::from(Span::styled(
            "ENCRYPT USB DRIVE",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        );
        frame.render_widget(title, chunks[0]);

        Self::draw_input_field(
            frame, chunks[1], "Device: ", &self.device_path, "/dev/sdX",
            self.input_field == 0, " Device Path ",
        );

        Self::draw_password_field(
            frame, chunks[2], "Password: ", &self.password, "enter password...",
            self.input_field == 1, " Password ",
        );

        if self.state == AppState::EncryptConfirm {
            Self::draw_password_field(
                frame, chunks[3], "Confirm:  ", &self.confirm_password, "re-enter password...",
                self.input_field == 2, " Confirm Password ",
            );
        }

        let instr = Paragraph::new(Line::from(vec![
            Span::styled("Tab: switch field", Style::default().fg(Color::DarkGray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter: next", Style::default().fg(Color::DarkGray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc: back", Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(instr, chunks[4]);
    }

    fn draw_decrypt_form(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(1),
            ])
            .split(area);

        let title = Paragraph::new(Line::from(Span::styled(
            "DECRYPT USB DRIVE",
            Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
        frame.render_widget(title, chunks[0]);

        Self::draw_input_field(
            frame, chunks[1], "Device: ", &self.device_path, "/dev/sdX",
            self.input_field == 0, " Device Path ",
        );

        Self::draw_password_field(
            frame, chunks[2], "Password: ", &self.password, "enter password...",
            self.input_field == 1, " Password ",
        );

        let instr = Paragraph::new(Line::from(vec![
            Span::styled("Tab: switch field", Style::default().fg(Color::DarkGray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter: next", Style::default().fg(Color::DarkGray)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc: back", Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(instr, chunks[3]);
    }

    fn draw_progress(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(4)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(area);

        let title = Paragraph::new(Line::from(Span::styled(
            &self.status_message,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);

        let progress_bar = Gauge::default()
            .block(
                Block::default()
                    .title(" Progress ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .gauge_style(Style::default().fg(Color::Green).bg(Color::DarkGray))
            .percent((self.progress * 100.0) as u16);
        frame.render_widget(progress_bar, chunks[1]);
    }

    fn draw_success(&self, frame: &mut Frame, area: Rect) {
        let text = Text::from(vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "SUCCESS!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                &self.status_message,
                Style::default().fg(Color::White),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press Enter to continue.",
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Remember: write your password on paper!",
                Style::default().fg(Color::Yellow),
            )]),
        ]);

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .title(" DONE ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .padding(Padding::uniform(2)),
            );
        frame.render_widget(paragraph, area);
    }

    fn draw_centered_overlay(&self, frame: &mut Frame, area: Rect, widget: Paragraph) {
        let overlay = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };
        frame.render_widget(Clear, overlay);
        frame.render_widget(widget, overlay);
    }
}

pub fn launch() -> io::Result<()> {
    let mut app = App::new();
    app.run()
}
