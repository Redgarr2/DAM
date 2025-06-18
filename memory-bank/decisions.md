# Technical Decisions & Design Trade-offs

## Decision Log for External LLM Diagnostic Support

This document provides a chronological record of major technical decisions, their rationale, alternatives considered, and known risks/limitations. Use this for troubleshooting, architecture analysis, and understanding system dependencies.

---

## 🏗️ **Architecture Decisions**

### **Decision 1: Multi-Crate Workspace Structure**
**Date**: Project inception
**Decision**: 7-crate modular architecture vs monolithic structure
**Rationale**: 
- Clean compilation boundaries enable parallel development
- Reduces rebuild times during development
- Enables selective feature compilation
- Forces clear API boundaries

**Alternatives Rejected**:
- Single crate: Would create coupling issues and slow compilation
- More crates (10+): Over-engineering for project scope

**Known Risks & Diagnostic Points**:
- ⚠️ **Circular Dependencies**: Watch for crate dependency cycles
- ⚠️ **Version Conflicts**: Multiple crates can cause dependency version mismatches
- 🔍 **Debug**: Check `cargo tree` for dependency conflicts
- 🔍 **Performance**: Inter-crate communication overhead (minimal in practice)

### **Decision 2: Rust Language Choice**
**Date**: Project inception  
**Decision**: Rust vs C++, Go, or Python
**Rationale**:
- Memory safety without garbage collection
- Excellent async/await support via Tokio
- Zero-cost abstractions for performance
- Strong type system prevents common bugs
- Great ecosystem for systems programming

**Alternatives Rejected**:
- **C++**: Memory safety concerns, complex build systems
- **Go**: Garbage collection unsuitable for real-time processing
- **Python**: Too slow for large-scale file processing

**Known Risks & Diagnostic Points**:
- ⚠️ **Learning Curve**: Ownership/borrowing concepts
- ⚠️ **Compile Times**: Can be slow with many dependencies
- 🔍 **Debug**: Use `cargo build --timings` for build analysis
- 🔍 **Memory**: Rust prevents leaks but can have high peak usage

---

## 🔍 **Search & Indexing Decisions**

### **Decision 3: Tantivy for Full-Text Search**
**Date**: Early development
**Decision**: Tantivy vs Elasticsearch, SQLite FTS, or custom solution
**Rationale**:
- Pure Rust, no external dependencies
- Excellent performance for local search
- Supports complex queries and ranking
- Embedded, no server required

**Alternatives Rejected**:
- **Elasticsearch**: Too heavy, requires JVM and separate server
- **SQLite FTS**: Limited ranking and query capabilities
- **Custom**: Would take months to implement properly

**Known Risks & Diagnostic Points**:
- ⚠️ **Index Corruption**: Power loss during indexing can corrupt indices
- ⚠️ **Memory Usage**: Large indices consume significant RAM
- ⚠️ **Disk Space**: Indices can be 10-30% of original content size
- 🔍 **Debug**: Check `data/index/` directory for corruption
- 🔍 **Performance**: Monitor index size and RAM usage
- 🔍 **Recovery**: Can rebuild indices from original files

### **Decision 4: Custom Vector Storage vs FAISS**
**Date**: Mid development
**Decision**: Simple in-memory vector storage vs FAISS bindings
**Rationale**:
- FAISS C++ bindings add complexity
- Current scale doesn't require FAISS optimizations
- Simpler debugging and maintenance
- Pure Rust implementation

**Alternatives Rejected**:
- **FAISS**: Excellent performance but complex bindings and deployment
- **External Vector DB**: Adds network dependency

**Known Risks & Diagnostic Points**:
- ⚠️ **Scalability**: Current solution may not scale beyond 100K+ assets
- ⚠️ **Memory**: All vectors loaded into RAM
- 🔍 **Debug**: Monitor memory usage with large asset collections
- 🔍 **Performance**: Vector search becomes slow with >50K assets
- 🔍 **Migration Path**: Can upgrade to FAISS later if needed

---

## 🌐 **UI & Framework Decisions**

### **Decision 5: Tauri vs Electron for Desktop App**
**Date**: Early development
**Decision**: Tauri 2.0 vs Electron vs native GUI
**Rationale**:
- Much smaller bundle size than Electron
- Better performance and security
- Native OS integration
- Rust backend integration

**Alternatives Rejected**:
- **Electron**: Large bundle size, security concerns, memory usage
- **Native GUI (egui/iced)**: More complex for rich web-like interfaces
- **Web-only**: Users wanted desktop app option

**Known Risks & Diagnostic Points**:
- ⚠️ **Platform Differences**: Behavior varies between Windows/macOS/Linux
- ⚠️ **WebView Issues**: System webview bugs affect app
- ⚠️ **Build Complexity**: Requires Node.js toolchain
- 🔍 **Debug**: Check webview console for frontend errors
- 🔍 **Platform**: Test on target OS, don't assume cross-platform compatibility

### **Decision 6: Embedded Bevy 3D vs Web-based Three.js**
**Date**: Mid development
**Decision**: Bevy compiled to WASM vs Three.js for 3D previews
**Rationale**:
- Unified Rust codebase
- Better integration with asset processing
- High performance 3D rendering

**Alternatives Rejected**:
- **Three.js**: Would require JavaScript maintenance and data marshaling
- **Native 3D Window**: Complex platform integration

**Known Risks & Diagnostic Points**:
- ⚠️ **WASM Size**: Large WASM bundles affect loading times
- ⚠️ **WebGL Compatibility**: Older browsers/drivers may fail
- ⚠️ **Memory**: 3D assets can consume significant GPU memory
- 🔍 **Debug**: Check browser console for WebGL errors
- 🔍 **Performance**: Monitor GPU memory usage with large models

---

## 🤖 **AI & Processing Decisions**

### **Decision 7: Candle vs PyTorch Bindings for Local AI**
**Date**: Mid development
**Decision**: Candle (pure Rust) vs Python/PyTorch integration
**Rationale**:
- No Python runtime dependency
- Better integration with Rust ecosystem
- Easier deployment and distribution
- Type safety for model operations

**Alternatives Rejected**:
- **PyTorch**: Requires Python runtime, complex deployment
- **ONNX Runtime**: Good but still external dependency
- **TensorFlow Lite**: Limited model support

**Known Risks & Diagnostic Points**:
- ⚠️ **Model Compatibility**: Candle supports fewer models than PyTorch
- ⚠️ **Performance**: May be slower than optimized PyTorch for some operations
- ⚠️ **Model Format**: Requires .safetensors format conversion
- 🔍 **Debug**: Check model loading errors carefully
- 🔍 **Memory**: Monitor GPU/CPU memory during inference
- 🔍 **Fallback**: Always provide CPU fallback for GPU operations

### **Decision 8: Whisper.cpp FFI vs Local Whisper Implementation**
**Date**: Early development
**Decision**: Use whisper.cpp via FFI vs pure Rust implementation
**Rationale**:
- Whisper.cpp is battle-tested and optimized
- Supports all Whisper model variants
- Much faster than pure Rust alternatives

**Alternatives Rejected**:
- **Candle Whisper**: Slower and less mature
- **Python Whisper**: Would require Python dependency

**Known Risks & Diagnostic Points**:
- ⚠️ **C++ Dependency**: Requires C++ compiler for building
- ⚠️ **Platform Issues**: Different linking requirements per platform
- ⚠️ **Memory Management**: C/Rust FFI boundary requires careful handling
- 🔍 **Debug**: Check FFI boundary for memory leaks
- 🔍 **Build**: Ensure whisper.cpp compiles correctly on target platform
- 🔍 **Model Loading**: Verify .ggml model format compatibility

---

## 📁 **File Processing Decisions**

### **Decision 9: Direct File Parsing vs External Tools**
**Date**: Early development
**Decision**: Pure Rust parsers vs calling Blender/FFmpeg executables
**Rationale**:
- Better error handling and control
- No external tool dependencies
- Faster processing (no process spawning)
- Cross-platform consistency

**Alternatives Rejected**:
- **Blender Headless**: Slow, requires Blender installation
- **FFmpeg**: Complex deployment, licensing issues

**Known Risks & Diagnostic Points**:
- ⚠️ **Format Support**: Rust parsers may not support all format variants
- ⚠️ **Complex Formats**: Some formats (advanced PSD) may need external tools
- ⚠️ **Version Drift**: File format specifications change over time
- 🔍 **Debug**: Test with diverse file samples
- 🔍 **Fallback**: Provide graceful degradation for unsupported formats
- 🔍 **Validation**: Always validate parsed data before using

### **Decision 10: Async File Processing vs Blocking I/O**
**Date**: Early development
**Decision**: Tokio async I/O vs traditional blocking file operations
**Rationale**:
- Better UI responsiveness
- Can process multiple files concurrently
- Scales better with large directories

**Alternatives Rejected**:
- **Blocking I/O**: Would freeze UI during large operations
- **Thread Pool**: More complex than async/await

**Known Risks & Diagnostic Points**:
- ⚠️ **File Handle Limits**: Async can hit OS file descriptor limits
- ⚠️ **Error Propagation**: Async errors can be harder to trace
- ⚠️ **Cancellation**: Long operations must handle cancellation properly
- 🔍 **Debug**: Monitor file descriptor usage
- 🔍 **Performance**: Watch for async task scheduling overhead
- 🔍 **Limits**: Check `ulimit -n` on Unix systems

---

## 🔒 **Security & Privacy Decisions**

### **Decision 11: Local-Only vs Cloud Hybrid Architecture**
**Date**: Project inception
**Decision**: Complete offline operation vs cloud-enhanced features
**Rationale**:
- Privacy-first design principle
- No internet dependency
- Faster operations (no network latency)
- User control over data

**Alternatives Rejected**:
- **Cloud AI**: Better models but privacy concerns
- **Hybrid**: Complexity in offline/online modes

**Known Risks & Diagnostic Points**:
- ⚠️ **Limited AI Models**: Local models smaller than cloud alternatives
- ⚠️ **Update Mechanism**: No automatic model updates
- ⚠️ **Backup**: Users responsible for their own backups
- 🔍 **Storage**: Monitor local disk usage growth
- 🔍 **Models**: Provide clear model download/update instructions

---

## 🚨 **Critical Failure Points & Dependencies**

### **System Requirements Failures**:
- **Rust Compiler**: Wrong version breaks candle compatibility
- **C++ Compiler**: Required for whisper.cpp FFI
- **GPU Drivers**: CUDA/ROCm issues affect AI performance
- **File Permissions**: Can't write to data/ directory
- **Disk Space**: Index and preview generation requires significant space

### **Runtime Dependencies**:
- **Tokio Runtime**: All async operations depend on this
- **File System**: Requires readable source files and writable cache directories
- **Memory**: AI models require 2-8GB RAM depending on size
- **Network**: LAN server requires available ports

### **Development Dependencies**:
- **Node.js**: Required for Tauri frontend build
- **wasm-pack**: For Bevy 3D preview compilation
- **Git**: For version control functionality

### **Common Diagnostic Commands**:
```bash
# Check build dependencies
cargo tree --duplicates
cargo build --timings

# Check runtime status
lsof -p <pid>  # File handles (Unix)
htop           # Memory and CPU usage

# Check disk usage
du -sh data/ previews/ models/

# Check network ports
netstat -tulpn | grep :8080
```

---

## 🔧 **Performance Bottlenecks & Optimization Points**

### **Identified Bottlenecks**:
1. **Large File Parsing**: Multi-GB files can block processing
2. **Vector Search**: Linear search becomes slow >50K assets
3. **Preview Generation**: High-resolution images slow to process
4. **Index Building**: Initial indexing of large collections takes time
5. **Memory Usage**: AI models consume significant RAM

### **Optimization Strategies**:
- **Chunked Processing**: Break large operations into smaller pieces
- **LRU Caches**: Cache frequently accessed data
- **Background Processing**: Move heavy operations off UI thread
- **Progressive Loading**: Load and display results incrementally
- **Memory Pooling**: Reuse allocated buffers for AI inference

This decision log provides the technical context needed for external LLMs to understand system architecture, identify likely failure points, and suggest appropriate debugging approaches.
