# DAM - Digital Asset Management Tool

## Project Goals
Build a standalone, offline digital asset management tool in Rust that handles all types of digital assets with AI-powered features and local-only operation.

## Core Requirements

### Asset Support
- **3D Assets**: .blend, .fbx, .obj, .gltf, Maya files
- **Images**: .psd (with layer support), .png, .jpg, .tiff, design files
- **Media**: .wav, .mp4, .mov, audio/video files
- **Documents**: Any digital asset type

### Key Features
1. **Browser-style 3D Previews**: Interactive orbit/zoom/pan for 3D models
2. **AI Processing**: 
   - Transcription via whisper.cpp (offline)
   - Visual tagging via CLIP/BLIP models
   - Generative image editing via Stable Diffusion
   - Semantic search across all content
3. **Version Control**: Git-backed snapshots with visual diffs for PSD layers and 3D models
4. **LAN Sharing**: Private server with permissions and access logging
5. **Fully Offline**: Zero internet dependencies, all AI models local

### Architecture Constraints
- **Language**: Rust
- **Workspace Structure**: Multiple crates for clean boundaries
- **UI Framework**: Tauri with embedded Bevy 3D previews  
- **Search**: Tantivy + vector embeddings
- **AI Framework**: candle for ML inference
- **Server**: Actix Web for LAN sharing

## Non-Goals
- Cloud integration
- External API dependencies  
- Online model downloads
- Remote collaboration beyond LAN
