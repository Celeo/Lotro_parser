use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clone, Serialize)]
pub enum CombatEventType {
    DAMAGE,
    HEAL,
}

#[derive(Clone, Serialize)]
pub struct CombatEvent {
    source: String,
    target: String,
    method: String,
    value: u64,
    event_type: CombatEventType,
    timestamp: SystemTime,
}

pub struct Parser {
    reader: BufReader<File>,
}

impl Parser {
    pub fn new(file_name: &str) -> Parser {
        let f = File::open(file_name).unwrap();
        let reader = BufReader::new(f);
        Parser { reader }
    }

    pub fn read_loop(&mut self, rx: &Sender<Vec<CombatEvent>>) {
        let mut lines = vec![];
        loop {
            loop {
                let mut line = String::new();
                self.reader.read_line(&mut line).unwrap();
                if line.len() == 0 {
                    break;
                }
                let parsed = Parser::parse_line(&line);
                if parsed.is_some() {
                    lines.push(parsed.unwrap());
                }
            }
            if lines.len() < 10 {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            println!("Sending {} lines to the channel", lines.len());
            rx.send(lines.clone()).unwrap();
            lines.clear();
        }
    }

    fn parse_line(line: &str) -> Option<CombatEvent> {
        lazy_static! {
            static ref RE_DAMAGE: Regex = Regex::new(r"^([a-zA-Z0-9-_ ]+) scored a [a-zA-Z0-9 ]*hit with ([a-zA-Z0-9 ]+) on ([a-zA-Z0-9-_ ]+) for (\d+) [\w]+ damage to [a-zA-Z0-9- ]+.$").unwrap();
            static ref RE_HEAL: Regex = Regex::new(
                r"^([a-zA-Z0-9-_ ]+) applied a heal to ([a-zA-Z0-9-_ ]+) restoring (\d+) points to Morale.$",
            )
            .unwrap();
            // static ref RE_KILL: Regex = Regex::new(r"^(Your|[a-zA-Z0-9']+) mighty blow defeated ([a-zA-Z0-9-_ ]+).$").unwrap();
        }

        let test = line.trim();
        if RE_DAMAGE.is_match(test) {
            let info = RE_DAMAGE.captures(test).unwrap();
            let source = String::from(&info[1]);
            let method = String::from(&info[2]);
            let target = String::from(&info[3]);
            let value = info[4].parse::<u64>().unwrap();
            Some(CombatEvent {
                source,
                target,
                method,
                value,
                event_type: CombatEventType::DAMAGE,
                timestamp: SystemTime::now(),
            })
        } else if RE_HEAL.is_match(test) {
            let info = RE_HEAL.captures(test).unwrap();
            let method = String::from(&info[1]);
            let target = String::from(&info[2]);
            let value = info[3].parse::<u64>().unwrap();;
            Some(CombatEvent {
                source: String::from(""),
                target,
                method,
                value,
                event_type: CombatEventType::HEAL,
                timestamp: SystemTime::now(),
            })
        } else {
            None
        }
    }
}
