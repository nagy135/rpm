
// {{{ constants
pub mod constants {
    pub static PASSWORD_HASH_HOLDER: &'static str = "rpm/pass_hash";
    pub const PASS_DELAY: u64 = 600;
    pub static STORAGE: &'static str = "rpm/storage.db";
    pub static IV: &[u8] = b"\x00\x01\x02\x03\x04\x05\x06\x07\x00\x01\x02\x03\x04\x05\x06\x07";

    pub enum Reason {
        PasswordEmpty,
        PasswordInvalid
    }
    impl Reason {
        pub fn to_string(&self) -> String {
            match self {
                PasswordInvalid => String::from("Password is invalid"),
                PasswordEmpty => String::from("Password is empty")
            }
        }
    }

    // {{{ Enum Event
    #[derive(Debug, Copy, Clone)]
    pub enum Event {
        New = 1,
        Get = 2,
        Validate = 3,
        List = 4,
        ChangeMP = 5,
        Init = 6,
        Delete = 7,
        Change = 8,
    }
    impl Event {
        pub fn to_u8(&self) -> u8 {
            *self as u8
        }
    }
    impl From<&u8> for Event {
        fn from(i: &u8) -> Self {
            match i {
                1 => Event::New,
                2 => Event::Get,
                3 => Event::Validate,
                4 => Event::List,
                5 => Event::ChangeMP,
                6 => Event::Init,
                7 => Event::Delete,
                8 => Event::Change,
                _ => Event::Validate
            }
        }
    }

    // }}}

}
// }}}

// {{{ tui
pub mod rpmtui {
    use std::io;
    use std::time::Duration;

    use termion::event::Key;
    use termion::input::MouseTerminal;
    use termion::raw::IntoRawMode;
    use termion::screen::AlternateScreen;
    use tui::backend::TermionBackend;
    use tui::Terminal;

    use crate::event;
    use crate::app;
    use crate::ui;

    // use crate::util::event::{Config, Event, Events};


    pub fn start() {
        let events = event::Events::with_config(event::Config {
            tick_rate: Duration::from_millis(200),
            ..event::Config::default()
        });

        let stdout = io::stdout().into_raw_mode().unwrap();
        let stdout = MouseTerminal::from(stdout);
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.hide_cursor().unwrap();

        let mut app = app::App::new("Rusty Password Manager");
        loop {
            ui::draw(&mut terminal, &app).unwrap();
            match events.next().unwrap() {
                event::Event::Input(key) => match key {
                    Key::Char(c) => {
                        app.on_key(c);
                    }
                    Key::Up => {
                        app.on_up();
                    }
                    Key::Down => {
                        app.on_down();
                    }
                    Key::Left => {
                        app.on_left();
                    }
                    Key::Right => {
                        app.on_right();
                    }
                    _ => {}
                },
                event::Event::Tick => {
                    app.on_tick();
                }
            }
            if app.should_quit {
                break;
            }
        }

        // Ok(())
    }
}
// }}}

// {{{ ui
mod ui {
    use std::io;

    use tui::backend::Backend;
    use tui::layout::{Constraint, Direction, Layout, Rect};
    use tui::style::{Color, Modifier, Style};
    use tui::widgets::canvas::{Canvas, Line, Map, MapResolution, Rectangle};
    use tui::widgets::{
        Axis, BarChart, Block, Borders, Chart, Dataset, Gauge, List, Marker, Paragraph, Row,
        SelectableList, Sparkline, Table, Tabs, Text, Widget,
    };
    use tui::{Frame, Terminal};

    use crate::app::App as App;

    pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<(), io::Error> {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());
            Tabs::default()
                .block(Block::default().borders(Borders::ALL).title(app.title))
                .titles(&app.tabs.titles)
                .style(Style::default().fg(Color::Green))
                .highlight_style(Style::default().fg(Color::Yellow))
                .select(app.tabs.index)
                .render(&mut f, chunks[0]);
            match app.tabs.index {
                0 => draw_first_tab(&mut f, &app, chunks[1]),
                1 => draw_second_tab(&mut f, &app, chunks[1]),
                _ => {}
            };
        })
    }

    fn draw_first_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints(
                [
                Constraint::Percentage(100)
                ]
                .as_ref(),
            )
            .split(area);
        draw_list(f, &app, chunks[0]);
    }

    fn draw_list<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where
        B: Backend,
    {
        SelectableList::default()
            .block(Block::default().borders(Borders::ALL).title("Key list"))
            .items(&app.tasks.items)
            .select(Some(app.tasks.selected))
            .highlight_style(Style::default().fg(Color::Yellow).modifier(Modifier::BOLD))
            .highlight_symbol(">")
            .render(f, area);
    }

    fn draw_second_tab<B>(f: &mut Frame<B>, app: &App, area: Rect)
    where
        B: Backend,
    {
    }
}
// }}}

// {{{ event
mod event {
    use std::io;
    use std::sync::mpsc;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::thread;
    use std::time::Duration;

    use termion::event::Key;
    use termion::input::TermRead;

    pub enum Event<I> {
        Input(I),
        Tick,
    }

    /// A small event handler that wrap termion input and tick events. Each event
    /// type is handled in its own thread and returned to a common `Receiver`
    pub struct Events {
        rx: mpsc::Receiver<Event<Key>>,
        input_handle: thread::JoinHandle<()>,
        ignore_exit_key: Arc<AtomicBool>,
        tick_handle: thread::JoinHandle<()>,
    }

#[derive(Debug, Clone, Copy)]
    pub struct Config {
        pub exit_key: Key,
        pub tick_rate: Duration,
    }

    impl Default for Config {
        fn default() -> Config {
            Config {
                exit_key: Key::Char('q'),
                tick_rate: Duration::from_millis(250),
            }
        }
    }

    impl Events {
        pub fn new() -> Events {
            Events::with_config(Config::default())
        }

        pub fn with_config(config: Config) -> Events {
            let (tx, rx) = mpsc::channel();
            let ignore_exit_key = Arc::new(AtomicBool::new(false));
            let input_handle = {
                let tx = tx.clone();
                let ignore_exit_key = ignore_exit_key.clone();
                thread::spawn(move || {
                    let stdin = io::stdin();
                    for evt in stdin.keys() {
                        match evt {
                            Ok(key) => {
                                if let Err(_) = tx.send(Event::Input(key)) {
                                    return;
                                }
                                if !ignore_exit_key.load(Ordering::Relaxed) && key == config.exit_key {
                                    return;
                                }
                            }
                            Err(_) => {}
                        }
                    }
                })
            };
            let tick_handle = {
                let tx = tx.clone();
                thread::spawn(move || {
                    let tx = tx.clone();
                    loop {
                        tx.send(Event::Tick).unwrap();
                        thread::sleep(config.tick_rate);
                    }
                })
            };
            Events {
                rx,
                ignore_exit_key,
                input_handle,
                tick_handle,
            }
        }

        pub fn next(&self) -> Result<Event<Key>, mpsc::RecvError> {
            self.rx.recv()
        }

        pub fn disable_exit_key(&mut self) {
            self.ignore_exit_key.store(true, Ordering::Relaxed);
        }

        pub fn enable_exit_key(&mut self) {
            self.ignore_exit_key.store(false, Ordering::Relaxed);
        }
    }
}
// }}}

// {{{ app
mod app {
    use crate::util::TabsState;

    const TASKS: [&'static str; 24] = [
        "Item1", "Item2", "Item3", "Item4", "Item5", "Item6", "Item7", "Item8", "Item9", "Item10",
        "Item11", "Item12", "Item13", "Item14", "Item15", "Item16", "Item17", "Item18", "Item19",
        "Item20", "Item21", "Item22", "Item23", "Item24",
    ];

    pub struct ListState<I> {
        pub items: Vec<I>,
        pub selected: usize,
    }

    impl<I> ListState<I> {
        fn new(items: Vec<I>) -> ListState<I> {
            ListState { items, selected: 0 }
        }
        fn select_previous(&mut self) {
            if self.selected > 0 {
                self.selected -= 1;
            }
        }
        fn select_next(&mut self) {
            if self.selected < self.items.len() - 1 {
                self.selected += 1
            }
        }
    }

    pub struct App<'a> {
        pub title: &'a str,
        pub should_quit: bool,
        pub tabs: TabsState<'a>,
        pub tasks: ListState<&'a str>,
    }

    impl<'a> App<'a> {
        pub fn new(title: &'a str) -> App<'a> {
            App {
                title,
                should_quit: false,
                tabs: TabsState::new(vec!["View", "Remove", "Add"]),
                tasks: ListState::new(TASKS.to_vec()),
            }
        }

        pub fn on_up(&mut self) {
            self.tasks.select_previous();
        }

        pub fn on_down(&mut self) {
            self.tasks.select_next();
        }

        pub fn on_right(&mut self) {
            self.tabs.next();
        }

        pub fn on_left(&mut self) {
            self.tabs.previous();
        }

        pub fn on_key(&mut self, c: char) {
            match c {
                'q' => {
                    self.should_quit = true;
                },
                'j' => {
                    self.on_down();
                },
                'k' => {
                    self.on_up();
                },
                'h' => {
                    self.on_left();
                },
                'l' => {
                    self.on_right();
                }
                _ => {}
            }
        }

        pub fn on_tick(&mut self) {
            // Update progress
            // nothing yet
        }
    }
}
// }}}

// {{{ util
mod util {
#[cfg(feature = "termion")]
    pub mod event;

    pub struct TabsState<'a> {
        pub titles: Vec<&'a str>,
        pub index: usize,
    }

    impl<'a> TabsState<'a> {
        pub fn new(titles: Vec<&'a str>) -> TabsState {
            TabsState { titles, index: 0 }
        }
        pub fn next(&mut self) {
            self.index = (self.index + 1) % self.titles.len();
        }

        pub fn previous(&mut self) {
            if self.index > 0 {
                self.index -= 1;
            } else {
                self.index = self.titles.len() - 1;
            }
        }
    }
}
// }}}
