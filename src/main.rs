
//use std::thread;
extern crate futures;
//extern crate tokio_core;
extern crate tokio;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use futures::future::lazy;

const ID_ALL: u8 = 0;
const ID_ZIPGATEWAY: u8 = 10;
const ID_HUBCORE: u8 = 12;
const ID_STDIN: u8 = 14;
const ID_STDOUT: u8 = 16;

const MESSAGE_TYPE_DATA: u8 = 2;
const MESSAGE_TYPE_COMPLETE_FUTURE: u8 = 4;


struct ChannelMessage {
    sender_id: u8,
    subscribers: Vec<u8>,
    message_type: u8,
    data: Vec::<u8>,
    text: String
}

impl ChannelMessage {
    fn clone(&self) -> ChannelMessage {
        return ChannelMessage { 
            sender_id: self.sender_id, 
            data: self.data.clone(),
            text: self.text.clone(),
            message_type: self.message_type,
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
                ID_STDIN => {
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



fn read_stdin (id: u8, 
               tx_broker: Sender<ChannelMessage>) -> Result<(), ()> {
    
    println!("Future start: read STDIN");
    loop {
        let mut input = String::new();
        let mut subscribers = Vec::<u8>::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(input_length) => {
                let mut data: Vec::<u8>; // = Vec::<u8>::new();
                match input.as_ref() {
                    "exit\n" => {
                        data = Vec::<u8>::new();
                        subscribers.push(ID_ALL);
                        let message = ChannelMessage { sender_id: id, message_type: MESSAGE_TYPE_COMPLETE_FUTURE, text: input, data: data, subscribers: subscribers};
                        tx_broker.send(message).unwrap();
                        break;
                    }
                    _ => {
                        // noop
                        data = (&input.as_bytes()).to_vec();
                    }
                }
                if input_length > 0 {
                    println!("print from read_stdin: {:02X?}", data);
                    subscribers.push(ID_ALL);
                    let message = ChannelMessage { sender_id: id, message_type: MESSAGE_TYPE_DATA, text: input, data: data, subscribers: subscribers};
                    tx_broker.send(message).unwrap();
                }

            }
            Err(err) => {
                    println!("Error: {}", err);
                    //return;
            }
        }
    }
    println!("Future exit: read STDIN");
    return Ok(());
}



fn broker(rx_broker: Receiver<ChannelMessage>,
          subscribers: Vec::<SubscriberChannel>) -> Result<(), ()> {
        println!("Future start: broker");
        loop {
            let message: ChannelMessage = rx_broker.recv().unwrap();
            for subscriber in &subscribers{
                if subscriber.id != message.sender_id 
                    && (message.subscribers.contains(&ID_ALL) 
                    || message.subscribers.contains(&subscriber.id)) {
                    subscriber.send(message.clone());
                }
            }
            match message.message_type {
                MESSAGE_TYPE_COMPLETE_FUTURE => {
                    break;
                }
                _ => {
                    //noop
                }
            }
        }
    println!("Future exit: broker");
    return Ok(());
}

fn write_stdout(rx_stdio: Receiver<ChannelMessage>)-> Result<(), ()>
{
    println!("Future start: write STDOUT");
    loop {
        let message = rx_stdio.recv().unwrap();

        match message.message_type {
            MESSAGE_TYPE_COMPLETE_FUTURE => {
                break;
            }
            _ => {
                //noop
                message.print();
            }
        }

    }
    println!("Future END: write STDOUT");
    return Ok(());
}

fn main() {
    println!("Started");

    let (tx_brocker, rx_brocker) = channel();
    let (tx_stdio, rx_stdio) = channel();

    let mut brocker_tx_channels = Vec::<SubscriberChannel>::new();
    brocker_tx_channels.push( SubscriberChannel{id: ID_STDOUT, tx: tx_stdio.clone()});

    let tx_broker_clone = tx_brocker.clone();
    let stdin_future = lazy(move || {
        match read_stdin(ID_STDIN, tx_broker_clone) {
            _ => {
                // noop
            }
        }
        Ok(())
    });

    let broker_future = lazy(move || {
        match broker(rx_brocker, brocker_tx_channels) {
            _ => {
                // noop
            }
        }
        Ok(())
    });

    let stdout_future = lazy(move || {
        match write_stdout(rx_stdio) {
            _ => {
                // noop
            }
        }
        Ok(())
    });

    tokio::run( lazy(move || {
            tokio::spawn(stdin_future );
            tokio::spawn(broker_future );
            tokio::spawn(stdout_future );
            Ok(())
        })
    );

    println!("Exit main");
}
