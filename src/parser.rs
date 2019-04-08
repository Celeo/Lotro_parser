use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clone, Debug, PartialEq)]
pub enum CombatEventType {
    DAMAGE,
    HEAL,
}

#[derive(Clone)]
pub struct CombatEvent {
    pub source: String,
    pub target: String,
    pub method: String,
    pub value: u64,
    pub event_type: CombatEventType,
    pub timestamp: SystemTime,
}

impl fmt::Display for CombatEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "'{} | {} | {} | {} | {:?} | {}'",
            if self.source != "" {
                &self.source
            } else {
                "* unknown *"
            },
            self.target,
            self.method,
            self.value,
            self.event_type,
            self.timestamp.elapsed().unwrap().as_millis()
        )
    }
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

    pub fn read_loop(&mut self, data_tx: &Sender<Vec<CombatEvent>>, cancel_rx: &Receiver<()>) {
        let mut lines = vec![];
        loop {
            match cancel_rx.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    break;
                }
                Err(TryRecvError::Empty) => {}
            };
            loop {
                let mut line = String::new();
                self.reader.read_line(&mut line).unwrap();
                if line.is_empty() {
                    break;
                }
                let parsed = Parser::parse_line(&line);
                if parsed.is_some() {
                    lines.push(parsed.unwrap());
                }
            }
            if lines.is_empty() {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            data_tx.send(lines.clone()).unwrap();
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
