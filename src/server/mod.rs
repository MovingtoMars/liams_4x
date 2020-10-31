use crate::common::*;

pub fn run_server() {
    println!("Starting server...");

    let mut game_world = generate_game_world(20, 10);

    let mut connection = Connection::new(SERVER, CLIENT);

    if let MessageToServer { message_type: MessageToServerType::Hello } = connection.receive_message_blocking() {
        // Everything went as expected
    } else {
        panic!("unknown first message to server");
    }

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
                    game_world.apply_event(&event);
                    connection.send_message(MessageToClient { message_type: MessageToClientType::Event(event) })
                }
            }
            _ => panic!("Unexpected message: {:?}", message),
        }
    }

    println!("Stopping server...");
}
