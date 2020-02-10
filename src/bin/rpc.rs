extern crate rpassword;


use rpassword::read_password;
use std::env;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;

use rpm::constants::Event as Event;

fn main() {
    std::process::exit(match run_app() {
        Ok(output) => {
            print!("{}", output);
            0
        }
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

fn run_app() -> Result<String, String>{
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let command: &String = &args[1];
        let response = match command.as_ref() {
            "new" => new(&args),
            "get" => get(&args),
            "delete" => delete(&args),
            "validate" => validate(&args),
            "changeMP" => change_mp(),
            "change" => change(&args),
            "init" => init(),
            "list" => list(),
            "tui" => tui(),
            "--help" => help(),
            _ => Err("Unknown command...try --help".to_string())
        };
        return response;
    }
    Err(String::from("No command specified...try --help"))
}

// Starts fully fledged tui interface with a daemon
fn tui() -> Result<String,String>{
    rpmtui::start();
    Ok("".to_string())
}

fn first_zero(data: &[u8; 200]) -> usize {
    for i in 1..200 { // skipping first index, might be validation zero
        if data[i] == 0 {
            return i
        }
    }
    200
}

fn send_to_daemon(message: String, event: Event) -> Result<String, String>{
    let result: Result<String, String>;
    match TcpStream::connect("localhost:3333") {
        Ok(mut stream) => {
            let mut msg_vec = message.into_bytes();
            msg_vec.insert(0, event.to_u8());
            let msg: &[u8] = &msg_vec; // c: &[u8]
            stream.write(msg).unwrap();

            let mut data = [0 as u8; 200];
            match stream.read(&mut data) {
                Ok(_) => {
                    let zero_index = first_zero(&data);
                    let validated = &data[0];
                    let text = from_utf8(&data[1..zero_index]).unwrap().to_string();
                    if *validated == 0 {
                        match event {
                            Event::Init => result = Ok(text.clone()),
                            _           => result = Err("Validation failed! use validate command".to_string())
                        }
                    } else {
                        result = Ok(text.clone());
                    }
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                    result = Err("Read failed".to_string());
                }
            }
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            result = Err("Connect failed".to_string());
        }
    }
    result
}

// {{{ utils

fn concat_vec(args: &Vec<String>) -> String {
    let mut result = String::new();
    for (i,arg) in args.iter().enumerate() {
        if i > 1 {
            result.push_str(arg);
            if i < args.len() - 1 {
                result.push_str("#@#");
            }
        }
    }
    result
}

// }}} utils

fn new(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 4 {
        return Err("Need at least 4 arguments !!! new, key, password (or new, key, login, password)".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::New);
    response
}

fn get(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 3 {
        return Err("Need at least 3 arguments !!! get, key (returns password, login returned by -l flag)".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::Get);
    response
}

fn delete(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 3 {
        return Err("Need 2 arguments, delete and key of record which we try to delete".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::Delete);
    response
}
fn validate(args: &Vec<String>) -> Result<String, String>{
    let password;
    if args.len() < 3 {
        println!("Type master password: ");
        password = read_password().unwrap();
    } else {
        password = args[2].clone();
    }
    let response = send_to_daemon(password, Event::Validate);
    response
}
fn init() -> Result<String, String>{
    println!("Type INITIAL master password (removes all records from storage): ");
    let init_password = read_password().unwrap();
    let response = send_to_daemon(init_password, Event::Init);
    response
}

fn change_mp() -> Result<String, String>{
    println!("Type NEW master password: ");
    let new_password = read_password().unwrap();
    let response = send_to_daemon(new_password, Event::ChangeMP);
    response
}
fn change(args: &Vec<String>) -> Result<String, String>{
    if args.len() < 4 {
        return Err("Need at least 4 arguments !!! change, key, password (or new, key, login, password)".to_string());
    }
    let response = send_to_daemon(concat_vec(&args), Event::Change);
    response
}
fn list() -> Result<String, String>{
    let response = send_to_daemon("".to_string(),  Event::List);
    response
}

fn help() -> Result<String, String>{
    println!("Rusty password daemon - password manager written in rust running in background

usage: rpc [command] [args...] [--help]
commands: [new, get, validate, change, list]
    new          - checks validation, prompts for key then password and saves hashed
    get [-l] key - checks validation, returns unencrypted password (or login with -l)
    validate     - prompts for master password and unlocks for PASS_DELAY seconds
    changeMP     - prompts for old master pass and then new twice
    list         - prints all keys for rofi integration
    ");
    Ok(String::new())
}

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
        pub tasks: ListState<String>,
    }

    impl<'a> App<'a> {
        pub fn new(title: &'a str) -> App<'a> {
            let response = super::list();
            let values_list = match response {
                Err(err) => panic!("Not validated"),
                Ok(values) => values
            };
            let values_list: Vec<&str> = values_list.split("\n").collect();
            let values_list: Vec<String> = values_list.iter().map(|s| s.to_string()).collect();
                // TODO HERE INCLUDE MY DATA
            App {
                title,
                should_quit: false,
                tabs: TabsState::new(vec!["View", "Remove", "Add"]),
                tasks: ListState::new(values_list),
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
