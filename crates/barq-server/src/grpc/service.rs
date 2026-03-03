use std::sync::Arc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use barq_proto::barqvault::v1::barq_vault_server::BarqVault;
use barq_proto::barqvault::v1::{
    ProtoChunkDownload, ProtoChunkUpload, ProtoDeleteRequest, ProtoDeleteResponse,
    ProtoFetchRequest, ProtoIngestRequest, ProtoIngestResponse, ProtoPingRequest,
    ProtoPingResponse, ProtoSearchRequest, ProtoSearchResult, ProtoSearchResponse,
    ProtoStatsRequest, ProtoStatsResponse,
};
use barq_types::{IngestRequest, SearchRequest};

use crate::state::AppState;

pub struct BarqVaultService {
    pub state: Arc<AppState>,
}

impl BarqVaultService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl BarqVault for BarqVaultService {
    async fn ingest(
        &self,
        request: Request<ProtoIngestRequest>,
    ) -> Result<Response<ProtoIngestResponse>, Status> {
        let proto_req = request.into_inner();
        let domain_req: IngestRequest = proto_req
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("Invalid request: {}", e)))?;

        let records = self
            .state
            .ingest_pipeline
            .run(domain_req)
            .await
            .map_err(|e| Status::internal(format!("Pipeline error: {}", e)))?;

        let mut index = self.state.index.write().await;
        let mut total_compressed = 0u64;
        let mut first_id = Uuid::nil();

        for record in records {
            if first_id.is_nil() {
                first_id = record.id;
            }

            if let Some(payload) = &record.compressed_payload {
                self.state
                    .store
                    .put_payload(record.id, payload)
                    .map_err(|e| Status::internal(format!("Store payload error: {}", e)))?;
                total_compressed += payload.len() as u64;
            }

            self.state
                .store
                .put_metadata(record.id, &record.metadata)
                .map_err(|e| Status::internal(format!("Store metadata error: {}", e)))?;

            self.state
                .store
                .put_record(&record)
                .map_err(|e| Status::internal(format!("Store record error: {}", e)))?;

            index.index_new(&record);
        }

        Ok(Response::new(ProtoIngestResponse {
            id: first_id.to_string(),
            success: true,
            compressed_size: total_compressed,
            compression_ratio: Default::default(), // To be precise: ratio of total_orig / total_comp
            error: String::new(),
        }))
    }

    async fn ingest_stream(
        &self,
        _request: Request<tonic::Streaming<ProtoChunkUpload>>,
    ) -> Result<Response<ProtoIngestResponse>, Status> {
        Err(Status::unimplemented("IngestStream is not yet implemented"))
    }

    async fn search(
        &self,
        request: Request<ProtoSearchRequest>,
    ) -> Result<Response<ProtoSearchResponse>, Status> {
        let t0 = std::time::Instant::now();
        let proto_req = request.into_inner();
        let domain_req: SearchRequest = proto_req
            .try_into()
            .map_err(|e| Status::invalid_argument(format!("Invalid search request: {}", e)))?;

        let search_params = barq_index::SearchParams {
            query_embedding: domain_req.query_embedding,
            query_text: domain_req.query_text,
            vector_weight: domain_req.vector_weight,
            top_k: domain_req.top_k,
            modality_filter: domain_req.modality_filter.map(|m| m.to_string()),
            metadata_filters: domain_req.metadata_filters,
        };

        let index = self.state.index.read().await;
        let results = index
            .search(search_params)
            .map_err(|e| Status::internal(format!("Search error: {}", e)))?;

        let mut out_results = Vec::new();
        for (id, score) in results {
            if let Ok(Some(record)) = self.state.store.get_record(id) {
                let mut p = ProtoSearchResult::from(record);
                p.score = score;
                out_results.push(p);
            }
        }

        let took_ms = t0.elapsed().as_millis() as u64;

        Ok(Response::new(ProtoSearchResponse {
            total_found: out_results.len() as u32,
            results: out_results,
            took_ms,
        }))
    }

    type SearchStreamStream = ReceiverStream<Result<ProtoSearchResult, Status>>;

    async fn search_stream(
        &self,
        _request: Request<tonic::Streaming<ProtoSearchRequest>>,
    ) -> Result<Response<Self::SearchStreamStream>, Status> {
        Err(Status::unimplemented("SearchStream is not yet implemented"))
    }

    type FetchStream = ReceiverStream<Result<ProtoChunkDownload, Status>>;

    async fn fetch(
        &self,
        request: Request<ProtoFetchRequest>,
    ) -> Result<Response<Self::FetchStream>, Status> {
        let req = request.into_inner();
        let id_str = req.id;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;

        if !self.state.store.record_exists(id) {
            return Err(Status::not_found("Record not found"));
        }

        let payload = self
            .state
            .store
            .get_payload(id)
            .map_err(|e| Status::internal(format!("Fetch payload err: {}", e)))?
            .ok_or_else(|| Status::not_found("Payload deleted or not stored"))?;

        let (tx, rx) = tokio::sync::mpsc::channel(4);
        let data_len = payload.len();

        tokio::spawn(async move {
            let chunk_size = 1024 * 1024; // 1MB chunks
            let mut offset = 0;
            let mut c_idx = 0;
            while offset < payload.len() {
                let end = (offset + chunk_size).min(payload.len());
                let slice = &payload[offset..end];
                let msg = ProtoChunkDownload {
                    data: slice.to_vec(),
                    chunk_index: c_idx,
                    total_bytes: data_len as u64,
                };
                if tx.send(Ok(msg)).await.is_err() {
                    break;
                }
                offset = end;
                c_idx += 1;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn delete(
        &self,
        request: Request<ProtoDeleteRequest>,
    ) -> Result<Response<ProtoDeleteResponse>, Status> {
        let req = request.into_inner();
        let id = Uuid::parse_str(&req.id)
            .map_err(|_| Status::invalid_argument("Invalid UUID format"))?;

        if let Ok(Some(record)) = self.state.store.get_record(id) {
            self.state
                .store
                .delete_record(id)
                .map_err(|e| Status::internal(format!("Delete error: {}", e)))?;
            let mut index = self.state.index.write().await;
            index.remove(id, &record);
        } else {
            return Err(Status::not_found("Record not found"));
        }

        Ok(Response::new(ProtoDeleteResponse {
            success: true,
            error: String::new(),
        }))
    }

    async fn ping(
        &self,
        _request: Request<ProtoPingRequest>,
    ) -> Result<Response<ProtoPingResponse>, Status> {
        let uptime_secs = self.state.started_at.elapsed().as_secs();
        Ok(Response::new(ProtoPingResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs,
        }))
    }

    async fn stats(
        &self,
        _request: Request<ProtoStatsRequest>,
    ) -> Result<Response<ProtoStatsResponse>, Status> {
        // Very rough approximations -- full stats would require DB scan or maintained counters
        // For MVP we just return uptime and defaults
        let uptime_secs = self.state.started_at.elapsed().as_secs();
        Ok(Response::new(ProtoStatsResponse {
            total_records: 0,
            total_payload_bytes: 0,
            total_compressed_bytes: 0,
            avg_compression_ratio: 0.0,
            uptime_secs,
            index_size: 0,
        }))
    }
}
