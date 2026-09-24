// Configuration settings for Omni-Graph orchestrator
// Implements: R-024 (Docker DNS hostnames)

use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub surreal_url: String,
    pub surreal_user: String,
    pub surreal_pass: String,
    pub surreal_ns: String,
    pub surreal_db: String,
    pub tei_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let surreal_url = env::var("SURREAL_URL")
            .unwrap_or_else(|_| "http://graph-db:8000".to_string());

        let surreal_user = env::var("SURREAL_USER")
            .unwrap_or_else(|_| "root".to_string());

        let surreal_pass = env::var("SURREAL_PASS")
            .unwrap_or_else(|_| "root".to_string());

        let surreal_ns = env::var("SURREAL_NS")
            .unwrap_or_else(|_| "omni".to_string());

        let surreal_db = env::var("SURREAL_DB")
            .unwrap_or_else(|_| "graph".to_string());

        let tei_url = env::var("TEI_URL")
            .unwrap_or_else(|_| "http://embedding-engine:80".to_string());

        Self {
            port,
            surreal_url,
            surreal_user,
            surreal_pass,
            surreal_ns,
            surreal_db,
            tei_url,
        }
    }
}
