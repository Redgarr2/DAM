# DAM - Digital Asset Management System

A powerful, privacy-first digital asset management system built in Rust. DAM handles all your digital assets - from images and 3D models to audio, video, and design files - completely offline with AI-powered search and organization.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## ✨ Features

### 🔍 Universal File Support
- **Images**: PNG, JPG, GIF, WebP, PSD (with layer detection)
- **3D Models**: Blender (.blend), FBX, OBJ, GLTF, GLB
- **Audio/Video**: WAV, MP3, MP4, AVI, MOV, and more
- **Documents**: PDF, TXT, CSV, JSON
- **Archives**: ZIP, RAR detection and cataloguing

### 🧠 AI-Powered Features
- **Offline Transcription**: Whisper.cpp integration for audio/video
- **Image Tagging**: CLIP/BLIP models for automatic content recognition
- **Semantic Search**: Find assets by meaning, not just keywords
- **Smart Categorization**: Automatic organization by content and type

### 🚀 Advanced Capabilities
- **Directory Import**: Recursive scanning with progress feedback
- **Real-time Search**: Text + AI-powered similarity search
- **Preview Generation**: Thumbnails for all supported formats
- **Version Control**: Git-based asset tracking with visual diffs
- **LAN Sharing**: Secure local network sharing with permissions
- **Privacy First**: Completely offline - no cloud, no tracking

### 💻 Multiple Interfaces
- **Web Interface**: Modern browser-based UI at localhost:8080
- **Desktop App**: Native Tauri application
- **CLI Tools**: Command-line asset processing

## 🏗️ Architecture

DAM is built with a modular 7-crate Rust architecture:

```
├── schema/      # Shared types and error handling
├── ingest/      # File processing and preview generation  
├── process/     # AI services (transcription, tagging, generation)
├── index/       # Search engine (text + vector)
├── ui/          # Tauri desktop application
├── server/      # LAN sharing server
├── versioning/  # Git-based version control
└── orchestrator/ # Task coordination
```

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- [Node.js](https://nodejs.org/) (for Tauri UI)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/Redgarr2/DAM.git
   cd DAM
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the web interface**
   ```bash
   cargo run --bin gui-demo
   ```
   Then open http://localhost:8080 in your browser

4. **Or run the desktop app**
   ```bash
   cargo tauri dev -p ui
   ```

### First Import

1. Open the web interface at http://localhost:8080
2. Click "Import Files" 
3. Enter a directory path (e.g., `C:\Users\YourName\Pictures`)
4. Watch DAM recursively scan and import all your assets!

## 📁 Usage Examples

### Import a Directory
```bash
# Web interface: http://localhost:8080 -> "Import Files"
# Enter any directory path, DAM will scan recursively
```

### Search Your Assets
```bash
# Text search: "vacation photos"
# Semantic search: "red car" (finds red cars even without those exact words)
# File type: "type:image" or "type:3d"
```

### CLI Processing
```bash
# Process individual files
cargo run --bin dam-demo -- import /path/to/file.blend

# Scan directories
cargo run --bin dam-demo -- scan /path/to/directory
```

## 🔧 Configuration

### AI Models (Optional)
Download AI models for enhanced features:
- **Whisper**: Place whisper models in `models/whisper/`
- **CLIP**: Place CLIP models in `models/clip/`
- **Stable Diffusion**: Place SD models in `models/sd/`

### Storage
- **Assets**: Original files remain in place
- **Index**: Search data stored in `data/index/`
- **Previews**: Generated thumbnails in `previews/`

## 🏛️ Technical Details

### Performance
- **Async Architecture**: Tokio-based concurrent processing
- **Memory Safe**: Rust's ownership system prevents crashes
- **Scalable**: Handles thousands of assets efficiently
- **Cross-Platform**: Windows, macOS, and Linux support

### Search Technology
- **Full-Text**: Tantivy search engine with TF-IDF scoring
- **Vector Search**: HNSW-based similarity search
- **Hybrid Ranking**: Combined text and semantic relevance

### Privacy & Security
- **Offline Only**: No external network calls
- **Local Processing**: All AI runs on your machine
- **No Tracking**: Your assets never leave your computer
- **Secure Sharing**: LAN-only with authentication

## 🗺️ Roadmap

- [ ] **Enhanced AI Models**: Larger model support
- [ ] **3D Viewer**: Interactive 3D model preview
- [ ] **Plugin System**: Extensible format support
- [ ] **Mobile App**: iOS/Android companion
- [ ] **Cloud Sync**: Optional encrypted cloud backup

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. **Clone and build**
   ```bash
   git clone https://github.com/Redgarr2/DAM.git
   cd DAM
   cargo build
   ```

2. **Run tests**
   ```bash
   cargo test
   ```

3. **Format code**
   ```bash
   cargo fmt
   ```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) for offline transcription
- [Tantivy](https://github.com/quickwit-oss/tantivy) for search functionality
- [Tauri](https://tauri.app/) for desktop application framework
- [Actix Web](https://actix.rs/) for web server
- [Candle](https://github.com/huggingface/candle) for ML inference

## 🚀 Get Started Today

Transform how you manage your digital assets. DAM provides enterprise-grade asset management with complete privacy and offline operation.

**Ready to organize your digital life?**
```bash
git clone https://github.com/Redgarr2/DAM.git
cd DAM
cargo run --bin gui-demo
# Open http://localhost:8080 and start importing!
```

---

**Built with ❤️ in Rust** • **Privacy First** • **Offline Always**
