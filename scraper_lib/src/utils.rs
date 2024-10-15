use url::Url;

pub fn normalize_url(input: &str) -> Option<String> {
    // Try parsing the URL as is
    if let Ok(url) = Url::parse(input) {
        return Some(normalize_url_parts(url));
    }

    // Try adding "https://" prefix
    let https_prefixed_input = format!("https://{}", input);
    if let Ok(url) = Url::parse(&https_prefixed_input) {
        return Some(normalize_url_parts(url));
    }

    // Try adding "http://" prefix
    let http_prefixed_input = format!("http://{}", input);
    if let Ok(url) = Url::parse(&http_prefixed_input) {
        return Some(normalize_url_parts(url));
    }

    // If all parsing attempts fail, return None
    None
}

fn normalize_url_parts(mut url: Url) -> String {
    // Remove unnecessary components for consistent comparison
    url.set_fragment(None);
    url.set_username("").ok();
    url.set_password(None).ok();
    url.set_query(None);

    // Optionally, normalize "www." prefix
    if let Some(original_host) = url.host_str() {
        let trimmed_host = original_host.trim_start_matches("www.").to_owned();
        url.set_host(Some(&trimmed_host)).ok();
    }

    // Return the normalized URL
    url.to_string()
}