//! For interacting with the various Mojang APIs
//!
//! Remember that requests are rate limited, avoid sending too many requests,
//! and cache what you can.
//!
//! In general, you may want to read [wiki.vg/Mojang
//! API](http://wiki.vg/Mojang_API) for further documentation about the
//! requests and their responses.
//!
//! Is missing the requests that require authentication.

pub use json::*;

use std::io::{self, Read};

use reqwest::Client;
use reqwest::header::ContentType;

use serde_json;

/// Trait for structs that represent requests to the Mojang API
pub trait APIRequest {
    type Response;

    fn perform(&self) -> io::Result<Self::Response>;
    fn get_endpoint() -> String;
}

/// Make a request to check the status of the Mojang APIs
#[derive(Debug, new)]
pub struct APIStatus();
impl APIRequest for APIStatus {
    type Response = APIStatusResponse;

    fn perform(&self) -> io::Result<APIStatusResponse> {
        let res = get_request(&Self::get_endpoint())?;
        /* Flatten the list, and turn it into an object.
         * For some reason this response is given in a really weird way, and
         * this fixes it so that it can be parsed more easily */
        let res = res.replace(|c| match c {
                                  '{' | '}' => true,
                                  _ => false,
                              },
                              "")
            .replace('[', "{")
            .replace(']', "}");
        // Ok(serde_json::from_str(&get_request(&Self::get_endpoint())?).
        // unwrap())
        Ok(serde_json::from_str(&res).unwrap())
    }

    fn get_endpoint() -> String {
        "https://status.mojang.com/check".to_string()
    }
}

/// Make a Username -> UUID (at time) request
///
/// Returns information about which account had the given name at the point in
/// time, where the time is specified as epoch time. If at is not specified, it
/// will default to the current time.
///
/// If unable to find the player at the given point in time, will return an
/// error.
#[derive(Debug, new)]
pub struct NameToUUID {
    username: String,
    at: Option<i64>,
}
impl APIRequest for NameToUUID {
    type Response = NameUUID;

    fn get_endpoint() -> String {
        "https://api.mojang.com/users/profiles/minecraft/".to_string()
    }
    fn perform(&self) -> io::Result<NameUUID> {
        let url = match self.at {
            Some(x) => {
                format!("https://api.mojang.com/users/profiles/minecraft/{}?at={}",
                        self.username,
                        x)
            },
            None => {
                format!("https://api.mojang.com/users/profiles/minecraft/{}",
                        self.username)
            },
        };
        let res = get_request(&url)?;
        Ok(serde_json::from_str(&res).unwrap())
    }
}

/// A UUID -> Username history request
///
/// The UUID must be given as a string without hyphens.
#[derive(Debug, new)]
pub struct UUIDToHistory {
    uuid: String,
}
impl APIRequest for UUIDToHistory {
    type Response = Vec<NameHistory>;

    fn get_endpoint() -> String {
        "https://api.mojang.com/user/profiles/{}/names".to_string()
    }
    fn perform(&self) -> io::Result<Vec<NameHistory>> {
        let url = format!("https://api.mojang.com/user/profiles/{}/names",
                          self.uuid);
        let res = get_request(&url)?;
        Ok(serde_json::from_str(&res).unwrap())
    }
}

/// A Playernames -> UUIDs request.
///
/// Can request up to 100 UUIDs at a time.
#[derive(Debug)]
pub struct PlayernamesToUUIDs {
    usernames: Vec<String>,
}
impl APIRequest for PlayernamesToUUIDs {
    type Response = Vec<NameUUID>;

    fn get_endpoint() -> String {
        "https://api.mojang.com/profiles/minecraft".to_string()
    }
    fn perform(&self) -> io::Result<Self::Response> {
        let body = serde_json::to_string(&self.usernames).unwrap();
        println!("body: {}", body);
        let res = post_request(&Self::get_endpoint(), &body)?;
        Ok(serde_json::from_str(&res).unwrap())
    }
}
impl PlayernamesToUUIDs {
    /// Create a new instance of this request.
    ///
    /// # Panics
    ///
    /// Panics if usernames.len() > 100. The API limits this request to 100
    /// usernames.
    pub fn new(usernames: Vec<String>) -> Self {
        if usernames.len() > 100 {
            panic!("PlayernamesToUUIDs got more than 100 usernames");
        }
        PlayernamesToUUIDs {
            usernames: usernames,
        }
    }
}

/// Represents a UUID -> Profile + Skin and Cape request
#[derive(Debug, new)]
pub struct UUIDToProfile {
    uuid: String,
    /// Whether you want the response signed by the yggdrasil private key
    signed: bool,
}
impl APIRequest for UUIDToProfile {
    type Response = Profile;

    fn get_endpoint() -> String {
        "https://sessionserver.mojang.com/session/minecraft/profile/"
            .to_string()
    }
    fn perform(&self) -> io::Result<Self::Response> {
        let url = if self.signed {
            format!("https://sessionserver.mojang.com/session/minecraft/profile/{}?unsigned=false",
                    self.uuid)
        } else {
            format!("https://sessionserver.mojang.com/session/minecraft/profile/{}",
                    self.uuid)
        };
        let res = get_request(&url)?;
        println!("res: {}", res);
        Ok(serde_json::from_str(&res).unwrap())
    }
}

/// Get the blocked server's hashes
#[derive(Debug, new)]
pub struct BlockedServers();
impl APIRequest for BlockedServers {
    type Response = Vec<String>;

    fn get_endpoint() -> String {
        "https://sessionserver.mojang.com/blockedservers".to_string()
    }
    fn perform(&self) -> io::Result<Self::Response> {
        let res: String = get_request(&Self::get_endpoint())?;
        Ok(res.split('\n')
               .filter_map(|e| if !e.is_empty() {
                               Some(e.to_string())
                           } else {
                               None
                           })
               .collect())
    }
}

/// Get the orders statistics
///
/// The API will respond with the sum of sales for the selected types. E.g. by
/// setting item_sold_minecraft and prepaid_card_redeemed_minecraft to true,
/// you'll get the sum of sales for those two types.
#[derive(Debug)]
pub struct Statistics {
    item_sold_minecraft: bool,
    prepaid_card_redeemed_minecraft: bool,
    item_sold_cobalt: bool,
    item_sold_scrolls: bool,
}
impl APIRequest for Statistics {
    type Response = StatisticsResponse;

    fn get_endpoint() -> String {
        "https://api.mojang.com/orders/statistics".to_string()
    }
    fn perform(&self) -> io::Result<Self::Response> {
        let mut query: Vec<&str> = Vec::new();
        if self.item_sold_minecraft {
            query.push("item_sold_minecraft");
        }
        if self.prepaid_card_redeemed_minecraft {
            query.push("prepaid_card_redeemed_minecraft");
        }
        if self.item_sold_cobalt {
            query.push("item_sold_cobalt");
        }
        if self.item_sold_scrolls {
            query.push("item_sold_scrolls");
        }
        let payload = json!({
                                "metricKeys": query
                            });
        let res = post_request(&Self::get_endpoint(), &payload.to_string())?;
        Ok(serde_json::from_str(&res).unwrap())
    }
}
impl Statistics {
    /// Create a new request for requesting the sum of sales of the specified
    /// types.
    ///
    /// # Panics
    ///
    /// Panics if not at least one of the values is true.
    pub fn new(item_sold_minecraft: bool,
               prepaid_card_redeemed_minecraft: bool,
               item_sold_cobalt: bool,
               item_sold_scrolls: bool)
               -> Self {
        if !(item_sold_minecraft | prepaid_card_redeemed_minecraft |
             item_sold_cobalt | item_sold_scrolls) {
            panic!("You must specify at least one type of sale in the Statistics request");
        }
        Statistics {
            item_sold_minecraft: item_sold_minecraft,
            prepaid_card_redeemed_minecraft: prepaid_card_redeemed_minecraft,
            item_sold_cobalt: item_sold_cobalt,
            item_sold_scrolls: item_sold_scrolls,
        }
    }
    /// Get the sum of everything
    pub fn all() -> Self {
        Statistics {
            item_sold_minecraft: true,
            prepaid_card_redeemed_minecraft: true,
            item_sold_cobalt: true,
            item_sold_scrolls: true,
        }
    }
    /// Get just the amount of Minecraft sales
    pub fn minecraft() -> Self {
        Statistics {
            item_sold_minecraft: true,
            prepaid_card_redeemed_minecraft: true,
            item_sold_cobalt: false,
            item_sold_scrolls: false,
        }
    }
}

/// Helper function for performing a GET request to the given URL, returning
/// the response content
fn get_request(url: &str) -> io::Result<String> {
    let client = Client::new().expect("Error creating reqwest client");
    let res = client.get(url).send();

    let mut res = match res {
        Ok(x) => x,
        Err(e) => {
            return io_error!("Error sending GET request to {}: {}", url, e);
        },
    };

    if !res.status().is_success() {
        return io_error!("Got {} response from {}", res.status(), url);
    }

    let mut ret = String::new();
    res.read_to_string(&mut ret)?;
    Ok(ret)
}

/// Helper function for performing a POST request to the given URL,
/// posting the given data to it, and returning the response content.
fn post_request(url: &str, post: &str) -> io::Result<String> {
    let client = Client::new().expect("Error creating reqwest client");
    let res = client.post(url)
        .header(ContentType::json())
        .body(post)
        .send();

    let mut res = match res {
        Ok(x) => x,
        Err(e) => {
            return io_error!("Error sending POST request to {}: {}", url, e);
        },
    };

    if !res.status().is_success() {
        return io_error!("Got {} response from {}", res.status(), url);
    }

    let mut ret = String::new();
    res.read_to_string(&mut ret)?;
    Ok(ret)
}
