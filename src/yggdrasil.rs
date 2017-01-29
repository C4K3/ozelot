//! For interacting with the Yggdrasil API
//!
//! The idea is to have everything necessary for yggdrasil, see
//! http://wiki.vg/Authentication for info about the various available
//! requests, but not all of them are implemented here yet. It also contains
//! a few utility functions that may be needed.
use std::fmt::Write;
use std::io;
use std::io::Read;

use ring::digest;
use ring::rand::SystemRandom;

use openssl::rsa::{Rsa, PKCS1_PADDING};

use reqwest::Client;
use reqwest::header::ContentType;

use rustc_serialize::json::Json;

lazy_static! {
    static ref RANDOM: SystemRandom = SystemRandom::new();
}

/// Create a shared secret as used by yggdrasil
pub fn create_shared_secret() -> [u8; 16] {
    let mut ret = [0; 16];
    match RANDOM.fill(&mut ret) {
        Ok(()) => (),
        Err(_) => {
            panic!("yggdrasil.create_shared_secret encountered an error");
        },
    }
    ret
}

/// Conduct yggdrasil authentication with Mojang, if successful returns
/// (accessToken, clientToken, username, uuid)
#[allow(non_snake_case)]
pub fn authenticate(login: &str, password: &str)
    -> io::Result<(String, String, String, String)> {

        let client = Client::new().expect("Error creating reqwest client");
        let payload = format!("{{\"agent\":{{\"name\":\"Minecraft\",\"version\":1}},\
    \"username\":\"{}\",\
    \"password\":\"{}\"}}",
    login, password);
    let res = client.post("https://authserver.mojang.com/authenticate")
        .header(ContentType::json())
        .body(payload)
        .send();

    let mut res = match res {
        Ok(x) => x,
        Err(e) => return io_error!(
            "Got yggdrasil::authenticate error sending http request, {:?}", e),
    };

    if !res.status().is_success() {
        return io_error!(
            "yggdrasil::authenticate got non-200 response for server, likely wrong username/password");
    }

    let mut tmp = String::new();
    res.read_to_string(&mut tmp)?;
    let data = match Json::from_str(&tmp) {
        Ok(x) => x,
        Err(_) => return io_error!("yggdrasil::authenticate error parsing json"),
    };
    let accessToken = match data.find("accessToken") {
        Some(&Json::String(ref x)) => x.to_string(),
        _ => return io_error!(
            "client::authenticate did not contain accessToken"),
    };
    let clientToken = match data.find("accessToken") {
        Some(&Json::String(ref x)) => x.to_string(),
        _ => return io_error!(
            "client::authenticate did not contain clientToken"),
    };
    let data = match data.find("selectedProfile") {
        Some(x) => x,
        None => return io_error!(
            "client::authenticate did not contain selectedProfile"),
    };
    let uuid = match data.find("id") {
        Some(&Json::String(ref x)) => x.to_string(),
        _ => return io_error!(
            "client::authenticate did not contain uuid"),
    };
    let username = match data.find("name") {
        Some(&Json::String(ref x)) => x.to_string(),
        _ => return io_error!(
            "client::authenticate did not contain name"),
    };
    Ok((accessToken, clientToken, username, uuid))
}

/// Post the join to Mojang, must be done immediately before sending
/// the EncryptionResponse. This does not receive a response.
pub fn session_join(access_token: &str,
                 uuid: &str,
                 server_id: &str,
                 shared_secret: &[u8],
                 server_public_key: &[u8])
    -> io::Result<()> {

    let client = Client::new().expect("Error creating reqwest client");
    let hash = sha1(server_id, shared_secret, server_public_key);
    let payload = format!("{{\"accessToken\":\"{}\",\"selectedProfile\":\"{}\",\"serverId\":\"{}\"}}",
                          access_token,
                          uuid,
                          hash);

    let res = client.post(
        "https://sessionserver.mojang.com/session/minecraft/join")
        .header(ContentType::json())
        .body(payload)
        .send();

    let res = match res {
        Ok(x) => x,
        Err(e) => return io_error!(
            "Got yggdrasil::session_join error sending http request, {:?}", e),
    };

    if !res.status().is_success() {
        return io_error!(
            "yggdrasil::session_join got non-200 response for server");
    }
    Ok(())
}

/// Given a public key in DER format (the format you get it in in the EncryptionRequest packet), and some data, RSA encrypt the data
///
/// For use with the EncryptionResponse packet.
pub fn rsa_encrypt(pubkey: &[u8], data: &[u8]) -> io::Result<Vec<u8>> {
    let key = match Rsa::public_key_from_der(pubkey) {
        Ok(x) => x,
        Err(e) => return io_error!(
            "rsa_encrypt: Got error trying to read public key: {:?}", e),
    };

    let padding = PKCS1_PADDING;

    let mut ret = vec![0; 128];
    match key.public_encrypt(data, &mut ret, padding) {
        Ok(128) => (),
        _ => return io_error!("yggdrasil::rsa_encrypt error encrypting data"),
    }

    Ok(ret)
}

/// Calculate the sha1 for the client to use to authenticate with yggdrasil
fn sha1(server_id: &str, shared_secret: &[u8], server_public_key: &[u8])
    -> String {

    let mut ctx = digest::Context::new(&digest::SHA1);
    ctx.update(server_id.as_bytes());
    ctx.update(shared_secret);
    ctx.update(server_public_key);
    let mut digest = Vec::new();
    digest.extend_from_slice(ctx.finish().as_ref());

    let mut tmp = String::new();
    let mut negative = false;

    if digest[0] >= 128 {
        /* This means we have to calculate the twos complement */
        negative = true;

        for byte in digest.iter_mut() {
            *byte ^= 0xff;
        }

        /* Add 1 to the number */
        for byte in digest.iter_mut().rev() {
            if *byte == 255 {
                *byte = 0;
            } else {
                *byte += 1;
                break;
            }
        }

        for byte in &digest {
            write!(&mut tmp, "{:02x}", byte)
                .expect("yggdrasil.sha1 failed writing to string");
        }

    } else {
        for byte in &digest {
            write!(&mut tmp, "{:02x}", byte)
                .expect("yggdrasil.sha1 failed writing to string");
        }
    }

    /* Now we copy the string a last time, to remove leading zeros and add the
     * the leading minus if applicable */
    let mut ret = String::new();
    if negative {
        write!(&mut ret, "-").expect("yggdrasil.sha1 failed writing to string");
    }

    let mut non_zero = false;
    for character in tmp.chars() {
        if character != '0' {
            non_zero = true;
        }
        if non_zero {
            ret.push(character);
        }
    }

    ret
}

