use barq_proto::{
    barqvault::v1::{
        ProtoDeleteRequest, ProtoFetchRequest, ProtoIngestRequest, ProtoPingRequest,
        ProtoSearchRequest, ProtoStatsRequest,
    },
    BarqVaultClient,
};
use barq_types::{BarqError, BarqResult, IngestRequest, SearchRequest, SearchResult};
use tonic::transport::Channel;
use uuid::Uuid;

/// High-level Rust client for BarqVault gRPC API.
#[derive(Clone)]
pub struct BarqClient {
    inner: BarqVaultClient<Channel>,
}

impl BarqClient {
    /// Connect to a BarqVault server given a gRPC endpoint URL (e.g., "http://127.0.0.1:50051").
    pub async fn connect(url: impl Into<String>) -> BarqResult<Self> {
        let endpoint = url.into();
        let channel = Channel::from_shared(endpoint)
            .map_err(|e| BarqError::ProviderError(format!("Invalid URL: {}", e)))?
            .connect()
            .await
            .map_err(|e| BarqError::ProviderError(format!("Connection failed: {}", e)))?;

        Ok(Self {
            inner: BarqVaultClient::new(channel),
        })
    }

    /// Ingest a single file or chunk.
    pub async fn ingest(&mut self, request: IngestRequest) -> BarqResult<Uuid> {
        let proto_req: ProtoIngestRequest = request.into();

        let response = self
            .inner
            .ingest(tonic::Request::new(proto_req))
            .await
            .map_err(|e| BarqError::ProviderError(format!("gRPC ingest err: {}", e)))?
            .into_inner();

        if !response.success {
            return Err(BarqError::ProviderError(format!("Ingest failed: {}", response.error)));
        }

        Uuid::parse_str(&response.id).map_err(|_| BarqError::ProviderError("Invalid UUID returned".to_string()))
    }

    /// Search the multimodal index.
    pub async fn search(&mut self, request: SearchRequest) -> BarqResult<Vec<SearchResult>> {
        let proto_req: ProtoSearchRequest = request.into();

        let response = self
            .inner
            .search(tonic::Request::new(proto_req))
            .await
            .map_err(|e| BarqError::ProviderError(format!("gRPC search err: {}", e)))?
            .into_inner();

        let mut results = Vec::new();
        for hit in response.results {
            let metadata = if hit.metadata_json.is_empty() {
                serde_json::Value::Object(Default::default())
            } else {
                serde_json::from_str(&hit.metadata_json)
                    .unwrap_or_else(|_| serde_json::Value::Object(Default::default()))
            };

            let modality = hit.modality.parse().unwrap_or(barq_types::Modality::Text);

            results.push(SearchResult {
                id: Uuid::parse_str(&hit.id).unwrap_or_default(),
                summary: hit.summary,
                filename: if hit.filename.is_empty() { None } else { Some(hit.filename) },
                modality,
                score: hit.score,
                has_payload: hit.has_payload,
                metadata,
            });
        }

        Ok(results)
    }

    /// Delete a record by ID.
    pub async fn delete(&mut self, id: Uuid) -> BarqResult<()> {
        let req = ProtoDeleteRequest { id: id.to_string() };
        let response = self
            .inner
            .delete(tonic::Request::new(req))
            .await
            .map_err(|e| BarqError::ProviderError(format!("gRPC delete err: {}", e)))?
            .into_inner();

        if !response.success {
            Err(BarqError::ProviderError(response.error))
        } else {
            Ok(())
        }
    }

    /// Ping the server to check health and get version/uptime.
    pub async fn ping(&mut self) -> BarqResult<(String, u64)> {
        let res = self
            .inner
            .ping(tonic::Request::new(ProtoPingRequest {}))
            .await
            .map_err(|e| BarqError::ProviderError(e.to_string()))?
            .into_inner();
        Ok((res.version, res.uptime_secs))
    }
}
