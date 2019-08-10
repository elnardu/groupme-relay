use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct GroupmeMessage {
    pub attachments: Vec<GroupmeAttachment>,
    avatar_url: String,
    created_at: u32,
    group_id: String,
    id: String,
    pub name: String,
    sender_id: String,
    source_guid: String,
    system: bool,
    pub text: String,
    user_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GroupmeAttachment {
    r#type: String,
    url: String,
}
