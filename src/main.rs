// mod parser;

use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use ws::{
    listen, CloseCode, Error as SocketError, Handler, Handshake, Message, Result as SocketResult,
    Sender,
};

// use parser::Parser;
// Parser::new("data.txt").read_loop();

struct Client {
    out: Sender,
    connected: bool,
    comm_thread: Option<JoinHandle<()>>,
}

impl Handler for Client {
    fn on_open(&mut self, _: Handshake) -> SocketResult<()> {
        self.connected = true;
        self.comm_thread = Some(thread::spawn(|| {
            while self.connected {
                println!("Thread started");
                thread::sleep(Duration::from_millis(1000));
            }
        }));
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> SocketResult<()> {
        println!("Got message from client: {}", msg);
        let new_msg = format!("Server response: '{}'", msg);
        self.out.send(new_msg)
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        self.connected = false;
        match code {
            CloseCode::Normal => println!("The client is done with the connection."),
            CloseCode::Away => println!("The client is leaving the site."),
            _ => println!("The client encountered an error: {}", reason),
        };
    }

    fn on_error(&mut self, err: SocketError) {
        println!("The server encountered an error: {:?}", err);
    }
}

fn main() {
    env_logger::init();
    listen("127.0.0.1:5000", |out| Client {
        out: out,
        connected: false,
        comm_thread: None,
    })
    .unwrap()
}
