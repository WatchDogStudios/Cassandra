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
use cncore::platform::models::{
    ModeratedContent, ModerationQuery, ModerationState, NewModeratedContent,
};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateContentRequest {
    pub project_id: Uuid,
    pub filename: String,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub labels: HashMap<String, String>,
    pub attributes: HashMap<String, String>,
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateContentStateRequest {
    pub state: String,
    pub reason: Option<String>,
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListContentQuery {
    pub tenant_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub state: Option<String>,
}

#[utoipa::path(
    post,
    path = "/ugc/content",
    tag = "ugc",
    request_body = CreateContentRequest,
    responses(
        (status = 201, body = ModeratedContent),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn create_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateContentRequest>,
) -> Result<(StatusCode, Json<ModeratedContent>), HttpError> {
    ensure_scope(&headers, "ugc:write")?;

    // In a real app, we'd resolve tenant_id from the user.
    // For now, we'll assume the user has access to the project's tenant.
    // We need to fetch the project to get the tenant_id.

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

        let store = &state.moderation_store;

        let input = NewModeratedContent {
            id: Uuid::new_v4(),
            tenant_id: project.tenant_id,
            project_id: req.project_id,
            filename: req.filename,
            mime_type: req.mime_type,
            size_bytes: req.size_bytes,
            labels: req.labels,
            attributes: req.attributes,
        };

        let content = store.create_content(input).await.map_err(|_| {
            HttpError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to create content",
            )
        })?;

        Ok((StatusCode::CREATED, Json(content)))
    }

    #[cfg(not(feature = "db"))]
    Err(HttpError::new(
        StatusCode::NOT_IMPLEMENTED,
        "database required",
    ))
}

#[utoipa::path(
    get,
    path = "/ugc/content",
    tag = "ugc",
    params(ListContentQuery),
    responses(
        (status = 200, body = Vec<ModeratedContent>),
        (status = 401, description = "Unauthorized")
    ),
    security(("BearerAuth" = []))
)]
pub async fn list_content(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListContentQuery>,
) -> Result<Json<Vec<ModeratedContent>>, HttpError> {
    ensure_scope(&headers, "ugc:read")?;

    #[cfg(feature = "db")]
    {
        let store = &state.moderation_store;

        let state_enum = if let Some(s) = query.state {
            Some(
                s.parse::<ModerationState>()
                    .map_err(|_| HttpError::new(StatusCode::BAD_REQUEST, "invalid state"))?,
            )
        } else {
            None
        };

        let db_query = ModerationQuery {
            tenant_id: query.tenant_id,
            project_id: query.project_id,
            state: state_enum,
        };

        let content = store.list_content(db_query).await.map_err(|_| {
            HttpError::new(StatusCode::INTERNAL_SERVER_ERROR, "failed to list content")
        })?;

        Ok(Json(content))
    }

    #[cfg(not(feature = "db"))]
    Err(HttpError::new(
        StatusCode::NOT_IMPLEMENTED,
        "database required",
    ))
}

#[utoipa::path(
    put,
    path = "/ugc/content/{id}/state",
    tag = "ugc",
    params(("id" = Uuid, Path, description = "Content ID")),
    request_body = UpdateContentStateRequest,
    responses(
        (status = 200, body = ModeratedContent),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Content not found")
    ),
    security(("BearerAuth" = []))
)]
pub async fn update_content_state(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContentStateRequest>,
) -> Result<Json<ModeratedContent>, HttpError> {
    ensure_scope(&headers, "ugc:write")?;

    #[cfg(feature = "db")]
    {
        let store = &state.moderation_store;

        let new_state = req
            .state
            .parse::<ModerationState>()
            .map_err(|_| HttpError::new(StatusCode::BAD_REQUEST, "invalid state"))?;

        let content = store
            .update_content_state(id, new_state, req.reason)
            .await
            .map_err(|_| {
                HttpError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to update content",
                )
            })?;

        Ok(Json(content))
    }

    #[cfg(not(feature = "db"))]
    Err(HttpError::new(
        StatusCode::NOT_IMPLEMENTED,
        "database required",
    ))
}
