;;;;;; This file contains only the definitions for all the packets.
(ns packet-definitions)

(def packets
  {:serverbound
   {"Handshake"
    [{:name "Handshake"
      :id 0
      :fields [{:name "protocol_version" :type "i32" :read "varint" :getter "Get the client's protocol version"}
               {:name "server_address" :type "String" :getter "Get the hostname of the server the client connected to"}
               {:name "server_port" :type "u16" :getter "Get the port of the server the client connected to"}
               {:name "next_state" :type "i32" :read "varint" :getter "Get the next state"}]}]
    "Status"
    [{:name "StatusRequest"
      :id 0
      :automatic-serialize false
      :fields  []}
     {:name "StatusPing"
      :id 1
      :fields  [{:name "id" :type "u64" :getter "Get the id"}]}]
    "Login"
    [{:name "LoginStart"
      :id 0
      :fields [{:name "name" :type "String" :getter "Get the player's claimed username"}]}
     {:name "EncryptionResponse"
      :id 1
      :fields  [{:name "shared_secret" :type "Vec<u8>" :read "prefixed_bytearray" :getter "Get the (raw encrypted) shared secret"}
                {:name "verify_token" :type "Vec<u8>" :read "prefixed_bytearray" :getter "Get the (raw encrypted) verify token"}]}]
    "Play"
    [{:name "TeleportConfirm"
      :id 0
      :fields [{:name "id" :type "i32" :read "varint" :getter "Get the teleport id"}]}
     {:name "TabComplete"
      :id 1
      :automatic-serialize false
      :fields [{:name "text" :type "String" :getter "Get the text"}
               {:name "assume_command" :type "bool" :getter "Get whether the server should parse the text even if it doesn't start with a /"}
               {:name "looked_at_block" :type "Option<(i32, i32, i32)>" :getter "Get the position of the block being looked at (if any)"}]}
     {:name "ChatMessage"
      :id 2
      :fields [{:name "message" :type "String" :getter "Get the chat message (not json)"}]}
     {:name "ClientStatus"
      :id 3
      :fields [{:name "action" :type "i32" :read "varint" :getter "Get the action ID varint enum"}]}
     {:name "ClientSettings"
      :id 4
      :fields [{:name "locale" :type "String" :getter "Get the client's locale"}
               {:name "view_distance" :type "u8" :getter "Get the client's view distance in chunks"}
               {:name "chat_mode" :type "i32" :read "varint" :getter "Get the client's chat mode as varint enum"}
               {:name "chat_colors" :type "bool" :getter "Get whether the player has chat colors enabled"}
               {:name "displayed_skin_parts" :type "u8" :getter "Get the displayed skin parts as a raw bit mask"}
               {:name "main_hand" :type "i32" :read "varint" :getter "Get the player's main hand as a varint enum"}]}
     {:name "ConfirmTransaction"
      :id 5
      :fields [{:name "window_id" :type "u8" :getter "Get the window id"}
               {:name "id" :type "i16" :getter "Get the action number"}
               {:name "accepted" :type "bool" :getter "Get whether the action was accepted"}]}
     {:name "EnchantItem"
      :id 6
      :fields [{:name "window_id" :type "u8" :getter "Get the window id"}
               {:name "enchantment" :type "i8" :getter "Get the position of the chosen enchantment"}]}
     {:name "ClickWindow"
      :id 7
      :fields [{:name "window_id" :type "u8" :getter "Get the window id"}
               {:name "slot_id" :type "i16" :getter "Get the clicked slot number"}
               {:name "button" :type "i8" :getter "Get the button clicked byte enum"}
               {:name "id" :type "i16" :getter "Get the action number id"}
               {:name "mode" :type "i32" :read "varint" :getter "Get the action/mode"}
               {:name "slot" :type "Vec<u8>" :read "bytearray" :getter "Get the raw unprocessed slot data"}]}
     {:name "CloseWindow"
      :id 8
      :fields [{:name "window_id" :type "u8" :getter "Get the window id"}]}
     {:name "PluginMessage"
      :id 9
      :fields [{:name "channel" :type "String" :getter "Get the channel"}
               {:name "data" :type "Vec<u8>" :read "bytearray" :getter "Get the data"}]}
     {:name "UseEntity"
      :id 10
      :automatic-serialize false
      :fields [{:name "target" :type "i32" :read "varint" :getter "Get the target eid"}
               {:name "action" :type "i32" :read "varint" :getter "Get the action type varint enum"}
               {:name "location" :type "Option<(f32, f32, f32)>" :getter "Get the target location (if any)"}
               {:name "hand" :type "Option<i32>" :getter "Get the hand used as a varint enum (if any)"}]}
     {:name "KeepAlive"
      :id 11
      :fields [{:name "id" :type "i32" :read "varint" :getter "Get the keep alive ID"}]}
     {:name "PlayerPosition"
      :id 12
      :fields [{:name "x" :type "f64" :getter "Get the X coordinate"}
               {:name "y" :type "f64" :getter "Get the Y coordinate (feet)"}
               {:name "z" :type "f64" :getter "Get the Z coordinate"}
               {:name "on_ground" :type "bool" :getter "Get whether on the ground"}]}
     {:name "PlayerPositionAndLook"
      :id 13
      :fields [{:name "x" :type "f64" :getter "Get the X coordinate"}
               {:name "y" :type "f64" :getter "Get the Y coordinate"}
               {:name "z" :type "f64" :getter "Get the Z coordinate"}
               {:name "yaw" :type "f32" :getter "Get the yaw"}
               {:name "pitch" :type "f32" :getter "Get the pitch"}
               {:name "on_ground" :type "bool" :getter "Get whether on the ground"}]}
     {:name "PlayerLook"
      :id 14
      :fields [{:name "yaw" :type "f32" :getter "Get the yaw"}
               {:name "pitch" :type "f32" :getter "Get the pitch"}
               {:name "on_ground" :type "bool" :getter "Get whether on the ground"}]}
     {:name "Player"
      :id 15
      :fields [{:name "on_ground" :type "bool" :getter "Get whether on the ground"}]}
     {:name "VehicleMove"
      :id 16
      :fields [{:name "x" :type "f64" :getter "Get the (absolute) X coordinate"}
               {:name "y" :type "f64" :getter "Get the (absolute) Y coordinate"}
               {:name "z" :type "f64" :getter "Get the (absolute) Z coordinate"}
               {:name "yaw" :type "f32" :getter "Get the (absolute) yaw"}
               {:name "pitch" :type "f32" :getter "Get the (absolute) pitch"}]}
     {:name "SteerBoat"
      :id 17
      :fields [{:name "right" :type "bool" :getter "Get whether the right paddle is turning"}
               {:name "left" :type "bool" :getter "Get whether the left paddle is turning"}]}
     {:name "PlayerAbilities"
      :id 18
      :fields [{:name "flags" :type "u8" :getter "Get the raw player abilities bit mask"}
               {:name "flying_speed" :type "f32" :getter "Get the player's flying speed"}
               {:name "walking_speed" :type "f32" :getter "Get the player's walking speed"}]}
     {:name "PlayerDigging"
      :id 19
      :fields [{:name "status" :type "i32" :read "varint" :getter "Get the status as a raw varint enum"}
               {:name "location" :type "(i32, i32, i32)" :read "position" :getter "Get the location of the block"}
               {:name "face" :type "u8" :getter "Get the face of the block being hit as a raw byte enum"}]}
{:name "EntityAction"
 :id 20
 :fields [{:name "entity_id" :type "i32" :read "varint" :getter "Get the player's eid"}
          {:name "action" :type "i32" :read "varint" :getter "Get the action as a raw varint enum"}
          {:name "jump_boost" :type "i32" :read "varint" :getter "Get the jump boost, used if the player is riding a horse"}]}
{:name "SteerVehicle"
 :id 21
 :fields [{:name "sideways" :type "f32" :getter "Get the sideways movement, positiev is to the left of the player"}
          {:name "forward" :type "f32" :getter "Get the forward movement"}
          {:name "flags" :type "u8" :getter "Get the raw flags byte enum"}]}
{:name "ResourcePackStatus"
 :id 22
 :fields [{:name "result" :type "i32" :read "varint" :getter "Get the result as a raw varint enum"}]}
{:name "HeldItemChange"
 :id 23
 :fields [{:name "slot" :type "i16" :getter "Get the slot the player has selected"}]}
{:name "CreativeInventoryAction"
 :id 24
 :fields [{:name "slot_id" :type "i16" :getter "Get the inventory slot number"}
          {:name "slot" :type "Vec<u8>" :read "bytearray" :getter "Get the raw unprocessed slot data"}]}
{:name "UpdateSign"
 :id 25
 :fields [{:name "location" :type "(i32, i32, i32)" :read "position" :getter "Get the block coordinates"}
          {:name "line1" :type "String" :getter "Get line 1"}
          {:name "line2" :type "String" :getter "Get line 2"}
          {:name "line3" :type "String" :getter "Get line 3"}
          {:name "line4" :type "String" :getter "Get line 4"}]}
{:name "Animation"
 :id 26
 :fields [{:name "hand" :type "i32" :read "varint" :getter "Get which arm was used as a raw varint enum"}]}
{:name "Spectate"
 :id 27
 :fields [{:name "target" :type "u128" :read "uuid" :getter "Get the uuid of the selected target"}]}
{:name "PlayerBlockPlacement"
 :id 28
 :fields [{:name "location" :type "(i32, i32, i32)" :read "position" :getter "Get the location of the placed block"}
          {:name "face" :type "i32" :read "varint" :getter "Get the face of the block as a raw varint enum"}
          {:name "hand" :type "i32" :read "varint" :getter "Get the hand from which the block was placed as a raw varint enum"}
          {:name "x" :type "f32" :getter "Get the X position of the crosshair on the block"}
          {:name "y" :type "f32" :getter "Get the Y position of the crosshair on the block"}
          {:name "z" :type "f32" :getter "Get the Z position of the crosshair on the block"}]}
{:name "UseItem"
 :id 29
 :fields [{:name "hand" :type "i32" :read "varint" :getter "Get which hand contained the used item as a raw varint enum"}]}]}
:clientbound
{"Handshake" []
 "Status"
 [{:name "StatusResponse"
   :id 0
   :fields [{:name "json" :type "String" :getter "Get the raw json response"}]}
  {:name "StatusPong"
   :id 1
   :fields [{:name "id" :type "u64" :getter "Get the id of the ping/pong"}]}]
 "Login"
 [{:name "LoginDisconnect"
   :id 0
   :fields [{:name "raw_chat" :type "String" :getter "Get the raw chat json"}]}
  {:name "EncryptionRequest"
   :id 1
   :fields [{:name "server_id" :type "String" :getter "Get the server id"}
            {:name "public_key" :type "Vec<u8>" :getter "Get the public key" :read "prefixed_bytearray"}
            {:name "verify_token" :type "Vec<u8>" :getter "Get the verify token" :read "prefixed_bytearray"}]}
  {:name "LoginSuccess"
   :id 2
   :fields [{:name "uuid" :type "u128" :getter "Get the player's uuid" :read "uuid_str"}
            {:name "username" :type "String" :getter "Get the player's name"}]}
  {:name "SetCompression"
   :id 3
   :fields [{:name "threshold" :type "i32" :getter "Get the compression threshold" :read "varint"}]}]
 "Play"
 [{:name "SpawnObject"
   :id 0
   :fields [{:name "entity_id" :type "i32" :getter "Get the ID of the created object" :read "varint"}
            {:name "object_uuid" :type "u128" :getter "Get the UUID of the created object" :read "uuid"}
            {:name "object_type" :type "u8"}
            {:name "x" :type "f64" :getter "Get the X coordinate"}
            {:name "y" :type "f64" :getter "Get the Y coordinate"}
            {:name "z" :type "f64" :getter "Get the Z coordinate"}
            {:name "pitch" :type "i8" :getter "Get the pitch"}
            {:name "yaw" :type "i8" :getter "Get the yaw"}
            {:name "data" :type "i32"}
            {:name "velocity_x" :type "i16" :getter "Get the X velocity"}
            {:name "velocity_y" :type "i16" :getter "Get the Y velocity"}
            {:name "velocity_z" :type "i16" :getter "Get the Z velocity"}]}
  {:name "SpawnExperienceOrb"
   :id 1
   :fields [{:name "entity_id" :type "i32" :getter "Get the ID of the orb" :read "varint"}
            {:name "x" :type "f64" :getter "Get the X coordinate"}
            {:name "y" :type "f64" :getter "Get the Y coordinate"}
            {:name "z" :type "f64" :getter "Get the Z coordinate"}
            {:name "count" :type "i16" :getter "Get the amount of experience this orb will reward"}]}
  {:name "SpawnGlobalEntity"
   :id 2
   :fields [{:name "entity_id" :type "i32" :getter "Get the ID of the entity" :read "varint"}
            {:name "entity_type" :type "u8"}
            {:name "x" :type "f64" :getter "Get the X coordinate"}
            {:name "y" :type "f64" :getter "Get the Y coordinate"}
            {:name "z" :type "f64" :getter "Get the Z coordinate"}]}
  {:name "SpawnMob"
   :id 3
   :fields [{:name "entity_id" :type "i32" :getter "Get the ID of the mob" :read "varint"}
            {:name "uuid" :type "u128" :getter "Get the UUID of the mob" :read "uuid"}
            {:name "mob_type" :type "i32" :getter "Get the type ID of the mob" :read "varint"}
            {:name "x" :type "f64" :getter "Get the X coordinate"}
            {:name "y" :type "f64" :getter "Get the Y coordinate"}
            {:name "z" :type "f64" :getter "Get the Z coordinate"}
            {:name "yaw" :type "i8" :getter "Get the yaw of the mob"}
            {:name "pitch" :type "i8" :getter "Get the pitch of the mob"}
            {:name "head_pitch" :type "i8" :getter "Get the pitch of the head of the mob"}
            {:name "velocity_x" :type "i16" :getter "Get the X velocity"}
            {:name "velocity_y" :type "i16" :getter "Get the Y velocity"}
            {:name "velocity_z" :type "i16" :getter "Get the Z velocity"}
            {:name "metadata" :type "Vec<u8>" :getter "Get the raw metadata field. This library does not currently attempt to parse this field" :read "bytearray_to_end"}]}
  {:name "SpawnPainting"
   :id 4
   :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID of the painting" :read "varint"}
            {:name "uuid" :type "u128" :getter "Get the UUID of the painting" :read "uuid"}
            {:name "title" :type "String" :getter "Get the title of the painting"}
            {:name "center_location" :type "(i32, i32, i32)" :read "position"}
            {:name "direction" :type "u8" :getter "The direction in which the painting faces"}]}
  {:name "SpawnPlayer"
   :id 5
   :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID of the player" :read "varint"}
            {:name "uuid" :type "u128" :getter "Get the UUID of the player" :read "uuid"}
            {:name "x" :type "f64" :getter "Get the X coordinate"}
            {:name "y" :type "f64" :getter "Get the Y coordinate"}
            {:name "z" :type "f64" :getter "Get the Z coordinate"}
            {:name "yaw" :type "i8" :getter "Get the yaw"}
            {:name "pitch" :type "i8" :getter "Get the pitch"}
            {:name "metadata" :type "Vec<u8>" :getter "Get the raw metadata in the packet, this library does not attempt to parse this field" :read "bytearray_to_end"}]}
  {:name "ClientboundAnimation"
   :id 6
   :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID doing the animation" :read "varint"}
            {:name "animation" :type "u8" :getter "Get the byte ID for what kind of animation it is"}]}
  {:name "Statistics"
   :id 7
   :automatic-serialize false
   :fields  [{:name "values" :type "BTreeMap<String, i32>" :getter "Get the statistics, with the key being the name of the statistic and the value being the value."}]}
  {:name "BlockBreakAnimation"
   :id 8
   :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID doing the animation" :read "varint"}]}
  {:name "UpdateBlockEntity"
   :id 9
   :fields [{:name "position" :type "(i32, i32, i32)" :getter "Get the (x, y, z) position" :read "position"}
            {:name "action" :type "u8" :getter "Get the action ID being performed"}
            {:name "nbt" :type "Vec<u8>" :getter "Get the raw NBT bytes" :read "bytearray_to_end"}]}
  {:name "BlockAction"
   :id 10
   :fields [{:name "position" :type "(i32, i32, i32)" :getter "Get the (x, y, z) position" :read "position"}
            {:name "action_id" :type "u8" :getter "Get the action ID"}
            {:name "action_param" :type "u8" :getter "Get the action parameter"}
            {:name "block_type" :type "i32" :getter "Get the block type" :read "varint"}]}
  {:name "BlockChange"
   :id 11
   :fields [{:name "position" :type "(i32, i32, i32)" :getter "Get the (x, y, z) position" :read "position"}
            {:name "new_block" :type "i32" :getter "Get the new block state ID for the block" :read "varint"}]}
  {:name "BossBar"
   :id 12
   :fields [{:name "data" :type "Vec<u8>" :getter "Get the raw data from this packet. Parsing this is very dependent on the specific client, and doing so would be out of scope for this library, therefore parsing this packet is left to the user of the library." :read "bytearray_to_end"}]}
  {:name "ServerDifficulty"
   :id 13
   :fields [{:name "difficulty" :type "u8" :getter "Get the difficulty"}]}
  {:name "ClientboundTabComplete"
   :id 14
   :automatic-serialize false
   :fields [{:name "matches" :type "Vec<String>" :getter "Get the matches"}]}
  {:name "ChatMessage"
   :id 15
   :fields [{:name "chat" :type "String" :getter "Get the raw JSON data of the chat message"}
            {:name "position" :type "u8" :getter "Get the position of the chat message"}]}
  {:name "MultiBlockChange"
   :id 16
   :automatic-serialize false
   :fields [{:name "chunk_x" :type "i32" :getter "Get the chunk X coordinate"}
            {:name "chunk_z" :type "i32" :getter "Get the chunk Z coordinate"}
            {:name "changes" :type "Vec<(u8, u8, u8, i32)>" :getter "Get the changes as a vector, in the form of Vec<(x, y, z, new_block_state)>, where the x, y, z are relative to the chunk."}]}
{:name "ClientboundConfirmTransaction"
 :id 17
 :fields [{:name "window_id" :type "u8" :getter "Get the window ID"}
          {:name "action_id" :type "i16" :getter "Get the action ID (nonce)"}
          {:name "accepted" :type "bool"}]}
{:name "ClientboundCloseWindow"
 :id 18
 :fields [{:name "window_id" :type "u8" :getter "Get the window ID"}]}
{:name "OpenWindow"
 :id 19
 :automatic-serialize false
 :fields [{:name "window_id" :type "u8" :getter "Get the window ID"}
          {:name "window_type" :type "String" :getter "Get the window type"}
          {:name "window_title" :type "String" :getter "Get the raw JSON of the window title"}
          {:name "number_of_slots" :type "u8" :getter "Get the number of slots in the window"}
          {:name "entity_id" :type "Option<i32>" :getter "Get the entity ID if the window type is `EntityHorse`, else `None`"}]}
{:name "WindowItems"
 :id 20
 :fields [{:name "window_id" :type "u8" :getter "Get the window ID"}
          {:name "slots" :type "Vec<u8>" :getter "Get the remaining slot data, that is the last two fields described at http://wiki.vg/Protocol#Window_Items" :read "bytearray_to_end"}]}
{:name "WindowProperty"
 :id 21
 :fields [{:name "window_id" :type "u8" :getter "Get the window ID"}
          {:name "property" :type "i16" :getter "Get the property being updated"}
          {:name "new_value" :type "i16" :getter "Get the new value of the propery"}]}
{:name "SetSlot"
 :id 22
 :fields [{:name "window_id" :type "u8" :getter "Get the window ID"}
          {:name "slot_id" :type "i16" :getter "Get the ID of the slot to be updated"}
          {:name "slot_data" :type "Vec<u8>" :getter "Get the slot data of the packet in raw, unprocessed format" :read "bytearray_to_end"}]}
{:name "SetCooldown"
 :id 23
 :fields [{:name "item_id" :type "i32" :getter "Get the ID of the item the cool applied to" :read "varint"}
          {:name "cooldown" :type "i32" :getter "Get the cooldown on the item specified in ticks" :read "varint"}]}
{:name "ClientboundPluginMessage"
 :id 24
 :fields [{:name "channel" :type "String" :getter "Get the channel"}
          {:name "data" :type "Vec<u8>" :getter "Get the raw data" :read "bytearray_to_end"}]}
{:name "NamedSoundEffect"
 :id 25
 :fields [{:name "sound_name" :type "String" :getter "Get the name of the sound"}
          {:name "sound_category" :type "i32" :getter "Get the category of the sound" :read "varint"}
          {:name "x" :type "i32" :getter "Get the X coordinate multiplied by 8"}
          {:name "y" :type "i32" :getter "Get the Y coordinate multiplied by 8"}
          {:name "z" :type "i32" :getter "Get the Z coordinate multiplied by 8"}
          {:name "volume" :type "f32" :getter "Get the volume"}
          {:name "pitch" :type "f32" :getter "Get the pitch"}]}
{:name "PlayDisconnect"
 :id 26
 :fields [{:name "reason" :type "String" :getter "Get the reason in raw json format"}]}
{:name "EntityStatus"
 :id 27
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID"}
          {:name "status" :type "u8" :getter "Get the status of the entity"}]}
{:name "Explosion"
 :id 28
 :automatic-serialize false
 :fields [{:name "x" :type "f32" :getter "Get the X coordinate"}
          {:name "y" :type "f32" :getter "Get the Y coordinate"}
          {:name "z" :type "f32" :getter "Get the Z coordinate"}
          {:name "radius" :type "f32" :getter "Get the radius"}
          {:name "affected_blocks" :type "Vec<(i8, i8, i8)>" :getter "A list of the blocks affected by the explosion, given in Vec<(x, y, z)> where x, y, z are offsets from the explosions center"}
          {:name "motion_x" :type "f32" :getter "Get the motion in the X direction that the player suffers as a result of the explosion"}
          {:name "motion_y" :type "f32" :getter "Get the motion in the Y direction that the player suffers as a result of the explosion"}
          {:name "motion_z" :type "f32" :getter "Get the motion in the Z direction that the player suffers as a result of the explosion"}]}
{:name "UnloadChunk"
 :id 29
 :fields [{:name "chunk_x" :type "i32" :getter "Get the chunk's X coordinate"}
          {:name "chunk_z" :type "i32" :getter "Get the chunk's Z coordinate"}]}
{:name "ChangeGameState"
 :id 30
 :fields [{:name "action" :type "u8" :getter "Get the reason for the change as a byte"}
          {:name "value" :type "f32" :getter "Get the value, its meaning depends on the action"}]}
{:name "KeepAlive"
 :id 31
 :fields [{:name "id" :type "i32" :getter "Get the ID of the keep alive packet" :read "varint"}]}
{:name "ChunkData"
 :id 32
 :fields [{:name "data" :type "Vec<u8>" :getter "Get all the data contained in this packet. Currently it's decided that this library shouldn't try to interpret complex data structures that are likely dependent on the specific implementation, so the parsing of this packet is left up to the client." :read "bytearray_to_end"}]}
{:name "Effect"
 :id 33
 :fields [{:name "effect_id" :type "i32" :getter "Get the ID of the effect"}
          {:name "location" :type "(i32, i32, i32)" :read "position"}
          {:name "data" :type "i32" :getter "Get the data for this effect"}
          {:name "disable_relative_volume" :type "bool" :getter "Get whether to disable relative volume"}]}
{:name "Particle"
 :id 34
 :automatic-serialize false
 :fields [{:name "particle_id" :type "i32" :getter "Get the particle ID"}
          {:name "use_long_distance" :type "bool" :getter "Get whether to use long distance (65536) instead of short (256)"}
          {:name "x" :type "f32" :getter "Get the X coordinate"}
          {:name "y" :type "f32" :getter "Get the Y coordinate"}
          {:name "z" :type "f32" :getter "Get the Z coordinate"}
          {:name "offset_x" :type "f32" :getter "Get the X offset"}
          {:name "offset_y" :type "f32" :getter "Get the Y offset"}
          {:name "offset_z" :type "f32" :getter "Get the Z offset"}
          {:name "particle_data" :type "f32" :getter "Get the particle data for each particle"}
          {:name "count" :type "i32" :getter "Get the amount of particles to create"}
          {:name "id" :type "Option<i32>" :getter "Get the id for iconcrack, blockcrack and blockdust"}
          {:name "crack_data" :type "Option<i32>" :getter "Get the data for iconcrack and blockcrack"}]}
{:name "JoinGame"
 :id 35
 :fields [{:name "entity_id" :type "i32" :getter "Get the player's entity ID"}
          {:name "gamemode" :type "u8" :getter "Get the player's gamemode"}
          {:name "dimension" :type "i32" :getter "Get the dimension the player is in. Not the specific world (in case of servers with multiworld), but the kind of world"}
          {:name "difficulty" :type "u8" :getter "Get the difficulty"}
          {:name "max_players" :type "u8"}; Apparently this field is no longer used but for some reason hasn't been removed yet
          {:name "level_type" :type "String" :getter "Get the level type of the world the player joined in"}
          {:name "reduced_debug" :type "bool" :getter "Get whether to show reduced debug info"}]}
{:name "Map"
 :id 36
 :fields [{:name "data" :type "Vec<u8>" :getter "Get the raw data from this packet. Parsing this is very dependent on the specific client, and doing so would be out of scope for this library, therefore parsing this packet is left to the user of the library." :read "bytearray_to_end"}]}
{:name "EntityRelativeMove"
 :id 37
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "x" :type "i16" :getter "Get the relative distanced moved in the X direction times 128"}
          {:name "y" :type "i16" :getter "Get the relative distanced moved in the Y direction times 128"}
          {:name "z" :type "i16" :getter "Get the relative distanced moved in the Z direction times 128"}
          {:name "on_ground" :type "bool" :getter "Get whether the entity is on the ground"}]}
{:name "EntityLookRelativeMove"
 :id 38
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "x" :type "i16" :getter "Get the relative distanced moved in the X direction times 128"}
          {:name "y" :type "i16" :getter "Get the relative distanced moved in the Y direction times 128"}
          {:name "z" :type "i16" :getter "Get the relative distanced moved in the Z direction times 128"}
          {:name "yaw" :type "i8" :getter "Get the yaw"}
          {:name "pitch" :type "i8" :getter "Get the pitch"}
          {:name "on_ground" :type "bool" :getter "Get whether the entity is on the grouns"}]}
{:name "EntityLook"
 :id 39
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "yaw" :type "i8" :getter "Get the (absolute) angle"}
          {:name "pitch" :type "i8" :getter "Get the (absolute) pitch"}
          {:name "on_ground" :type "bool" :getter "Get whether on the ground"}]}
{:name "Entity"
 :id 40
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}]}
{:name "ClientboundVehicleMove"
 :id 41
 :fields [{:name "x" :type "f64" :getter "Get the X coordinate"}
          {:name "y" :type "f64" :getter "Get the Y coordinate"}
          {:name "z" :type "f64" :getter "Get the Z coordinate"}
          {:name "yaw" :type "i8" :getter "Get the (absolute) angle"}
          {:name "pitch" :type "i8" :getter "Get the (absolute) pitch"}]}
{:name "OpenSignEditor"
 :id 42
 :fields [{:name "position" :type "(i32, i32, i32)" :getter "Get the (x, y, z) position" :read "position"}]}
{:name "PlayerAbilities"
 :id 43
 :fields [{:name "flags" :type "u8"}
          {:name "flying_speed" :type "f32" :getter "Get the player's allowed flying speed"}
          {:name "fov" :type "f32" :getter "Get the player's field of view modifier"}]}
{:name "CombatEvent"
 :id 44
 :automatic-serialize false
 :fields [{:name "event" :type "i32"}
          {:name "duration_playerid" :type "Option<i32>"}
          {:name "entity_id" :type "Option<i32>" :getter "Get the entity ID if packet action is 'end combat' or 'entity dead'"}
          {:name "message" :type "Option<String>"}]}
{:name "PlayerListItem"
 :id 45
 :fields [{:name "data" :type "Vec<u8>" :getter "Get the raw data from this packet. This library does not attempt to parse this packet." :read "bytearray_to_end"}]}
{:name "PlayerPositionAndLook"
 :id 46
 :fields [{:name "x" :type "f64" :getter "Get the x coordinate"}
          {:name "y" :type "f64" :getter "Get the y coordinate"}
          {:name "z" :type "f64" :getter "Get the z coordinate"}
          {:name "yaw" :type "f32" :getter "Get the yaw"}
          {:name "pitch" :type "f32" :getter "Get the pitch"}
          {:name "flags" :type "u8" :getter "Get the raw flags bitmask"}
          {:name "teleport_id" :type "i32" :getter "Get the teleport ID to be used in the serverbound TeleportConfirm packet." :read "varint"}]}
{:name "UseBed"
 :id 47
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID of the player sleeping" :read "varint"}
          {:name "position" :type "(i32, i32, i32)" :getter "Get the (x, y, z) position" :read "position"}]}
{:name "DestroyEntities"
 :id 48
 :fields [{:name "entity_ids" :type "Vec<i32>" :getter "Get the list of entity IDs that have been destroyed" :read "prefixed_varintarray"}]}
{:name "RemoveEntityEffect"
 :id 49
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID to remove the effect from" :read "varint"}
          {:name "effect_id" :type "u8"}]}
{:name "ResourcePackSend"
 :id 50
 :fields [{:name "url" :type "String" :getter "Get the URL to the resource pack"}
          {:name "hash" :type "String" :getter "Get the expected SHA-1 hash of the resource pack"}]}
{:name "Respawn"
 :id 51
 :fields  [{:name "dimension" :type "i32" :getter "Get the integer value for the dimension the player is spawning in"}
           {:name "difficulty" :type "u8" :getter "Get the integer value for the difficulty"}
           {:name "gamemode" :type "u8" :getter "Get the integer value for the gamemode"}
           {:name "level_type" :type "String" :getter "Get the level type"}]}
{:name "EntityHeadLook"
 :id 52
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "head_yaw" :type "i8" :getter "Get the new head yaw"}]}
{:name "WorldBorder"
 :id 53
 :fields [{:name "data" :type "Vec<u8>" :getter "Get this packet's raw data. This library does not attempt to parse this packet" :read "bytearray_to_end"}]}
{:name "Camera"
 :id 54
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID to set the camera to" :read "varint"}]}
{:name "ClientboundHeldItemChange"
 :id 55
 :fields [{:name "slot" :type "u8" :getter "Get the slot number which the player has selected"}]}
{:name "DisplayScoreboard"
 :id 56
 :fields [{:name "position" :type "u8" :getter "Get the raw integer representing the scoreboard's position"}
          {:name "name" :type "String" :getter "Get the name of the scoreboard"}]}
{:name "EntityMetadata"
 :id 57
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID of the entity which metadata is being updated" :read "varint"}
          {:name "metadata" :type "Vec<u8>" :getter "Get the raw data for the metadata. This library does not attempt to parse the metadata." :read "bytearray_to_end"}]}
{:name "AttachEntity"
 :id 58
 :fields [{:name "attached_entity_id" :type "i32" :getter "Get the entity ID of the entity that has been attached"}
          {:name "holding_entity_id" :type "i32" :getter "Get the entity ID of the entity that has been attached to"}]}
{:name "EntityVelocity"
 :id 59
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "x_velocity" :type "i16" :getter "Get the X velocity"}
          {:name "y_velocity" :type "i16" :getter "Get the Y velocity"}
          {:name "z_velocity" :type "i16" :getter "Get the Z velocity"}]}
{:name "EntityEquipment"
 :id 60
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "slot_enum" :type "i32" :getter "Get the raw slot identification number" :read "varint"}
          {:name "slot_data" :type "Vec<u8>" :getter "Get the raw slot data. This library does not attempt to parse it" :read "bytearray_to_end"}]}
{:name "SetExperience"
 :id 61
 :fields [{:name "experience" :type "f32" :getter "Get how filled up the experience bar is"}
          {:name "level" :type "i32" :getter "Get the new level" :read "varint"}
          {:name "total_experience" :type "i32" :getter "Get the total experience" :read "varint"}]}
{:name "UpdateHealth"
 :id 62
 :fields [{:name "health" :type "f32" :getter "Get how much health the player has"}
          {:name "food" :type "i32" :getter "Get how much food the player has" :read "varint"}
          {:name "saturation" :type "f32" :getter "Get the saturation level"}]}
{:name "ScoreboardObjective"
 :id 63
 :automatic-serialize false
 :fields [{:name "name" :type "String" :getter "Get the name for the object"}
          {:name "mode" :type "u8" :getter "Get the raw mode enum integer"}
          {:name "value" :type "Option<String>" :getter "Get the text to be displayed"}
          {:name "objective_type" :type "Option<String>" :getter "Get the raw string representing the type (`integer` or `hearts`)"}]}
{:name "SetPassengers"
 :id 64
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "passengers" :type "Vec<i32>" :getter "Get the vec of all the passengers" :read "prefixed_varintarray"}]}
{:name "Teams"
 :id 65
 :fields [{:name "data" :type "Vec<u8>" :getter "Get the raw data, this library does not attempt to parse this packet." :read "bytearray_to_end"}]}
{:name "UpdateScore"
 :id 66
 :automatic-serialize false
 :fields [{:name "name" :type "String" :getter "Get the name of the score to be updated"}
          {:name "action" :type "u8" :getter "Get the action being performed"}
          {:name "objective_name" :type "String" :getter "Get the name of the objective the score belongs to"}
          {:name "value" :type "Option<i32>" :getter "Get the score to be displayed if this packet is updating a score, else `None`"}]}
{:name "SpawnPosition"
 :id 67
 :fields [{:name "position" :type "(i32, i32, i32)" :getter "Get the position" :read "position"}]}
{:name "TimeUpdate"
 :id 68
 :fields [{:name "world_age" :type "i64" :getter "Get the world's age in ticks"}
          {:name "time_of_day" :type "i64" :getter "Get the current time in ticks (0 is sunrise, 6000 is noon, ...)"}]}
{:name "Title"
 :id 69
 :automatic-serialize false
 :fields [{:name "action" :type "i32" :getter "Get the raw action enum integer"}
          {:name "text" :type "Option<String>" :getter "Get the title/subtitle/action bar text if action is set title/subtitle/action bar in raw json"}
          {:name "times" :type "Option<(i32, i32, i32)>" :getter "If action is 'set times and display' get `Some((fade_in, stay, fade_out))` else get `None`"}]}
{:name "SoundEffect"
 :id 70
 :fields [{:name "sound_id" :type "i32" :getter "Get the raw sound effect ID. Note that the meaning of this is liable to change between MC releases." :read "varint"}
          {:name "sound_category" :type "i32" :getter "Get the raw sound category ID." :read "varint"}
          {:name "x" :type "i32" :getter "Get the X effect multiplied by 8"}
          {:name "y" :type "i32" :getter "Get the Y effect multiplied by 8"}
          {:name "z" :type "i32" :getter "Get the Z effect multiplied by 8"}
          {:name "volume" :type "f32" :getter "Get the volume where 1.0 is 100%"}
          {:name "pitch" :type "f32" :getter "Get the pitch"}]}
{:name "PlayerListHeaderFooter"
 :id 71
 :fields [{:name "header" :type "String" :getter "Get the raw json data for the header"}
          {:name "footer" :type "String" :getter "Get the raw json data for the footer"}]}
{:name "CollectItem"
 :id 72
 :fields [{:name "collected_entity_id" :type "i32" :getter "Get the entity ID of the collected item" :read "varint"}
          {:name "collector_entity_id" :type "i32" :getter "Get the entity ID of the person picking up the item" :read "varint"}
          {:name "item_count" :type "i32" :getter "Get how many items were picked up" :read "varint"}]}
{:name "EntityTeleport"
 :id 73
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID of the entity teleported" :read "varint"}
          {:name "x" :type "f64" :getter "Get the X coordinate the entity moved to"}
          {:name "y" :type "f64" :getter "Get the Y coordinate the entity moved to"}
          {:name "z" :type "f64" :getter "Get the Z coordinate the entity moved to"}
          {:name "yaw" :type "i8" :getter "Get the (absolute) yaw"}
          {:name "pitch" :type "i8" :getter "Get the (absolute) pitch"}
          {:name "on_ground" :type "bool" :getter "Get whether the entity is now on the ground"}]}
{:name "EntityProperties"
 :id 74
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "data" :type "Vec<u8>" :getter "Get the raw data for this packet. This library does not attempt to parse the packet" :read "bytearray_to_end"}]}
{:name "EntityEffect"
 :id 75
 :fields [{:name "entity_id" :type "i32" :getter "Get the entity ID" :read "varint"}
          {:name "effect_id" :type "u8" :getter "Get the raw effect ID integer enum"}
          {:name "amplifier" :type "i8" :getter "Get the amplifier = effect level - 1"}
          {:name "duration" :type "i32" :getter "Get the duration of the effect in seconds" :read "varint"}
          {:name "flags" :type "u8" :getter "Get the raw flags byte"}]}
]}})
