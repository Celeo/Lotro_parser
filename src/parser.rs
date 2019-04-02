use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::prelude::BufRead;
use std::io::BufReader;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, SystemTime};

#[derive(Clone)]
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
}

impl Parser {
    pub fn new(file_name: &str) -> Parser {
        let f = File::open(file_name).unwrap();
        let reader = BufReader::new(f);
        Parser { reader }
    }

    pub fn read_loop(&mut self, rx: &Sender<LineItem>) {
        lazy_static! {
            static ref RE_DAMAGE: Regex = Regex::new(r"^([a-zA-Z0-9-_ ]+) scored a [a-zA-Z0-9 ]*hit with ([a-zA-Z0-9 ]+) on [a-zA-Z0-9-_ ]+ for (\d+) [\w]+ damage to [a-zA-Z0-9- ]+.$").unwrap();
            static ref RE_HEAL: Regex = Regex::new(
                r"^([a-zA-Z0-9-_ ]+) applied a heal to ([a-zA-Z0-9-_ ]+) restoring (\d+) points to Morale.$",
            )
            .unwrap();
            static ref RE_KILL: Regex = Regex::new(r"^(Your|[a-zA-Z0-9']+) mighty blow defeated ([a-zA-Z0-9-_ ]+).$").unwrap();
        }

        let mut lines = vec![];
        loop {
            println!("File read");
            loop {
                let mut line = String::new();
                self.reader.read_line(&mut line).unwrap();
                if line.len() == 0 {
                    break;
                }
                lines.push(LineItem::new(line));
            }
            if lines.len() < 10 {
                thread::sleep(Duration::from_secs(1));
                continue;
            }
            for line in lines.clone() {
                rx.send(line).unwrap();
            }
            lines.clear();
        }
    }

    // fn update_stats(&mut self) {
    //     for item in &self.lines {
    //         let test = &item.raw.trim();
    //         if RE_DAMAGE.is_match(test) {
    //             // let info = RE_DAMAGE.captures(test).unwrap();
    //             // let agressor = &info[1];
    //             // let target = &info[2];
    //             // let value = &info[3];
    //             // println!("{}, {}, {}", agressor, target, value);
    //         } else if RE_HEAL.is_match(test) {
    //             // let info = RE_HEAL.captures(test).unwrap();
    //             // let skill_name = &info[1];
    //             // let target = &info[2];
    //             // let value = &info[3];
    //             // println!("{}, {}, {}.", target, value, skill_name);
    //         }
    //     }
    // }
}
