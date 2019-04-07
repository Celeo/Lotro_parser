mod parser;

use crossterm::{terminal, ClearType, InputEvent, KeyEvent, Screen};
use parser::{CombatEvent, Parser};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    let screen = Screen::new(true);
    let term = terminal();
    let input = crossterm::TerminalInput::from_output(&screen.stdout);
    let mut stdin = input.read_async();

    let (data_tx, data_rx) = mpsc::channel::<Vec<CombatEvent>>();
    let (cancel_tx, cancel_rx) = mpsc::channel();
    let mut parser = Parser::new("data.txt");
    let parser_thread = thread::spawn(move || parser.read_loop(&data_tx, &cancel_rx));
    let mut running = true;

    term.clear(ClearType::All).unwrap();

    while running {
        if let Ok(data) = data_rx.try_recv() {
            for event in data {
                print!("{}\n\r", event);
            }
        }
        if let Some(key_event) = stdin.next() {
            match key_event {
                InputEvent::Keyboard(key) => match key {
                    KeyEvent::Char('q') | KeyEvent::Ctrl('c') => running = false,
                    _ => {}
                },
                _ => {}
            };
        } else {
            thread::sleep(Duration::from_millis(500));
        }
    }

    cancel_tx.send(()).unwrap();
    parser_thread.join().unwrap();
}
