// SPDX-License-Identifier: GPL-3.0

//! Web search functionality for augmenting AI responses.
//!
//! Uses DuckDuckGo's instant answer API for quick searches.

use serde::Deserialize;

const SEARCH_URL: &str = "https://api.duckduckgo.com/";

#[derive(Debug, Deserialize)]
struct DdgResponse {
    #[serde(rename = "Abstract")]
    abstract_text: String,
    #[serde(rename = "AbstractSource")]
    abstract_source: String,
    #[serde(rename = "AbstractURL")]
    abstract_url: String,
    #[serde(rename = "RelatedTopics")]
    related_topics: Vec<RelatedTopic>,
}

#[derive(Debug, Deserialize)]
struct RelatedTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
}

/// Search result from DuckDuckGo.
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub summary: String,
    pub source: String,
    pub url: String,
    pub related: Vec<String>,
}

/// Perform a web search using DuckDuckGo's instant answer API.
pub async fn search(query: &str) -> Option<SearchResult> {
    let client = reqwest::Client::new();

    let response = client
        .get(SEARCH_URL)
        .query(&[
            ("q", query),
            ("format", "json"),
            ("no_html", "1"),
            ("skip_disambig", "1"),
        ])
        .send()
        .await
        .ok()?;

    let ddg: DdgResponse = response.json().await.ok()?;

    // If we have an abstract, use it
    if !ddg.abstract_text.is_empty() {
        return Some(SearchResult {
            summary: ddg.abstract_text,
            source: ddg.abstract_source,
            url: ddg.abstract_url,
            related: ddg
                .related_topics
                .iter()
                .filter_map(|t| t.text.clone())
                .take(3)
                .collect(),
        });
    }

    // Otherwise try to get info from related topics
    let related: Vec<String> = ddg
        .related_topics
        .iter()
        .filter_map(|t| t.text.clone())
        .take(5)
        .collect();

    if !related.is_empty() {
        return Some(SearchResult {
            summary: related.join("\n\n"),
            source: "DuckDuckGo".to_string(),
            url: format!("https://duckduckgo.com/?q={}", urlencoding::encode(query)),
            related: vec![],
        });
    }

    None
}

/// Format search results for inclusion in context.
pub fn format_results(result: &SearchResult) -> String {
    let mut output = format!("## Web Search Results\n\n{}", result.summary);

    if !result.source.is_empty() {
        output.push_str(&format!("\n\nSource: {} ({})", result.source, result.url));
    }

    if !result.related.is_empty() {
        output.push_str("\n\nRelated info:\n");
        for item in &result.related {
            output.push_str(&format!("- {}\n", item));
        }
    }

    output
}
