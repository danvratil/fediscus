// SPDX-FileCopyrightText: 2024 Daniel Vrátil <me@dvratil.cz>
// SPDX-FileCopyrightText: 2025 Daniel Vrátil <me@dvratil.cz>
//
// SPDX-License-Identifier: MIT

use activitypub_federation::{fetch::object_id::ObjectId, kinds::object::NoteType, traits::ActivityHandler};
use chrono::{DateTime, Utc};
use html_parser::Dom;
use serde::Deserialize;
use url::Url;

use crate::storage;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub r#type: String,
    pub href: Url,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub r#type: NoteType,
    pub id: ObjectId<storage::Note>,
    pub in_reply_to: Option<ObjectId<storage::Note>>,
    pub published: DateTime<Utc>,
    pub url: Url,
    pub attributed_to: ObjectId<storage::Account>,
    pub content: String,
    pub tag: Vec<Tag>,
}

impl Note {
    pub fn has_tag(&self) -> bool {
        self.tag.iter().any(|tag| {
            tag.r#type.eq_ignore_ascii_case("hashtag") && tag.name.eq_ignore_ascii_case("#fediscus")
        })
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

        Ok(links)
    }
}
