#!/usr/bin/env bash
set -e

echo "BarqVault Setup Script"
echo "======================"

MISSING_DEPS=()

command -v clang >/dev/null 2>&1 || MISSING_DEPS+=("clang")
command -v tesseract >/dev/null 2>&1 || MISSING_DEPS+=("tesseract")
command -v ffmpeg >/dev/null 2>&1 || MISSING_DEPS+=("ffmpeg")

# Check for libs via pkg-config if possible (macOS/Linux)
if command -v pkg-config >/dev/null 2>&1; then
    pkg-config --exists liblzma || MISSING_DEPS+=("liblzma")
    pkg-config --exists liblz4 || MISSING_DEPS+=("liblz4")
else
    MISSING_DEPS+=("liblzma" "liblz4") # Assume missing if no pkg-config
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "Missing dependencies: ${MISSING_DEPS[*]}"
    echo ""
    echo "Please install them using your package manager:"
    echo "  Ubuntu/Debian : sudo apt install clang liblzma-dev liblz4-dev tesseract-ocr ffmpeg"
    echo "  macOS         : brew install llvm xz lz4 tesseract ffmpeg"
    echo "  Arch Linux    : sudo pacman -S clang xz lz4 tesseract tesseract-data-eng ffmpeg"
    echo ""
    echo "Note: Whisper binary (if using local STT) must be installed manually: https://github.com/ggerganov/whisper.cpp"
    exit 1
fi

echo "All system dependencies found!"
echo "Building workspace..."
cargo build --workspace

echo ""
echo "Setup complete! You can now run:"
echo "  cargo run -p barq-server"
echo "  cargo run -p barq-cli -- help"
