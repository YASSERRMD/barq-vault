use rocksdb::{BlockBasedOptions, ColumnFamilyDescriptor, Options};

/// Column family names for the barq-vault RocksDB database.
pub const CF_RECORDS: &str = "records";
pub const CF_PAYLOADS: &str = "payloads";
pub const CF_METADATA: &str = "metadata";
pub const CF_INDEX_META: &str = "index_meta";

/// Build base RocksDB options tuned for barq-vault's write-heavy workload.
pub fn build_db_options() -> Options {
    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.create_missing_column_families(true);
    opts.set_compression_type(rocksdb::DBCompressionType::Zstd);
    opts.increase_parallelism(num_cpus::get() as i32);
    opts.set_max_write_buffer_number(4);
    opts.set_write_buffer_size(128 * 1024 * 1024); // 128 MB
    opts.set_max_background_jobs(4);

    // Bloom filter for faster point lookups
    let mut block_opts = BlockBasedOptions::default();
    block_opts.set_bloom_filter(10.0, false);
    opts.set_block_based_table_factory(&block_opts);

    opts
}

/// Build column family descriptors for all 4 CFs with default options.
pub fn build_cf_descriptors() -> Vec<ColumnFamilyDescriptor> {
    let cf_opts = Options::default();
    vec![
        ColumnFamilyDescriptor::new(CF_RECORDS, cf_opts.clone()),
        ColumnFamilyDescriptor::new(CF_PAYLOADS, cf_opts.clone()),
        ColumnFamilyDescriptor::new(CF_METADATA, cf_opts.clone()),
        ColumnFamilyDescriptor::new(CF_INDEX_META, cf_opts),
    ]
}
