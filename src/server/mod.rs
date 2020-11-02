use crate::common::*;

use std::net::TcpListener;

pub fn run_server() {
    println!("Server started.");
    let listener = TcpListener::bind(SERVER).unwrap();

    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                handle_connection(Connection::new(stream));
                break;
            },
            Err(e) => panic!("{:?}", e),
        }
    }
    println!("Server stopped.");
}

fn handle_connection(mut connection: Connection<MessageToClient, MessageToServer>) {
    println!("Handling connection...");

    let mut game_world = generate_game_world(60, 40);

    if let MessageToServer { message_type: MessageToServerType::Hello } = connection.receive_message_blocking() {
        // Everything went as expected
    } else {
        panic!("unknown first message to server");
    }

    connection.send_message(MessageToClient { message_type: MessageToClientType::Nothing });
    connection.send_message(MessageToClient { message_type: MessageToClientType::InitializeWorld(game_world.clone()) });

    loop {
        let message = connection.receive_message_blocking();
        match message.message_type {
            MessageToServerType::Goodbye => {
                break;
            }
            MessageToServerType::Action(action) => {
                let events = game_world.process_action(&action);
                for event in events {
                    connection.send_message(MessageToClient { message_type: MessageToClientType::Event(event) })
                }
            }
            MessageToServerType::NextTurn => {
                let events = game_world.next_turn();
                for event in events {
                    connection.send_message(MessageToClient { message_type: MessageToClientType::Event(event) })
                }
            }
            MessageToServerType::Hello => panic!("Unexpected message: {:?}", message),
        }
    }

    println!("Finishing with connection...");
}
