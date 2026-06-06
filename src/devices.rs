// Device endpoints (all require a valid JWT). Devices are scoped per user, so a
// user only ever sees / updates their own devices.

use axum::{
    extract::{ConnectInfo, Query, State},
    http::StatusCode,
    Json,
};
use futures::TryStreamExt;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::auth::AuthUser;
use crate::error::AppError;
use crate::models::Device;
use crate::state::AppState;

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Deserialize)]
pub struct DeviceReq {
    pub node_id: String,
    pub name: String,
    pub ip: Option<String>,
    pub tcp_port: i32,
    pub udp_port: i32,
}

#[derive(Serialize)]
pub struct DeviceOut {
    pub node_id: String,
    pub name: String,
    pub ip: String,
    pub tcp_port: i32,
    pub udp_port: i32,
    pub last_seen: i64,
}

/// Register or update (heartbeat / network-change) the caller's device.
pub async fn register(
    State(s): State<AppState>,
    user: AuthUser,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<DeviceReq>,
) -> Result<Json<serde_json::Value>, AppError> {
    let ip = req
        .ip
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| addr.ip().to_string());
    let device = Device {
        user_id: user.user_id.clone(),
        node_id: req.node_id.clone(),
        name: req.name,
        ip,
        tcp_port: req.tcp_port,
        udp_port: req.udp_port,
        last_seen: now(),
    };
    s.db.collection::<Device>(crate::models::DEVICES_COLL)
        .replace_one(
            doc! { "user_id": &user.user_id, "node_id": &req.node_id },
            &device,
        )
        .upsert(true)
        .await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

#[derive(Deserialize)]
pub struct PeersQuery {
    pub exclude: Option<String>,
}

/// List the caller's devices (optionally excluding their own node_id).
pub async fn list(
    State(s): State<AppState>,
    user: AuthUser,
    Query(q): Query<PeersQuery>,
) -> Result<Json<Vec<DeviceOut>>, AppError> {
    let mut filter = doc! { "user_id": &user.user_id };
    if let Some(ex) = q.exclude.filter(|v| !v.is_empty()) {
        filter.insert("node_id", doc! { "$ne": ex });
    }
    let devices: Vec<Device> = s
        .db
        .collection::<Device>(crate::models::DEVICES_COLL)
        .find(filter)
        .await?
        .try_collect()
        .await?;
    let out = devices
        .into_iter()
        .map(|d| DeviceOut {
            node_id: d.node_id,
            name: d.name,
            ip: d.ip,
            tcp_port: d.tcp_port,
            udp_port: d.udp_port,
            last_seen: d.last_seen,
        })
        .collect();
    Ok(Json(out))
}

#[derive(Deserialize)]
pub struct UnregReq {
    pub node_id: String,
}

pub async fn unregister(
    State(s): State<AppState>,
    user: AuthUser,
    Json(req): Json<UnregReq>,
) -> Result<StatusCode, AppError> {
    s.db.collection::<Device>(crate::models::DEVICES_COLL)
        .delete_one(doc! { "user_id": &user.user_id, "node_id": &req.node_id })
        .await?;
    Ok(StatusCode::OK)
}
