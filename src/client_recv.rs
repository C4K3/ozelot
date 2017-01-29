//! Contains all the clientbound packets
//!
//! See the clientbound sections on http://wiki.vg/Protocol for information
//! about all the available packets.
//!
//! The goal is also to add a bunch of useful helper functions to the packets,
//! if you feel such a function is missing, open an issue.

use {ClientState, u128};
use read::*;

use std::io;
use std::io::Read;
use std::ops::Deref;
use std::collections::BTreeMap;

/* See packets.clj for information about this include */
include!("./.client_recv.generated.rs");

/* Now come the manual definitions for the edge cases that don't fit into
 * packets.clj's code generation */

impl Statistics {
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let chunk_x = read_i32(r)?;
        let chunk_z = read_i32(r)?;
        let count = read_varint(r)?;
        let mut tmp: Vec<(u8, u8, u8, i32)> = Vec::with_capacity(count as usize);
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
        let window_id = read_u8(r)?;
        let window_type = read_String(r)?;
        let window_title = read_String(r)?;
        let number_of_slots = read_u8(r)?;
        let entity_id = match window_type.deref() {
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
            duration_playerid: duration_playerid,
            entity_id: entity_id,
            message: message,
        }))
    }
}

impl ScoreboardObjective {
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
            objective_type: objective_type,
        }))
    }
}

impl UpdateScore {
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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
    fn new<R: Read>(r: &mut R) -> io::Result<ClientboundPacket> {
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

