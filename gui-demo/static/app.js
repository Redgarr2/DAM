// DAM Web GUI JavaScript

class DamApp {
    constructor() {
        this.currentQuery = '';
        this.results = [];
        this.stats = {};
        
        this.initializeElements();
        this.bindEvents();
        this.loadStats();
        this.showEmptyState();
    }

    initializeElements() {
        // Search elements
        this.searchInput = document.getElementById('search-input');
        this.searchBtn = document.getElementById('search-btn');
        this.resultsGrid = document.getElementById('results-grid');
        this.emptyState = document.getElementById('empty-state');
        this.loading = document.getElementById('loading');
        
        // Stats elements
        this.totalAssetsEl = document.getElementById('total-assets');
        this.aiProcessedEl = document.getElementById('ai-processed');
        
        // Button elements
        this.importBtn = document.getElementById('import-btn');
        this.scanLibraryBtn = document.getElementById('scan-library-btn');
        this.importFirstBtn = document.getElementById('import-first-btn');
        this.settingsBtn = document.getElementById('settings-btn');
        
        // Modal elements
        this.settingsModal = document.getElementById('settings-modal');
        this.closeSettingsBtn = document.getElementById('close-settings');
        this.saveSettingsBtn = document.getElementById('save-settings');
        this.cancelSettingsBtn = document.getElementById('cancel-settings');
        
        // Status
        this.statusText = document.getElementById('status-text');
    }

    bindEvents() {
        // Search functionality
        this.searchBtn.addEventListener('click', () => this.performSearch());
        this.searchInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                this.performSearch();
            }
        });
        
        // Import functionality  
        this.importBtn.addEventListener('click', () => this.showImportDialog());
        this.importFirstBtn.addEventListener('click', () => this.showImportDialog());
        
        // Library scan
        this.scanLibraryBtn.addEventListener('click', () => this.scanLibrary());
        
        // Settings modal
        this.settingsBtn.addEventListener('click', () => this.showSettings());
        this.closeSettingsBtn.addEventListener('click', () => this.hideSettings());
        this.cancelSettingsBtn.addEventListener('click', () => this.hideSettings());
        this.saveSettingsBtn.addEventListener('click', () => this.saveSettings());
        
        // Close modal on background click
        this.settingsModal.addEventListener('click', (e) => {
            if (e.target === this.settingsModal) {
                this.hideSettings();
            }
        });
    }

    async loadStats() {
        try {
            const response = await fetch('/api/stats');
            if (response.ok) {
                this.stats = await response.json();
                this.updateStatsDisplay();
            }
        } catch (error) {
            console.error('Failed to load stats:', error);
            this.setStatus('Failed to load library stats');
        }
    }

    updateStatsDisplay() {
        if (this.totalAssetsEl) {
            this.totalAssetsEl.textContent = this.stats.total_documents || 0;
        }
        if (this.aiProcessedEl) {
            this.aiProcessedEl.textContent = this.stats.visual_embeddings || 0;
        }
    }

    async performSearch() {
        const query = this.searchInput.value.trim();
        
        if (!query) {
            this.showEmptyState();
            return;
        }

        this.currentQuery = query;
        this.showLoading();
        this.setStatus(`Searching for "${query}"...`);

        try {
            const response = await fetch(`/api/search?q=${encodeURIComponent(query)}&limit=20`);
            
            if (response.ok) {
                const data = await response.json();
                this.results = data.results;
                this.displayResults();
                this.setStatus(`Found ${data.total} results for "${query}"`);
            } else {
                throw new Error(`Search failed: ${response.statusText}`);
            }
        } catch (error) {
            console.error('Search error:', error);
            this.setStatus('Search failed. Please try again.');
            this.showEmptyState();
        } finally {
            this.hideLoading();
        }
    }

    displayResults() {
        this.hideEmptyState();
        this.resultsGrid.innerHTML = '';

        if (this.results.length === 0) {
            this.resultsGrid.innerHTML = `
                <div class="no-results">
                    <h3>No results found</h3>
                    <p>Try a different search term or import more files.</p>
                </div>
            `;
            return;
        }

        this.results.forEach(result => {
            const resultElement = this.createResultElement(result);
            this.resultsGrid.appendChild(resultElement);
        });
    }

    createResultElement(result) {
        const element = document.createElement('div');
        element.className = 'result-item';
        
        const fileType = this.getFileTypeIcon(result.filename);
        const truncatedContent = result.content.length > 150 
            ? result.content.substring(0, 150) + '...'
            : result.content;

        element.innerHTML = `
            <div class="result-header">
                <div class="file-icon">${fileType}</div>
                <div class="file-info">
                    <h4 class="filename">${this.escapeHtml(result.filename)}</h4>
                    <p class="file-path">${this.escapeHtml(result.path)}</p>
                </div>
                <div class="result-score">${result.score.toFixed(2)}</div>
            </div>
            <div class="result-content">
                <p>${this.escapeHtml(truncatedContent)}</p>
            </div>
        `;

        element.addEventListener('click', () => {
            this.openAsset(result);
        });

        return element;
    }

    getFileTypeIcon(filename) {
        const ext = filename.split('.').pop()?.toLowerCase();
        
        switch (ext) {
            case 'txt': case 'md': case 'doc': case 'docx':
                return '📄';
            case 'jpg': case 'jpeg': case 'png': case 'gif': case 'bmp':
                return '🖼️';
            case 'mp4': case 'avi': case 'mov': case 'mkv':
                return '🎥';
            case 'mp3': case 'wav': case 'flac': case 'ogg':
                return '🎵';
            case 'pdf':
                return '📕';
            case 'zip': case 'rar': case '7z':
                return '📦';
            case 'blend':
                return '🎨';
            case 'psd':
                return '🎨';
            case 'fbx': case 'obj': case 'gltf':
                return '🧊';
            default:
                return '📄';
        }
    }

    openAsset(result) {
        // For now, just show the file path
        this.setStatus(`Opening: ${result.filename}`);
        
        // In a real implementation, this could open a preview modal
        // or launch the appropriate application
        console.log('Opening asset:', result);
    }

    showImportDialog() {
        // Improved prompt with better instructions
        const filePath = prompt(
            'Enter file or directory path to import:\n\n' +
            'Examples:\n' +
            '• C:\\MyProjects\\image.png (single file)\n' +
            '• C:\\Blender\\MyProjects (entire directory)\n' +
            '• /home/user/assets (Linux/Mac directory)'
        );
        if (filePath && filePath.trim()) {
            this.importPath(filePath.trim());
        }
    }

    async importPath(path) {
        this.setStatus(`Importing ${path}...`);
        
        try {
            const response = await fetch('/api/import', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ path: path })
            });

            if (response.ok) {
                const result = await response.json();
                
                if (result.type === 'directory') {
                    // Directory import with detailed feedback
                    const message = `Directory import complete!\n` +
                        `• ${result.imported_count} files successfully imported\n` +
                        `• ${result.failed_count} files failed to import\n` +
                        `• Path: ${result.path}`;
                    
                    this.setStatus(result.message);
                    console.log('Directory import details:', result);
                    
                    // Show detailed results to user
                    if (result.failed_count > 0) {
                        alert(`${result.message}\n\nNote: ${result.failed_count} files failed to import. Check console for details.`);
                    } else {
                        alert(`✅ ${result.message}`);
                    }
                } else {
                    // Single file import
                    this.setStatus(`Successfully imported ${result.asset_type}: ${path}`);
                    alert(`✅ Successfully imported ${result.asset_type} file`);
                }
                
                this.loadStats(); // Refresh stats
                
                // If we have a current search, refresh results
                if (this.currentQuery) {
                    this.performSearch();
                }
                
                // Clear the "no results" state if we're showing empty state
                if (this.emptyState && !this.emptyState.classList.contains('hidden')) {
                    this.showEmptyState();
                }
                
            } else {
                const error = await response.json();
                throw new Error(error.message || error.error || 'Import failed');
            }
        } catch (error) {
            console.error('Import error:', error);
            const errorMessage = `Failed to import: ${error.message}`;
            this.setStatus(errorMessage);
            alert(`❌ ${errorMessage}`);
        }
    }

    async scanLibrary() {
        this.setStatus('Scanning library...');
        
        // For demo, just reload stats and show a message
        setTimeout(() => {
            this.loadStats();
            this.setStatus('Library scan complete');
        }, 2000);
    }

    showSettings() {
        this.settingsModal.classList.remove('hidden');
    }

    hideSettings() {
        this.settingsModal.classList.add('hidden');
    }

    saveSettings() {
        // Get values from settings form
        const aiEnabled = document.getElementById('ai-enabled').checked;
        const aiTier = document.getElementById('ai-tier').value;
        const autoTag = document.getElementById('auto-tag').checked;
        const autoTranscribe = document.getElementById('auto-transcribe').checked;

        // In a real app, these would be saved via API
        console.log('Saving settings:', {
            aiEnabled,
            aiTier,
            autoTag,
            autoTranscribe
        });

        this.setStatus('Settings saved successfully');
        this.hideSettings();
    }

    showLoading() {
        this.loading.classList.remove('hidden');
        this.resultsGrid.classList.add('hidden');
        this.emptyState.classList.add('hidden');
    }

    hideLoading() {
        this.loading.classList.add('hidden');
        this.resultsGrid.classList.remove('hidden');
    }

    showEmptyState() {
        this.emptyState.classList.remove('hidden');
        this.resultsGrid.classList.add('hidden');
        this.loading.classList.add('hidden');
    }

    hideEmptyState() {
        this.emptyState.classList.add('hidden');
    }

    setStatus(message) {
        if (this.statusText) {
            this.statusText.textContent = message;
        }
        console.log('Status:', message);
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize the app when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    new DamApp();
});
