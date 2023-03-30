use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use starknet::core::types::FieldElement;
use std::{
    io,
    time::{Duration, Instant},
};

use color_eyre::Result;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use tui_textarea::{Input, TextArea};

use crate::{
    dump::{DumpState, StorageSlot},
    Config,
};

struct StatefulStorageList {
    state: ListState,
    total_items: usize,
    storages: Vec<(FieldElement, FieldElement)>,
}

impl StatefulStorageList {
    fn with_items(items: Vec<StorageSlot>) -> StatefulStorageList {
        StatefulStorageList {
            total_items: 0,
            state: ListState::default(),
            storages: items.iter().map(|i| (i.key, i.value.value)).collect(),
        }
    }

    fn next(&mut self) {
        if self.total_items == 0 {
            self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.total_items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.total_items == 0 {
            self.state.select(None);
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.total_items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    #[allow(unused)]
    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App {
    to: u64,
    from: u64,
    contract: FieldElement,
    items: StatefulStorageList,
}

impl App {
    fn new(contract: FieldElement, from: u64, to: u64, storages: Vec<StorageSlot>) -> App {
        App {
            to,
            from,
            contract,
            items: StatefulStorageList::with_items(storages),
        }
    }
}

pub fn execute_ui(dump_state: &DumpState, config: Config) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let storages = dump_state
        .storage
        .iter()
        .map(|(key, value)| StorageSlot {
            key: *key,
            value: value.clone(),
        })
        .collect::<Vec<_>>();

    let tick_rate = Duration::from_millis(250);
    let app = App::new(
        config.contract,
        config.from_block,
        config.to_block,
        storages,
    );
    let res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err}")
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let last_tick = Instant::now();

    let mut textarea = TextArea::default();
    textarea.set_cursor_line_style(Style::default());
    textarea.set_style(Style::default().fg(Color::Yellow));
    textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));

    loop {
        terminal.draw(|f| ui(f, &mut app, &textarea))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Esc => return Ok(()),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    KeyCode::Enter => {}
                    _ => {
                        textarea.input(Input::from(key));
                    }
                }
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, textarea: &TextArea) {
    // Create two chunks with equal horizontal screen space
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());

    render_header(f, area[0], &app.contract, app.from, app.to);

    let prefix = &textarea.lines()[0];

    // Render the widget
    render_list(f, app, area[1], prefix);
    f.render_widget(textarea.widget(), area[2]);
}

fn render_header<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    contract_address: &FieldElement,
    from_block: u64,
    to_block: u64,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(area);

    let address = Paragraph::new(format!("{contract_address:#x}"))
        .block(Block::default().borders(Borders::ALL).title("Contract"));

    let from = Paragraph::new(from_block.to_string())
        .block(Block::default().borders(Borders::ALL).title("From block"));

    let to = Paragraph::new(to_block.to_string())
        .block(Block::default().borders(Borders::ALL).title("To block"));

    f.render_widget(address, chunks[0]);
    f.render_widget(from, chunks[1]);
    f.render_widget(to, chunks[2]);
}

fn render_list<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect, prefix: &str) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(area);

    let (keys, values): (Vec<ListItem>, Vec<ListItem>) = if prefix.is_empty() {
        app.items.total_items = app.items.storages.len();
        app.items
            .storages
            .iter()
            .map(|(key, value)| {
                (
                    ListItem::new(format!("{key:#x}")).style(Style::default()),
                    ListItem::new(format!("{value:#x}")).style(Style::default()),
                )
            })
            .unzip()
    } else {
        let items: (Vec<ListItem>, Vec<ListItem>) = app
            .items
            .storages
            .iter()
            .filter(|(key, _)| format!("{key:#x}").starts_with(prefix))
            .map(|(key, value)| {
                (
                    ListItem::new(format!("{key:#x}")).style(Style::default()),
                    ListItem::new(format!("{value:#x}")).style(Style::default()),
                )
            })
            .unzip();

        app.items.total_items = items.0.len();

        items
    };

    let highlight_style = Style::default()
        .fg(Color::Black)
        .bg(Color::White)
        .add_modifier(Modifier::BOLD);

    let key_items = List::new(keys)
        .block(Block::default().borders(Borders::ALL).title("Index"))
        .highlight_style(highlight_style);

    let value_items = List::new(values)
        .block(Block::default().borders(Borders::ALL).title("Value"))
        .highlight_style(highlight_style);

    f.render_stateful_widget(key_items, chunks[0], &mut app.items.state);
    f.render_stateful_widget(value_items, chunks[1], &mut app.items.state);
}
