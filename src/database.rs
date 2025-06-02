// over complicated shit
use crate::ip_ratelimit;
use serde::{Deserialize, Serialize};
use std::fs;
const USER_MSG_SIZE: usize = 80; // 64 character
const MAX_TENOR_GIF_SIZE: usize = 30;
const USERNAME_SIZE: usize = 10;
#[derive(Serialize, Deserialize)]
pub struct UserInfo {
    msg: String,
    username: String,
    gif_id: String,
    is_owner: bool,
}
pub struct Database {
    db: Vec<UserInfo>,
    pub ip_ratelimit: ip_ratelimit::IpRatelimit,
    owner_key: String,
}
pub enum AddMsgError {
    Ratelimit,
    UsernameOrMsgEmpty,
    UsernameTooBig,
    MessageTooBig,
    GifIdTooBig,
    Success,
}

impl Database {
    pub fn add_msg(
        &mut self,
        ip: String,
        username: String,
        msg: String,
        gif_id: String,
        owner_key: String,
    ) -> AddMsgError {
        if !self.ip_ratelimit.remove_ratelimit(&ip) {
            return AddMsgError::Ratelimit;
        }
        if username.is_empty() || msg.is_empty() {
            return AddMsgError::UsernameOrMsgEmpty;
        }
        if username.len() > USERNAME_SIZE {
            return AddMsgError::UsernameTooBig;
        }
        if msg.len() > USER_MSG_SIZE {
            //no
            return AddMsgError::MessageTooBig;
        }
        if gif_id.len() > MAX_TENOR_GIF_SIZE {
            // no
            return AddMsgError::GifIdTooBig;
        }
        self.ip_ratelimit.ratelimit(ip);
        self.db.push(UserInfo {
            username: username,
            msg: msg,
            gif_id: gif_id,
            is_owner: self.owner_key == owner_key,
        });

        AddMsgError::Success
    }

    pub fn remove_msg(&mut self, cursor: usize, owner_key: String) -> bool {
        if owner_key != self.owner_key {
            return false;
        }
        self.db.remove(cursor);
        true
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(&self.db).unwrap()
    }

    pub fn save(&self) {
        fs::write("db.json", self.to_json());
    }

    pub fn load(&mut self) {
        let json_data = fs::read("db.json");
        match json_data {
            Ok(data) => {
                let parsed: Vec<UserInfo> = serde_json::from_slice(&data).unwrap();
                self.db = parsed;
            }
            Err(error) => {
                print!("{:?}", error);
            }
        }
    }

}

pub fn new(api_key: String) -> Database {
    let mut db = Database {
        db: Vec::new(),
        ip_ratelimit: ip_ratelimit::new(),
        owner_key: api_key,
    };

    db.load();
    db
}
