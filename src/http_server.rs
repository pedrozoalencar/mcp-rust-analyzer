use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::server::McpServer;

#[derive(Clone)]
pub struct AppState {
    mcp_server: Arc<RwLock<McpServer>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

pub async fn start_http_server(mcp_server: McpServer, port: u16) -> Result<()> {
    let state = AppState {
        mcp_server: Arc::new(RwLock::new(mcp_server)),
    };

    let app = Router::new()
        .route("/", get(health_check))
        .route("/jsonrpc", post(handle_jsonrpc))
        .route("/initialize", post(handle_initialize))
        .route("/tools/list", get(handle_tools_list))
        .route("/tools/call", post(handle_tools_call))
        .route("/resources/list", get(handle_resources_list))
        .route("/resources/read", post(handle_resources_read))
        .route("/prompts/list", get(handle_prompts_list))
        .route("/prompts/get", post(handle_prompts_get))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("127.0.0.1:{}", port);
    info!("HTTP server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "mcp-rust-analyzer",
        "version": "0.1.0"
    }))
}

async fn handle_jsonrpc(
    State(state): State<AppState>,
    Json(request): Json<Value>,
) -> impl IntoResponse {
    let request_str = serde_json::to_string(&request).unwrap_or_default();
    
    match state.mcp_server.read().await.handle_request(&request_str).await {
        Ok(response) => {
            match serde_json::from_str::<Value>(&response) {
                Ok(json_response) => (StatusCode::OK, Json(json_response)),
                Err(_) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32603,
                            "message": "Invalid response format"
                        }
                    }))
                ),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": format!("Internal error: {}", e)
                }
            }))
        ),
    }
}

async fn handle_initialize(State(state): State<AppState>) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}

async fn handle_tools_list(State(state): State<AppState>) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}

async fn handle_tools_call(
    State(state): State<AppState>,
    Json(params): Json<Value>,
) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": params
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}

async fn handle_resources_list(State(state): State<AppState>) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/list",
        "params": {}
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}

async fn handle_resources_read(
    State(state): State<AppState>,
    Json(params): Json<Value>,
) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "resources/read",
        "params": params
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}

async fn handle_prompts_list(State(state): State<AppState>) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/list",
        "params": {}
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}

async fn handle_prompts_get(
    State(state): State<AppState>,
    Json(params): Json<Value>,
) -> impl IntoResponse {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "prompts/get",
        "params": params
    });
    
    handle_jsonrpc(State(state), Json(request)).await
}