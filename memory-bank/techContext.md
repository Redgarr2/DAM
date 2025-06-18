# Technical Context & Dependencies

## Core Technologies

### Rust Ecosystem
- **Edition**: 2021
- **MSRV**: 1.75+ (required for latest candle)
- **Workspace**: Multi-crate structure for modularity

### UI Framework
- **Tauri 2.0**: Cross-platform desktop app framework
- **Frontend**: HTML/CSS/JS with embedded Bevy WASM
- **3D Rendering**: Bevy 0.12+ compiled to WebGL2/WASM
- **WebView**: System webview with message passing

### AI/ML Stack
- **candle**: Pure Rust ML framework for local inference
- **whisper.cpp**: C++ transcription via FFI bindings
- **Models Required**:
  - CLIP ViT-B/32 or ViT-L/14 (.safetensors)
  - BLIP for image captioning (.safetensors)  
  - Stable Diffusion 2.1 complete pipeline (.safetensors)
  - Whisper base/small model (.ggml)

### Search & Storage
- **Tantivy**: Fast full-text search engine
- **sled**: Embedded key-value store for metadata
- **git2**: Git repository management
- **Vector Storage**: Custom implementation or FAISS bindings

### File Format Support
- **Images**: `image`, `psd` crates
- **3D Models**: `gltf`, `obj-rs`, `fbxcel-dom`
- **Audio/Video**: `ffmpeg-sys-next`, `symphonia`
- **Archives**: `zip`, `tar`

### Web Server
- **Actix Web 4**: HTTP server for LAN sharing
- **Authentication**: JWT tokens or custom auth
- **TLS**: Optional rustls for HTTPS
- **Static Serving**: Asset streaming with range requests

## External Dependencies

### System Requirements
- **Blender**: Headless CLI for .blend file processing
- **FFmpeg**: Media file processing (optional, prefer Rust alternatives)
- **Git**: For version control backend

### Build Dependencies
- **C/C++ Compiler**: For whisper.cpp compilation
- **CMake**: For complex C++ builds
- **pkg-config**: For system library detection
- **WASM Tools**: wasm-pack, wasm-bindgen

## Development Environment

### Code Style Guide
- **Formatting**: rustfmt with default settings
- **Linting**: clippy with recommended lints
- **Documentation**: Comprehensive rustdoc for all public APIs
- **Testing**: Unit tests in each crate, integration tests in workspace root

### Error Handling Patterns
- **Library Errors**: `thiserror` for structured error types
- **Application Errors**: `anyhow` for error propagation
- **Logging**: `tracing` with structured logging
- **Async Runtime**: tokio for async operations

### Configuration Management
- **Settings**: TOML configuration files
- **Secrets**: Local file-based storage (no external key management)
- **Paths**: Configurable asset directories
- **Models**: Configurable model paths and parameters

## Security Constraints

### Network Isolation
- **No Outbound**: Zero external network calls allowed
- **LAN Only**: Server binds to local network interfaces only
- **Firewall Friendly**: No need for NAT or port forwarding
- **Audit Trail**: All network access logged

### Data Privacy
- **Local Storage**: All data remains on local filesystem
- **No Telemetry**: Zero usage analytics or error reporting
- **Encrypted Storage**: Optional disk encryption for sensitive assets
- **Access Control**: File-based user permissions

## Performance Requirements

### Resource Usage
- **Memory**: Efficient for large asset collections (1TB+)
- **CPU**: Multi-threaded processing for AI tasks
- **Storage**: Incremental indexing, efficient thumbnails
- **GPU**: Optional CUDA/ROCm for faster ML inference

### Scalability Targets
- **Assets**: Handle 100,000+ files efficiently
- **Search**: Sub-second search across all indexed content
- **Preview**: Fast 3D model loading and rendering
- **Concurrent**: Multi-user LAN access without blocking
