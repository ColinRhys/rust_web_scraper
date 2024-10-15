pub mod manager;
pub mod utils;

use reqwest::Client;
use scraper::{Html, Selector};
use dashmap::DashSet;
use url::Url;
use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}};
use log::{info, error, debug};
use utils::normalize_url;

pub struct Scraper {
    client: Client,
    visited_links: Arc<DashSet<String>>,
}

impl Scraper {
    pub fn new() -> Self {
        Scraper {
            client: Client::builder()
                .cookie_store(true)
                .build()
                .unwrap(),
            visited_links: Arc::new(DashSet::new()),
        }
    }

    pub async fn scrape(&self, start_url: &str, links: Arc<Mutex<HashMap<String, Vec<String>>>>) {
        debug!("Starting to scrape: {}", start_url);

        // Normalize the start URL
        let normalized_start_url = match normalize_url(start_url) {
            Some(url) => url,
            None => {
                error!("Invalid start URL: {}", start_url);
                return;
            }
        };

        let mut to_visit = VecDeque::new();
        to_visit.push_back(normalized_start_url.clone());

        // Parse the base domain from normalized_start_url
        let base_domain = match Url::parse(&normalized_start_url) {
            Ok(url) => url.domain().map(|s| s.to_string()),
            Err(err) => {
                error!("Invalid start URL {}: {}", normalized_start_url, err);
                return;
            }
        };

        if base_domain.is_none() {
            error!("Could not extract domain from {}", normalized_start_url);
            return;
        }
        let base_domain = base_domain.unwrap();

        while let Some(url) = to_visit.pop_front() {
            // Normalize the URL
            let normalized_url = match normalize_url(&url) {
                Some(url) => url,
                None => continue,
            };

            if self.visited_links.contains(&normalized_url) {
                continue;
            }
            self.visited_links.insert(normalized_url.clone());

            info!("Visiting - {}", normalized_url);

            // Fetch the page
            let res = match self.client.get(&normalized_url).send().await {
                Ok(res) => res,
                Err(err) => {
                    error!("Error fetching {}: {}", normalized_url, err);
                    continue;
                }
            };

            // Read the body
            let body = match res.text().await {
                Ok(text) => text,
                Err(err) => {
                    error!("Error reading body {}: {}", normalized_url, err);
                    continue;
                }
            };

            // Parse the document
            let document = Html::parse_document(&body);
            let selector = Selector::parse("a").unwrap();

            for element in document.select(&selector) {
                if let Some(link) = element.value().attr("href") {
                    // Resolve the link to an absolute URL
                    let absolute_link = resolve_link(&normalized_url, link);

                    // Normalize the absolute link
                    let normalized_link = match normalize_url(&absolute_link) {
                        Some(url) => url,
                        None => continue,
                    };

                    if is_valid_link(&normalized_link, &base_domain) {
                        // Save the link
                        {
                            let mut links_lock = links.lock().unwrap();
                            let entry = links_lock.entry(normalized_start_url.clone()).or_insert_with(Vec::new);
                            if !entry.contains(&normalized_link) {
                                entry.push(normalized_link.clone());
                            }
                        }

                        if !self.visited_links.contains(&normalized_link) {
                            to_visit.push_back(normalized_link);
                        }
                    }
                }
            }
        }

        info!("Finished scraping {}", normalized_start_url);
    }
}

impl Clone for Scraper {
    fn clone(&self) -> Self {
        Scraper {
            client: self.client.clone(),
            visited_links: Arc::clone(&self.visited_links),
        }
    }
}

fn is_valid_link(link: &str, base_domain: &str) -> bool {
    if link.starts_with('#') || link.starts_with("mailto:") {
        return false;
    }

    match Url::parse(link) {
        Ok(url) => {
            if let Some(domain) = url.domain() {
                domain == base_domain
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

fn resolve_link(base: &str, link: &str) -> String {
    match Url::parse(base) {
        Ok(base_url) => match base_url.join(link) {
            Ok(url) => url.to_string(),
            Err(_) => link.to_string(),
        },
        Err(_) => link.to_string(),
    }
}