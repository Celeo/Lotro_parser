use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct LineItem {
    raw: String,
    timestamp: SystemTime,
}

impl LineItem {
    pub fn new(raw: String) -> LineItem {
        LineItem {
            raw,
            timestamp: SystemTime::now(),
        }
    }
}

pub struct Parser {
    reader: BufReader<File>,
    lines: Vec<LineItem>,
}

impl Parser {
    pub fn new(file_name: &str) -> Parser {
        let f = File::open(file_name).unwrap();
        Parser {
            reader: BufReader::new(f),
            lines: vec![],
        }
    }

    pub fn read_loop(&mut self) {
        let mut line = String::new();
        loop {
            self.reader.read_line(&mut line).unwrap();
            if line.len() == 0 {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            self.lines.push(LineItem::new(line.clone()));
            self.update_stats();
            line.clear();
        }
    }

    fn update_stats(&mut self) {
        lazy_static! {
            static ref RE_DAMAGE: Regex = Regex::new(r"^([a-zA-Z0-9-_ ]+) scored a [a-zA-Z0-9 ]*hit with ([a-zA-Z0-9 ]+) on [a-zA-Z0-9-_ ]+ for (\d+) [\w]+ damage to [a-zA-Z0-9- ]+.$").unwrap();
            static ref RE_HEAL: Regex = Regex::new(
                r"^([a-zA-Z0-9-_ ]+) applied a heal to ([a-zA-Z0-9-_ ]+) restoring (\d+) points to Morale.$",
            )
            .unwrap();
            static ref RE_KILL: Regex = Regex::new(r"^(Your|[a-zA-Z0-9']+) mighty blow defeated ([a-zA-Z0-9-_ ]+).$").unwrap();
        }

        // have to somewhere delete already-processed lines
        for item in &self.lines {
            let test = &item.raw.trim();
            if RE_DAMAGE.is_match(test) {
                let info = RE_DAMAGE.captures(test).unwrap();
                let agressor = &info[1];
                let target = &info[2];
                let value = &info[3];
                println!("{}, {}, {}", agressor, target, value);
            } else if RE_HEAL.is_match(test) {
                let info = RE_HEAL.captures(test).unwrap();
                let skill_name = &info[1];
                let target = &info[2];
                let value = &info[3];
                println!("{}, {}, {}.", target, value, skill_name);
            }
        }

        // do something with the data
    }
}
