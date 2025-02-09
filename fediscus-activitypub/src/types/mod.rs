// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, time};
use url::Url;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Post {
    pub url: Url,
    pub instance: Url,
    pub author: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCountRequest {
    pub posts: Vec<Post>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetCountResponse {
    counts: HashMap<Url, usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommentsRequest {
    pub post: Url,
    pub instance: Url,
    pub author: String,

    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Emoji {
    pub shortcode: String,
    pub url: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Author {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub url: Url,
    pub avatar: Option<Url>,
    pub emojis: Vec<Emoji>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: Url,
    pub content: String,
    pub in_reply_to_id: Option<String>,
    pub in_reply_to_account_id: Option<String>,
    pub published: time::SystemTime,
    pub url: Url,
    pub likes_count: usize,
    pub shares_count: usize,
    pub author: Author,
    pub emojis: Vec<Emoji>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CommentsResponse {
    pub post: Url,
    pub total: usize,
    pub page: usize,

    pub comments: Vec<Comment>,
}
