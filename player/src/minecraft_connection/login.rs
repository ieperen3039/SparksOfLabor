use crate::voxels::world::World;
use std::{
    collections::{BTreeMap, HashMap},
    io,
    net::TcpStream,
};

use super::{network, player_character::PlayerCharacter};
use minecraft_protocol::{
    components as mc_components,
    nbt::NbtTag,
    packets::{
        self as mc_packets, play_clientbound::ClientboundPacket as PlayClientbound,
        play_serverbound::ServerboundPacket as PlayServerbound, status, Array,
    },
    MinecraftPacketPart,
};
use sol_voxel_lib::vector_alias::Position;

#[derive(Debug)]
pub enum CommunicationError {
    ConnectionClosed,
    UnexpectedPackage {
        expected: String,
        received: String,
    },
    UnexpectedState {
        from: mc_packets::ConnectionState,
        to: mc_packets::ConnectionState,
    },
    SerializationError(String),
    DeserializationError(String),
    IoError(io::Error),
    InternalError(String),
}

impl CommunicationError {
    pub fn wrong_package<'a, PacketType>(expected: &str, received: PacketType) -> CommunicationError
    where
        PacketType: MinecraftPacketPart<'a> + std::fmt::Debug,
    {
        Self::UnexpectedPackage {
            expected: String::from(expected),
            received: format!("{received:?}"),
        }
    }
}

impl From<io::Error> for CommunicationError {
    fn from(value: io::Error) -> Self {
        Self::IoError(value)
    }
}

pub struct PlayerConnectionData {
    username: String,
    uuid: u128,
}

pub fn login(stream: &mut TcpStream) -> Result<PlayerConnectionData, CommunicationError> {
    // Receive login start
    let mut buffer = Vec::new();
    let packet: mc_packets::login::ServerboundPacket =
        network::receive_packet(stream, &mut buffer)?;

    let mc_packets::login::ServerboundPacket::LoginStart {
        username,
        player_uuid,
    } = packet
    else {
        return Err(CommunicationError::wrong_package("LoginStart", packet));
    };
    println!("LoginStart: {username}");

    // copy username out of the buffer to drop the buffer early
    // We must own the username anyway if we later want to move the username
    let username = username.to_owned();
    drop(buffer);

    // OPTIONAL encryption

    // OPTIONAL compression

    // Send login success
    let login_success = mc_packets::login::ClientboundPacket::LoginSuccess {
        uuid: player_uuid,
        username: &username,
        properties: Array::default(),
    };

    network::send_packet(stream, login_success)?;
    println!("LoginSuccess sent");

    // Receive login acknowledged
    let mut buffer = Vec::new();
    let packet: mc_packets::login::ServerboundPacket =
        network::receive_packet(stream, &mut buffer)?;
    let mc_packets::login::ServerboundPacket::LoginAcknowledged = packet else {
        return Err(CommunicationError::wrong_package(
            "LoginAcknowledged",
            packet,
        ));
    };
    println!("LoginAcknowledged received");

    // Ignore encryption response if any
    let mut buffer = Vec::new();
    let packet: mc_packets::login::ServerboundPacket =
        network::receive_packet(stream, &mut buffer)?;
    if let mc_packets::login::ServerboundPacket::EncryptionResponse { .. } = packet {
        println!("EncryptionResponse received and ignored");
    }

    Ok(PlayerConnectionData {
        username,
        uuid: player_uuid,
    })
}

pub struct PlayerInfo {
    pub socket: TcpStream,
    pub username: String,
    pub uuid: u128,
    pub locale: String,
    pub render_distance: usize,
    pub chat_mode: mc_components::chat::ChatMode,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: mc_components::players::MainHand,
    pub enable_text_filtering: bool,
    pub allow_server_listing: bool,
    pub chunks_per_tick: f32,
}

pub fn initialize_client(
    mut socket: TcpStream,
    logged_in_player_info: PlayerConnectionData,
    character: &PlayerCharacter,
) -> Result<PlayerInfo, CommunicationError> {
    let stream = &mut socket;

    // Receive client information
    let mut buffer = Vec::new();
    let packet = network::receive_packet(stream, &mut buffer)?;
    let mc_packets::config::ServerboundPacket::ClientInformations {
        locale,
        render_distance,
        chat_mode,
        chat_colors,
        displayed_skin_parts,
        main_hand,
        enable_text_filtering,
        allow_server_listing,
    } = packet
    else {
        return Err(CommunicationError::wrong_package(
            "ClientInformations",
            packet,
        ));
    };
    println!("ClientInformation received");

    // Send server agent
    let server_agent = mc_packets::config::ClientboundPacket::PluginMessage {
        channel: "minecraft:brand",
        data: mc_packets::RawBytes {
            data: &[6, 83, 112, 105, 103, 111, 116],
        },
    };
    network::send_packet(stream, server_agent)?;
    println!("PluginMessage sent");

    // Send feature flags
    let feature_flags = mc_packets::config::ClientboundPacket::FeatureFlags {
        features: Array::from(vec!["minecraft:vanilla"]),
    };
    network::send_packet(stream, feature_flags)?;
    println!("FeatureFlags sent");

    // Send registry data
    // TODO this can be used to make our own block set
    network::send_packet_raw(stream, include_bytes!("raw/registry_codec.mc_packet"))?;
    println!("RegistryData sent");

    // Update tags
    let update_tags = mc_packets::config::ClientboundPacket::UpdateTags {
        tags: mc_packets::Map::default(),
    };
    network::send_packet(stream, update_tags)?;
    println!("UpdateTags sent");

    // Send finish configuration
    let finish_configuration = mc_packets::config::ClientboundPacket::FinishConfiguration;
    network::send_packet(stream, finish_configuration)?;
    println!("FinishConfiguration sent");

    // Receive finish configuration
    let mut buffer = Vec::new();
    let packet: mc_packets::config::ServerboundPacket =
        network::receive_packet(stream, &mut buffer)?;
    let mc_packets::config::ServerboundPacket::FinishConfiguration = packet else {
        return Err(CommunicationError::wrong_package(
            "FinishConfiguration",
            packet,
        ));
    };
    println!("FinishConfiguration received");

    // Send join game
    let player_id: usize = 0; // TODO how does player_id work with other players?

    let join_game = PlayClientbound::JoinGame {
        player_id: player_id as i32,
        is_hardcore: false,
        dimensions_names: Array::from(vec!["minecraft:overworld"]),
        max_players: mc_packets::VarInt::from(1000),
        render_distance: mc_packets::VarInt::from(12),
        simulation_distance: mc_packets::VarInt::from(8),
        reduced_debug_info: false,
        enable_respawn_screen: true,
        do_limited_crafting: false,
        dimension_type: "minecraft:overworld",
        dimension_name: "minecraft:overworld",
        hashed_seed: 42,
        gamemode: mc_components::gamemode::Gamemode::Creative,
        previous_gamemode: mc_components::gamemode::PreviousGamemode::Creative,
        is_debug: false,
        is_flat: true,
        death_location: None,
        portal_cooldown: mc_packets::VarInt::from(0),
    };
    network::send_packet(stream, join_game)?;
    println!("JoinGame sent");

    // Set difficulty
    let change_difficulty = PlayClientbound::ChangeDifficulty {
        difficulty: mc_components::difficulty::Difficulty::Normal,
        difficulty_locked: false,
    };
    network::send_packet(stream, change_difficulty)?;
    println!("ChangeDifficulty sent");

    // Set player abilities
    let change_player_abilities = PlayClientbound::PlayerAbilities {
        flags: 0,
        flying_speed: 0.05,
        field_of_view_modifier: 0.1,
    };
    network::send_packet(stream, change_player_abilities)?;
    println!("PlayerAbilities sent");

    // Set held item
    let held_item_change = PlayClientbound::SetHeldItem {
        slot: 0, // TODO should be the same as when disconnected
    };
    network::send_packet(stream, held_item_change)?;
    println!("SetHeldItem sent");

    // Update recipes
    let update_recipes = PlayClientbound::UpdateRecipes {
        data: mc_packets::RawBytes { data: &[0] },
    };
    network::send_packet(stream, update_recipes)?;
    println!("UpdateRecipes sent");

    // Entity event
    let entity_event = PlayClientbound::EntityEvent {
        entity_id: player_id as i32,
        entity_status: 28, // set permission level to 4
    };
    network::send_packet(stream, entity_event)?;
    println!("EntityEvent sent");

    // Declare commands
    let declare_commands = PlayClientbound::DeclareCommands {
        count: mc_packets::VarInt(0),
        data: mc_packets::RawBytes { data: &[0] },
    };
    network::send_packet(stream, declare_commands)?;
    println!("DeclareCommands sent");

    // Unlock recipes
    let unlock_recipes = PlayClientbound::UnlockRecipes {
        action: minecraft_protocol::components::recipes::UnlockRecipesAction::Init {
            crafting_recipe_book_open: false,
            crafting_recipe_book_filter_active: false,
            smelting_recipe_book_open: false,
            smelting_recipe_book_filter_active: false,
            blast_furnace_recipe_book_open: false,
            blast_furnace_recipe_book_filter_active: false,
            smoker_recipe_book_open: false,
            smoker_recipe_book_filter_active: false,
            displayed_recipes: Array::default(),
            added_recipes: Array::default(),
        },
    };
    network::send_packet(stream, unlock_recipes)?;
    println!("UnlockRecipes sent");

    // Spawn player
    let player_position = character.position;
    let player_look_dir = character.head_rotation;

    let (player_yaw, player_pitch, _) = player_look_dir.euler_angles();
    let player_position_packet = PlayClientbound::PlayerPositionAndLook {
        x: player_position.x as f64,
        y: player_position.y as f64,
        z: player_position.z as f64,
        yaw: player_yaw,
        pitch: player_pitch,
        flags: 0x00,
        teleport_id: mc_packets::VarInt(1),
    };
    network::send_packet(stream, player_position_packet)?;
    println!("PlayerPositionAndLook sent");

    // Send server metadata
    let server_data = PlayClientbound::ServerData {
        motd: "{\"text\":\"Not like any other Minecraft Server\"}",
        icon: None,
        enforces_secure_chat: false,
    };
    network::send_packet(stream, server_data)?;
    println!("ServerData sent");

    // Spawn message
    let spawn_message = PlayClientbound::SystemChatMessage {
        content: "{\"text\":\"Welcome to Sparks of Labor!\"}",
        overlay: false,
    };
    network::send_packet(stream, spawn_message)?;
    println!("SystemChatMessage sent");

    // TODO: update players info (x2)

    // Set entity metadata
    let mut entity_metadata = BTreeMap::new();
    entity_metadata.insert(
        9,
        mc_components::entity::EntityMetadataValue::Float { value: 20.0 },
    );
    entity_metadata.insert(
        16,
        mc_components::entity::EntityMetadataValue::VarInt {
            value: mc_packets::VarInt(18),
        },
    );
    entity_metadata.insert(
        17,
        mc_components::entity::EntityMetadataValue::Byte { value: 127 },
    );
    let set_entity_metadata = PlayClientbound::SetEntityMetadata {
        entity_id: mc_packets::VarInt::from(player_id),
        metadata: mc_components::entity::EntityMetadata {
            items: entity_metadata.clone(),
        },
    };
    network::send_packet(stream, set_entity_metadata)?;
    println!("SetEntityMetadata sent");

    // Initialize world border
    let world_border_init = PlayClientbound::InitializeWorldBorder {
        x: 0.0,
        y: 0.0,
        old_diameter: 60000000.0,
        new_diameter: 60000000.0,
        speed: mc_packets::VarLong(0),
        portal_teleport_boundary: mc_packets::VarInt(29999984),
        warning_blocks: mc_packets::VarInt(5),
        warning_time: mc_packets::VarInt(15),
    };
    network::send_packet(stream, world_border_init)?;
    println!("InitializeWorldBorder sent");

    // Update time
    let time_update = PlayClientbound::UpdateTime {
        world_age: 0,
        time_of_day: 0,
    };
    network::send_packet(stream, time_update)?;
    println!("UpdateTime sent");

    // Set spawn position
    let set_spawn_position = PlayClientbound::SetSpawnPosition {
        location: minecraft_protocol::packets::Position { x: 0, y: 70, z: 0 },
        angle: 0.0,
    };
    network::send_packet(stream, set_spawn_position)?;
    println!("SetSpawnPosition sent");

    // Set center chunk
    let set_center_chunk = PlayClientbound::SetCenterChunk {
        chunk_x: mc_packets::VarInt(0), // TODO: should be the same as when disconnected
        chunk_z: mc_packets::VarInt(0), // TODO: should be the same as when disconnected
    };
    network::send_packet(stream, set_center_chunk)?;
    println!("SetCenterChunk sent");

    // Set inventory
    let set_container_content = PlayClientbound::SetContainerContent {
        window_id: 0,
        state_id: mc_packets::VarInt(1),
        slots: Array::default(),
        carried_item: mc_components::slots::Slot { item: None },
    };
    network::send_packet(stream, set_container_content)?;
    println!("SetContainerContent sent");

    // Set entity metadata (again)
    let set_entity_metadata = PlayClientbound::SetEntityMetadata {
        entity_id: mc_packets::VarInt::from(player_id),
        metadata: mc_components::entity::EntityMetadata {
            items: entity_metadata,
        },
    };
    network::send_packet(stream, set_entity_metadata)?;
    println!("SetEntityMetadata sent");

    // Update entity attributes
    let mut entity_attributes = BTreeMap::new();
    entity_attributes.insert(
        "minecraft:generic.attack_speed",
        mc_components::entity::EntityAttribute {
            value: 4.0,
            modifiers: Array::default(),
        },
    );
    entity_attributes.insert(
        "minecraft:generic.max_health",
        mc_components::entity::EntityAttribute {
            value: 20.0,
            modifiers: Array::default(),
        },
    );
    entity_attributes.insert(
        "minecraft:generic.movement_speed",
        mc_components::entity::EntityAttribute {
            value: 0.10000000149011612,
            modifiers: Array::default(),
        },
    );
    let update_entity_attributes = PlayClientbound::UpdateEntityAttributes {
        entity_id: mc_packets::VarInt::from(player_id),
        attributes: mc_packets::Map::from(entity_attributes),
    };
    network::send_packet(stream, update_entity_attributes)?;
    println!("UpdateEntityAttributes sent");

    // Update advancements
    let update_advancements = PlayClientbound::UpdateAdvancements {
        reset: true,
        advancement_mapping: mc_packets::Map::default(),
        advancements_to_remove: Array::default(),
        progress_mapping: mc_packets::Map::default(),
    };
    network::send_packet(stream, update_advancements)?;
    println!("UpdateAdvancements sent");

    // Set health
    let set_health = PlayClientbound::SetHealth {
        health: 20.0,
        food: mc_packets::VarInt(20),
        food_saturation: 5.0,
    };
    network::send_packet(stream, set_health)?;
    println!("UpdateHealth sent");

    // Set experience
    let set_experience = PlayClientbound::SetExperience {
        experience_level: mc_packets::VarInt(0),
        experience_bar: 0.0,
        total_experience: mc_packets::VarInt(0),
    };
    network::send_packet(stream, set_experience)?;
    println!("SetExperience sent");

    Ok(PlayerInfo {
        socket,
        username: logged_in_player_info.username,
        uuid: logged_in_player_info.uuid,
        locale: locale.to_owned(),
        render_distance: render_distance.try_into().unwrap_or(5),
        chat_mode,
        chat_colors,
        displayed_skin_parts,
        main_hand,
        enable_text_filtering,
        allow_server_listing,
        chunks_per_tick: 1.0,
    })
}

pub fn send_initial_chunk_data(
    stream: &mut TcpStream,
    world: &World,
    player_position: Position,
) -> Result<f32, CommunicationError> {
    // Chunk batch start
    let chunk_data = PlayClientbound::ChunkBatchStart;
    network::send_packet(stream, chunk_data)?;
    println!("ChunkBatchStart sent");

    let chunks = world.get_area(player_position);
    for chunk_column in chunks {
        let serialized = chunk_column
            .to_minecraft()
            .map_err(|e| CommunicationError::SerializationError(String::from(e)))?;

        let mut heightmaps = HashMap::new();
        heightmaps.insert(
            String::from("WORLD_SURFACE"),
            NbtTag::LongArray(serialized.heightmap_world_surface),
        );
        heightmaps.insert(
            String::from("MOTION_BLOCKING"),
            NbtTag::LongArray(serialized.heightmap_motion_blocking),
        );

        let chunk_data = PlayClientbound::ChunkData {
            value: mc_components::chunk::ChunkData {
                chunk_x: serialized.chunk_x_16,
                chunk_z: serialized.chunk_z_16,
                heightmaps: NbtTag::Compound(heightmaps),
                data: Array::from(serialized.chunk_sections),
                block_entities: Array::from(serialized.block_entities),
                sky_light_mask: Array::default(),
                block_light_mask: Array::default(),
                empty_sky_light_mask: Array::default(),
                empty_block_light_mask: Array::default(),
                sky_light: Array::default(),
                block_light: Array::default(),
            },
        };
        network::send_packet(stream, chunk_data)?;
    }

    println!("ChunkData sent");

    // Chunk batch end
    let chunk_data = PlayClientbound::ChunkBatchFinished {
        batch_size: mc_packets::VarInt(49),
    };
    network::send_packet(stream, chunk_data)?;
    println!("ChunkBatchFinished sent");

    // Get chunk batch acknoledgement
    let mut buffer = Vec::new();
    let packet: PlayServerbound = network::receive_packet(stream, &mut buffer)?;
    let PlayServerbound::ChunkBatchReceived { chunks_per_tick } = packet else {
        return Err(CommunicationError::wrong_package(
            "ChunkBatchReceived",
            packet,
        ));
    };
    println!("ChunkBatchReceived received");

    Ok(chunks_per_tick)
}

pub fn send_status_response(stream: &mut TcpStream) -> Result<(), CommunicationError> {
    let status_msg = status::ClientboundPacket::Response {
        json_response: include_str!("raw/status_response.json"),
    };
    network::send_packet(stream, status_msg)?;
    Ok(())
}

pub(crate) fn pong(stream: &mut TcpStream, payload: i64) -> Result<(), CommunicationError> {
    network::send_packet(
        stream,
        mc_packets::status::ClientboundPacket::Pong { payload },
    )?;
    Ok(())
}
