//! Contains all packets that can be sent by an ozelot client
//!
//! See the Serverbound sections on http://wiki.vg/Protocol for information
//! about each of the packets.

use read::*;
use write::*;
use connection::Packet;
use {ClientState, u128, yggdrasil};

use std::io::Read;
use std::io;

/* See packets.clj for information about this include */
include!("./.serverbound-enum.generated.rs");
include!("./.serverbound-packets.generated.rs");

/* Now come to the manual definitions of packets that don't fit into the
 * code generation */

impl Handshake {
    /// Get the next state as a ClientState, if the value is valid
    pub fn get_next_clientstate(&self) -> Option<ClientState> {
        match self.next_state {
            1 => Some(ClientState::Status),
            2 => Some(ClientState::Login),
            _ => None,
        }
    }
}

impl StatusRequest {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&StatusRequest::get_packet_id(), &mut ret)?;
        Ok(ret)
    }
    fn deserialize<R: Read>(_: &mut R) -> io::Result<ServerboundPacket> {
        Ok(ServerboundPacket::StatusRequest(StatusRequest { }))
    }
}

impl TabComplete {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&TabComplete::get_packet_id(), &mut ret)?;
        write_String(&self.text, &mut ret)?;
        write_bool(&self.assume_command, &mut ret)?;
        match self.looked_at_block {
            Some(x) => {
                write_bool(&true, &mut ret)?;
                write_position(&x, &mut ret)?;
            },
            None => {
                write_bool(&false, &mut ret)?;
            },
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ServerboundPacket> {
        let text = read_String(r)?;
        let assume_command = read_bool(r)?;
        let has_position = read_bool(r)?;
        let looked_at_block = if has_position {
            Some(read_position(r)?)
        } else {
            None
        };

        Ok(ServerboundPacket::TabComplete(TabComplete {
            text: text,
            assume_command: assume_command,
            looked_at_block: looked_at_block,
        }))
    }
}

impl UseEntity {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&UseEntity::get_packet_id(), &mut ret)?;
        write_varint(&self.target, &mut ret)?;
        write_varint(&self.action, &mut ret)?;
        match self.action {
            2 => {
                if let Some((x, y, z)) = self.location {
                    write_f32(&x, &mut ret)?;
                    write_f32(&y, &mut ret)?;
                    write_f32(&z, &mut ret)?;
                } else {
                    return io_error!("UseEntity had invalid values. Location was None even though action was 2.");
                }
            },
            _ => (),
        }

        match self.action {
            0 | 1 => {
                if let Some(x) = self.hand {
                    write_varint(&x, &mut ret)?;
                } else {
                    return io_error!("UseEntity had invalid values. Hand was none even though action was {}",
                                     self.action);
                }
            },
            _ => (),
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ServerboundPacket> {
        let target = read_varint(r)?;
        let action = read_varint(r)?;

        let location = if action == 2 {
            Some((read_f32(r)?, read_f32(r)?, read_f32(r)?))
        } else {
            None
        };

        let hand = if action == 0 || action == 1 {
            Some(read_varint(r)?)
        } else {
            None
        };
        Ok(ServerboundPacket::UseEntity(UseEntity {
            target: target,
            action: action,
            location: location,
            hand: hand,
        }))
    }
}

impl EncryptionResponse {
    /// Create the EncryptionResponse packet from the unencrypted shared secret
    /// and verify token, and the server's public key in DER format.
    pub fn new_unencrypted(key: &[u8],
                           shared_secret: &[u8],
                           verify_token: &[u8])
        -> io::Result<ServerboundPacket> {
            let ss_encrypted = yggdrasil::rsa_encrypt(key, shared_secret)?;
            let verify_encrypted = yggdrasil::rsa_encrypt(key, verify_token)?;

            Ok(EncryptionResponse::new(ss_encrypted, verify_encrypted))
        }
}

