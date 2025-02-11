// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use std::sync::LazyLock;

use activitypub_federation::{fetch::object_id::ObjectId, kinds::object::NoteType};
use chrono::{DateTime, Utc};
use html_parser::Dom;
use regex::Regex;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::storage;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub r#type: String,
    pub href: Url,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub r#type: NoteType,
    pub id: ObjectId<storage::Note>,
    pub in_reply_to: Option<ObjectId<storage::Note>>,
    pub published: Option<DateTime<Utc>>,
    pub url: Option<Url>,
    pub attributed_to: ObjectId<storage::Account>,
    pub content: String,
    #[serde(default)]
    pub tag: Vec<Tag>,
}

impl Note {
    pub fn has_tag(&self) -> bool {
        self.tag.iter().any(|tag| {
            tag.r#type.eq_ignore_ascii_case("hashtag") && tag.name.eq_ignore_ascii_case("#fediscus")
        }) || self.content.contains("#fediscus")
    }

    /// Retrieves all the hyperlinks (URLs) from the HTML content.
    pub fn get_links(&self) -> Result<Vec<Url>, html_parser::Error> {
        let dom = Dom::parse(&self.content)?;
        let mut links = Vec::new();
        let mut stack = Vec::new();

        for node in dom.children.iter() {
            stack.push(node);
        }

        while let Some(node) = stack.pop() {
            if let html_parser::Node::Element(element) = node {
                if element.name.eq_ignore_ascii_case("a") {
                    if let Some(Some(href)) = element.attributes.get("href") {
                        if let Ok(url) = Url::parse(href) {
                            links.push(url);
                        }
                    }
                }
                for child in &element.children {
                    stack.push(child);
                }
            }
        }

        // Fallback to simply looking for an HTTP(s) URL in the content
        static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"https?://[^\s]+").unwrap());
        RE.find_iter(&self.content)
            .filter_map(|m| Url::parse(m.as_str()).ok())
            .for_each(|url| links.push(url));

        Ok(links)
    }
}
