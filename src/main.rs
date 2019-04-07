mod parser;

use crossterm::{ClearType, Crossterm};
use parser::{CombatEvent, Parser};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;

fn main() {
    let ct = Crossterm::new();
    let term = ct.terminal();
    let input = ct.input();
    let mut cursor = ct.cursor();
    term.clear(ClearType::All).unwrap();

    let (data_tx, data_rx) = mpsc::channel::<Vec<CombatEvent>>();
    let (cancel_tx, cancel_rx) = mpsc::channel();
    let mut parser = Parser::new("data.txt");
    let parser_thread = thread::spawn(move || parser.read_loop(&data_tx, &cancel_rx));

    loop {
        if let Ok(data) = data_rx.try_recv() {
            for event in data {
                println!("{}", event);
            }
        }
        print!("> ");
        io::stdout().flush().unwrap();
        let key = input.read_char().unwrap();
        term.clear(ClearType::CurrentLine).unwrap();
        cursor.move_left(cursor.pos().0);
        if key == 'q' {
            break;
        }
    }

    cancel_tx.send(()).unwrap();
    parser_thread.join().unwrap();
}
