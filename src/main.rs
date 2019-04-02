mod parser;

use parser::{LineItem, Parser};
use std::sync::mpsc;
use std::thread;
use ws::{
    listen, CloseCode, Error as SocketError, Handler, Handshake, Result as SocketResult, Sender,
};

struct Client<'a> {
    out: Sender,
    connected: bool,
    rx: &'a mpsc::Receiver<LineItem>,
}

impl<'a> Handler for Client<'a> {
    fn on_open(&mut self, _: Handshake) -> SocketResult<()> {
        self.connected = true;
        println!("Client connected");
        Ok(())
    }

    // fn on_message(&mut self, msg: Message) -> SocketResult<()> {
    //     println!("Got message from client: {}", msg);
    //     let new_msg = format!("Server response: '{}'", msg);
    //     self.out.send(new_msg)
    // }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        self.connected = false;
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

/**
 * Next tasks -
 *
 * 1. Something in the handler has to be waiting on the receiver to get data. When it gets data, some code
 *      will need to run that will take that data and send it up to the client.
 * 2. The channel should get a single collection of multiple items instead of multiple single items.
 * 3. Somewhere, there will need to be a struct -> JSON string conversion so the client can make
 *      easy use of the data that it receives.
 */

fn main() {
    env_logger::init();
    let (tx, rx) = mpsc::channel::<LineItem>();

    let socket_thread = thread::spawn(move || {
        listen("127.0.0.1:5000", |out| Client {
            out,
            connected: false,
            rx: &rx,
        })
        .unwrap()
    });
    println!("Socket thread started");

    let mut parser = Parser::new("data.txt");
    let parser_thread = thread::spawn(move || parser.read_loop(&tx));
    println!("Parser thread started");

    socket_thread.join().unwrap();
    parser_thread.join().unwrap();
}
