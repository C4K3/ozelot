//! This module contains json serializable structs for use for interaction with
//! the various Mojang APIs.

/// Contains the status about each of the Mojang APIs
#[derive(Debug, Deserialize)]
pub struct APIStatusResponse {
    #[serde(rename="minecraft.net")]
    pub minecraftnet: String,
    #[serde(rename="session.minecraft.net")]
    pub sessionminecraftnet: String,
    #[serde(rename="account.mojang.com")]
    pub accountmojangcom: String,
    #[serde(rename="auth.mojang.com")]
    pub authmojangcom: String,
    #[serde(rename="skins.minecraft.net")]
    pub skinsminecraftnet: String,
    #[serde(rename="authserver.mojang.com")]
    pub authservermojangcom: String,
    #[serde(rename="sessionserver.mojang.com")]
    pub sessionservermojangcom: String,
    #[serde(rename="api.mojang.com")]
    pub apimojangcom: String,
    #[serde(rename="textures.minecraft.net")]
    pub texturesminecraftnet: String,
    #[serde(rename="mojang.com")]
    pub mojangcom: String,
}

/// Represents a single username - UUID mapping.
///
/// This struct is used in both PlayernamesToUUIDs and NameToUUID.
#[derive(Debug, Deserialize)]
pub struct NameUUID {
    /// The uuid in hex without dashes
    pub id: String,
    /// Name of the player at the present point in time
    pub name: String,
    #[serde(default="always_false")]
    pub legacy: bool,
    #[serde(default="always_false")]
    pub demo: bool,
}

/// Represents a player Profile, as returned in a UUIDToProfile lookup
#[derive(Debug, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub properties: Vec<ProfileProperties>,
}
/// Represents the properties part of a Profile response
///
/// Used in the UUIDToProfile and SessionHasJoined requests.
#[derive(Debug, Deserialize)]
pub struct ProfileProperties {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

/// The requested statistics
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct StatisticsResponse {
    pub total: u64,
    pub last24h: u64,
    pub saleVelocityPerSeconds: f64,
}

/// Represents a single historic name for a given account. Used in the
/// UUIDToHistory request.
#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct NameHistory {
    pub name: String,
    pub changedToAt: Option<u64>,
}

/// Represents a response to a successful authentication
#[derive(Debug, Deserialize)]
pub struct AuthenticationResponse {
    pub accessToken: String,
    pub clientToken: Option<String>,
    pub availableProfiles: Option<Vec<NameUUID>>,
    pub selectedProfile: NameUUID,
}

/// Response about whether the client has posted a join to Mojang
#[derive(Debug, Deserialize)]
pub struct SessionHasJoinedResponse {
    /// The uuid
    id: String,
    name: String,
    properties: ProfileProperties,
}





/// For use with Serde default values
fn always_false() -> bool {
    false
}
