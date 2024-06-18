use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct NavigationLink {
    pub href: Box<str>,
    pub title: Box<str>,
}

static NAVIGATION_LINKS: Lazy<Mutex<Arc<[NavigationLink]>>> =
    Lazy::new(|| Mutex::new(Arc::from([])));

pub fn set_navigation_links(links: Arc<[NavigationLink]>) -> Result<()> {
    let mut value = NAVIGATION_LINKS
        .lock()
        .map_err(|e| anyhow!("Failed to acquire lock: {:?}", e))?;
    *value = links;
    Ok(())
}

pub fn get_navigation_links() -> Arc<[NavigationLink]> {
    NAVIGATION_LINKS
        .lock()
        .map(|value| value.clone())
        .unwrap_or(Arc::from([]))
}
