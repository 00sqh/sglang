// SPDX-FileCopyrightText: Copyright (c) 2026 The SGLang Authors
// SPDX-License-Identifier: Apache-2.0

use serde_json::Value;
use sha2::{Digest, Sha256};

/// The first image stays stable as later turns add text or more images.
pub fn key(request: &Value) -> Option<String> {
    for message in request.get("messages")?.as_array()? {
        for part in message["content"].as_array().into_iter().flatten() {
            let reference = match part["type"].as_str() {
                Some("image_url" | "input_image") => part["image_url"]
                    .as_str()
                    .or_else(|| part["image_url"]["url"].as_str()),
                Some("image") => part["image"].as_str(),
                _ => continue,
            }?;
            if reference.is_empty() {
                return None;
            }
            return Some(format!("{:x}", Sha256::digest(reference.as_bytes())));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn image_key_survives_new_turns_and_additional_images() {
        let mut request = json!({"messages":[{"role":"user","content":[
            {"type":"text","text":"describe"},{"type":"image_url","image_url":{"url":"data:first"}}
        ]}]});
        let first = key(&request).unwrap();
        request["messages"]
            .as_array_mut()
            .unwrap()
            .push(json!({"role":"user","content":[
                {"type":"image_url","image_url":"data:second"}
            ]}));
        assert_eq!(key(&request).as_deref(), Some(first.as_str()));
        request["messages"][0]["content"][1]["image_url"]["url"] = "different".into();
        assert_ne!(key(&request).unwrap(), first);
    }

    #[test]
    fn text_audio_and_empty_images_have_no_key() {
        for part in [
            json!({"type":"text","text":"hi"}),
            json!({"type":"input_audio","input_audio":"x"}),
            json!({"type":"image_url","image_url":{"url":""}}),
        ] {
            assert!(key(&json!({"messages":[{"content":[part]}]})).is_none());
        }
    }
}
