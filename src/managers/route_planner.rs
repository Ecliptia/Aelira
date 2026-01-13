use std::collections::HashMap;
use std::sync::Mutex;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct FailingAddress {
    #[serde(rename = "failingAddress")]
    pub address: String,
    #[serde(rename = "failingTimestamp")]
    pub timestamp: u64,
    #[serde(rename = "failingTime")]
    pub time: String,
}

#[derive(Serialize, Clone)]
pub struct RoutePlannerStatus {
    pub class: Option<String>,
    pub details: Option<RoutePlannerDetails>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlannerDetails {
    pub ip_block: IpBlock,
    pub failing_addresses: Vec<FailingAddress>,
    pub rotate_index: String,
    pub ip_index: String,
    pub current_address: String,
}

#[derive(Serialize, Clone)]
pub struct IpBlock {
    pub r#type: String,
    pub size: String,
}

pub struct RoutePlannerManager {
    pub banned_ips: Mutex<HashMap<String, u64>>,
}

impl RoutePlannerManager {
    pub fn new() -> Self {
        Self {
            banned_ips: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_status(&self) -> RoutePlannerStatus {
        let banned = self.banned_ips.lock().unwrap();
        let failing_addresses: Vec<FailingAddress> = banned.iter()
            .map(|(ip, timestamp)| {
                let datetime: chrono::DateTime<chrono::Local> = std::time::SystemTime::UNIX_EPOCH
                    .checked_add(std::time::Duration::from_millis(*timestamp))
                    .unwrap_or(std::time::SystemTime::now())
                    .into();
                
                FailingAddress {
                    address: ip.clone(),
                    timestamp: *timestamp,
                    time: datetime.to_rfc3339(),
                }
            })
            .collect();

        if failing_addresses.is_empty() {
            return RoutePlannerStatus { class: None, details: None };
        }

        RoutePlannerStatus {
            class: Some("RotatingIpRoutePlanner".to_string()),
            details: Some(RoutePlannerDetails {
                ip_block: IpBlock {
                    r#type: "Inet4Address".to_string(),
                    size: "1".to_string(),
                },
                failing_addresses,
                rotate_index: "0".to_string(),
                ip_index: "0".to_string(),
                current_address: "0.0.0.0".to_string(),
            }),
        }
    }

    pub fn unmark_address(&self, address: &str) {
        let mut banned = self.banned_ips.lock().unwrap();
        banned.remove(address);
    }

    pub fn unmark_all_addresses(&self) {
        let mut banned = self.banned_ips.lock().unwrap();
        banned.clear();
    }
}
