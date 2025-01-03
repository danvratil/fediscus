// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use axum::extract::{Json, State};
use axum::http::StatusCode;

use crate::types::{CommentsRequest, CommentsResponse, GetCountRequest, GetCountResponse};
use crate::AppState;


pub async fn post_comments(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<CommentsRequest>,
) -> Result<Json<CommentsResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn get_count(
    State(_state): State<Arc<AppState>>,
    Json(_request): Json<GetCountRequest>,
) -> Result<Json<GetCountResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
    /*
    let (mut counts, misses) = db::get_counts(&state.db, &request.posts).await?;

     misses.iter().for_each(|post| {
        let fedi_post_id = fediverse::locate_users_post(&post).await;
        let comments = fediverse::get_comments(&fedi_post_id).await;
        db::store_comments(&comments).await;
        counts.insert(post.url, comments.len());
    });

    Ok(Json(GetCountResponse { counts }))
    */
}
