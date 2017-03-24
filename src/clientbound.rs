//! Contains all the clientbound packets
//!
//! See the clientbound sections on http://wiki.vg/Protocol for information
//! about all the available packets.
//!
//! The goal is also to add a bunch of useful helper functions to the packets,
//! if you feel such a function is missing, open an issue.

use read::*;
use write::*;
use connection::Packet;
use {ClientState, u128};

use std::collections::BTreeMap;
use std::fmt;
use std::io::Read;
use std::io;

/* See packets.clj for information about this include */
include!("./.clientbound-enum.generated.rs");
include!("./.clientbound-packets.generated.rs");

/* Now come the manual definitions for the edge cases that don't fit into
 * packets.clj's code generation */

impl Statistics {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Statistics::get_packet_id(), &mut ret)?;
        write_varint(&(self.values.len() as i32), &mut ret)?;
        for (key, value) in self.values.iter() {
            write_String(key, &mut ret)?;
            write_varint(value, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let count = read_varint(r)?;
        let mut tmp = BTreeMap::new();
        for _ in 0..count {
            tmp.insert(read_String(r)?, read_varint(r)?);
        }
        Ok(ClientboundPacket::Statistics(Statistics {
                                             values: tmp,
                                         }))
    }
}

impl ClientboundTabComplete {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&ClientboundTabComplete::get_packet_id(), &mut ret)?;
        write_varint(&(self.matches.len() as i32), &mut ret)?;
        for value in &self.matches {
            write_String(value, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let count = read_varint(r)?;
        let mut tmp = Vec::with_capacity(count as usize);
        for _ in 0..count {
            tmp.push(read_String(r)?);
        }
        Ok(ClientboundPacket::ClientboundTabComplete(ClientboundTabComplete {
                                                         matches: tmp,
                                                     }))
    }
}

impl MultiBlockChange {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&MultiBlockChange::get_packet_id(), &mut ret)?;
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
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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

impl OpenWindow {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&OpenWindow::get_packet_id(), &mut ret)?;
        write_u8(&self.window_id, &mut ret)?;
        write_String(&self.window_type, &mut ret)?;
        write_String(&self.window_title, &mut ret)?;
        write_u8(&self.number_of_slots, &mut ret)?;
        if let Some(entity_id) = self.entity_id {
            write_i32(&entity_id, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let window_id = read_u8(r)?;
        let window_type = read_String(r)?;
        let window_title = read_String(r)?;
        let number_of_slots = read_u8(r)?;
        let entity_id = match &*window_type {
            "EntityHorse" => Some(read_i32(r)?),
            _ => None,
        };

        Ok(ClientboundPacket::OpenWindow(OpenWindow {
                                             window_id: window_id,
                                             window_type: window_type,
                                             window_title: window_title,
                                             number_of_slots: number_of_slots,
                                             entity_id: entity_id,
                                         }))
    }
}

impl Explosion {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Explosion::get_packet_id(), &mut ret)?;
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
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Particle::get_packet_id(), &mut ret)?;
        write_i32(&self.particle_id, &mut ret)?;
        write_bool(&self.use_long_distance, &mut ret)?;
        write_f32(&self.x, &mut ret)?;
        write_f32(&self.y, &mut ret)?;
        write_f32(&self.z, &mut ret)?;
        write_f32(&self.offset_x, &mut ret)?;
        write_f32(&self.offset_y, &mut ret)?;
        write_f32(&self.offset_z, &mut ret)?;
        write_f32(&self.particle_data, &mut ret)?;
        write_i32(&self.count, &mut ret)?;
        if let Some(x) = self.id {
            write_varint(&x, &mut ret)?;
        }
        if let Some(x) = self.crack_data {
            write_varint(&x, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let particle_id = read_i32(r)?;
        let long_distance = read_bool(r)?;
        let x = read_f32(r)?;
        let y = read_f32(r)?;
        let z = read_f32(r)?;
        let offset_x = read_f32(r)?;
        let offset_y = read_f32(r)?;
        let offset_z = read_f32(r)?;
        let particle_data = read_f32(r)?;
        let count = read_i32(r)?;
        let id = match particle_id {
            36 | 37 | 38 => Some(read_varint(r)?),
            _ => None,
        };
        let crack_data = match particle_id {
            36 | 37 => Some(read_varint(r)?),
            _ => None,
        };
        Ok(ClientboundPacket::Particle(Particle {
                                           particle_id: particle_id,
                                           use_long_distance: long_distance,
                                           x: x,
                                           y: y,
                                           z: z,
                                           offset_x: offset_x,
                                           offset_y: offset_y,
                                           offset_z: offset_z,
                                           particle_data: particle_data,
                                           count: count,
                                           id: id,
                                           crack_data: crack_data,
                                       }))
    }
}

impl CombatEvent {
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&CombatEvent::get_packet_id(), &mut ret)?;
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
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let event = read_varint(r)?;
        let (duration_playerid, entity_id, message) = match event {
            0 => (None, None, None),
            1 => (Some(read_varint(r)?), Some(read_i32(r)?), None),
            2 => {
                (Some(read_varint(r)?),
                 Some(read_i32(r)?),
                 Some(read_String(r)?))
            },
            _ => return io_error!("Invalid event in CombatEvent"),
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
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&ScoreboardObjective::get_packet_id(), &mut ret)?;
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
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&UpdateScore::get_packet_id(), &mut ret)?;
        write_String(&self.name, &mut ret)?;
        write_u8(&self.action, &mut ret)?;
        write_String(&self.objective_name, &mut ret)?;
        if let Some(x) = self.value {
            write_varint(&x, &mut ret)?;
        }
        Ok(ret)
    }
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let name = read_String(r)?;
        let action = read_u8(r)?;
        let objective_name = read_String(r)?;
        let value = match action {
            1 => Some(read_varint(r)?),
            _ => None,
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
    fn to_u8(&self) -> io::Result<Vec<u8>> {
        let mut ret = Vec::new();
        write_varint(&Title::get_packet_id(), &mut ret)?;
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
    fn deserialize<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
