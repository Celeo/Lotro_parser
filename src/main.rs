mod parser;

use crossterm::{terminal, ClearType, InputEvent, KeyEvent, Screen};
use num_format::{Locale, ToFormattedString};
use parser::{CombatEvent, CombatEventType, Parser};
use prettytable::{cell, format, row, Table};
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/**
 * Okay, pulling back. I didn't know that the LotRO combat log only records damage and healing done by or to
 * the user's own character. It does not record damage or healing done by or to the rest of the fellowship or
 * raid, or even by enemies done to others - the only events that are recorded are combat events that originate
 * with or are targeted on the user's own character.
 *
 * Because of this, I cannot create a parser that expects to print out the DPS or HPS of anything other than
 * the user's character or enemies attacking the character.
 */

fn display(data: &Vec<CombatEvent>) {
    let mut map = HashMap::<&str, u64>::new();
    for item in data
        .iter()
        .filter(|e| e.event_type == CombatEventType::DAMAGE)
    {
        *map.entry(&item.source).or_insert(0) += item.value;
    }

    // TODO need to actually calculate DPS, not just total damage

    let mut v: Vec<(&str, u64)> = map.iter().map(|e| (*e.0, *e.1)).collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.set_titles(row!["Name", "Damage"]);
    for (name, damage) in v {
        table.add_row(row![name, damage.to_formatted_string(&Locale::en)]);
    }
    let output = table.to_string();
    print!("{}", output.replace("\n", "\n\r"));
}

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

    let mut all_data = vec![];
    while running {
        if let Ok(data) = data_rx.try_recv() {
            all_data.extend(data);
            term.clear(ClearType::All).unwrap();
            display(&all_data);
        }
        if let Some(key_event) = stdin.next() {
            match key_event {
                InputEvent::Keyboard(key) => match key {
                    KeyEvent::Char('q') | KeyEvent::Ctrl('c') => running = false,
                    _ => {}
                },
                _ => {}
            };
        }
        thread::sleep(Duration::from_millis(500));
    }

    cancel_tx.send(()).unwrap();
    parser_thread.join().unwrap();
}
