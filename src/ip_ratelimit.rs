const RATELIMIT_SECS: u64 = 60;
use std::collections::HashMap;
use std::time::SystemTime;
pub struct IpRatelimit {
    ip_list: HashMap<String, SystemTime>,
}

impl IpRatelimit {
    pub fn remove_stale(&mut self) {
        // let queue: Vec<_> = Vec::new();
        for ip in self.ip_list.clone().into_keys() {
            let ip = ip.to_string();
            self.remove_ratelimit(&ip);
        }
    }
    pub fn ratelimit(&mut self, ip: String) {
        self.ip_list.insert(ip, SystemTime::now());
    }
    pub fn remove_ratelimit(&mut self, ip: &String) -> bool {
        let time_left = self.ratelimit_left(ip);
        if time_left == 0 {
            self.ip_list.remove(ip);
            true
        } else {
            false
        }
    }
    pub fn ratelimit_left(&self, ip: &String) -> u64 {
        let lookup = self.ip_list.get(ip);
        match lookup {
            Some(time) => {
                let now = SystemTime::now();
                let distance = now.duration_since(*time).expect("wtf ip time error");
                let time_left = distance.as_secs().checked_sub(RATELIMIT_SECS);
                if time_left.is_some() {
                    time_left.unwrap()
                } else {
                    0
                }
            }
            None => 0,
        }
        //RATELIMIT_SECS
    }
}

pub fn new() -> IpRatelimit {
    IpRatelimit {
        ip_list: HashMap::new(),
    }
}
