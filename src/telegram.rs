use crate::object_types::GroupmeMessage;
use futures::future::Future;
use hyper::header::HeaderValue;
use hyper::{Body, Method, Request};
use serde_json::json;
use tokio;

const TG_API_BASE: &str = "https://api.telegram.org/bot";

#[derive(Debug)]
pub struct Telegram {
    token: String,
}

impl Telegram {
    pub fn new(token: &String) -> Telegram {
        let token = token.clone();
        Telegram { token }
    }

    pub fn relay_message(&self, mes: GroupmeMessage, chat_id: &String) {
        let text = if mes.attachments.len() > 0 {
            "This message contains attachments".to_string()
        } else {
            mes.text
        };

        let json = json!({
            "chat_id": chat_id,
            "text": format!("`{}:`\n{}", mes.name, text),
            "parse_mode": "Markdown",
        });

        let client = reqwest::r#async::Client::new();
        let resp = client
            .post(&format!("{}{}/sendMessage", TG_API_BASE, self.token))
            .body(serde_json::to_string(&json).unwrap())
            .header(
                hyper::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            )
            .send()
            .map_err(|e| eprintln!("error: {}", e))
            .and_then(|r| Ok(()));

        tokio::spawn(resp);
    }
}
