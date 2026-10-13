// DB-only endpoints: without the `db` feature every handler returns 501.
#![cfg_attr(not(feature = "db"), allow(unused))]

use crate::http::ensure_scope;
use crate::http::HttpError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use cncore::platform::models::{MessagePriority, MessageQuery, MessageRecord, NewMessageRecord};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct PublishMessageRequest {
    pub project_id: Uuid,
    pub topic: String,
    pub key: Option<String>,
    pub payload: String, // Base64 encoded or just string? Let's assume string for now, but model has Vec<u8>
    pub priority: Option<String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListMessagesQuery {
    pub tenant_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub topic: String,
    pub limit: Option<u32>,
}

#[utoipa::path(
    post,
    path = "/messaging/publish",
    tag = "messaging",
    request_body = PublishMessageRequest,
    responses(
        (status = 201, body = MessageRecord),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn publish_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<PublishMessageRequest>,
) -> Result<(StatusCode, Json<MessageRecord>), HttpError> {
    ensure_scope(&headers, "messaging:write")?;

    #[cfg(feature = "db")]
    {
        let project_store = state.project_store.as_ref().ok_or_else(|| {
            HttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "project store not available",
            )
        })?;

        let project = project_store
            .get_project(req.project_id)
            .await
            .map_err(|_| HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
            .ok_or_else(|| HttpError::new(StatusCode::NOT_FOUND, "project not found"))?;

        let store = &state.messaging_store;

        let priority = if let Some(p) = req.priority {
            p.parse::<MessagePriority>()
                .map_err(|_| HttpError::new(StatusCode::BAD_REQUEST, "invalid priority"))?
        } else {
            MessagePriority::Normal
        };

        let input = NewMessageRecord {
            id: Uuid::new_v4(),
            tenant_id: project.tenant_id,
            project_id: req.project_id,
            topic: req.topic,
            key: req.key,
            payload: req.payload.into_bytes(),
            priority,
            attributes: req.attributes,
        };

        let message = store.enqueue_message(input).await.map_err(|_| {
            HttpError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to enqueue message",
            )
        })?;

        Ok((StatusCode::CREATED, Json(message)))
    }

    #[cfg(not(feature = "db"))]
    Err(HttpError::new(
        StatusCode::NOT_IMPLEMENTED,
        "database required",
    ))
}

#[utoipa::path(
    get,
    path = "/messaging/messages",
    tag = "messaging",
    params(ListMessagesQuery),
    responses(
        (status = 200, body = Vec<MessageRecord>),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListMessagesQuery>,
) -> Result<Json<Vec<MessageRecord>>, HttpError> {
    ensure_scope(&headers, "messaging:read")?;

    #[cfg(feature = "db")]
    {
        let store = &state.messaging_store;

        let db_query = MessageQuery {
            tenant_id: query.tenant_id,
            project_id: query.project_id,
            topic: query.topic,
            limit: query.limit,
        };

        let messages = store.list_messages(db_query).await.map_err(|_| {
            HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "failed to list messages")
        })?;

        Ok(Json(messages))
    }

    #[cfg(not(feature = "db"))]
    Err(HttpError::new(
        StatusCode::NOT_IMPLEMENTED,
        "database required",
    ))
}

#[utoipa::path(
    post,
    path = "/messaging/messages/{id}/ack",
    tag = "messaging",
    params(("id" = Uuid, Path, description = "Message ID")),
    request_body = serde_json::Value, // Expecting {"topic": "..."}
    responses(
        (status = 204, description = "Message acknowledged"),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn ack_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<serde_json::Value>,
) -> Result<StatusCode, HttpError> {
    ensure_scope(&headers, "messaging:write")?;

    let topic = req
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| HttpError::new(StatusCode::BAD_REQUEST, "topic required"))?;

    #[cfg(feature = "db")]
    {
        let store = &state.messaging_store;

        store.ack_message(topic, id).await.map_err(|_| {
            HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "failed to ack message")
        })?;

        Ok(StatusCode::NO_CONTENT)
    }

    #[cfg(not(feature = "db"))]
    Err(HttpError::new(
        StatusCode::NOT_IMPLEMENTED,
        "database required",
    ))
}
