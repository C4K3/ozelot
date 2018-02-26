//! A very basic text-only client that connects to a server, prints chat
//! messages, and lets you write chat messages to the server.
//!
//! Connect to the specified server (--host) using the specified username
//! (--user) optionally specifying --port and --unauthenticated (defaults to
//! authenticated.) If authenticated, will prompt for password on commandline.
//! Once connected, will print out (as much as possible, translate components
//! are only partially supported) of received ChatMessages. Lines entered to
//! stdin are sent to the server also as ChatMessages.
//!
//! Note the liberal use of unwrap, which leads to some hard to understand error
//! messages.
extern crate ozelot;
extern crate getopts;
extern crate rpassword;

use std::{thread, time};
use std::sync::mpsc::{channel, Sender};
use std::process::exit;
use std::io;
use std::env;

use ozelot::{Client, serverbound, mojang, utils};
use ozelot::clientbound::*;

use getopts::Options;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.reqopt("u", "username", "your username", "C4K3");
    opts.reqopt("h", "host", "the server's hostname", "minecraft.example.com");
    opts.optopt("p", "port", "the server's port", "25565");
    opts.optflag("", "unauthenticated", "connect unauthenticated instead of authenticated");
    let matches = opts.parse(&args[1..]).unwrap();

    let username = matches.opt_str("username").expect("unreachable");
    let host = matches.opt_str("host").expect("unreachable");
    let port = if let Some(x) = matches.opt_str("port") {
        x.parse::<u16>().unwrap()
    } else {
        25565
    };
    let authenticated: bool = matches.opt_present("unauthenticated") == false;

    let mut client = if authenticated {
        let password = rpassword::prompt_password_stdout("Enter password: ").unwrap();
        let auth = mojang::Authenticate::new(username, password).perform().unwrap();
        match Client::connect_authenticated(&host, port, &auth) {
            Ok(x) => x,
            Err(e) => {
                println!("Error connecting to {}:{}: {:?}", host, port, e);
                exit(1);
            },
        }
    } else {
        match Client::connect_unauthenticated(&host, port, &username) {
            Ok(x) => x,
            Err(e) => {
                println!("Error connecting unauthenticated to {}:{}: {:?}", host, port, e);
                exit(1);
            },
        }
    };

    let (tx, rx) = channel();
    thread::spawn(move || {
        read_stdin(tx);
    });

    'main: loop {
        let packets = client.read().unwrap();
        for packet in packets {
            match packet {
                ClientboundPacket::PlayDisconnect(ref p) => {
                    println!("Got disconnect packet, exiting ...");
                    println!("Reason: {}", utils::chat_to_str(p.get_reason()).unwrap());
                    break 'main;
                },
                ClientboundPacket::ChatMessage(ref p) => {
                    let msg = utils::chat_to_str(p.get_chat()).unwrap();
                    println!("{}", msg);
                },
                _ => (),
            }
        }

        if let Ok(msg) = rx.try_recv() {
            let msg = msg.trim_right().to_string();
            let chat = serverbound::ChatMessage::new(msg);
            client.send(chat).unwrap();
        }

        thread::sleep(time::Duration::from_millis(50));
    }
}

fn read_stdin(tx: Sender<String>) {
    loop {
        let mut tmp = String::new();
        let _: usize = io::stdin().read_line(&mut tmp).unwrap();
        tx.send(tmp).unwrap();
    }
}
