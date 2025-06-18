# DAM - Digital Asset Management System

A powerful, privacy-first **desktop** digital asset management system built in Rust. DAM handles all your digital assets - from images and 3D models to audio, video, and design files - completely offline with AI-powered search and organization.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## ✨ Features

### 🖥️ **Native Desktop Application**
- **Pure Desktop Experience**: No browser or web server required
- **Single Portable Executable**: Runs anywhere on Windows, macOS, Linux
- **Native Performance**: Built with Tauri for optimal speed and responsiveness
- **Offline First**: Complete privacy - never connects to the internet

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
- **LAN Sharing**: Optional secure local network sharing with permissions
- **Privacy First**: Completely offline - no cloud, no tracking

## 🏗️ Architecture

DAM is built with a modular 7-crate Rust architecture:

```
├── schema/      # Shared types and error handling
├── ingest/      # File processing and preview generation  
├── process/     # AI services (transcription, tagging, generation)
├── index/       # Search engine (text + vector)
├── ui/          # Tauri desktop application (PRIMARY INTERFACE)
├── server/      # Optional LAN sharing server
├── versioning/  # Git-based version control
└── orchestrator/ # Task coordination
```

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) 1.75 or later
- [Node.js](https://nodejs.org/) 16+ (for building the desktop app)

### Installation & Launch

1. **Clone the repository**
   ```bash
   git clone https://github.com/Redgarr2/DAM.git
   cd DAM
   ```

2. **Launch the desktop application**
   ```bash
   cd crates/ui
   cargo tauri dev
   ```

   A native desktop window will open with the DAM interface!

3. **Or build a portable executable**
   ```bash
   cd crates/ui
   cargo tauri build --release
   ```
   
   Find your portable `.exe` in `src-tauri/target/release/`

### First Import

1. The desktop application opens automatically
2. Click "Import Files" in the interface
3. Select files or directories using the native file dialog
4. Watch DAM recursively scan and import all your assets!

## 📁 Usage Examples

### Desktop Application Interface
- **Import Assets**: Use "Import Files" button with native file dialogs
- **Directory Scanning**: Select any folder to recursively import all supported files
- **Search**: Use the search bar for text or semantic similarity search
- **Library Management**: View statistics and organize your collection

### Search Your Assets
- **Text search**: "vacation photos"
- **Semantic search**: "red car" (finds red cars even without those exact words)
- **File type filters**: Search within specific asset types
- **AI-powered**: Finds content by meaning, not just filename

### Build Distribution
```bash
# Create portable executable
cd crates/ui
cargo tauri build --release

# The executable will be in src-tauri/target/release/
# Copy anywhere and run - no installation needed!
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
- **Portable**: All data folders created relative to executable

## 🏛️ Technical Details

### Desktop Application
- **Framework**: Tauri 2.0 for native desktop experience
- **Frontend**: HTML/CSS/JS embedded in native window
- **Backend**: Pure Rust with full access to system APIs
- **Distribution**: Single executable with embedded assets

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
- **Desktop Native**: No web browser or server dependencies

## 🎯 Use Cases

### Creative Professionals
- **3D Artists**: Organize Blender files, textures, and references
- **Photographers**: AI-powered photo organization and search
- **Video Editors**: Transcribe and search video content
- **Designers**: Manage PSD files with layer detection

### Personal Use
- **Media Libraries**: Organize personal photos, videos, music
- **Document Management**: Search through PDFs and documents
- **Project Organization**: Keep creative projects organized
- **Backup Solution**: Catalog and search backup drives

## 🗺️ Roadmap

- [ ] **Enhanced 3D Viewer**: Interactive 3D model preview in desktop app
- [ ] **Advanced AI Models**: Larger model support for better accuracy
- [ ] **Plugin System**: Extensible format support
- [ ] **Cloud Sync**: Optional encrypted cloud backup
- [ ] **Mobile Companion**: iOS/Android app for remote browsing

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. **Clone and build**
   ```bash
   git clone https://github.com/Redgarr2/DAM.git
   cd DAM
   cargo build
   ```

2. **Run desktop app in development**
   ```bash
   cd crates/ui
   cargo tauri dev
   ```

3. **Run tests**
   ```bash
   cargo test
   ```

4. **Format code**
   ```bash
   cargo fmt
   ```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Whisper.cpp](https://github.com/ggerganov/whisper.cpp) for offline transcription
- [Tantivy](https://github.com/quickwit-oss/tantivy) for search functionality
- [Tauri](https://tauri.app/) for desktop application framework
- [Candle](https://github.com/huggingface/candle) for ML inference

## 🚀 Get Started Today

Transform how you manage your digital assets with a powerful desktop application that puts privacy first.

**Ready to organize your digital life?**
```bash
git clone https://github.com/Redgarr2/DAM.git
cd DAM/crates/ui
cargo tauri dev
# Desktop app opens automatically!
```

### Why Choose DAM?

✅ **Complete Privacy** - Everything stays on your computer  
✅ **No Installation** - Single portable executable  
✅ **Universal Support** - Handles all your file types  
✅ **AI-Powered** - Smart search and organization  
✅ **Professional Grade** - Built for serious asset management  

---

**Built with ❤️ in Rust** • **Desktop First** • **Privacy Always** • **Offline Forever**
