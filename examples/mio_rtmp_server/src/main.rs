extern crate mio;
extern crate slab;
extern crate rml_rtmp;

mod connection;

use mio::*;
use mio::net::{TcpListener};
use slab::Slab;
use ::connection::{Connection, ConnectionState};

const SERVER: Token = Token(std::usize::MAX - 1);

fn main() {
    let addr = "127.0.0.1:1935".parse().unwrap();
    let server = TcpListener::bind(&addr).unwrap();
    let mut poll = Poll::new().unwrap();

    println!("Listening for connections");
    poll.register(&server, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);
    let mut connections = Slab::new();

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (socket, _) = server.accept().unwrap();
                    let mut connection = Connection::new(socket);
                    let token = connections.insert(connection);
                    connections[token].token = Some(Token(token));
                    connections[token].register(&mut poll).unwrap();
                },

                Token(value) => {
                    let mut should_close_connection = false;
                    {
                        let mut connection = match connections.get_mut(value) {
                            Some(connection) => connection,
                            None => continue,
                        };

                        if event.readiness().is_readable() {
                            let state = connection.readable(&mut poll).unwrap();
                            if state == ConnectionState::Closed {
                                should_close_connection = true;
                            }
                        }

                        if event.readiness().is_writable() {
                            connection.writable(&mut poll).unwrap();
                        }
                    }

                    if should_close_connection {
                        println!("Connection closed");
                        connections.remove(value);
                    }
                }
            }
        }
    }
}