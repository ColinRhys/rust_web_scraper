use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::info;
use tokio::task::JoinHandle;
use crate::Scraper;

pub struct ScraperManager {
    tasks: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    links: Arc<Mutex<HashMap<String, Vec<String>>>>,
    statuses: Arc<Mutex<HashMap<String, String>>>,
}

impl ScraperManager {
    pub fn new() -> Self {
        ScraperManager {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            links: Arc::new(Mutex::new(HashMap::new())),
            statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_scraping(&self, url: String) {
        let scraper = Scraper::new();
        let links = Arc::clone(&self.links);
        let statuses = Arc::clone(&self.statuses);

        // Clone url for use in the closure
        let url_for_task = url.clone();

        let handle = tokio::spawn(async move {
            scraper.scrape(&url_for_task, links).await;
            info!("Finished scraping {}", url_for_task);
            // Update status to "finished" for the URL
            let mut statuses = statuses.lock().unwrap();
            statuses.insert(url_for_task, "finished".to_string());
        });

        // Use the original url here
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(url.clone(), handle);

        let mut statuses = self.statuses.lock().unwrap();
        statuses.insert(url, "crawling".to_string());
    }

    pub fn stop_scraping(&self, url: &str) {
        let tasks = self.tasks.lock().unwrap();
        if let Some(handle) = tasks.get(url) {
            handle.abort();
            // Update the status to "stopped"
            self.statuses.lock().unwrap().insert(url.to_string(), "stopped".to_string());
            info!("Stopped scraping {}", url);
        }
    }

    pub fn list_tasks(&self) -> Vec<(String, String)> {
        let statuses = self.statuses.lock().unwrap();
        statuses.iter().map(|(url, status)| (url.clone(), status.clone())).collect()
    }

    pub fn get_links(&self, url: &str) -> Option<Vec<String>> {
        let links = self.links.lock().unwrap();
        links.get(url).cloned()
    }
}