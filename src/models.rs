use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// Dedicated collection names. The `device_registry` DB is shared with another
// project that uses `users`/`devices` with a different schema, so we namespace
// ours to avoid colliding with its documents and indexes.
pub const USERS_COLL: &str = "md_users";
pub const DEVICES_COLL: &str = "md_devices";

/// A user account. Password is stored only as an Argon2id hash.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password_hash: String,
}

/// A device owned by a user. `node_id` is the per-session id the client uses;
/// `ip`/`tcp_port`/`udp_port` are exactly what that client listens on, so peers
/// can connect directly once they look it up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub user_id: String,
    pub node_id: String,
    pub name: String,
    pub ip: String,
    pub tcp_port: i32,
    pub udp_port: i32,
    pub last_seen: i64,
}
