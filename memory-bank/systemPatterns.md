# System Architecture & Design Patterns

## Workspace Structure

```
dam-workspace/
├── crates/
│   ├── ingest/         # Asset parsing and format detection
│   ├── process/        # AI processing pipeline
│   ├── index/          # Search and indexing
│   ├── ui/             # Tauri desktop application
│   ├── server/         # LAN sharing server
│   ├── versioning/     # Git-backed version control
│   └── orchestrator/   # Task coordination
├── schema/             # Shared types and IPC
├── assets/             # AI models and resources
├── vendor/             # External dependencies (whisper.cpp)
└── memory-bank/        # Project documentation
```

## Core Design Patterns

### 1. Crate Boundaries & Responsibilities

**ingest**: Entry point for all assets
- File format detection
- Metadata extraction  
- Preview generation triggers
- Input validation and sanitization

**process**: AI and computation pipeline
- Whisper.cpp FFI for transcription
- CLIP/BLIP visual tagging via candle
- Stable Diffusion image editing
- Embedding generation for search

**index**: Unified search system
- Tantivy full-text search
- Vector similarity search
- Hybrid ranking algorithms
- Asset metadata storage

**ui**: Desktop application interface
- Tauri-based desktop app
- Embedded Bevy 3D previews (WASM)
- Asset browser and search UI
- Background task management

**server**: LAN sharing capabilities
- Actix Web HTTP server
- Authentication and authorization
- Access logging and permissions
- Asset streaming and API endpoints

**versioning**: Asset history management
- Git2-backed snapshots
- Visual diff generation
- PSD layer comparison
- 3D model change detection

**orchestrator**: Cross-crate coordination
- Task queue management
- Pipeline orchestration
- Event-driven communication
- Progress tracking and error handling

### 2. Data Flow Architecture

```
Asset Input → ingest → process → index
                ↓        ↓       ↓
            versioning → orchestrator → ui
                              ↓
                           server (LAN)
```

### 3. Communication Patterns

**IPC Messages**: Structured communication between crates
- Asset processing events
- Search queries and results
- UI state updates
- Server requests/responses

**Event-Driven**: Async task coordination
- File system watchers
- Background processing
- Progress notifications
- Error propagation

### 4. Storage Patterns

**Local-First**: All data stored on local filesystem
- Asset files in organized directory structure
- Search indices in local database
- AI model weights in assets/ directory
- Version history in git repositories

**Privacy-by-Design**: No external communication
- All AI inference runs locally
- No telemetry or analytics
- LAN-only server binding
- Audit logs for access tracking
