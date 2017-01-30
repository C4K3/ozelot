//! Contains all packets that can be sent by an ozelot client
//!
//! See the Serverbound sections on http://wiki.vg/Protocol for information
//! about each of the packets.

use write::*;
use {ClientState, Sendable, yggdrasil, u128};

use std::io;

/* See packets.clj for information about this include */
include!("./.client_send.generated.rs");

impl TabComplete {
    #[inline(always)]
    fn to_u8_custom(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&TabComplete::get_id(), &mut ret)?;
        write_String(&self.text, &mut ret)?;
        write_bool(&self.assume_command, &mut ret)?;
        match self.looked_at_block {
            Some(x) => {
                write_bool(&true, &mut ret)?;
                write_u64(&x, &mut ret)?;
            },
            None => {
                write_bool(&false, &mut ret)?;
            },
        }
        Ok(ret)
    }
}

impl UseEntity {
    #[inline(always)]
    fn to_u8_custom(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&UseEntity::get_id(), &mut ret)?;
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
}

impl EncryptionResponse {
    /// Create the EncryptionResponse packet from the unencrypted shared secret
    /// and verify token, and the server's public key in DER format.
    pub fn new_unencrypted(key: &[u8],
                           shared_secret: &[u8],
                           verify_token: &[u8]) -> io::Result<Self> {
        let ss_encrypted = yggdrasil::rsa_encrypt(key, shared_secret)?;
        let verify_encrypted = yggdrasil::rsa_encrypt(key, verify_token)?;

        Ok(EncryptionResponse::new(ss_encrypted, verify_encrypted))
    }
}

