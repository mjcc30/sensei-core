use std::io::{self};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

// Use ratatui's re-export of crossterm
use ratatui::crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use reqwest::Client;
use tui_textarea::TextArea;

// Internal event bus
enum AppEvent {
    Input(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    ServerResponse(String),
}

#[derive(PartialEq)]
enum ActiveArea {
    Input,
    Chat,
}

pub async fn run_tui(client: Client, base_url: String) -> anyhow::Result<()> {
    // 1. Setup Terminal with Mouse Support
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Setup Async Communication
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(250);

    // Event Polling Thread
    let tx_input = tx.clone();
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll failed") {
                match event::read().expect("read failed") {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press
                            && tx_input.send(AppEvent::Input(key)).is_err()
                        {
                            return;
                        }
                    }
                    Event::Mouse(mouse) => {
                        if tx_input.send(AppEvent::Mouse(mouse)).is_err() {
                            return;
                        }
                    }
                    _ => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if tx_input.send(AppEvent::Tick).is_err() {
                    return;
                }
                last_tick = Instant::now();
            }
        }
    });

    // 3. App State
    let mut messages: Vec<String> = vec![
        "🦈 SENSEI CORE v0.1.3 - TUI ENHANCED".to_string(),
        "Controls: [TAB] Switch Focus | [Mouse] Scroll | [Up/Down] History".to_string(),
        "".to_string(),
    ];

    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_placeholder_text("Enter query...");

    let mut is_loading = false;
    let mut list_state = ListState::default();
    let mut active_area = ActiveArea::Input;
    let mut auto_scroll = true;

    // History State
    let mut input_history: Vec<String> = Vec::new();
    let mut history_pos = 0;

    // Spinner
    let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let mut spinner_index = 0;

    // 4. Main Loop
    loop {
        // Auto-scroll logic
        if auto_scroll && !messages.is_empty() {
            list_state.select(Some(messages.len() - 1));
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1),    // Chat
                    Constraint::Length(3), // Input
                ])
                .split(f.area());

            // --- Chat Area ---
            let mut inside_code_block = false;
            let chat_items: Vec<ListItem> = messages
                .iter()
                .map(|m| {
                    let mut style = Style::default().fg(Color::White); // Default White

                    if m.starts_with("```") {
                        inside_code_block = !inside_code_block;
                        style = style.fg(Color::Cyan);
                    } else if inside_code_block {
                        style = style.fg(Color::Cyan);
                    } else if m.starts_with("> ") {
                        style = style.fg(Color::Yellow);
                    } else if m.starts_with("Error:") {
                        style = style.fg(Color::Red);
                    } else if m.starts_with("# ") || m.starts_with("## ") {
                        style = style.fg(Color::Blue).add_modifier(Modifier::BOLD);
                    } else if m.contains("🦈 SENSEI") {
                        style = style.fg(Color::Magenta).add_modifier(Modifier::BOLD);
                    }

                    ListItem::new(Line::from(vec![Span::styled(m, style)]))
                })
                .collect();

            // Dynamic Border Color based on Focus
            let chat_border_color = if active_area == ActiveArea::Chat {
                Color::Green
            } else {
                Color::Magenta
            };

            let chat_block = Block::default()
                .borders(Borders::ALL)
                .title(" SENSEI TERMINAL (TAB to Switch) ")
                .border_style(Style::default().fg(chat_border_color));

            let chat_list = List::new(chat_items)
                .block(chat_block)
                .highlight_symbol(">> ")
                .highlight_style(Style::default().bg(Color::Rgb(30, 30, 30)));

            f.render_stateful_widget(chat_list, chunks[0], &mut list_state);

            // --- Input Area ---
            if is_loading {
                let loading_block = Block::default()
                    .borders(Borders::ALL)
                    .title(" STATUS ")
                    .border_style(Style::default().fg(Color::Yellow));

                let frame = spinner_frames[spinner_index];
                let text = format!("{} ANALYZING DATA STREAM...", frame);
                let spinner = Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .block(loading_block);
                f.render_widget(spinner, chunks[1]);
            } else {
                let input_border_color = if active_area == ActiveArea::Input {
                    Color::Green
                } else {
                    Color::Cyan
                };

                textarea.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" COMMAND INPUT ")
                        .border_style(Style::default().fg(input_border_color)),
                );
                f.render_widget(&textarea, chunks[1]);
            }
        })?;

        let event = rx.recv()?;
        match event {
            // --- MOUSE Handling ---
            AppEvent::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::ScrollDown => {
                        auto_scroll = false;
                        let i = list_state.selected().unwrap_or(0);
                        // Scroll Down
                        let new_idx = (i + 3).min(messages.len().saturating_sub(1));
                        list_state.select(Some(new_idx));
                        if new_idx >= messages.len().saturating_sub(1) {
                            auto_scroll = true;
                        }
                    }
                    MouseEventKind::ScrollUp => {
                        auto_scroll = false;
                        let i = list_state.selected().unwrap_or(0);
                        // Scroll Up
                        list_state.select(Some(i.saturating_sub(3)));
                    }
                    _ => {}
                }
            }

            // --- KEYBOARD Handling ---
            AppEvent::Input(key) => {
                if is_loading {
                    continue;
                }

                // Global Shortcuts
                if key.code == KeyCode::Tab {
                    active_area = match active_area {
                        ActiveArea::Input => ActiveArea::Chat,
                        ActiveArea::Chat => ActiveArea::Input,
                    };
                    continue;
                }

                if active_area == ActiveArea::Chat {
                    // Chat Navigation Mode
                    match key.code {
                        KeyCode::Down | KeyCode::PageDown => {
                            auto_scroll = false;
                            let i = list_state.selected().unwrap_or(0);
                            let new_idx = (i + 1).min(messages.len().saturating_sub(1));
                            list_state.select(Some(new_idx));
                            if new_idx >= messages.len().saturating_sub(1) {
                                auto_scroll = true;
                            }
                        }
                        KeyCode::Up | KeyCode::PageUp => {
                            auto_scroll = false;
                            let i = list_state.selected().unwrap_or(0);
                            list_state.select(Some(i.saturating_sub(1)));
                        }
                        KeyCode::Esc => break, // Quit from Chat mode too? Yes.
                        _ => {}
                    }
                } else {
                    // Input Mode
                    match key.code {
                        KeyCode::Esc => break,

                        // History Navigation (Up/Down in Input Mode)
                        KeyCode::Up => {
                            if history_pos > 0 {
                                history_pos -= 1;
                                textarea = TextArea::default();
                                textarea.set_cursor_line_style(Style::default());
                                textarea.set_placeholder_text("Enter query...");
                                textarea.insert_str(&input_history[history_pos]);
                            }
                        }
                        KeyCode::Down => {
                            if history_pos < input_history.len() {
                                history_pos += 1;
                                textarea = TextArea::default();
                                textarea.set_cursor_line_style(Style::default());
                                textarea.set_placeholder_text("Enter query...");
                                if history_pos < input_history.len() {
                                    textarea.insert_str(&input_history[history_pos]);
                                }
                            }
                        }

                        KeyCode::Enter => {
                            let input = textarea.lines()[0].trim().to_string();
                            if !input.is_empty() {
                                if input == "exit" {
                                    break;
                                }

                                messages.push(format!("> {}", input));
                                input_history.push(input.clone());
                                history_pos = input_history.len();

                                textarea.delete_line_by_head();
                                textarea.delete_line_by_end();

                                is_loading = true;
                                auto_scroll = true; // Snap to bottom on send
                                let client = client.clone();
                                let base_url = base_url.clone();
                                let tx = tx.clone();

                                tokio::spawn(async move {
                                    let res = crate::ask_api(&client, &base_url, &input).await;
                                    let msg = match res {
                                        Ok(content) => content,
                                        Err(e) => format!("Error: {}", e),
                                    };
                                    tx.send(AppEvent::ServerResponse(msg)).unwrap();
                                });
                            }
                        }
                        _ => {
                            textarea.input(key);
                        }
                    }
                }
            }
            AppEvent::Tick => {
                if is_loading {
                    spinner_index = (spinner_index + 1) % spinner_frames.len();
                }
            }
            AppEvent::ServerResponse(msg) => {
                messages.push("".to_string());
                for line in msg.lines() {
                    messages.push(line.to_string());
                }
                messages.push("".to_string());
                is_loading = false;
                auto_scroll = true;
            }
        }
    }

    // 5. Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
