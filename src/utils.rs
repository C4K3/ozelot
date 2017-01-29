//! Miscellaneous utility functions

use rustc_serialize::json::Json;

/// Parses a Chat component (given as json) into a String containing only the
/// visible plaintext without any formatting.
///
/// FIXME Does not yet work with translate components
///
/// # Panics
///
/// If the input is not valid json.
pub fn chat_to_str(chat: &str) -> String {
    let mut ret = String::new();
    let data = Json::from_str(chat)
        .expect("Invalid json passed to chat_to_str");

    chat_to_str_parse_json(&data, &mut ret);

    ret
}

fn chat_to_str_parse_json(json: &Json, ret: &mut String) {
    match json.find("text") {
        Some(&Json::String(ref x)) => ret.push_str(x),
        _ => (),
    }

    match json.find("extra") {
        Some(&Json::Array(ref x)) => {
            for object in x {
                chat_to_str_parse_json(object, ret);
            }
        },
        Some(ref x) => chat_to_str_parse_json(x, ret),
        None => ()
    }
}

