
//use std::thread;
extern crate futures;
//extern crate tokio_core;
extern crate tokio;

use futures::future::lazy;


struct ChannelMessage {
    sender_id: u8,
    subscribers: Vec<u8>,
    data: Vec::<u8>,
    text: String
}

impl ChannelMessage {
    fn clone(&self) -> ChannelMessage {
        return ChannelMessage { 
            sender_id: self.sender_id, 
            data: self.data.clone(),
            text: self.text.clone(),
            subscribers: self.subscribers.clone()
        };
    }

    fn print(&self) {
            println! ("----------------------------------------------------");
            match self.sender_id {
                ID_HUBCORE => {
                    println! (" Recieved from HUB: {}", self.text );
                }
                ID_ZIPGATEWAY => {
                    println! ("Recieved from ZIPGATEWAY: {}", self.text);
                }
                ID_STDIO => {
                    println! ("Recieved from STDIO: {}", self.text);
                }
                _ => {
                    println! (" This should never happen: unknown sender ");
                }
            }
            println! ("{:02X?}", self.data ); //0<#04X
            println!("Sending to: {:?}", self.subscribers);
            println! ("----------------------------------------------------");
    }
}

//  Subscriber channel

struct SubscriberChannel {
    tx: std::sync::mpsc::Sender<ChannelMessage>,
    id: u8
}

impl SubscriberChannel {
    fn send (&self, message: ChannelMessage) {
        if self.id != message.sender_id {
            self.tx.send(message).unwrap();
        }
    }
}



fn read_stdin () -> Result<(), std::io::Error> {

    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(input_length) => {
                let mut data: Vec::<u8>; // = Vec::<u8>::new();
                match input.as_ref() {
                    "exit\n" => {
                        break;
                    }
                    _ => {
                        // noop
                        data = (&input.as_bytes()).to_vec();
                    }
                }
                if input_length > 0 {
                    println!("Data: {:02X?}", data);
                }

            }
            Err(err) => {
                    println!("Error: {}", err);
                    return Err(err);
            }
        }
    }
    println!("Future exit: Exit reading consiole input");

    return Ok(());
}

fn main() {
    println!("Started");

    // Thread -based
    /*
    let stdio_thread = thread::spawn(move|| {
        read_stdin();
    });
    */
    //use tokio_core::reactor::Core;
    //use futures::future::lazy;
    //use futures::Future;

    let stdin_future = lazy(move || {
        match read_stdin() {
            _ => {
                // noop
            }
        }
        Ok(())
    });

    tokio::run( lazy(move || {
            tokio::spawn(stdin_future );
            Ok(())
        })
    );

    println!("Exit main");
}
