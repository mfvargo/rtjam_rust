use serde_json::Value;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize)]

pub enum WebsockMessage {
    Chat(Value),
    API(String, Value),
}

#[cfg(test)]
mod test_websock_message {

    use super::*;
    use serde_json::json;

    #[test]
    fn make_chat() {
        let chat = WebsockMessage::Chat(json!({ "name": "bob", "age": 32}));
        let cjson = serde_json::to_value(chat).unwrap();
        println!(
            "chat json: {}",
            serde_json::to_string_pretty(&cjson).unwrap()
        );
        let api = WebsockMessage::API(String::from("saveStats"), json!({"some": "stats"}));
        dbg!(api);
    }
}
