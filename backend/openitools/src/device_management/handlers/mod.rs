use openitools_idevice::LockdownClient;
use plist::Value;

pub mod battery;
pub mod hardware;
pub mod os;
pub mod storage;

pub async fn get_string_value_or_default(
    device: &mut LockdownClient,
    domain: Option<&str>,
    key: Option<&str>,
) -> Option<String> {
    match device.get_value(domain, key).await {
        Ok(value) => Some(
            value
                .as_string()
                .map(ToString::to_string)
                .unwrap_or_default(),
        ),
        Err(e) => {
            log::error!("Failed to get value {domain:?}:{key:?}: {e:?}");
            None
        }
    }
}

pub trait RecursiveFind {
    fn rfind(&self, key: &str) -> Option<Value>;
}

impl RecursiveFind for plist::Dictionary {
    fn rfind(&self, key: &str) -> Option<Value> {
        for (k, v) in self {
            if k == key {
                return Some(v.clone());
            }

            if let Value::Dictionary(dict) = v {
                if let Some(found) = dict.rfind(key) {
                    return Some(found);
                }
            }
        }
        None
    }
}
