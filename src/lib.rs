//! [(Link to repository page.)](https://github.com/C4K3/Ozelot)
//!
//! This is a library made for interacting with the Minecraft protocl
//! (MCMODERN only, not MCPE.)
//!
//! It is mostly a low-level library, its goals are to
//! handle everything network related, but nothing beyond that. For example
//! this library does not support any types beyond ones found in standard Rust,
//! this results in certain packets that contain compound datatypes not having
//! their complex fields serialized by this library, but instead handing the
//! raw binary data to consumers of this library to parse however they wish.
//! One example of this is packets that contain NBT data, the packets are read
//! but the NBT data is not in any way parsed by this library. You'll probably
//! want to see what the meanings of each of the packets are, which is
//! documented on [wiki.vg](http://wiki.vg/Main_Page). The [protocol
//! documentation](http://wiki.vg/Protocol) in particular is likely to be a
//! necessary companion to using this library.
//!
//! Currently the library is entirely synchronous, requiring consumers to handle
//! concurrency however they wish. It would be cool to add an asynchronous API
//! though, maybe using mio.
//!
//! The library nominally supports every packet used in MCMODERN, but the goal
//! is still to add lots more helper functions and other useful things. There
//! are still many errors and inconvenient things in the library, where changes
//! or helper functions would be good to add. If you find any errors or
//! suggestions for improvement, no matter how trivial, then post it on the
//! bugtracker.
//!
//! The library does not yet have any stable releases, not because it may
//! contain bugs (which it certainly does,) but because the API may be
//! significantly restructured without warning.
//!
//! # Examples
//!
//! Connecting to a remote server as an authenticated user, printing all chat
//! messages, and then echoing back messages that appear not to be written by us
//! (else we'd get into an infinite loop echoing our own messages.)
//!
//! ```rust,no_run
//! use ozelot::{mojang, Client, serverbound, utils};
//! use ozelot::clientbound::ClientboundPacket;
//!
//! let auth = mojang::Authenticate::new("my_email@example.com".to_string(),
//!                                      "my_password".to_string())
//!     .perform().unwrap();
//!
//!
//! /* By using connect_authenticated, auto_handle will be true and thus ozelot
//!  * will respond to keepalives automatically */
//! let mut client = Client::connect_authenticated("minecraft.example.com",
//!                                                25565, &auth).unwrap();
//!
//! let username = auth.selectedProfile.name;
//!
//! 'main: loop {
//!     let packets = client.read().unwrap();
//!     for packet in packets {
//!         match packet {
//!         ClientboundPacket::PlayDisconnect(ref p) => {
//!             println!("Disconnected, reason: {}",
//!                      utils::chat_to_str(p.get_reason()));
//!             break 'main;
//!         },
//!         ClientboundPacket::ChatMessage(ref p) => {
//!             let msg = utils::chat_to_str(p.get_chat());
//!             println!("{}", msg);
//!             if !msg.contains(&username) {
//!                 /* Since we don't want an infinite loop, we don't echo back
//!                  * our own messages. We say that a received message wasn't
//!                  * written by us if it doesn't contain our username
//!                  *
//!                  * Note that this echoes back the raw chat message, that is
//!                  * playername and everything, and also non-chat messages. */
//!                 let response = serverbound::ChatMessage::new(msg);
//!                 client.send(response).unwrap();
//!             }
//!         },
//!         /* We throw away all other packets */
//!         _ => (),
//!         }
//!     }
//! }
//! ```
extern crate byteorder;
extern crate flate2;
extern crate netbuf;
extern crate openssl;
extern crate reqwest; /* Holy fuck the dependencies needed just to make 2
                         HTTP POST requests ... */
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate error_chain;

mod client;
mod connection;
#[allow(non_snake_case)]
mod json;
mod server;
mod yggdrasil;
pub mod clientbound;
pub mod errors;
#[allow(non_snake_case)]
pub mod mojang;
pub mod read;
pub mod serverbound;
pub mod utils;
pub mod write;
#[cfg(test)]
mod tests;

pub use client::Client;
pub use server::Server;

use std::fmt;

use flate2::Compression;

const COMPRESSION_LEVEL: Compression = Compression::Default;
/// The protocol version supported by this version of ozelot
pub const PROTOCOL_VERSION: i32 = 316;

/// FIXME temporary struct until u128 lands in stable rust, used for uuids
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_camel_case_types)]
pub struct u128(pub u64, pub u64);

/// This tracks which state of play the client is in. The value of this changes
/// the meaning of the different packet ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    Handshake,
    Status,
    Login,
    Play,
}
impl fmt::Display for ClientState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "{}",
               match self {
                   &ClientState::Handshake => "Handshake",
                   &ClientState::Status => "Status",
                   &ClientState::Login => "Login",
                   &ClientState::Play => "Play",
               })
    }
}
