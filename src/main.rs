use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::thread;
use std::time::{Duration, SystemTime};

struct Stats {
    total: HashMap<String, u64>,
    dps: HashMap<String, u64>,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            total: HashMap::new(),
            dps: HashMap::new(),
        }
    }
}

struct LineItem {
    raw: String,
    timestamp: SystemTime,
}

impl LineItem {
    fn new(raw: String) -> LineItem {
        LineItem {
            raw,
            timestamp: SystemTime::now(),
        }
    }
}

struct Parser<'a> {
    file_name: &'a str,
    reader: BufReader<File>,
    lines: Vec<LineItem>,
}

impl<'a> Parser<'a> {
    fn new(file_name: &str) -> Parser {
        let f = File::open(file_name).unwrap();
        Parser {
            file_name,
            reader: BufReader::new(f),
            lines: vec![],
        }
    }

    fn read_loop(&mut self) {
        let mut line = String::new();
        loop {
            self.reader.read_line(&mut line).unwrap();
            if line.len() == 0 {
                // thread::sleep(Duration::from_millis(100));
                // continue;
                break;
            }
            self.lines.push(LineItem::new(line.clone()));
            self.update_stats();
            line.clear();
        }
    }

    fn update_stats(&mut self) -> Stats {
        lazy_static! {
            static ref RE_DAMAGE: Regex = Regex::new(r"^([a-zA-Z0-9-_ ]+) scored a [a-zA-Z0-9 ]*hit with ([a-zA-Z0-9 ]+) on ([a-zA-Z0-9-_ ]+) for (\d+) [\w]+ damage to [a-zA-Z0-9- ]+.$").unwrap();
            static ref RE_HEAL: Regex = Regex::new(
                r"^([a-zA-Z0-9-_ ]+) applied a heal to ([a-zA-Z0-9-_ ]+) restoring (\d+) points to Morale.$",
            )
            .unwrap();
            static ref RE_KILL: Regex = Regex::new(r"^(Your|[a-zA-Z0-9']+) mighty blow defeated ([a-zA-Z0-9-_ ]+).$").unwrap();
        }

        for item in &self.lines {
            let test = &item.raw.trim();
            if RE_DAMAGE.is_match(test) {
                let info = RE_DAMAGE.captures(test).unwrap();
                let agressor = &info[1];
                let attack = &info[2];
                let target = &info[3];
                let value = &info[4];
                println!("{} hit {} with {} for {}", agressor, target, attack, value);
            } else if RE_HEAL.is_match(test) {
                let info = RE_HEAL.captures(test).unwrap();
                let skill_name = &info[1];
                let target = &info[2];
                let value = &info[3];
                println!("{} was healed for {} by {}.", target, value, skill_name);
            }
        }

        Stats::new()
    }
}

fn main() {
    let mut parser = Parser::new("data.txt");
    parser.read_loop();
}
