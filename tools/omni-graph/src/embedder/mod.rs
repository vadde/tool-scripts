// HuggingFace Text Embeddings Inference (TEI) client
// Implements: R-004 (TEI 384-dim vector embedding)
// Target model: BAAI/bge-small-en-v1.5

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct EmbedderClient {
    base_url: String,
    client: Client,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    inputs: Vec<&'a str>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct TeiHealthResponse {
    pub status: Option<String>,
}

impl EmbedderClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { base_url, client }
    }

    /// Health check for TEI inference engine
    pub async fn health(&self) -> bool {
        let url = format!("{}/health", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) => resp.status().is_success(),
            Err(e) => {
                debug!("TEI healthcheck failed at {}: {}", url, e);
                false
            }
        }
    }

    /// Generate 384-dimensional vector embeddings in batch
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, String> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let url = format!("{}/embed", self.base_url);
        let mut all_embeddings = Vec::with_capacity(texts.len());

        // TEI maximum batch size is 32 on CPU / ARM64
        for chunk in texts.chunks(32) {
            let payload = EmbedRequest {
                inputs: chunk.to_vec(),
            };

            let response = self
                .client
                .post(&url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Failed to connect to TEI at {}: {}", url, e))?;

            if !response.status().is_success() {
                let status = response.status();
                let err_body = response.text().await.unwrap_or_default();
                return Err(format!("TEI returned error status {}: {}", status, err_body));
            }

            let chunk_embeddings: Vec<Vec<f32>> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse TEI response vectors: {}", e))?;

            // Validate dimension matches 384
            for (i, emb) in chunk_embeddings.iter().enumerate() {
                if emb.len() != 384 {
                    return Err(format!(
                        "Embedding dimension mismatch at index {}: expected 384, got {}",
                        i,
                        emb.len()
                    ));
                }
            }

            all_embeddings.extend(chunk_embeddings);
        }

        Ok(all_embeddings)
    }

    /// Generate vector embedding for a single text prompt
    pub async fn embed_single(&self, text: &str) -> Result<Vec<f32>, String> {
        let mut results = self.embed_batch(&[text]).await?;
        results
            .pop()
            .ok_or_else(|| "No embedding returned for single text".to_string())
    }
}
