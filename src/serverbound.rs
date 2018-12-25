//! Contains all serverbound packets
//!
//! See the Serverbound sections on http://wiki.vg/Protocol for information
//! about each of the packets.

use connection::Packet;
use errors::Result;
use read::*;
use write::*;
use {ClientState, utils};

use std::fmt;
use std::io::Read;

use openssl::rsa::Rsa;
use openssl::pkey::Private;

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

impl EncryptionResponse {
    pub fn get_decrypted_shared_secret(&self, key: &Rsa<Private>) -> Result<[u8; 16]> {
        let tmp = utils::rsa_decrypt(key, &self.shared_secret)?;
        if tmp.len() != 16 {
            bail!("Decrypted shared secret was not 16 bytes long");
        }
        let mut ret = [0; 16];
        for i in 0..16 {
            ret[i] = tmp[i];
        }
        Ok(ret)
    }
    pub fn get_decrypted_verify_token(&self, key: &Rsa<Private>) -> Result<Vec<u8>> {
        utils::rsa_decrypt(key, &self.verify_token)
    }
    /// Create the EncryptionResponse packet from the unencrypted shared secret
    /// and verify token, and the server's public key in DER format.
    pub fn new_unencrypted(key: &[u8],
                           shared_secret: &[u8],
                           verify_token: &[u8])
                           -> Result<ServerboundPacket> {
        let ss_encrypted = utils::rsa_encrypt(key, shared_secret)?;
        let verify_encrypted = utils::rsa_encrypt(key, verify_token)?;

        Ok(EncryptionResponse::new(ss_encrypted, verify_encrypted))
    }
}

impl StatusRequest {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&StatusRequest::PACKET_ID, &mut ret)?;
        Ok(ret)
    }
    fn deserialize<R: Read>(_: &mut R) -> Result<ServerboundPacket> {
        Ok(ServerboundPacket::StatusRequest(StatusRequest {}))
    }
}

impl UseEntity {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&UseEntity::PACKET_ID, &mut ret)?;
        write_varint(&self.target, &mut ret)?;
        write_varint(&self.action, &mut ret)?;
        match self.action {
            2 => {
                if let Some((x, y, z)) = self.location {
                    write_f32(&x, &mut ret)?;
                    write_f32(&y, &mut ret)?;
                    write_f32(&z, &mut ret)?;
                } else {
                    bail!("UseEntity had invalid values. Location was None even though action was 2.");
                }
            },
            _ => (),
        }

        match self.action {
            0 | 2 => {
                if let Some(x) = self.hand {
                    write_varint(&x, &mut ret)?;
                } else {
                    bail!("UseEntity had invalid values. Hand was none even though action was {}", self.action);
                }
            },
            _ => (),
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ServerboundPacket> {
        let target = read_varint(r)?;
        let action = read_varint(r)?;

        let location = if action == 2 {
            Some((read_f32(r)?, read_f32(r)?, read_f32(r)?))
        } else {
            None
        };

        let hand = if action == 0 || action == 2 {
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

impl RecipeBookData {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Self::PACKET_ID, &mut ret)?;

        if let Some(ref x) = self.displayed_recipe {
            write_varint(&0, &mut ret)?;
            write_String(x, &mut ret)?;
        } else if let Some((a, b, c, d)) = self.recipe_book_states {
            write_varint(&1, &mut ret)?;
            write_bool(&a, &mut ret)?;
            write_bool(&b, &mut ret)?;
            write_bool(&c, &mut ret)?;
            write_bool(&d, &mut ret)?;
        } else {
            bail!("Invalid RecipeBookData packet");
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ServerboundPacket> {
        let type_ = read_varint(r)?;
        let (displayed_recipe, recipe_book_states) = match type_ {
            0 => (Some(read_String(r)?), None),
            1 => (None, Some((read_bool(r)?, read_bool(r)?, read_bool(r)?, read_bool(r)?))),
            _ => bail!("CraftingBookData got invalid type {}", type_),
        };
        Ok(ServerboundPacket::RecipeBookData(RecipeBookData{
            displayed_recipe,
            recipe_book_states,
        }))
    }
}

impl AdvancementTab {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Self::PACKET_ID, &mut ret)?;

        if let Some(ref tab_id) = self.tab_id {
            write_varint(&0, &mut ret)?;
            write_String(tab_id, &mut ret)?;
        } else {
            write_varint(&1, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ServerboundPacket> {
        let action = read_varint(r)?;
        let tab_id = match action {
            0 => Some(read_String(r)?),
            1 => None,
            _ => bail!("Advancement Tab got invalid action {}", action),
        };
        Ok(ServerboundPacket::AdvancementTab(AdvancementTab {
                                                 tab_id: tab_id,
                                             }))
    }
}

