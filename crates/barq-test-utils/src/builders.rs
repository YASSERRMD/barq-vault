use barq_types::{BarqRecord, Modality, StorageMode};
use uuid::Uuid;

pub struct BarqRecordBuilder {
    record: BarqRecord,
}

impl Default for BarqRecordBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BarqRecordBuilder {
    pub fn new() -> Self {
        let mut record = BarqRecord::default();
        record.id = Uuid::new_v4();
        let dim = 384;
        record.embedding = random_embedding(dim);
        record.embedding_dim = dim as u32;
        record.summary = "mock summary".to_string();
        record.modality = Modality::Text;
        record.storage_mode = StorageMode::TextOnly;
        Self { record }
    }

    pub fn modality(mut self, modality: Modality) -> Self {
        self.record.modality = modality;
        self
    }

    pub fn storage_mode(mut self, storage_mode: StorageMode) -> Self {
        self.record.storage_mode = storage_mode;
        self
    }

    pub fn summary(mut self, summary: String) -> Self {
        self.record.summary = summary;
        self
    }

    pub fn embedding(mut self, embedding: Vec<f32>) -> Self {
        self.record.embedding_dim = embedding.len() as u32;
        self.record.embedding = embedding;
        self
    }

    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.record.original_size = payload.len() as u64;
        self.record.compressed_size = payload.len() as u64; // mock
        self.record.compressed_payload = Some(payload);
        self
    }

    pub fn filename(mut self, filename: String) -> Self {
        self.record.filename = Some(filename);
        self
    }

    pub fn chunk(mut self, index: u32, total: u32) -> Self {
        self.record.chunk_index = index;
        self.record.total_chunks = total;
        self
    }

    pub fn build(self) -> BarqRecord {
        self.record
    }
}

// Simple LCG PRNG for vector mocking without extra dependencies
struct SimplePrng {
    state: u64,
}
impl SimplePrng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_f32(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let val = (self.state >> 32) as u32;
        (val as f32) / (u32::MAX as f32)
    }
    // Box-Muller transform for normal distribution
    fn next_gaussian(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-7); // avoid 0
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

pub fn random_embedding(dim: usize) -> Vec<f32> {
    // Seed using random uuid bytes to ensure varying embeddings across test calls
    let seed_bytes = Uuid::new_v4().as_bytes().to_vec();
    let seed = u64::from_le_bytes(seed_bytes[0..8].try_into().unwrap());
    
    let mut prng = SimplePrng::new(seed);
    let mut vec = Vec::with_capacity(dim);
    let mut norm_sq = 0.0;
    
    for _ in 0..dim {
        let val = prng.next_gaussian();
        vec.push(val);
        norm_sq += val * val;
    }
    
    let norm = norm_sq.sqrt().max(1e-7);
    for v in &mut vec {
        *v /= norm;
    }
    vec
}

pub fn similar_embedding(base: &[f32], noise: f32) -> Vec<f32> {
    let seed_bytes = Uuid::new_v4().as_bytes().to_vec();
    let seed = u64::from_le_bytes(seed_bytes[0..8].try_into().unwrap());
    let mut prng = SimplePrng::new(seed);
    
    let mut vec = Vec::with_capacity(base.len());
    let mut norm_sq = 0.0;
    
    for val in base {
        let noise_val = prng.next_gaussian() * noise;
        let new_val = val + noise_val;
        vec.push(new_val);
        norm_sq += new_val * new_val;
    }
    
    let norm = norm_sq.sqrt().max(1e-7);
    for v in &mut vec {
        *v /= norm;
    }
    vec
}

pub fn orthogonal_embedding(dim: usize) -> Vec<f32> {
    let mut vec = vec![0.0; dim];
    if dim > 0 {
        vec[0] = 1.0;
    }
    vec
}
