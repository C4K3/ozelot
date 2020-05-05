//! Contains all the clientbound packets
//!
//! See the clientbound sections on http://wiki.vg/Protocol for information
//! about all the available packets.
//!
//! The goal is also to add a bunch of useful helper functions to the packets,
//! if you feel such a function is missing, open an issue.

use connection::Packet;
use errors::Result;
use read::*;
use write::*;
use ClientState;

use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;

/* See packets.clj for information about this include */
include!("./.clientbound-enum.generated.rs");
include!("./.clientbound-packets.generated.rs");

/* Now come the manual definitions for the edge cases that don't fit into
 * packets.clj's code generation */

impl Statistics {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Statistics::PACKET_ID, &mut ret)?;
        write_varint(&(self.values.len() as i32), &mut ret)?;
        for (key, value) in self.values.iter() {
            write_String(key, &mut ret)?;
            write_varint(value, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let count = read_varint(r)?;
        let mut tmp = BTreeMap::new();
        for _ in 0..count {
            let _: Option<i32> = tmp.insert(read_String(r)?, read_varint(r)?);
        }
        Ok(ClientboundPacket::Statistics(Statistics {
                                             values: tmp,
                                         }))
    }
}

impl ClientboundTabComplete {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&ClientboundTabComplete::PACKET_ID, &mut ret)?;

        write_varint(&self.transaction_id, &mut ret)?;
        write_varint(&self.start, &mut ret)?;
        write_varint(&self.length, &mut ret)?;
        write_varint(&(self.matches.len() as i32), &mut ret)?;
        for (match_, tooltip) in &self.matches {
            write_String(match_, &mut ret)?;
            match tooltip {
                Some(x) => {
                    write_bool(&true, &mut ret)?;
                    write_String(x, &mut ret)?;
                },
                None => {
                    write_bool(&false, &mut ret)?;
                },
            }
        }

        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let transaction_id = read_varint(r)?;
        let start = read_varint(r)?;
        let length = read_varint(r)?;

        let count = read_varint(r)?;
        let mut matches: Vec<(String, Option<String>)> = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let match_ = read_String(r)?;
            let has_tooltip = read_bool(r)?;
            let tooltip = if has_tooltip {
                Some(read_String(r)?)
            } else {
                None
            };
            matches.push((match_, tooltip));
        }

        Ok(ClientboundPacket::ClientboundTabComplete(ClientboundTabComplete {
            transaction_id,
            start,
            length,
            matches,
        }))
    }
}

impl MultiBlockChange {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&MultiBlockChange::PACKET_ID, &mut ret)?;
        write_i32(&self.chunk_x, &mut ret)?;
        write_i32(&self.chunk_z, &mut ret)?;
        write_varint(&(self.changes.len() as i32), &mut ret)?;
        for &(x, y, z, new_state) in &self.changes {
            let xz: u8 = (x << 4) | z;
            write_u8(&xz, &mut ret)?;
            write_u8(&y, &mut ret)?;
            write_varint(&new_state, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let chunk_x = read_i32(r)?;
        let chunk_z = read_i32(r)?;
        let count = read_varint(r)?;
        let mut tmp: Vec<(u8, u8, u8, i32)> = Vec::with_capacity(count as
                                                                 usize);
        for _ in 0..count {
            let hori_pos = read_u8(r)?;
            let y = read_u8(r)?;
            let block_state = read_varint(r)?;
            let x = (hori_pos & 0xF0) >> 4;
            let z = hori_pos & 0x0F;
            tmp.push((x, y, z, block_state));
        }
        Ok(ClientboundPacket::MultiBlockChange(MultiBlockChange {
                                                   chunk_x: chunk_x,
                                                   chunk_z: chunk_z,
                                                   changes: tmp,
                                               }))
    }
}

impl Explosion {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Explosion::PACKET_ID, &mut ret)?;
        write_f32(&self.x, &mut ret)?;
        write_f32(&self.y, &mut ret)?;
        write_f32(&self.z, &mut ret)?;
        write_f32(&self.radius, &mut ret)?;
        write_i32(&(self.affected_blocks.len() as i32), &mut ret)?;
        for &(x, y, z) in &self.affected_blocks {
            write_i8(&x, &mut ret)?;
            write_i8(&y, &mut ret)?;
            write_i8(&z, &mut ret)?;
        }
        write_f32(&self.motion_x, &mut ret)?;
        write_f32(&self.motion_y, &mut ret)?;
        write_f32(&self.motion_z, &mut ret)?;
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let x = read_f32(r)?;
        let y = read_f32(r)?;
        let z = read_f32(r)?;
        let radius = read_f32(r)?;
        let count = read_i32(r)?;
        let mut tmp: Vec<(i8, i8, i8)> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            tmp.push((read_i8(r)?, read_i8(r)?, read_i8(r)?));
        }
        let motion_x = read_f32(r)?;
        let motion_y = read_f32(r)?;
        let motion_z = read_f32(r)?;
        Ok(ClientboundPacket::Explosion(Explosion {
                                            x: x,
                                            y: y,
                                            z: z,
                                            radius: radius,
                                            affected_blocks: tmp,
                                            motion_x: motion_x,
                                            motion_y: motion_y,
                                            motion_z: motion_z,
                                        }))
    }
}

impl Particle {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Particle::PACKET_ID, &mut ret)?;
        write_i32(&self.particle_id, &mut ret)?;
        write_bool(&self.use_long_distance, &mut ret)?;
        write_f64(&self.x, &mut ret)?;
        write_f64(&self.y, &mut ret)?;
        write_f64(&self.z, &mut ret)?;
        write_f32(&self.offset_x, &mut ret)?;
        write_f32(&self.offset_y, &mut ret)?;
        write_f32(&self.offset_z, &mut ret)?;
        write_f32(&self.particle_data, &mut ret)?;
        write_i32(&self.count, &mut ret)?;
        write_bytearray_to_end(&self.data, &mut ret)?;
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let particle_id = read_i32(r)?;
        let use_long_distance = read_bool(r)?;
        let x = read_f64(r)?;
        let y = read_f64(r)?;
        let z = read_f64(r)?;
        let offset_x = read_f32(r)?;
        let offset_y = read_f32(r)?;
        let offset_z = read_f32(r)?;
        let particle_data = read_f32(r)?;
        let count = read_i32(r)?;
        let data = read_bytearray_to_end(r)?;
        Ok(ClientboundPacket::Particle(Particle {
            particle_id,
            use_long_distance,
            x,
            y,
            z,
            offset_x,
            offset_y,
            offset_z,
            particle_data,
            count,
            data,
        }))
    }
}

impl CombatEvent {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&CombatEvent::PACKET_ID, &mut ret)?;
        write_varint(&self.event, &mut ret)?;
        if let Some(x) = self.duration_playerid {
            write_varint(&x, &mut ret)?;
        }
        if let Some(x) = self.entity_id {
            write_i32(&x, &mut ret)?;
        }
        if let &Some(ref x) = &self.message {
            write_String(x, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let event = read_varint(r)?;
        let (duration_playerid, entity_id, message) = match event {
            0 => (None, None, None),
            1 => (Some(read_varint(r)?), Some(read_i32(r)?), None),
            2 => {
                (Some(read_varint(r)?),
                 Some(read_i32(r)?),
                 Some(read_String(r)?))
            },
            _ => bail!("Invalid event in CombatEvent"),
        };
        Ok(ClientboundPacket::CombatEvent(CombatEvent {
                                              event: event,
                                              duration_playerid:
                                                  duration_playerid,
                                              entity_id: entity_id,
                                              message: message,
                                          }))
    }
}

impl ScoreboardObjective {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&ScoreboardObjective::PACKET_ID, &mut ret)?;
        write_String(&self.name, &mut ret)?;
        write_u8(&self.mode, &mut ret)?;
        if let &Some(ref x) = &self.value {
            write_String(x, &mut ret)?;
        }
        if let &Some(ref x) = &self.objective_type {
            write_String(x, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let name = read_String(r)?;
        let mode = read_u8(r)?;
        let (value, objective_type) = match mode {
            0 | 2 => (Some(read_String(r)?), Some(read_String(r)?)),
            _ => (None, None),
        };
        Ok(ClientboundPacket::ScoreboardObjective(ScoreboardObjective {
                                                      name: name,
                                                      mode: mode,
                                                      value: value,
                                                      objective_type:
                                                          objective_type,
                                                  }))
    }
}

impl UpdateScore {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&UpdateScore::PACKET_ID, &mut ret)?;
        write_String(&self.name, &mut ret)?;
        write_u8(&self.action, &mut ret)?;
        write_String(&self.objective_name, &mut ret)?;
        if let Some(x) = self.value {
            write_varint(&x, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let name = read_String(r)?;
        let action = read_u8(r)?;
        let objective_name = read_String(r)?;
        let value = match action {
            1 => None,
            _ => Some(read_varint(r)?),
        };
        Ok(ClientboundPacket::UpdateScore(UpdateScore {
                                              name: name,
                                              action: action,
                                              objective_name: objective_name,
                                              value: value,
                                          }))
    }
}

impl Title {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Title::PACKET_ID, &mut ret)?;
        write_varint(&self.action, &mut ret)?;
        if let &Some(ref x) = &self.text {
            write_String(x, &mut ret)?;
        } else if let Some((a, b, c)) = self.times {
            write_i32(&a, &mut ret)?;
            write_i32(&b, &mut ret)?;
            write_i32(&c, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let action = read_varint(r)?;
        let text = match action {
            0 | 1 | 2 => Some(read_String(r)?),
            _ => None,
        };
        let times = match action {
            3 => Some((read_i32(r)?, read_i32(r)?, read_i32(r)?)),
            _ => None,
        };
        Ok(ClientboundPacket::Title(Title {
                                        action: action,
                                        text: text,
                                        times: times,
                                    }))
    }
}

impl PlayerAbilities {
    /// Get whether the player is invulnerable
    pub fn is_invulnerable(&self) -> bool {
        (self.flags & 0x01) != 0
    }
    /// Get whether the player is flying
    pub fn is_flying(&self) -> bool {
        (self.flags & 0x02) != 0
    }
    /// Get whether the player is allowed to fly
    pub fn allow_flying(&self) -> bool {
        (self.flags & 0x04) != 0
    }
    /// Get whether player is in creative mode
    pub fn is_creative(&self) -> bool {
        (self.flags & 0x08) != 0
    }
}

impl FacePlayer {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Self::PACKET_ID, &mut ret)?;
        write_varint(&self.feet_or_eyes, &mut ret)?;
        write_f64(&self.x, &mut ret)?;
        write_f64(&self.y, &mut ret)?;
        write_f64(&self.z, &mut ret)?;

        if let (Some(id), Some(feet_or_eyes)) = (self.entity_id, self.entity_feet_or_eyes) {
            write_bool(&true, &mut ret)?;
            write_varint(&id, &mut ret)?;
            write_varint(&feet_or_eyes, &mut ret)?;
        } else {
            write_bool(&false, &mut ret)?;
        }

        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let feet_or_eyes = read_varint(r)?;
        let x = read_f64(r)?;
        let y = read_f64(r)?;
        let z = read_f64(r)?;
        let is_entity = read_bool(r)?;
        let entity_id = if is_entity {
            Some(read_varint(r)?)
        } else {
            None
        };
        let entity_feet_or_eyes = if is_entity {
            Some(read_varint(r)?)
        } else {
            None
        };

        Ok(ClientboundPacket::FacePlayer(FacePlayer {
            feet_or_eyes,
            x,
            y,
            z,
            entity_id,
            entity_feet_or_eyes,
                                                   }))
    }
}

impl UnlockRecipes {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Self::PACKET_ID, &mut ret)?;
        write_varint(&self.action, &mut ret)?;
        write_bool(&self.crafting_book_open, &mut ret)?;
        write_bool(&self.crafting_book_filter, &mut ret)?;
        write_bool(&self.smelting_book_open, &mut ret)?;
        write_bool(&self.smelting_book_filter, &mut ret)?;
        write_varint(&(self.recipes.len() as i32), &mut ret)?;
        for x in &self.recipes {
            write_String(x, &mut ret)?;
        }
        if self.action == 0 {
            write_varint(&(self.recipes2.len() as i32), &mut ret)?;
            for x in &self.recipes2 {
                write_String(x, &mut ret)?;
            }
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let action = read_varint(r)?;
        let crafting_book_open = read_bool(r)?;
        let crafting_book_filter = read_bool(r)?;
        let smelting_book_open = read_bool(r)?;
        let smelting_book_filter = read_bool(r)?;

        let count1 = read_varint(r)? as usize;
        let mut recipes = Vec::with_capacity(count1);
        for _ in 0..count1 {
            recipes.push(read_String(r)?);
        }

        let mut recipes2 = Vec::new();
        if action == 0 {
            let count2 = read_varint(r)? as usize;
            recipes2.reserve(count2);
            for _ in 0..count2 {
                recipes2.push(read_String(r)?);
            }
        }

        Ok(ClientboundPacket::UnlockRecipes(UnlockRecipes {
            action,
            crafting_book_open,
            crafting_book_filter,
            smelting_book_open,
            smelting_book_filter,
            recipes,
            recipes2,
        }))
    }
}

impl SelectAdvancementTab {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Self::PACKET_ID, &mut ret)?;
        if let Some(ref identifier) = self.identifier {
            if identifier.len() > 32767 {
                bail!("SelectAdvancementTab identifier is too long, is {} bytes long",
                      identifier.len());
            }
            write_bool(&true, &mut ret)?;
            write_String(identifier, &mut ret)?;
        } else {
            write_bool(&false, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let has_id = read_bool(r)?;
        let identifier = if has_id {
            let tmp = read_String(r)?;
            if tmp.len() > 32767 {
                bail!("SelectAdvancementTab identifier is too long, is {} bytes long",
                      tmp.len());
            }
            Some(tmp)
        } else {
            None
        };

        Ok(ClientboundPacket::SelectAdvancementTab(SelectAdvancementTab {
                                                       identifier,
                                                   }))
    }
}

impl StopSound {
    fn to_u8(&self) -> Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Self::PACKET_ID, &mut ret)?;
        write_u8(&self.flags, &mut ret)?;

        let should_write_source = (self.flags & 0x1) != 0;
        let should_write_sound = (self.flags & 0x2) != 0;

        if should_write_source != self.source.is_some() {
            bail!("Value of flags does not match value of source in StopSound");
        }
        if should_write_sound != self.sound.is_some() {
            bail!("Value of flags does not match value of sound in StopSound");
        }

        if let Some(ref x) = self.source {
            write_varint(x, &mut ret)?;
        }
        if let Some(ref x) = self.sound {
            write_String(x, &mut ret)?;
        }

        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> Result<ClientboundPacket> {
        let flags = read_u8(r)?;
        let source = if (flags & 0x1) != 0 {
            Some(read_varint(r)?)
        } else {
            None
        };
        let sound = if (flags & 0x2) != 0 {
            Some(read_String(r)?)
        } else {
            None
        };

        Ok(ClientboundPacket::StopSound(StopSound {
            flags,
            source,
            sound,
        }))
    }
}
