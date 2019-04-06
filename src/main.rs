mod parser;

use parser::{CombatEvent, Parser};
use serde_json;
use std::sync::mpsc;
use std::thread;
use ws::{
    listen, CloseCode, Error as SocketError, Handler, Message, Result as SocketResult, Sender,
};

struct Client<'a> {
    out: Sender,
    rx: &'a mpsc::Receiver<Vec<CombatEvent>>,
}

impl<'a> Handler for Client<'a> {
    fn on_message(&mut self, msg: Message) -> SocketResult<()> {
        println!("Got message from client: {}", msg);
        match self.rx.try_recv() {
            Ok(data) => {
                let actual = data;
                let json = serde_json::to_string(&actual).unwrap();
                self.out.send(json)
            }
            Err(e) => {
                match e {
                    mpsc::TryRecvError::Empty => {
                        println!("Can not send client data - no remaining data")
                    }
                    mpsc::TryRecvError::Disconnected => eprintln!("Client isn't connected"),
                }
                Ok(())
            }
        }
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        match code {
            CloseCode::Normal => println!("Client is done with the connection."),
            CloseCode::Away => println!("Client is leaving the site."),
            _ => println!("Client encountered an error: {}", reason),
        };
    }

    fn on_error(&mut self, err: SocketError) {
        println!("The server encountered an error: {:?}", err);
    }
}

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel::<Vec<CombatEvent>>();

    let socket_thread =
        thread::spawn(move || listen("127.0.0.1:5000", |out| Client { out, rx: &rx }).unwrap());
    println!("Socket thread started");

    let mut parser = Parser::new("data.txt");
    let parser_thread = thread::spawn(move || parser.read_loop(&tx));
    println!("Parser thread started");

    socket_thread.join().unwrap();
    parser_thread.join().unwrap();
}
