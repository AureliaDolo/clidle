use std::{
    collections::HashMap,
    fmt::{self},
    fs,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::{Deserialize, Serialize};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Default, Serialize, Deserialize)]
struct Item {
    cps: f64,
    cost: u64,
    id: u64,
    name: String,
    long_name: String,
}

enum InputMode {
    Buy,
    Sell,
    Normal,
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    owned_items: HashMap<u64, u64>,
    code_lines: f64,
    items_index: HashMap<u64, Item>,
    error: Option<ClidleError>,
}

impl App {
    fn new() -> App {
        let items: Vec<Item> = serde_json::from_str(
            &fs::read_to_string("items.json").expect("Should have been able to read the file"),
        )
        .unwrap();
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            owned_items: HashMap::new(),
            code_lines: 0.,
            items_index: items.into_iter().map(|i| (i.id, i)).collect(),
            error: None,
        }
    }
    fn update(&mut self) {
        for (item_id, item_count) in self.owned_items.iter() {
            let item_type = self.items_index.get(item_id).unwrap();
            self.code_lines += *item_count as f64 * item_type.cps;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[derive(Debug)]
enum ClidleError {
    BuyingItemNotKnown(String),
}

impl Error for ClidleError {}

impl fmt::Display for ClidleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// States of the game.
enum GameState {
    /// Item you wanna buy need to be parsed.
    BuyItem(String),
    /// Nothing from input, at least it's fast to manage
    Noop,
    /// Stop gaming, go code for work government said.
    Quit,
}

/// Check if you can buy an item and buy it.
///
/// ## Errors
///
/// May return the infamous `ClideError::BuyingItemNotKnown` if
/// your item is not known.
fn buy_item(app: &mut App, item: String) -> Result<(), ClidleError> {
    let (item_id, item_type) = app
        .items_index
        .iter()
        .find(|(_, i)| i.name == item)
        .ok_or_else(|| ClidleError::BuyingItemNotKnown(item.clone()))?;

    let count = 1; // TODO buy multiple
    if item_type.cost * count < app.code_lines.floor() as u64 {
        app.code_lines -= (item_type.cost * count) as f64;
        app.owned_items
            .entry(*item_id)
            .and_modify(|e| *e += count)
            .or_insert(count);
    }
    Ok(())
}

/// Handles inputs if it's successful you get a GameState if not you may end up with
/// an IO error.
fn handle_input(mut app: &mut App) -> io::Result<GameState> {
    if let Event::Key(key) = event::read()? {
        match app.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('b') => {
                    app.input_mode = InputMode::Buy;
                }
                KeyCode::Char('c') => {
                    app.code_lines += 1.;
                }
                KeyCode::Char('s') => {
                    app.input_mode = InputMode::Sell;
                }
                KeyCode::Char('q') => {
                    return Ok(GameState::Quit);
                }
                _ => {}
            },
            InputMode::Buy => match key.code {
                KeyCode::Char(c) => {
                    app.input.push(c);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Enter => {
                    return Ok(GameState::BuyItem(app.input.drain(..).collect()));
                }
                KeyCode::Esc => {
                    app.input_mode = InputMode::Normal;
                }
                _ => {}
            },
            InputMode::Sell => todo!(),
        }
    }
    Ok(GameState::Noop)
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>> {
    let mut last_tick = Instant::now();

    loop {
        if last_tick.elapsed() >= Duration::from_secs(1) {
            app.update();
            last_tick = Instant::now();
        }
        terminal.draw(|f| ui(f, &mut app))?;

        let state = handle_input(&mut app)?;
        match state {
            GameState::BuyItem(item_string) => {
                if let Err(e) = buy_item(&mut app, item_string) {
                    app.error = Some(e)
                }
            }
            GameState::Noop => {}
            GameState::Quit => return Ok(()),
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw(format!("Owning {:.2} code lines, ", app.code_lines)),
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to code, "),
                Span::styled("b", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start buying, "),
                Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start selling."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Buy => (
            vec![
                Span::raw(format!("Owning {:.2} code lines, ", app.code_lines)),
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop buying, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
        InputMode::Sell => (
            vec![
                Span::raw(format!("Owning {:.2} code lines, ", app.code_lines)),
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop selling, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to record the message"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Buy => Style::default().fg(Color::Green),
            InputMode::Sell => Style::default().fg(Color::Red),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Sell | InputMode::Buy => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let owned: Vec<ListItem> = app
        .owned_items
        .iter()
        .map(|(item_id, item_count)| {
            let item_type = app.items_index.get(item_id).unwrap();

            let content = vec![Spans::from(Span::raw(format!(
                "Owning {item_count} {} producing a total of {:.2} code line per second",
                item_type.long_name,
                *item_count as f64 * item_type.cps
            )))];
            ListItem::new(content)
        })
        .collect();
    let owned = List::new(owned).block(Block::default().borders(Borders::ALL).title("Owned"));
    f.render_widget(owned, chunks[2]);

    let mut messages: Vec<ListItem> = app
        .items_index
        .values()
        .map(|item| {
            let content = vec![Spans::from(Span::raw(format!(
                "Buy {}(as {}) producing {:.2} code lines per second",
                item.long_name, item.name, item.cps
            )))];
            ListItem::new(content)
        })
        .collect();

    if let Some(error) = app.error.take() {
        messages.push(ListItem::new(Spans::from(Span::raw(format!(
            "Error: {error}",
        )))))
    }

    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[3]);
}
