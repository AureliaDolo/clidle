use std::{
    collections::HashMap,
    fmt::{self},
    fs,
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
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

/// Un item pour produire du code
///
/// # note
/// les trois / indiquent un commentaire de documentation
/// Ils permettent d'indiquer que ce commentaire documente
/// ce qui suit (fonction, struct, mod, etc)
///
/// Ils permettent notamment de générer la documentation
/// complète du projet (et de ces dépendances) au format html avec `cargo doc`
///
/// les commentaire classique se font avec deux /
/// ou bien entre /* votre commentaire */
#[derive(Debug, Default, Serialize, Deserialize)]
struct Item {
    /// code par seconde
    cps: f64,
    /// cout unitaire
    cost: u64,
    /// identifiant unique
    #[serde(default)]
    id: usize,
    /// nom pour interagir avec l'item
    name: String,
    /// nom complet à afficher de l'item
    long_name: String,
}

/// Les input auront des effets différents selon
/// dans quel mode on se situe
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
    /// Items you bought (item_id, item count)
    owned_items: HashMap<usize, u64>,
    /// total owned code lines
    code_lines: f64,
    /// available items: index is item id
    items_index: Vec<Item>,
    /// some if an error occurred
    error: Result<(), ClidleError>,
}

impl App {
    fn new() -> App {
        let mut items_index: Vec<Item> = serde_json::from_str(
            &fs::read_to_string("items.json").expect("Should have been able to read the file"),
        )
        .unwrap();
        // set ids
        items_index
            .iter_mut()
            .enumerate()
            .for_each(|(id, item)| item.id = id);
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            owned_items: HashMap::new(),
            code_lines: 0.,
            items_index,
            error: Ok(()),
        }
    }

    fn update(&mut self) {
        for (item_id, item_count) in self.owned_items.iter() {
            let item_type = self.items_index.get(*item_id).unwrap();
            self.code_lines += *item_count as f64 * item_type.cps;
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    // le ? permet de faire un early return en cas d'erreur.
    // https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator
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
        println!("{err:?}")
    }

    Ok(())
}

#[derive(Debug)]
enum ClidleError {
    BuyingItemNotKnown(String),
}

// les deux blocs impl suivant sont des implémentaition concrètes de trait de la lib standard.
// par exemple, avoir implémenter le trait display, permet de directement
// println!("{}", clidle_error); où clidle_error est de type ClidleError

impl Error for ClidleError {}

impl fmt::Display for ClidleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
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
/// May return the infamous `ClidleError::BuyingItemNotKnown` if
/// your item is not known.
fn buy_item(app: &mut App, item: String) -> Result<(), ClidleError> {
    let (item_id, item_type) = app
        .items_index
        .iter()
        .enumerate()
        .find(|(_, i)| i.name == item)
        .ok_or_else(|| ClidleError::BuyingItemNotKnown(item.clone()))?;

    let count = 1; // TODO buy multiple
    if item_type.cost * count < app.code_lines.floor() as u64 {
        app.code_lines -= (item_type.cost * count) as f64;
        app.owned_items
            .entry(item_id)
            .and_modify(|e| *e += count)
            .or_insert(count);
    }
    Ok(())
}

/// Handles inputs if it's successful you get a GameState if not you may end up with
/// an IO error.
fn handle_input(app: &mut App) -> io::Result<GameState> {
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

/// contient la boucle de jeu
///
/// Ici Terminal est un type générique qui a besoin d'une interface (un trait)
/// pour fonctionner. Ici en particulier le trait backend permet d'interagir avec le terminal
/// (dessin, gestion du curseur, etc)
///
///
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>> {
    // pour vérifier si il faut mettre à jour l'état du jeu
    let mut last_tick = Instant::now();

    loop {
        // mise à jour de l'état du jeu
        if last_tick.elapsed() >= Duration::from_secs(1) {
            app.update();
            last_tick = Instant::now();
        }

        // ici l'argument de la fonction est une closure, une autre fonction anonyme
        terminal.draw(|f| ui(f, &mut app))?;

        // la fonction poll permet de vérifier si un evenement s'est rendu disponible
        // avant la fin du temps inparti
        if poll(Duration::from_millis(100))? {
            let state = handle_input(&mut app)?;
            match state {
                GameState::BuyItem(item_string) => {
                    // On veut pouvoir afficher l'erreur et sans paniquer
                    // en effet, on ne sait si ce que le joueur a entré est valide ou non
                    app.error = buy_item(&mut app, item_string)
                }
                GameState::Noop => {}
                GameState::Quit => return Ok(()),
            }
        }
    }
}

// Permet de gérer tout l'affichage
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
                Span::raw(" buy"),
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
                Span::raw(" to sell"),
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
            let item_type = app.items_index.get(*item_id).unwrap();

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
        .iter()
        .map(|item| {
            // TODO: on peut afficher le prix de chaque item
            let content = vec![Spans::from(Span::raw(format!(
                "Buy {}(as {}) producing {:.2} code lines per second",
                item.long_name, item.name, item.cps
            )))];
            ListItem::new(content)
        })
        .collect();

    if let Err(error) = app.error.as_ref() {
        messages.push(ListItem::new(Spans::from(Span::raw(format!(
            "Error: {error}",
        )))))
    }

    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[3]);
}
