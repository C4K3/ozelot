//! Miscellaneous utility functions

use errors::Result;

use serde_json::{self, Value};

/// Parses a Chat component (given as json) into a String containing only the
/// visible plaintext without any formatting.
///
/// FIXME Does not yet work with translate components
pub fn chat_to_str(chat: &str) -> Result<String> {
    let mut ret = String::new();
    let data = serde_json::from_str(chat)?;

    chat_to_str_parse_json(&data, &mut ret);

    Ok(ret)
}

fn chat_to_str_parse_json(json: &Value, ret: &mut String) {
    match json.get("text") {
        Some(&Value::String(ref x)) => ret.push_str(x),
        _ => (),
    }

    match json.get("extra") {
        Some(&Value::Array(ref x)) => {
            for object in x {
                chat_to_str_parse_json(object, ret);
            }
        },
        Some(ref x) => chat_to_str_parse_json(x, ret),
        None => (),
    }
}
