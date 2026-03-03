#!/usr/bin/env bash
set -e

echo "BarqVault CLI Benchmark Script"
echo "=============================="

# Ensure server is running (assumes background or separate terminal)
if ! curl -s http://127.0.0.1:8080/api/v1/ping > /dev/null; then
  echo "Error: Server is not running. Start it with 'cargo run --release -p barq-server'"
  exit 1
fi

CLI="cargo run --release -p barq-cli --"
DATA_DIR="tests/data"
mkdir -p "$DATA_DIR"

# Generate some dummy files
echo "Creating dummy files..."
for i in {1..10}; do
  echo "This is document number $i. It contains some text that we want to embed and search for later." > "$DATA_DIR/doc_$i.txt"
done

# We fake image/audio ingestion by just uploading text files but asserting different modalities
# (for the MVP bench script, we just care about ingestion throughput and basic integration)
mv "$DATA_DIR/doc_9.txt" "$DATA_DIR/image_9.jpg"
mv "$DATA_DIR/doc_10.txt" "$DATA_DIR/audio_10.wav"

files=("$DATA_DIR"/*)
echo "Starting ingestion of 10 files..."
START_TIME=$(date +%s)

for file in "${files[@]}"; $do
  modality="text"
  if [[ "$file" == *.jpg ]]; then modality="image"; fi
  if [[ "$file" == *.wav ]]; then modality="audio"; fi
  
  # For local benchmark, we skip heavy embedding/llm processing by leaving those dummy or mocked by the server config if not provided
  output=$($CLI ingest "$file" --modality "$modality" 2>&1)
  if [[ $output == *"successful"* ]]; then
      echo "  [OK] Ingested $file"
  else
      echo "  [FAIL] $file"
      echo "$output"
  fi
done

END_TIME=$(date +%s)
echo "Ingestion completed in $((END_TIME - START_TIME)) seconds."

echo "\nStarting Search Benchmark (5 queries)..."
queries=("document number 1" "some text" "embed" "search" "number 5")

START_TIME=$(date +%s)
for q in "${queries[@]}"; do
  echo "\nQuery: '$q'"
  $CLI search "$q" --top-k 3
done
END_TIME=$(date +%s)

echo "\nSearch completed in $((END_TIME - START_TIME)) seconds."

echo "\nCleaning up test data..."
rm -rf "$DATA_DIR"

echo "Benchmark finished."
