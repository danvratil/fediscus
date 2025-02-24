use aide::{axum::IntoApiResponse, openapi::OpenApi};
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetComments {
    pub url: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CommentResponse {
    pub url: String,
}

pub async fn get_comments(Json(request): Json<GetComments>) -> impl IntoApiResponse {
    Json(CommentResponse { url: request.url })
}
