# SCypher GUI - Address Derivation Implementation Guide

## Overview
This document provides a complete, autonomous implementation guide for adding multi-blockchain address derivation functionality to the existing SCypher GUI. The implementation adds a new tab-based interface while preserving all existing functionality.

## Project Context
- **Base Project**: SCypher GUI v3.0 - XOR-based BIP39 Seed Cipher with Tauri backend
- **Existing Functionality**: Seed phrase input, validation, and XOR+Argon2id transformation
- **New Requirement**: Multi-blockchain address derivation with professional UX

## Core Architecture Changes

### 1. Tab-Based Interface Structure
Transform the existing single-page interface into a two-tab system:

**Tab 1: "Transform Seed"** (existing functionality enhanced)
- Input Seed Phrase (current system)
- SCypher Security Settings (current system)
- Transformation Result (current system)

**Tab 2: "Derive Addresses"** (new functionality)
- Seed Source Selection
- BIP39 Standard Settings  
- Network Selection Grid
- Address Count Configuration
- Results Display with Export Options

### 2. State Management System
Implement global state coordination between tabs:

```javascript
const AppState = {
  inputSeed: null,           // User-entered seed phrase
  inputSeedWordCount: 0,     // Auto-detected: 12,15,18,21,24
  transformedSeed: null,     // SCypher-processed result
  scypherPassword: null,     // SCypher transformation password
  selectedSource: 'input',   // 'input' | 'transformed'
  bip39Passphrase: '',       // BIP39 standard passphrase
  selectedNetworks: [],      // Array of selected network names
  addressCount: 1,           // 1-100 addresses per network
  derivationResults: {},     // Results by network
  processing: {
    active: false,
    type: null,              // 'transform' | 'derive'
    progress: 0,
    current: '',
    cancelable: false
  }
}
```

### 3. Tab Communication and Lock System
- **Tab Lock**: During active processing, disable tab switching
- **State Synchronization**: Real-time updates between tabs
- **Smart Headers**: Dynamic tab titles reflecting current state

## Detailed Implementation Specifications

### Tab 1 Enhancements: "Transform Seed"

#### Current Functionality Fixes
1. **Placeholder Ghost Text Fix**:
   ```javascript
   // Fix the "Start typing or drag & drop..." placeholder not clearing
   function updateContainerState() {
     const container = document.getElementById('wordContainer');
     const hasContent = currentWords.length > 0 || 
                       document.getElementById('wordInput').value.length > 0;
     container.classList.toggle('has-content', hasContent);
     
     // Enhanced placeholder handling
     const placeholder = document.querySelector('.input-placeholder');
     if (hasContent) {
       placeholder.style.opacity = '0';
       placeholder.style.pointerEvents = 'none';
     } else {
       placeholder.style.opacity = '1';
       placeholder.style.pointerEvents = 'auto';
     }
   }
   ```

2. **Preserve Drag & Drop Functionality**: Maintain existing Tauri file drop system without modification

3. **Auto Word Count Detection**: Display detected word count in seed source preview

#### Visual Enhancements
- Maintain existing styling and functionality
- Add success state indicators when transformation completes
- Enhanced result display with "Use for Derivation →" button

### Tab 2 Implementation: "Derive Addresses"

#### Seed Source Selection Section
```html
<div class="section">
  <div class="section-title">🔄 Seed Source Selection</div>
  <div class="seed-source-container">
    <label class="source-option">
      <input type="radio" name="seedSource" value="input" checked>
      <div class="source-preview">
        <div class="source-title">Input seed phrase (<span id="inputWordCount">0</span> words)</div>
        <div class="source-text" id="inputSeedPreview">No seed phrase entered</div>
      </div>
    </label>
    
    <label class="source-option" id="transformedSourceOption" disabled>
      <input type="radio" name="seedSource" value="transformed" disabled>
      <div class="source-preview">
        <div class="source-title">SCypher-transformed seed ✅ Available</div>
        <div class="source-text" id="transformedSeedPreview">Transform a seed first</div>
      </div>
    </label>
  </div>
</div>
```

#### BIP39 Standard Settings Section
```html
<div class="section">
  <div class="section-title">🌐 BIP39 Standard Settings</div>
  <div class="bip39-section">
    <div class="info-card">
      <div class="info-title">BIP39 Standard: Official Bitcoin Improvement Proposal</div>
      <div class="info-desc">Purpose: Additional entropy during address derivation</div>
    </div>
    <div class="input-group">
      <label class="label">BIP39 Passphrase (optional):</label>
      <div class="password-input">
        <input type="password" class="input" id="bip39Passphrase" 
               placeholder="Enter BIP39 passphrase (leave empty for standard derivation)">
        <button class="password-toggle" type="button">👁️</button>
      </div>
    </div>
  </div>
</div>
```

#### Network Selection Grid
```html
<div class="section">
  <div class="section-title">🌍 Select Networks</div>
  <div class="network-grid">
    <button class="network-button" data-network="bitcoin" data-color="#F7931A">
      <span class="network-icon">₿</span>
      <span class="network-name">Bitcoin</span>
    </button>
    <button class="network-button" data-network="ethereum" data-color="#627EEA">
      <span class="network-icon">Ξ</span>
      <span class="network-name">Ethereum</span>
    </button>
    <button class="network-button" data-network="ergo" data-color="#FF5722">
      <span class="network-icon">⚡</span>
      <span class="network-name">Ergo</span>
    </button>
    <button class="network-button" data-network="tron" data-color="#FF0013">
      <span class="network-icon">🔺</span>
      <span class="network-name">TRON</span>
    </button>
    <button class="network-button" data-network="bsc" data-color="#F0B90B">
      <span class="network-icon">⚪</span>
      <span class="network-name">BSC</span>
    </button>
    <button class="network-button" data-network="polygon" data-color="#8247E5">
      <span class="network-icon">◆</span>
      <span class="network-name">Polygon</span>
    </button>
    <button class="network-button" data-network="cardano" data-color="#0033AD">
      <span class="network-icon">₳</span>
      <span class="network-name">Cardano</span>
    </button>
    <button class="network-button" data-network="dogecoin" data-color="#C2A633">
      <span class="network-icon">Ð</span>
      <span class="network-name">Dogecoin</span>
    </button>
    <button class="network-button" data-network="litecoin" data-color="#BFBBBB">
      <span class="network-icon">Ł</span>
      <span class="network-name">Litecoin</span>
    </button>
    <button class="network-button" data-network="solana" data-color="#00FFA3">
      <span class="network-icon">◉</span>
      <span class="network-name">Solana</span>
    </button>
  </div>
  
  <div class="network-controls">
    <button class="control-button select-all">🔥 Select All</button>
    <button class="control-button clear-all">❌ Clear All</button>
  </div>
  
  <div class="bip39-warning" id="bip39Warning" style="display: none;">
    <strong>⚠️ Notice:</strong> Cardano & Solana will derive WITHOUT BIP39 passphrase (not supported by these networks)
  </div>
</div>
```

#### Address Count Configuration
```html
<div class="section">
  <div class="section-title">⚙️ Address Count</div>
  <div class="count-config">
    <label class="label">Generate addresses per network:</label>
    <div class="count-input-group">
      <input type="number" class="count-input" id="addressCount" 
             value="1" min="1" max="100" placeholder="1">
      <span class="count-range">(1-100)</span>
    </div>
  </div>
</div>
```

#### Derive Button and Progress
```html
<div class="action-section">
  <button class="button derive-button" id="deriveButton" disabled>
    🚀 Derive Addresses
  </button>
  
  <div class="progress-section" id="progressSection" style="display: none;">
    <div class="progress-bar">
      <div class="progress-fill" id="deriveProgressFill"></div>
    </div>
    <div class="progress-text" id="progressText">Preparing derivation...</div>
    <button class="cancel-button" id="cancelDerivation">❌ Cancel Derivation</button>
  </div>
</div>
```

#### Results Display Section
```html
<div class="results-section" id="resultsSection" style="display: none;">
  <div class="section-title">📍 Derivation Results</div>
  
  <div class="results-header">
    <div class="results-info" id="resultsInfo">
      Derived from: <strong id="sourceIndicator">Input seed phrase</strong>
    </div>
    <div class="view-controls">
      <select class="view-selector" id="viewMode">
        <option value="detailed">Detailed View</option>
        <option value="compact">Compact View</option>
      </select>
      <select class="export-selector" id="exportFormat">
        <option value="">Export As...</option>
        <option value="json">JSON</option>
        <option value="csv">CSV</option>
        <option value="txt">TXT</option>
      </select>
    </div>
  </div>
  
  <div class="results-content" id="resultsContent">
    <!-- Dynamic content based on view mode -->
  </div>
</div>
```

## CSS Styling Specifications

### Tab System Styling
```css
.tab-container {
  background: rgba(40, 40, 40, 0.95);
  border-radius: 12px 12px 0 0;
  border-bottom: 1px solid #4a4a4a;
}

.tab-header {
  display: flex;
  background: rgba(30, 30, 30, 0.8);
  border-radius: 12px 12px 0 0;
}

.tab-button {
  flex: 1;
  background: none;
  border: none;
  color: #888;
  padding: 16px 24px;
  font-size: 1em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  border-bottom: 3px solid transparent;
  position: relative;
}

.tab-button.active {
  color: #ff9500;
  border-bottom-color: #ff9500;
  background: rgba(255, 149, 0, 0.05);
}

.tab-button.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  color: #555;
}

.tab-button .tab-status {
  font-size: 0.8em;
  margin-left: 8px;
  opacity: 0.7;
}

.tab-content {
  display: none;
  padding: 30px;
  animation: fadeIn 0.3s ease-in;
}

.tab-content.active {
  display: block;
}
```

### Network Selection Grid Styling
```css
.network-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}

.network-button {
  background: rgba(60, 60, 60, 0.5);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 16px 12px;
  color: #ccc;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
  font-weight: 500;
}

.network-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  border-color: rgba(255, 255, 255, 0.2);
}

.network-button.selected {
  border-color: var(--network-color);
  background: rgba(var(--network-color-rgb), 0.15);
  color: var(--network-color);
  box-shadow: 0 0 0 2px rgba(var(--network-color-rgb), 0.3);
}

.network-icon {
  font-size: 1.5em;
  font-weight: bold;
}

.network-name {
  font-size: 0.85em;
  text-align: center;
}
```

### Seed Source Selection Styling
```css
.seed-source-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.source-option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: rgba(60, 60, 60, 0.3);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.source-option:hover:not([disabled]) {
  border-color: rgba(255, 149, 0, 0.3);
  background: rgba(255, 149, 0, 0.05);
}

.source-option input[type="radio"]:checked + .source-preview {
  color: #ff9500;
}

.source-option[disabled] {
  opacity: 0.4;
  cursor: not-allowed;
}

.source-preview {
  flex: 1;
}

.source-title {
  font-weight: 600;
  margin-bottom: 8px;
  color: inherit;
}

.source-text {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.85em;
  color: #888;
  word-break: break-all;
  line-height: 1.4;
}

.source-option input[type="radio"]:checked + .source-preview .source-text {
  color: #ccc;
}
```

### Results Display Styling
```css
.results-section {
  background: rgba(30, 30, 30, 0.8);
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 20px;
  margin-top: 20px;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #444;
}

.results-info {
  font-size: 0.9em;
  color: #ccc;
}

.view-controls {
  display: flex;
  gap: 10px;
}

.view-selector, .export-selector {
  background: rgba(60, 60, 60, 0.8);
  border: 1px solid #555;
  color: #e0e0e0;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 0.85em;
}

/* Detailed View */
.network-result {
  margin-bottom: 24px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 8px;
  padding: 16px;
}

.network-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #555;
}

.network-header .network-icon {
  font-size: 1.2em;
  color: var(--network-color);
}

.network-title {
  font-size: 1em;
  font-weight: 600;
  color: #e0e0e0;
}

.address-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.address-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: rgba(20, 20, 20, 0.5);
  border: 1px solid #333;
  border-radius: 6px;
}

.address-index {
  font-size: 0.8em;
  color: #888;
  min-width: 20px;
}

.address-text {
  flex: 1;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.85em;
  color: #e0e0e0;
  word-break: break-all;
}

.address-actions {
  display: flex;
  gap: 4px;
}

.action-button {
  background: rgba(60, 60, 60, 0.8);
  border: 1px solid #555;
  color: #ccc;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75em;
  transition: all 0.2s ease;
}

.action-button:hover {
  background: rgba(80, 80, 80, 0.8);
  color: #fff;
}

/* Compact View */
.compact-result {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 6px;
  margin-bottom: 8px;
}

.compact-network {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 100px;
}

.compact-count {
  color: #888;
  font-size: 0.8em;
}

.compact-actions {
  margin-left: auto;
  display: flex;
  gap: 4px;
}
```

## JavaScript Implementation Details

### Core State Management
```javascript
// Global state object
const AppState = {
  inputSeed: null,
  inputSeedWordCount: 0,
  transformedSeed: null,
  scypherPassword: null,
  selectedSource: 'input',
  bip39Passphrase: '',
  selectedNetworks: [],
  addressCount: 1,
  derivationResults: {},
  processing: {
    active: false,
    type: null,
    progress: 0,
    current: '',
    cancelable: false
  }
};

// Tab management
function initializeTabs() {
  const tabButtons = document.querySelectorAll('.tab-button');
  const tabContents = document.querySelectorAll('.tab-content');
  
  tabButtons.forEach(button => {
    button.addEventListener('click', function() {
      if (this.classList.contains('disabled')) return;
      if (AppState.processing.active) return; // Block during processing
      
      const targetTab = this.dataset.tab;
      switchToTab(targetTab);
    });
  });
}

function switchToTab(tabName) {
  // Update tab buttons
  document.querySelectorAll('.tab-button').forEach(btn => {
    btn.classList.remove('active');
  });
  document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');
  
  // Update tab content
  document.querySelectorAll('.tab-content').forEach(content => {
    content.classList.remove('active');
  });
  document.getElementById(`${tabName}Tab`).classList.add('active');
  
  // Update tab-specific state
  updateTabState(tabName);
}

function updateTabState(tabName) {
  if (tabName === 'derive') {
    updateSeedSourcePreviews();
    updateDerivationButtonState();
    updateBip39Warning();
  }
}
```

### Network Selection Logic
```javascript
function initializeNetworkSelection() {
  const networkButtons = document.querySelectorAll('.network-button');
  
  networkButtons.forEach(button => {
    const network = button.dataset.network;
    const color = button.dataset.color;
    
    // Set CSS custom properties for dynamic coloring
    button.style.setProperty('--network-color', color);
    button.style.setProperty('--network-color-rgb', hexToRgb(color));
    
    button.addEventListener('click', function() {
      toggleNetworkSelection(network, this);
    });
  });
  
  // Control buttons
  document.querySelector('.select-all').addEventListener('click', selectAllNetworks);
  document.querySelector('.clear-all').addEventListener('click', clearAllNetworks);
}

function toggleNetworkSelection(network, buttonElement) {
  const index = AppState.selectedNetworks.indexOf(network);
  
  if (index > -1) {
    AppState.selectedNetworks.splice(index, 1);
    buttonElement.classList.remove('selected');
  } else {
    AppState.selectedNetworks.push(network);
    buttonElement.classList.add('selected');
  }
  
  updateDerivationButtonState();
  updateBip39Warning();
}

function updateBip39Warning() {
  const warning = document.getElementById('bip39Warning');
  const passphrase = document.getElementById('bip39Passphrase').value;
  
  if (passphrase && AppState.selectedNetworks.some(net => 
    ['cardano', 'solana'].includes(net))) {
    warning.style.display = 'block';
  } else {
    warning.style.display = 'none';
  }
}
```

### Address Derivation Process
```javascript
async function deriveAddresses() {
  const selectedSource = document.querySelector('input[name="seedSource"]:checked').value;
  const bip39Passphrase = document.getElementById('bip39Passphrase').value;
  const addressCount = parseInt(document.getElementById('addressCount').value);
  
  // Validate inputs
  if (AppState.selectedNetworks.length === 0) {
    showToast('Please select at least one network', 'warning');
    return;
  }
  
  // Get source seed
  const seedPhrase = selectedSource === 'input' ? 
    AppState.inputSeed : AppState.transformedSeed;
  
  if (!seedPhrase) {
    showToast('No valid seed phrase available', 'error');
    return;
  }
  
  // Start processing
  AppState.processing = {
    active: true,
    type: 'derive',
    progress: 0,
    current: 'Preparing derivation...',
    cancelable: true
  };
  
  updateUIForProcessing(true);
  
  try {
    const results = {};
    const totalNetworks = AppState.selectedNetworks.length;
    
    for (let i = 0; i < totalNetworks; i++) {
      if (!AppState.processing.active) break; // Check for cancellation
      
      const network = AppState.selectedNetworks[i];
      AppState.processing.current = `Generating ${network} addresses...`;
      AppState.processing.progress = ((i / totalNetworks) * 100);
      updateProgressDisplay();
      
      // Create network config for backend
      const networkConfigs = {};
      networkConfigs[network] = {
        count: addressCount,
        use_passphrase: !['cardano', 'solana'].includes(network) && !!bip39Passphrase
      };
      
      // Call backend
      const networkResult = await invoke('derive_addresses_with_config', {
        seedPhrase: seedPhrase,
        passphrase: bip39Passphrase || null,
        networkConfigs: networkConfigs
      });
      
      results[network] = networkResult[network] || [];
    }
    
    if (AppState.processing.active) {
      AppState.derivationResults = results;
      displayResults(results, selectedSource);
      showToast(`Successfully derived addresses for ${totalNetworks} networks`, 'success');
    }
    
  } catch (error) {
    console.error('Derivation error:', error);
    showToast(`Derivation failed: ${error.message}`, 'error');
  } finally {
    AppState.processing.active = false;
    updateUIForProcessing(false);
  }
}

function cancelDerivation() {
  if (AppState.processing.active && AppState.processing.cancelable) {
    AppState.processing.active = false;
    showToast('Derivation cancelled by user', 'info');
  }
}
```

### Results Display Management
```javascript
function displayResults(results, sourceType) {
  const resultsSection = document.getElementById('resultsSection');
  const sourceIndicator = document.getElementById('sourceIndicator');
  const resultsContent = document.getElementById('resultsContent');
  
  // Update source indicator
  sourceIndicator.textContent = sourceType === 'input' ? 
    'Input seed phrase' : 'SCypher-transformed seed';
  
  // Show results section
  resultsSection.style.display = 'block';
  resultsSection.classList.add('fade-in');
  
  // Generate content based on view mode
  const viewMode = document.getElementById('viewMode').value;
  
  if (viewMode === 'detailed') {
    resultsContent.innerHTML = generateDetailedView(results);
  } else {
    resultsContent.innerHTML = generateCompactView(results);
  }
  
  // Setup export functionality
  setupExportHandlers(results, sourceType);
}

function generateDetailedView(results) {
  let html = '';
  
  for (const [network, addresses] of Object.entries(results)) {
    if (addresses.length === 0) continue;
    
    const networkInfo = getNetworkInfo(network);
    html += `
      <div class="network-result" style="--network-color: ${networkInfo.color}">
        <div class="network-header">
          <span class="network-icon">${networkInfo.icon}</span>
          <span class="network-title">${networkInfo.name} (${addresses.length} addresses)</span>
        </div>
        <div class="address-list">
    `;
    
    addresses.forEach((addr, index) => {
      html += `
        <div class="address-item">
          <span class="address-index">${index + 1}.</span>
          <span class="address-text">${addr.address}</span>
          <div class="address-actions">
            <button class="action-button" onclick="copyAddress('${addr.address}')">📋</button>
            <button class="action-button" onclick="showQR('${addr.address}')">📱</button>
          </div>
        </div>
      `;
    });
    
    html += `
        </div>
      </div>
    `;
  }
  
  return html;
}

function generateCompactView(results) {
  let html = '';
  
  for (const [network, addresses] of Object.entries(results)) {
    if (addresses.length === 0) continue;
    
    const networkInfo = getNetworkInfo(network);
    html += `
      <div class="compact-result" style="--network-color: ${networkInfo.color}">
        <div class="compact-network">
          <span class="network-icon">${networkInfo.icon}</span>
          <span class="network-name">${networkInfo.name}</span>
          <span class="compact-count">(${addresses.length})</span>
        </div>
        <div class="compact-actions">
          <button class="action-button" onclick="expandNetwork('${network}')">Expand</button>
          <button class="action-button" onclick="copyNetworkAddresses('${network}')">Copy All</button>
        </div>
      </div>
    `;
  }
  
  return html;
}
```

### Memory Management and Confirmations
```javascript
function handleNewSeedInput() {
  // Check if there are existing results that would be cleared
  const hasResults = Object.keys(AppState.derivationResults).length > 0;
  const hasTransformed = AppState.transformedSeed !== null;
  
  if (hasResults || hasTransformed) {
    const message = hasResults ? 
      'This will clear existing derivation results. Continue?' :
      'This will clear the transformed seed. Continue?';
      
    if (!confirm(message)) {
      return false; // User cancelled
    }
  }
  
  // Clear relevant state
  AppState.derivationResults = {};
  AppState.transformedSeed = null;
  AppState.scypherPassword = null;
  
  // Update UI
  document.getElementById('resultsSection').style.display = 'none';
  updateSeedSourcePreviews();
  updateTabHeaders();
  
  return true;
}

function updateSeedSourcePreviews() {
  const inputPreview = document.getElementById('inputSeedPreview');
  const transformedPreview = document.getElementById('transformedSeedPreview');
  const inputWordCount = document.getElementById('inputWordCount');
  const transformedOption = document.getElementById('transformedSourceOption');
  
  // Update input seed preview
  if (AppState.inputSeed) {
    inputPreview.textContent = AppState.inputSeed;
    inputWordCount.textContent = AppState.inputSeedWordCount;
  } else {
    inputPreview.textContent = 'No seed phrase entered';
    inputWordCount.textContent = '0';
  }
  
  // Update transformed seed preview and availability
  if (AppState.transformedSeed) {
    transformedPreview.textContent = AppState.transformedSeed;
    transformedOption.removeAttribute('disabled');
    transformedOption.querySelector('input').disabled = false;
  } else {
    transformedPreview.textContent = 'Transform a seed first';
    transformedOption.setAttribute('disabled', true);
    transformedOption.querySelector('input').disabled = true;
    
    // Reset to input if transformed was selected
    if (AppState.selectedSource === 'transformed') {
      AppState.selectedSource = 'input';
      document.querySelector('input[name="seedSource"][value="input"]').checked = true;
    }
  }
}
```

### Export Functionality
```javascript
function setupExportHandlers(results, sourceType) {
  const exportSelector = document.getElementById('exportFormat');
  
  exportSelector.addEventListener('change', function() {
    const format = this.value;
    if (format) {
      exportResults(results, sourceType, format);
      this.value = ''; // Reset selector
    }
  });
}

async function exportResults(results, sourceType, format) {
  const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
  const filename = `SCypher_addresses_${timestamp}`;
  
  let content = '';
  let extension = '';
  
  switch (format) {
    case 'json':
      content = JSON.stringify({
        source: sourceType,
        bip39_passphrase: document.getElementById('bip39Passphrase').value ? 'used' : 'not_used',
        generated_at: new Date().toISOString(),
        networks: results
      }, null, 2);
      extension = 'json';
      break;
      
    case 'csv':
      content = generateCSVContent(results);
      extension = 'csv';
      break;
      
    case 'txt':
      content = generateTXTContent(results, sourceType);
      extension = 'txt';
      break;
  }
  
  try {
    await invoke('save_addresses_file', {
      content: content,
      filename: `${filename}.${extension}`
    });
    showToast(`Addresses exported as ${format.toUpperCase()}`, 'success');
  } catch (error) {
    console.error('Export error:', error);
    showToast(`Export failed: ${error.message}`, 'error');
  }
}

function generateCSVContent(results) {
  let csv = 'Network,Address_Type,Derivation_Path,Address\n';
  
  for (const [network, addresses] of Object.entries(results)) {
    addresses.forEach(addr => {
      csv += `${network},"${addr.address_type}","${addr.path}","${addr.address}"\n`;
    });
  }
  
  return csv;
}

function generateTXTContent(results, sourceType) {
  let txt = `SCypher Address Derivation Results\n`;
  txt += `Generated: ${new Date().toISOString()}\n`;
  txt += `Source: ${sourceType === 'input' ? 'Input seed phrase' : 'SCypher-transformed seed'}\n`;
  txt += `BIP39 Passphrase: ${document.getElementById('bip39Passphrase').value ? 'Used' : 'Not used'}\n`;
  txt += `\n${'='.repeat(80)}\n\n`;
  
  for (const [network, addresses] of Object.entries(results)) {
    const networkInfo = getNetworkInfo(network);
    txt += `${networkInfo.icon} ${networkInfo.name.toUpperCase()} (${addresses.length} addresses)\n`;
    txt += `${'-'.repeat(50)}\n`;
    
    addresses.forEach((addr, index) => {
      txt += `${index + 1}. ${addr.address_type}\n`;
      txt += `    Path: ${addr.path}\n`;
      txt += `    Address: ${addr.address}\n\n`;
    });
    
    txt += `\n`;
  }
  
  return txt;
}
```

### Helper Functions
```javascript
function getNetworkInfo(network) {
  const networkMap = {
    bitcoin: { name: 'Bitcoin', icon: '₿', color: '#F7931A' },
    ethereum: { name: 'Ethereum', icon: 'Ξ', color: '#627EEA' },
    ergo: { name: 'Ergo', icon: '⚡', color: '#FF5722' },
    tron: { name: 'TRON', icon: '🔺', color: '#FF0013' },
    bsc: { name: 'BSC', icon: '⚪', color: '#F0B90B' },
    polygon: { name: 'Polygon', icon: '◆', color: '#8247E5' },
    cardano: { name: 'Cardano', icon: '₳', color: '#0033AD' },
    dogecoin: { name: 'Dogecoin', icon: 'Ð', color: '#C2A633' },
    litecoin: { name: 'Litecoin', icon: 'Ł', color: '#BFBBBB' },
    solana: { name: 'Solana', icon: '◉', color: '#00FFA3' }
  };
  
  return networkMap[network] || { name: network, icon: '●', color: '#888' };
}

function hexToRgb(hex) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result ? 
    `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}` : 
    '136, 136, 136';
}

function updateTabHeaders() {
  const transformTab = document.querySelector('[data-tab="transform"]');
  const deriveTab = document.querySelector('[data-tab="derive"]');
  
  // Transform tab status
  if (AppState.transformedSeed) {
    transformTab.innerHTML = 'Transform Seed <span class="tab-status">✅</span>';
  } else {
    transformTab.innerHTML = 'Transform Seed';
  }
  
  // Derive tab status
  const sourceType = AppState.selectedSource === 'input' ? 'Input' : 'Transformed';
  deriveTab.innerHTML = `Derive Addresses <span class="tab-status">(${sourceType})</span>`;
}

function updateUIForProcessing(isProcessing) {
  // Lock/unlock tabs
  document.querySelectorAll('.tab-button').forEach(btn => {
    if (isProcessing) {
      btn.classList.add('disabled');
    } else {
      btn.classList.remove('disabled');
    }
  });
  
  // Update derive button
  const deriveButton = document.getElementById('deriveButton');
  const progressSection = document.getElementById('progressSection');
  
  if (isProcessing) {
    deriveButton.style.display = 'none';
    progressSection.style.display = 'block';
  } else {
    deriveButton.style.display = 'block';
    progressSection.style.display = 'none';
  }
}

function updateProgressDisplay() {
  const progressFill = document.getElementById('deriveProgressFill');
  const progressText = document.getElementById('progressText');
  
  progressFill.style.width = `${AppState.processing.progress}%`;
  progressText.textContent = AppState.processing.current;
}

async function copyAddress(address) {
  try {
    await navigator.clipboard.writeText(address);
    showToast('Address copied to clipboard', 'success', 2000);
  } catch (error) {
    console.error('Copy failed:', error);
    showToast('Failed to copy address', 'error');
  }
}

async function copyNetworkAddresses(network) {
  const addresses = AppState.derivationResults[network];
  if (!addresses) return;
  
  const addressList = addresses.map(addr => addr.address).join('\n');
  
  try {
    await navigator.clipboard.writeText(addressList);
    showToast(`${addresses.length} ${network} addresses copied`, 'success', 2000);
  } catch (error) {
    console.error('Copy failed:', error);
    showToast('Failed to copy addresses', 'error');
  }
}
```

## Tooltip System Implementation

### Tooltip HTML Structure
```html
<!-- Add to head section -->
<div id="tooltipContainer" class="tooltip-container">
  <div class="tooltip-content" id="tooltipContent"></div>
  <div class="tooltip-arrow" id="tooltipArrow"></div>
</div>
```

### Tooltip CSS
```css
.tooltip-container {
  position: absolute;
  background: rgba(20, 20, 20, 0.95);
  border: 1px solid #555;
  border-radius: 6px;
  padding: 12px 16px;
  max-width: 300px;
  font-size: 0.85em;
  color: #e0e0e0;
  z-index: 10000;
  opacity: 0;
  transform: translateY(10px);
  transition: all 0.3s ease;
  pointer-events: none;
  backdrop-filter: blur(10px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.tooltip-container.show {
  opacity: 1;
  transform: translateY(0);
}

.tooltip-content {
  line-height: 1.4;
}

.tooltip-arrow {
  position: absolute;
  width: 0;
  height: 0;
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-bottom: 6px solid rgba(20, 20, 20, 0.95);
  top: -6px;
  left: 50%;
  transform: translateX(-50%);
}
```

### Tooltip JavaScript
```javascript
function initializeTooltips() {
  const tooltipElements = [
    {
      element: '#bip39Passphrase',
      content: 'Official BIP39 standard feature. Adds entropy during address derivation. Supported by hardware wallets. Completely separate from SCypher.'
    },
    {
      element: '#passwordInput',
      content: 'SCypher proprietary method. Creates encrypted seed using XOR + Argon2id. NOT related to BIP39 standard.'
    },
    {
      element: '.seed-source-container',
      content: 'Input = What you typed/imported. Transformed = Processed with SCypher method.'
    },
    {
      element: '.network-grid',
      content: 'Each network will generate the specified number of addresses using standard derivation paths.'
    }
  ];
  
  tooltipElements.forEach(({ element, content }) => {
    const el = document.querySelector(element);
    if (el) {
      el.addEventListener('mouseenter', (e) => showTooltip(e, content));
      el.addEventListener('mouseleave', hideTooltip);
    }
  });
}

function showTooltip(event, content) {
  const tooltip = document.getElementById('tooltipContainer');
  const tooltipContent = document.getElementById('tooltipContent');
  
  tooltipContent.textContent = content;
  
  // Position tooltip
  const rect = event.target.getBoundingClientRect();
  tooltip.style.left = `${rect.left + (rect.width / 2)}px`;
  tooltip.style.top = `${rect.top - 10}px`;
  tooltip.style.transform = 'translateX(-50%) translateY(-100%)';
  
  tooltip.classList.add('show');
}

function hideTooltip() {
  document.getElementById('tooltipContainer').classList.remove('show');
}
```

## Backend Integration Requirements

### New Tauri Commands Needed

```rust
// Add to commands.rs

#[tauri::command]
pub async fn derive_addresses_with_config(
    seed_phrase: String,
    passphrase: Option<String>,
    network_configs: std::collections::HashMap<String, crate::addresses::NetworkConfig>,
) -> Result<crate::addresses::AddressSet, String> {
    crate::addresses::derive_addresses_with_config(&seed_phrase, passphrase.as_deref(), network_configs)
        .map_err(|e| format!("Address derivation failed: {}", e))
}

#[tauri::command]
pub async fn save_addresses_file(
    content: String,
    filename: String,
) -> Result<(), String> {
    use tauri::api::dialog::FileDialogBuilder;
    
    let path = FileDialogBuilder::new()
        .set_file_name(&filename)
        .save_file()
        .ok_or("File save cancelled")?;
    
    std::fs::write(path, content)
        .map_err(|e| format!("Failed to save file: {}", e))
}
```

### Update Main Tauri Setup
```rust
// Add to main.rs in tauri::Builder commands
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    derive_addresses_with_config,
    save_addresses_file
])
```

## Complete HTML Structure Update

### Replace Existing HTML Body
```html
<body>
    <div class="container">
        <div class="header">
            <div class="logo">SCypher</div>
            <div class="tagline">XOR-based BIP39 Seed Cipher v3.0</div>
            <button class="secure-mode-toggle" id="secureMode">
                🛡️ Secure Screen: OFF
            </button>
        </div>

        <!-- Tab Navigation -->
        <div class="tab-container">
            <div class="tab-header">
                <button class="tab-button active" data-tab="transform">
                    Transform Seed
                </button>
                <button class="tab-button" data-tab="derive">
                    Derive Addresses <span class="tab-status">(Select Source)</span>
                </button>
            </div>
        </div>

        <!-- Tab 1: Transform Seed (Enhanced Existing) -->
        <div class="tab-content active" id="transformTab">
            <div class="main-content" id="mainContent">
                <div class="warning-banner">
                    <strong>⚠️ Security Notice:</strong> Only process seed phrases you own. Never share encrypted results with untrusted parties.
                </div>

                <div class="section">
                    <div class="section-title">📝 Input Seed Phrase</div>
                    <!-- Existing input system preserved -->
                    <div class="input-controls">
                        <label style="color: #ddd; font-size: 0.9em;">Mode:</label>
                        <select class="word-count-selector auto" id="wordCount">
                            <option value="auto">AUTO (detect)</option>
                            <option value="12">12 words</option>
                            <option value="15">15 words</option>
                            <option value="18">18 words</option>
                            <option value="21">21 words</option>
                            <option value="24">24 words</option>
                        </select>

                        <div class="control-buttons">
                            <button class="small-button generate" id="generateSeed">🎲 Generate New Seed</button>
                            <button class="small-button clear" id="clearInput">🗑️ Clear</button>
                            <button class="small-button" id="browseFile">📁 Browse</button>
                        </div>
                    </div>

                    <div class="advanced-input-area">
                        <div class="word-input-container" id="wordContainer">
                            <div class="words-display" id="wordsDisplay"></div>
                            <input type="text" class="word-input" id="wordInput" autocomplete="off" spellcheck="false">
                            <div class="autocomplete-dropdown" id="autocompleteDropdown"></div>
                        </div>
                        <div class="input-placeholder" id="inputPlaceholder">Start typing or drag & drop a .txt file...</div>
                    </div>

                    <div class="validation-status" id="validationStatus">
                        <span class="status-progress">Ready to input seed phrase • AUTO mode active</span>
                    </div>
                </div>

                <div class="section">
                    <div class="section-title">🔐 SCypher Security Settings</div>
                    <div class="security-section">
                        <div class="info-card">
                            <div class="info-title">SCypher Method: Proprietary XOR + Argon2id transformation</div>
                            <div class="info-desc">Purpose: Create encrypted variant of your seed phrase</div>
                        </div>
                        
                        <div class="input-group">
                            <label class="label">SCypher Password:</label>
                            <div class="password-input">
                                <input type="password" class="input" id="passwordInput" placeholder="Enter a strong password">
                                <button class="password-toggle" id="passwordToggle" type="button">👁️</button>
                            </div>
                        </div>

                        <div class="argon2-display">
                            <div class="param-display">
                                <div class="param-label">Argon2id Iterations</div>
                                <div class="param-value">5</div>
                            </div>
                            <div class="param-display">
                                <div class="param-label">Memory Cost</div>
                                <div class="param-value">128 MB</div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="action-buttons">
                    <button class="button" id="processButton" disabled>⏳ Enter seed phrase and password...</button>
                </div>

                <div class="progress-bar" id="progressBar">
                    <div class="progress-fill" id="progressFill"></div>
                </div>

                <div class="result-area" id="resultArea">
                    <div class="section-title">✅ Transformation Result</div>
                    <div class="result-text" id="resultText"></div>
                    <div class="result-actions">
                        <button class="copy-button" onclick="copyToClipboard()">📋 Copy to Clipboard</button>
                        <button class="copy-button" onclick="saveToFile()">💾 Save to File</button>
                        <button class="copy-button" onclick="useSeedForDerivation()">🔗 Use for Derivation →</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Tab 2: Derive Addresses (New) -->
        <div class="tab-content" id="deriveTab">
            <div class="main-content">
                <!-- Seed Source Selection -->
                <div class="section">
                    <div class="section-title">🔄 Seed Source Selection</div>
                    <div class="seed-source-container">
                        <label class="source-option">
                            <input type="radio" name="seedSource" value="input" checked>
                            <div class="source-preview">
                                <div class="source-title">Input seed phrase (<span id="inputWordCount">0</span> words)</div>
                                <div class="source-text" id="inputSeedPreview">No seed phrase entered</div>
                            </div>
                        </label>
                        
                        <label class="source-option" id="transformedSourceOption" disabled>
                            <input type="radio" name="seedSource" value="transformed" disabled>
                            <div class="source-preview">
                                <div class="source-title">SCypher-transformed seed ✅ Available</div>
                                <div class="source-text" id="transformedSeedPreview">Transform a seed first</div>
                            </div>
                        </label>
                    </div>
                </div>

                <!-- BIP39 Settings -->
                <div class="section">
                    <div class="section-title">🌐 BIP39 Standard Settings</div>
                    <div class="bip39-section">
                        <div class="info-card">
                            <div class="info-title">BIP39 Standard: Official Bitcoin Improvement Proposal</div>
                            <div class="info-desc">Purpose: Additional entropy during address derivation</div>
                        </div>
                        <div class="input-group">
                            <label class="label">BIP39 Passphrase (optional):</label>
                            <div class="password-input">
                                <input type="password" class="input" id="bip39Passphrase" 
                                       placeholder="Enter BIP39 passphrase (leave empty for standard derivation)">
                                <button class="password-toggle" type="button" onclick="togglePasswordVisibility('bip39Passphrase', this)">👁️</button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Network Selection -->
                <div class="section">
                    <div class="section-title">🌍 Select Networks</div>
                    <div class="network-grid">
                        <button class="network-button" data-network="bitcoin" data-color="#F7931A">
                            <span class="network-icon">₿</span>
                            <span class="network-name">Bitcoin</span>
                        </button>
                        <button class="network-button" data-network="ethereum" data-color="#627EEA">
                            <span class="network-icon">Ξ</span>
                            <span class="network-name">Ethereum</span>
                        </button>
                        <button class="network-button" data-network="ergo" data-color="#FF5722">
                            <span class="network-icon">⚡</span>
                            <span class="network-name">Ergo</span>
                        </button>
                        <button class="network-button" data-network="tron" data-color="#FF0013">
                            <span class="network-icon">🔺</span>
                            <span class="network-name">TRON</span>
                        </button>
                        <button class="network-button" data-network="bsc" data-color="#F0B90B">
                            <span class="network-icon">⚪</span>
                            <span class="network-name">BSC</span>
                        </button>
                        <button class="network-button" data-network="polygon" data-color="#8247E5">
                            <span class="network-icon">◆</span>
                            <span class="network-name">Polygon</span>
                        </button>
                        <button class="network-button" data-network="cardano" data-color="#0033AD">
                            <span class="network-icon">₳</span>
                            <span class="network-name">Cardano</span>
                        </button>
                        <button class="network-button" data-network="dogecoin" data-color="#C2A633">
                            <span class="network-icon">Ð</span>
                            <span class="network-name">Dogecoin</span>
                        </button>
                        <button class="network-button" data-network="litecoin" data-color="#BFBBBB">
                            <span class="network-icon">Ł</span>
                            <span class="network-name">Litecoin</span>
                        </button>
                        <button class="network-button" data-network="solana" data-color="#00FFA3">
                            <span class="network-icon">◉</span>
                            <span class="network-name">Solana</span>
                        </button>
                    </div>
                    
                    <div class="network-controls">
                        <button class="control-button select-all">🔥 Select All</button>
                        <button class="control-button clear-all">❌ Clear All</button>
                    </div>
                    
                    <div class="bip39-warning" id="bip39Warning" style="display: none;">
                        <strong>⚠️ Notice:</strong> Cardano & Solana will derive WITHOUT BIP39 passphrase (not supported by these networks)
                    </div>
                </div>

                <!-- Address Count -->
                <div class="section">
                    <div class="section-title">⚙️ Address Count</div>
                    <div class="count-config">
                        <label class="label">Generate addresses per network:</label>
                        <div class="count-input-group">
                            <input type="number" class="count-input" id="addressCount" 
                                   value="1" min="1" max="100" placeholder="1">
                            <span class="count-range">(1-100)</span>
                        </div>
                    </div>
                </div>

                <!-- Derive Button and Progress -->
                <div class="action-section">
                    <button class="button derive-button" id="deriveButton" disabled>
                        🚀 Derive Addresses
                    </button>
                    
                    <div class="progress-section" id="progressSection" style="display: none;">
                        <div class="progress-bar">
                            <div class="progress-fill" id="deriveProgressFill"></div>
                        </div>
                        <div class="progress-text" id="progressText">Preparing derivation...</div>
                        <button class="cancel-button" id="cancelDerivation">❌ Cancel Derivation</button>
                    </div>
                </div>

                <!-- Results Display -->
                <div class="results-section" id="resultsSection" style="display: none;">
                    <div class="section-title">📍 Derivation Results</div>
                    
                    <div class="results-header">
                        <div class="results-info" id="resultsInfo">
                            Derived from: <strong id="sourceIndicator">Input seed phrase</strong>
                        </div>
                        <div class="view-controls">
                            <select class="view-selector" id="viewMode">
                                <option value="detailed">Detailed View</option>
                                <option value="compact">Compact View</option>
                            </select>
                            <select class="export-selector" id="exportFormat">
                                <option value="">Export As...</option>
                                <option value="json">JSON</option>
                                <option value="csv">CSV</option>
                                <option value="txt">TXT</option>
                            </select>
                        </div>
                    </div>
                    
                    <div class="results-content" id="resultsContent"></div>
                </div>
            </div>
        </div>

        <!-- Status Bar -->
        <div class="status-bar">
            <div class="status-text" id="statusText">Ready • AUTO mode • BIP39 validation enabled • Argon2id: 5 iterations, 128MB</div>
            <div class="security-indicator">
                <span>🛡️</span>
                <span id="securityStatus">Secure mode ready</span>
            </div>
        </div>
    </div>

    <!-- Toast Container -->
    <div class="toast-container" id="toastContainer"></div>
    
    <!-- Tooltip Container -->
    <div id="tooltipContainer" class="tooltip-container">
        <div class="tooltip-content" id="tooltipContent"></div>
        <div class="tooltip-arrow" id="tooltipArrow"></div>
    </div>

    <!-- Existing script preserved and enhanced -->
    <script>
        // Enhanced existing variables
        let currentWords = [];
        let currentMode = 'auto';
        let editingIndex = -1;
        let highlightedIndex = -1;
        let secureMode = false;
        let bip39WordList = [];
        let invoke = null;
        let tauriAvailable = false;

        // NEW: Global App State
        const AppState = {
            inputSeed: null,
            inputSeedWordCount: 0,
            transformedSeed: null,
            scypherPassword: null,
            selectedSource: 'input',
            bip39Passphrase: '',
            selectedNetworks: [],
            addressCount: 1,
            derivationResults: {},
            processing: {
                active: false,
                type: null,
                progress: 0,
                current: '',
                cancelable: false
            }
        };

        // Initialize application (enhanced)
        document.addEventListener('DOMContentLoaded', async function() {
            console.log('🚀 SCypher GUI v3.0 initializing...');

            try {
                await initializeTauri();
                bip39WordList = await invoke('get_bip39_wordlist');
                console.log(`Loaded ${bip39WordList.length} BIP39 words from backend`);

                // Enhanced initialization
                initializeEventListeners();
                initializeTabs();
                initializeNetworkSelection();
                initializeTooltips();
                updateValidationStatus();
                updateProcessButtonState();
                updateTabHeaders();

                console.log('✅ Application initialized successfully');
                initializeToasts();
            } catch (error) {
                console.error('Failed to initialize:', error);
                alert('Failed to initialize application. Please restart.');
            }
        });

        // Enhanced existing functions + all new functions from the detailed implementation above
        // ... [Include all the JavaScript functions detailed above]
        
    </script>
</body>
```

## Testing Scenarios

### Critical Test Cases
1. **Tab switching during processing** - Should be blocked
2. **Seed source switching** - Should update previews correctly
3. **BIP39 warning display** - Should show for Cardano/Solana when passphrase entered
4. **Network selection** - Colors and states should update properly
5. **Address count validation** - Should enforce 1-100 range
6. **Export functionality** - All formats should work
7. **Cancellation** - Should stop processing gracefully
8. **Memory confirmation** - Should prompt before clearing results
9. **Placeholder fix** - Should disappear when typing starts
10. **Drag & drop preservation** - Should maintain existing functionality

### Edge Cases
- Empty network selection
- Invalid address count
- Very long seed phrases
- Special characters in passwords
- Network timeout handling
- Multiple rapid tab switches

## Implementation Notes

### Preservation Requirements
- **CRITICAL**: Do not modify existing drag & drop functionality
- **CRITICAL**: Preserve all existing word input, validation, and autocomplete systems
- **CRITICAL**: Maintain existing Tauri file drop API implementation
- **CRITICAL**: Keep existing styling patterns and color schemes
- **CRITICAL**: Preserve existing toast notification system

### New File Requirements
No new files needed - all changes are modifications to existing `index.html`

### CSS Integration Strategy
1. Add new CSS classes without modifying existing ones
2. Use CSS custom properties for dynamic network colors
3. Maintain existing responsive design patterns
4. Preserve existing animation and transition styles

### JavaScript Integration Strategy
1. Extend existing global variables with AppState object
2. Enhance existing functions rather than replacing them
3. Add new event listeners without breaking existing ones
4. Maintain existing error handling patterns

### State Synchronization Points
1. **Word input changes** → Update AppState.inputSeed and previews
2. **Transform completion** → Update AppState.transformedSeed and enable Tab 2
3. **Tab switches** → Update headers and validate states
4. **Network selection** → Update AppState.selectedNetworks and warnings
5. **Source selection** → Update AppState.selectedSource and validation

## Complete CSS Additions

### Add to Existing Style Section
```css
/* ===== TAB SYSTEM STYLES ===== */
.tab-container {
  background: rgba(40, 40, 40, 0.95);
  border-radius: 12px 12px 0 0;
  border-bottom: 1px solid #4a4a4a;
}

.tab-header {
  display: flex;
  background: rgba(30, 30, 30, 0.8);
  border-radius: 12px 12px 0 0;
}

.tab-button {
  flex: 1;
  background: none;
  border: none;
  color: #888;
  padding: 16px 24px;
  font-size: 1em;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
  border-bottom: 3px solid transparent;
  position: relative;
}

.tab-button.active {
  color: #ff9500;
  border-bottom-color: #ff9500;
  background: rgba(255, 149, 0, 0.05);
}

.tab-button.disabled {
  opacity: 0.4;
  cursor: not-allowed;
  color: #555;
}

.tab-button .tab-status {
  font-size: 0.8em;
  margin-left: 8px;
  opacity: 0.7;
}

.tab-content {
  display: none;
  animation: fadeIn 0.3s ease-in;
}

.tab-content.active {
  display: block;
}

/* ===== SEED SOURCE SELECTION STYLES ===== */
.seed-source-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.source-option {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 16px;
  background: rgba(60, 60, 60, 0.3);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.source-option:hover:not([disabled]) {
  border-color: rgba(255, 149, 0, 0.3);
  background: rgba(255, 149, 0, 0.05);
}

.source-option input[type="radio"]:checked + .source-preview {
  color: #ff9500;
}

.source-option[disabled] {
  opacity: 0.4;
  cursor: not-allowed;
}

.source-option input[type="radio"] {
  margin-top: 2px;
  accent-color: #ff9500;
}

.source-preview {
  flex: 1;
}

.source-title {
  font-weight: 600;
  margin-bottom: 8px;
  color: inherit;
}

.source-text {
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.85em;
  color: #888;
  word-break: break-all;
  line-height: 1.4;
  max-height: 60px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.source-option input[type="radio"]:checked + .source-preview .source-text {
  color: #ccc;
}

/* ===== BIP39 SECTION STYLES ===== */
.bip39-section {
  background: rgba(30, 30, 30, 0.6);
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 20px;
}

.info-card {
  background: rgba(50, 50, 50, 0.6);
  border: 1px solid #555;
  border-radius: 6px;
  padding: 12px 16px;
  margin-bottom: 15px;
}

.info-title {
  font-weight: 600;
  color: #e0e0e0;
  margin-bottom: 4px;
}

.info-desc {
  font-size: 0.9em;
  color: #888;
}

/* ===== NETWORK SELECTION STYLES ===== */
.network-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 12px;
  margin-bottom: 20px;
}

.network-button {
  background: rgba(60, 60, 60, 0.5);
  border: 2px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 16px 12px;
  color: #ccc;
  cursor: pointer;
  transition: all 0.3s ease;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  font-size: 0.9em;
  font-weight: 500;
}

.network-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  border-color: rgba(255, 255, 255, 0.2);
}

.network-button.selected {
  border-color: var(--network-color);
  background: rgba(var(--network-color-rgb), 0.15);
  color: var(--network-color);
  box-shadow: 0 0 0 2px rgba(var(--network-color-rgb), 0.3);
}

.network-icon {
  font-size: 1.5em;
  font-weight: bold;
}

.network-name {
  font-size: 0.85em;
  text-align: center;
}

.network-controls {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-bottom: 15px;
}

.control-button {
  background: rgba(60, 60, 60, 0.8);
  border: 1px solid #555;
  color: #ccc;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9em;
  transition: all 0.3s ease;
}

.control-button:hover {
  background: rgba(80, 80, 80, 0.8);
  color: #fff;
}

.control-button.select-all {
  border-color: #ff9500;
  color: #ff9500;
}

.control-button.clear-all {
  border-color: #dc2626;
  color: #fca5a5;
}

.bip39-warning {
  background: rgba(255, 193, 7, 0.1);
  border: 1px solid rgba(255, 193, 7, 0.3);
  border-radius: 6px;
  padding: 12px;
  color: #ffc107;
  font-size: 0.9em;
}

/* ===== ADDRESS COUNT STYLES ===== */
.count-config {
  background: rgba(30, 30, 30, 0.6);
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 20px;
}

.count-input-group {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
}

.count-input {
  width: 80px;
  padding: 8px 12px;
  background: rgba(60, 60, 60, 0.5);
  border: 1px solid #555;
  border-radius: 6px;
  color: #e0e0e0;
  font-family: inherit;
  text-align: center;
  font-size: 0.95em;
}

.count-input:focus {
  outline: none;
  border-color: #ff9500;
  background: rgba(60, 60, 60, 0.8);
  box-shadow: 0 0 0 3px rgba(255, 149, 0, 0.1);
}

.count-range {
  color: #888;
  font-size: 0.85em;
}

/* ===== DERIVATION ACTION STYLES ===== */
.action-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
  margin: 30px 0;
}

.derive-button {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
  border: none;
  padding: 14px 24px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 1em;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.3);
  min-width: 180px;
}

.derive-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(16, 185, 129, 0.4);
}

.derive-button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: 0 4px 12px rgba(16, 185, 129, 0.1);
}

.progress-section {
  width: 100%;
  max-width: 400px;
  text-align: center;
}

.progress-text {
  font-size: 0.9em;
  color: #ccc;
  margin-bottom: 15px;
}

.cancel-button {
  background: rgba(220, 38, 38, 0.2);
  border: 1px solid #dc2626;
  color: #fca5a5;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.9em;
  transition: all 0.3s ease;
}

.cancel-button:hover {
  background: rgba(220, 38, 38, 0.3);
  color: #fff;
}

/* ===== RESULTS DISPLAY STYLES ===== */
.results-section {
  background: rgba(30, 30, 30, 0.8);
  border: 1px solid #4a4a4a;
  border-radius: 8px;
  padding: 20px;
  margin-top: 20px;
}

.results-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid #444;
}

.results-info {
  font-size: 0.9em;
  color: #ccc;
}

.view-controls {
  display: flex;
  gap: 10px;
}

.view-selector, .export-selector {
  background: rgba(60, 60, 60, 0.8);
  border: 1px solid #555;
  color: #e0e0e0;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 0.85em;
  cursor: pointer;
}

.view-selector:focus, .export-selector:focus {
  outline: none;
  border-color: #ff9500;
}

/* ===== DETAILED VIEW STYLES ===== */
.network-result {
  margin-bottom: 24px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 8px;
  padding: 16px;
  border-left: 4px solid var(--network-color);
}

.network-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #555;
}

.network-header .network-icon {
  font-size: 1.2em;
  color: var(--network-color);
}

.network-title {
  font-size: 1em;
  font-weight: 600;
  color: #e0e0e0;
}

.address-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.address-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: rgba(20, 20, 20, 0.5);
  border: 1px solid #333;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.address-item:hover {
  background: rgba(30, 30, 30, 0.5);
  border-color: #444;
}

.address-index {
  font-size: 0.8em;
  color: #888;
  min-width: 20px;
  font-weight: 600;
}

.address-text {
  flex: 1;
  font-family: 'Monaco', 'Menlo', monospace;
  font-size: 0.85em;
  color: #e0e0e0;
  word-break: break-all;
  line-height: 1.3;
}

.address-actions {
  display: flex;
  gap: 4px;
}

.action-button {
  background: rgba(60, 60, 60, 0.8);
  border: 1px solid #555;
  color: #ccc;
  padding: 4px 8px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.75em;
  transition: all 0.2s ease;
  min-width: 30px;
  text-align: center;
}

.action-button:hover {
  background: rgba(80, 80, 80, 0.8);
  color: #fff;
  border-color: #666;
}

/* ===== COMPACT VIEW STYLES ===== */
.compact-result {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: rgba(40, 40, 40, 0.6);
  border-radius: 6px;
  margin-bottom: 8px;
  border-left: 3px solid var(--network-color);
  transition: all 0.2s ease;
}

.compact-result:hover {
  background: rgba(50, 50, 50, 0.6);
}

.compact-network {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 120px;
}

.compact-network .network-icon {
  color: var(--network-color);
  font-size: 1.1em;
}

.compact-count {
  color: #888;
  font-size: 0.8em;
  margin-left: 4px;
}

.compact-actions {
  margin-left: auto;
  display: flex;
  gap: 6px;
}

/* ===== TOOLTIP STYLES ===== */
.tooltip-container {
  position: absolute;
  background: rgba(20, 20, 20, 0.95);
  border: 1px solid #555;
  border-radius: 6px;
  padding: 12px 16px;
  max-width: 300px;
  font-size: 0.85em;
  color: #e0e0e0;
  z-index: 10000;
  opacity: 0;
  transform: translateY(10px);
  transition: all 0.3s ease;
  pointer-events: none;
  backdrop-filter: blur(10px);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.tooltip-container.show {
  opacity: 1;
  transform: translateY(0);
}

.tooltip-content {
  line-height: 1.4;
}

.tooltip-arrow {
  position: absolute;
  width: 0;
  height: 0;
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-bottom: 6px solid rgba(20, 20, 20, 0.95);
  top: -6px;
  left: 50%;
  transform: translateX(-50%);
}

/* ===== RESPONSIVE STYLES ===== */
@media (max-width: 768px) {
  .tab-header {
    flex-direction: column;
  }

  .tab-button {
    border-bottom: none;
    border-right: 3px solid transparent;
  }

  .tab-button.active {
    border-right-color: #ff9500;
  }

  .network-grid {
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 8px;
  }

  .network-button {
    padding: 12px 8px;
  }

  .results-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .view-controls {
    justify-content: center;
  }

  .address-item {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .address-actions {
    justify-content: center;
  }

  .compact-result {
    flex-direction: column;
    align-items: stretch;
  }

  .compact-actions {
    margin-left: 0;
    justify-content: center;
  }
}

/* ===== ENHANCED PLACEHOLDER FIX ===== */
.input-placeholder {
  position: absolute;
  top: 50%;
  left: 16px;
  transform: translateY(-50%);
  color: #888;
  pointer-events: none;
  transition: all 0.3s ease;
  z-index: 1;
  font-size: 0.9em;
}

.word-input-container.has-content .input-placeholder,
.word-input-container:focus-within .input-placeholder {
  opacity: 0;
  transform: translateY(-70%);
  pointer-events: none;
}

/* ===== ADDITIONAL BUTTON STYLES ===== */
.result-actions .copy-button:last-child {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  color: white;
  border: none;
}

.result-actions .copy-button:last-child:hover {
  background: linear-gradient(135deg, #059669 0%, #047857 100%);
}
```

## Complete JavaScript Integration

### Key Integration Points

1. **Enhanced existing DOMContentLoaded** - Add new initializations
2. **Enhanced existing word input handling** - Update AppState
3. **Enhanced existing transform process** - Update AppState.transformedSeed
4. **New tab management system** - Complete implementation
5. **New network selection system** - Complete implementation
6. **New derivation process** - Complete implementation

### Critical Modification Points in Existing Code

```javascript
// MODIFY existing updateValidationStatus function
async function updateValidationStatus() {
    const container = document.getElementById('wordContainer');
    const statusElement = document.getElementById('validationStatus');
    const validation = await validateCurrentPhrase();

    // EXISTING CODE PRESERVED
    container.className = 'word-input-container';
    if (validation.status === 'valid') {
        container.classList.add('valid');
    } else if (validation.status === 'invalid') {
        container.classList.add('invalid');
    }

    // NEW: Update AppState
    if (validation.valid) {
        AppState.inputSeed = currentWords.join(' ');
        AppState.inputSeedWordCount = currentWords.length;
    } else {
        AppState.inputSeed = null;
        AppState.inputSeedWordCount = 0;
    }

    // NEW: Update seed source previews
    updateSeedSourcePreviews();
    updateTabHeaders();

    const statusClass = validation.status === 'valid' ? 'status-valid' :
                       validation.status === 'invalid' ? 'status-invalid' :
                       validation.status === 'progress' ? 'status-progress' : 'status-progress';

    statusElement.innerHTML = `<span class="${statusClass}">${validation.message}</span>`;

    return validation;
}

// MODIFY existing processSeed function
async function processSeed() {
    // EXISTING CODE PRESERVED...
    const phrase = currentWords.join(' ');
    const password = document.getElementById('passwordInput').value;

    const progressBar = document.getElementById('progressBar');
    const progressFill = document.getElementById('progressFill');
    const resultArea = document.getElementById('resultArea');
    const statusText = document.getElementById('statusText');
    const processButton = document.getElementById('processButton');

    processButton.disabled = true;
    processButton.textContent = '⏳ Processing...';

    progressBar.classList.add('show');
    progressFill.style.width = '10%';

    const startTime = Date.now();
    statusText.textContent = 'Starting real-time processing...';

    try {
        console.log('🔄 Starting processing at:', new Date().toISOString());

        progressFill.style.width = '50%';
        statusText.textContent = 'Processing with Argon2id and XOR...';

        const result = await invoke('transform_seed_phrase', {
            phrase: phrase,
            password: password,
            iterations: 5,
            memoryCost: 131072
        });

        const processingTime = Date.now() - startTime;
        console.log(`✅ Real processing time: ${processingTime}ms`);

        progressFill.style.width = '100%';
        statusText.textContent = `Completed in ${processingTime}ms (real time)`;

        await new Promise(resolve => setTimeout(resolve, 500));

        if (result.success && result.result) {
            document.getElementById('resultText').textContent = result.result;
            progressBar.classList.remove('show');
            resultArea.classList.add('show');
            statusText.textContent = `Ready • Real processing: ${processingTime}ms • ${currentWords.length} words`;
            processButton.disabled = false;
            processButton.textContent = '🔄 Process Seed Phrase';

            // NEW: Update AppState
            AppState.transformedSeed = result.result;
            AppState.scypherPassword = password;

            // NEW: Update UI
            updateSeedSourcePreviews();
            updateTabHeaders();

            showToast(`Transformation completed in ${processingTime}ms`, 'success');
        } else {
            throw new Error(result.error || 'Unknown error');
        }

    } catch (error) {
        const processingTime = Date.now() - startTime;
        console.error('❌ Processing error after', processingTime, 'ms:', error);

        progressBar.classList.remove('show');
        processButton.disabled = false;
        processButton.textContent = '🔄 Process Seed Phrase';
        statusText.textContent = `Error after ${processingTime}ms`;

        showToast(`Processing failed: ${error.message || error}`, 'error');
    }
}

// MODIFY existing clearAllWords function
function clearAllWords() {
    // NEW: Check for confirmation first
    if (!handleNewSeedInput()) {
        return; // User cancelled
    }

    // EXISTING CODE PRESERVED
    currentWords = [];
    editingIndex = -1;
    document.getElementById('wordInput').value = '';
    document.getElementById('passwordInput').value = '';
    document.getElementById('resultArea').classList.remove('show');
    document.getElementById('resultText').textContent = '';

    renderWords();
    hideAutocomplete();
    updateValidationStatus();
    updateProcessButtonState();
    focusWordInput();

    document.getElementById('statusText').textContent = 'Ready • AUTO mode • BIP39 validation enabled • Argon2id: 5 iterations, 128MB';
}

// NEW: Add to existing DOMContentLoaded initialization
document.addEventListener('DOMContentLoaded', async function() {
    console.log('🚀 SCypher GUI v3.0 initializing...');

    try {
        await initializeTauri();
        bip39WordList = await invoke('get_bip39_wordlist');
        console.log(`Loaded ${bip39WordList.length} BIP39 words from backend`);

        // EXISTING CODE PRESERVED
        initializeEventListeners();
        updateValidationStatus();
        updateProcessButtonState();

        // NEW: Enhanced initialization
        initializeTabs();
        initializeNetworkSelection();
        initializeTooltips();
        updateTabHeaders();

        console.log('✅ Application initialized successfully');
        initializeToasts();
    } catch (error) {
        console.error('Failed to initialize:', error);
        alert('Failed to initialize application. Please restart.');
    }
});
```

### All New JavaScript Functions

```javascript
// Tab Management System
function initializeTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');

    tabButtons.forEach(button => {
        button.addEventListener('click', function() {
            if (this.classList.contains('disabled')) return;
            if (AppState.processing.active) return; // Block during processing

            const targetTab = this.dataset.tab;
            switchToTab(targetTab);
        });
    });
}

function switchToTab(tabName) {
    // Update tab buttons
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
    });
    document.querySelector(`[data-tab="${tabName}"]`).classList.add('active');

    // Update tab content
    document.querySelectorAll('.tab-content').forEach(content => {
        content.classList.remove('active');
    });
    document.getElementById(`${tabName}Tab`).classList.add('active');

    // Update tab-specific state
    updateTabState(tabName);
}

function updateTabState(tabName) {
    if (tabName === 'derive') {
        updateSeedSourcePreviews();
        updateDerivationButtonState();
        updateBip39Warning();
    }
}

function updateTabHeaders() {
    const transformTab = document.querySelector('[data-tab="transform"]');
    const deriveTab = document.querySelector('[data-tab="derive"]');

    // Transform tab status
    if (AppState.transformedSeed) {
        transformTab.innerHTML = 'Transform Seed <span class="tab-status">✅</span>';
    } else {
        transformTab.innerHTML = 'Transform Seed';
    }

    // Derive tab status
    const sourceType = AppState.selectedSource === 'input' ? 'Input' : 'Transformed';
    deriveTab.innerHTML = `Derive Addresses <span class="tab-status">(${sourceType})</span>`;
}

// Network Selection System
function initializeNetworkSelection() {
    const networkButtons = document.querySelectorAll('.network-button');

    networkButtons.forEach(button => {
        const network = button.dataset.network;
        const color = button.dataset.color;

        // Set CSS custom properties for dynamic coloring
        button.style.setProperty('--network-color', color);
        button.style.setProperty('--network-color-rgb', hexToRgb(color));

        button.addEventListener('click', function() {
            toggleNetworkSelection(network, this);
        });
    });

    // Control buttons
    document.querySelector('.select-all').addEventListener('click', selectAllNetworks);
    document.querySelector('.clear-all').addEventListener('click', clearAllNetworks);

    // Address count input
    const addressCountInput = document.getElementById('addressCount');
    addressCountInput.addEventListener('click', function() {
        if (this.value === '1') {
            this.select();
        }
    });

    addressCountInput.addEventListener('input', function() {
        let value = parseInt(this.value);
        if (isNaN(value) || value < 1) {
            this.value = 1;
        } else if (value > 100) {
            this.value = 100;
        }
        AppState.addressCount = parseInt(this.value);
        updateDerivationButtonState();
    });

    // Source selection
    document.querySelectorAll('input[name="seedSource"]').forEach(radio => {
        radio.addEventListener('change', function() {
            AppState.selectedSource = this.value;
            updateDerivationButtonState();
            updateTabHeaders();
        });
    });

    // BIP39 passphrase
    document.getElementById('bip39Passphrase').addEventListener('input', function() {
        AppState.bip39Passphrase = this.value;
        updateBip39Warning();
    });

    // Derive button
    document.getElementById('deriveButton').addEventListener('click', deriveAddresses);

    // Cancel button
    document.getElementById('cancelDerivation').addEventListener('click', cancelDerivation);

    // View mode selector
    document.getElementById('viewMode').addEventListener('change', function() {
        if (Object.keys(AppState.derivationResults).length > 0) {
            displayResults(AppState.derivationResults, AppState.selectedSource);
        }
    });
}

function toggleNetworkSelection(network, buttonElement) {
    const index = AppState.selectedNetworks.indexOf(network);

    if (index > -1) {
        AppState.selectedNetworks.splice(index, 1);
        buttonElement.classList.remove('selected');
    } else {
        AppState.selectedNetworks.push(network);
        buttonElement.classList.add('selected');
    }

    updateDerivationButtonState();
    updateBip39Warning();
}

function selectAllNetworks() {
    const allNetworks = ['bitcoin', 'ethereum', 'ergo', 'tron', 'bsc', 'polygon', 'cardano', 'dogecoin', 'litecoin', 'solana'];
    AppState.selectedNetworks = [...allNetworks];

    document.querySelectorAll('.network-button').forEach(button => {
        button.classList.add('selected');
    });

    updateDerivationButtonState();
    updateBip39Warning();
}

function clearAllNetworks() {
    AppState.selectedNetworks = [];

    document.querySelectorAll('.network-button').forEach(button => {
        button.classList.remove('selected');
    });

    updateDerivationButtonState();
    updateBip39Warning();
}

function updateBip39Warning() {
    const warning = document.getElementById('bip39Warning');
    const passphrase = document.getElementById('bip39Passphrase').value;

    if (passphrase && AppState.selectedNetworks.some(net =>
        ['cardano', 'solana'].includes(net))) {
        warning.style.display = 'block';
    } else {
        warning.style.display = 'none';
    }
}

function updateDerivationButtonState() {
    const deriveButton = document.getElementById('deriveButton');

    const hasNetworks = AppState.selectedNetworks.length > 0;
    const hasValidSeed = (AppState.selectedSource === 'input' && AppState.inputSeed) ||
                        (AppState.selectedSource === 'transformed' && AppState.transformedSeed);

    deriveButton.disabled = !(hasNetworks && hasValidSeed);

    if (hasNetworks && hasValidSeed) {
        deriveButton.textContent = `🚀 Derive Addresses (${AppState.selectedNetworks.length} networks)`;
    } else if (!hasNetworks) {
        deriveButton.textContent = '🚀 Select networks to derive addresses';
    } else {
        deriveButton.textContent = '🚀 Select valid seed source';
    }
}

// Seed Source Management
function updateSeedSourcePreviews() {
    const inputPreview = document.getElementById('inputSeedPreview');
    const transformedPreview = document.getElementById('transformedSeedPreview');
    const inputWordCount = document.getElementById('inputWordCount');
    const transformedOption = document.getElementById('transformedSourceOption');

    // Update input seed preview
    if (AppState.inputSeed) {
        inputPreview.textContent = AppState.inputSeed;
        inputWordCount.textContent = AppState.inputSeedWordCount;
    } else {
        inputPreview.textContent = 'No seed phrase entered';
        inputWordCount.textContent = '0';
    }

    // Update transformed seed preview and availability
    if (AppState.transformedSeed) {
        transformedPreview.textContent = AppState.transformedSeed;
        transformedOption.removeAttribute('disabled');
        transformedOption.querySelector('input').disabled = false;
    } else {
        transformedPreview.textContent = 'Transform a seed first';
        transformedOption.setAttribute('disabled', true);
        transformedOption.querySelector('input').disabled = true;

        // Reset to input if transformed was selected
        if (AppState.selectedSource === 'transformed') {
            AppState.selectedSource = 'input';
            document.querySelector('input[name="seedSource"][value="input"]').checked = true;
        }
    }
}

function handleNewSeedInput() {
    // Check if there are existing results that would be cleared
    const hasResults = Object.keys(AppState.derivationResults).length > 0;
    const hasTransformed = AppState.transformedSeed !== null;

    if (hasResults || hasTransformed) {
        const message = hasResults ?
            'This will clear existing derivation results. Continue?' :
            'This will clear the transformed seed. Continue?';

        if (!confirm(message)) {
            return false; // User cancelled
        }
    }

    // Clear relevant state
    AppState.derivationResults = {};
    AppState.transformedSeed = null;
    AppState.scypherPassword = null;

    // Update UI
    document.getElementById('resultsSection').style.display = 'none';
    updateSeedSourcePreviews();
    updateTabHeaders();

    return true;
}

// Address Derivation Process
async function deriveAddresses() {
    const selectedSource = document.querySelector('input[name="seedSource"]:checked').value;
    const bip39Passphrase = document.getElementById('bip39Passphrase').value;
    const addressCount = parseInt(document.getElementById('addressCount').value);

    // Validate inputs
    if (AppState.selectedNetworks.length === 0) {
        showToast('Please select at least one network', 'warning');
        return;
    }

    // Get source seed
    const seedPhrase = selectedSource === 'input' ?
        AppState.inputSeed : AppState.transformedSeed;

    if (!seedPhrase) {
        showToast('No valid seed phrase available', 'error');
        return;
    }

    // Start processing
    AppState.processing = {
        active: true,
        type: 'derive',
        progress: 0,
        current: 'Preparing derivation...',
        cancelable: true
    };

    updateUIForProcessing(true);

    try {
        const results = {};
        const totalNetworks = AppState.selectedNetworks.length;

        for (let i = 0; i < totalNetworks; i++) {
            if (!AppState.processing.active) break; // Check for cancellation

            const network = AppState.selectedNetworks[i];
            AppState.processing.current = `Generating ${network} addresses...`;
            AppState.processing.progress = ((i / totalNetworks) * 100);
            updateProgressDisplay();

            // Create network config for backend
            const networkConfigs = {};
            networkConfigs[network] = {
                count: addressCount,
                use_passphrase: !['cardano', 'solana'].includes(network) && !!bip39Passphrase
            };

            // Call backend
            const networkResult = await invoke('derive_addresses_with_config', {
                seedPhrase: seedPhrase,
                passphrase: bip39Passphrase || null,
                networkConfigs: networkConfigs
            });

            results[network] = networkResult[network] || [];

            // Small delay to show progress
            await new Promise(resolve => setTimeout(resolve, 100));
        }

        if (AppState.processing.active) {
            AppState.derivationResults = results;
            displayResults(results, selectedSource);
            showToast(`Successfully derived addresses for ${totalNetworks} networks`, 'success');
        }

    } catch (error) {
        console.error('Derivation error:', error);
        showToast(`Derivation failed: ${error.message}`, 'error');
    } finally {
        AppState.processing.active = false;
        updateUIForProcessing(false);
    }
}

function cancelDerivation() {
    if (AppState.processing.active && AppState.processing.cancelable) {
        AppState.processing.active = false;
        showToast('Derivation cancelled by user', 'info');
    }
}

function updateUIForProcessing(isProcessing) {
    // Lock/unlock tabs
    document.querySelectorAll('.tab-button').forEach(btn => {
        if (isProcessing) {
            btn.classList.add('disabled');
        } else {
            btn.classList.remove('disabled');
        }
    });

    // Update derive button
    const deriveButton = document.getElementById('deriveButton');
    const progressSection = document.getElementById('progressSection');

    if (isProcessing) {
        deriveButton.style.display = 'none';
        progressSection.style.display = 'block';
    } else {
        deriveButton.style.display = 'block';
        progressSection.style.display = 'none';
    }
}

function updateProgressDisplay() {
    const progressFill = document.getElementById('deriveProgressFill');
    const progressText = document.getElementById('progressText');

    progressFill.style.width = `${AppState.processing.progress}%`;
    progressText.textContent = AppState.processing.current;
}

// Results Display Management
function displayResults(results, sourceType) {
    const resultsSection = document.getElementById('resultsSection');
    const sourceIndicator = document.getElementById('sourceIndicator');
    const resultsContent = document.getElementById('resultsContent');

    // Update source indicator
    sourceIndicator.textContent = sourceType === 'input' ?
        'Input seed phrase' : 'SCypher-transformed seed';

    // Show results section
    resultsSection.style.display = 'block';
    resultsSection.classList.add('fade-in');

    // Generate content based on view mode
    const viewMode = document.getElementById('viewMode').value;

    if (viewMode === 'detailed') {
        resultsContent.innerHTML = generateDetailedView(results);
    } else {
        resultsContent.innerHTML = generateCompactView(results);
    }

    // Setup export functionality
    setupExportHandlers(results, sourceType);
}

function generateDetailedView(results) {
    let html = '';

    for (const [network, addresses] of Object.entries(results)) {
        if (addresses.length === 0) continue;

        const networkInfo = getNetworkInfo(network);
        html += `
            <div class="network-result" style="--network-color: ${networkInfo.color}; --network-color-rgb: ${hexToRgb(networkInfo.color)}">
                <div class="network-header">
                    <span class="network-icon">${networkInfo.icon}</span>
                    <span class="network-title">${networkInfo.name} (${addresses.length} addresses)</span>
                </div>
                <div class="address-list">
        `;

        addresses.forEach((addr, index) => {
            html += `
                <div class="address-item">
                    <span class="address-index">${index + 1}.</span>
                    <span class="address-text">${addr.address}</span>
                    <div class="address-actions">
                        <button class="action-button" onclick="copyAddress('${addr.address}')">📋</button>
                        <button class="action-button" onclick="showAddressDetails('${network}', ${index})">ℹ️</button>
                    </div>
                </div>
            `;
        });

        html += `
                </div>
            </div>
        `;
    }

    return html || '<div style="text-align: center; color: #888; padding: 40px;">No addresses generated</div>';
}

function generateCompactView(results) {
    let html = '';

    for (const [network, addresses] of Object.entries(results)) {
        if (addresses.length === 0) continue;

        const networkInfo = getNetworkInfo(network);
        html += `
            <div class="compact-result" style="--network-color: ${networkInfo.color}; --network-color-rgb: ${hexToRgb(networkInfo.color)}">
                <div class="compact-network">
                    <span class="network-icon">${networkInfo.icon}</span>
                    <span class="network-name">${networkInfo.name}</span>
                    <span class="compact-count">(${addresses.length})</span>
                </div>
                <div class="compact-actions">
                    <button class="action-button" onclick="expandNetwork('${network}')">View</button>
                    <button class="action-button" onclick="copyNetworkAddresses('${network}')">Copy</button>
                </div>
            </div>
        `;
    }

    return html || '<div style="text-align: center; color: #888; padding: 40px;">No addresses generated</div>';
}

// Export Functionality
function setupExportHandlers(results, sourceType) {
    const exportSelector = document.getElementById('exportFormat');

    exportSelector.addEventListener('change', function() {
        const format = this.value;
        if (format) {
            exportResults(results, sourceType, format);
            this.value = ''; // Reset selector
        }
    });
}

async function exportResults(results, sourceType, format) {
    const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
    const filename = `SCypher_addresses_${timestamp}`;

    let content = '';
    let extension = '';

    switch (format) {
        case 'json':
            content = JSON.stringify({
                source: sourceType,
                bip39_passphrase: document.getElementById('bip39Passphrase').value ? 'used' : 'not_used',
                generated_at: new Date().toISOString(),
                networks: results
            }, null, 2);
            extension = 'json';
            break;

        case 'csv':
            content = generateCSVContent(results);
            extension = 'csv';
            break;

        case 'txt':
            content = generateTXTContent(results, sourceType);
            extension = 'txt';
            break;
    }

    try {
        await invoke('save_addresses_file', {
            content: content,
            filename: `${filename}.${extension}`
        });
        showToast(`Addresses exported as ${format.toUpperCase()}`, 'success');
    } catch (error) {
        console.error('Export error:', error);
        showToast(`Export failed: ${error.message}`, 'error');
    }
}

function generateCSVContent(results) {
    let csv = 'Network,Address_Type,Derivation_Path,Address\n';

    for (const [network, addresses] of Object.entries(results)) {
        addresses.forEach(addr => {
            csv += `${network},"${addr.address_type}","${addr.path}","${addr.address}"\n`;
        });
    }

    return csv;
}

function generateTXTContent(results, sourceType) {
    let txt = `SCypher Address Derivation Results\n`;
    txt += `Generated: ${new Date().toISOString()}\n`;
    txt += `Source: ${sourceType === 'input' ? 'Input seed phrase' : 'SCypher-transformed seed'}\n`;
    txt += `BIP39 Passphrase: ${document.getElementById('bip39Passphrase').value ? 'Used' : 'Not used'}\n`;
    txt += `\n${'='.repeat(80)}\n\n`;

    for (const [network, addresses] of Object.entries(results)) {
        const networkInfo = getNetworkInfo(network);
        txt += `${networkInfo.icon} ${networkInfo.name.toUpperCase()} (${addresses.length} addresses)\n`;
        txt += `${'-'.repeat(50)}\n`;

        addresses.forEach((addr, index) => {
            txt += `${index + 1}. ${addr.address_type}\n`;
            txt += `    Path: ${addr.path}\n`;
            txt += `    Address: ${addr.address}\n\n`;
        });

        txt += `\n`;
    }

    return txt;
}

// Utility Functions
function getNetworkInfo(network) {
    const networkMap = {
        bitcoin: { name: 'Bitcoin', icon: '₿', color: '#F7931A' },
        ethereum: { name: 'Ethereum', icon: 'Ξ', color: '#627EEA' },
        ergo: { name: 'Ergo', icon: '⚡', color: '#FF5722' },
        tron: { name: 'TRON', icon: '🔺', color: '#FF0013' },
        bsc: { name: 'BSC', icon: '⚪', color: '#F0B90B' },
        polygon: { name: 'Polygon', icon: '◆', color: '#8247E5' },
        cardano: { name: 'Cardano', icon: '₳', color: '#0033AD' },
        dogecoin: { name: 'Dogecoin', icon: 'Ð', color: '#C2A633' },
        litecoin: { name: 'Litecoin', icon: 'Ł', color: '#BFBBBB' },
        solana: { name: 'Solana', icon: '◉', color: '#00FFA3' }
    };

    return networkMap[network] || { name: network, icon: '●', color: '#888' };
}

function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ?
        `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}` :
        '136, 136, 136';
}

async function copyAddress(address) {
    try {
        await navigator.clipboard.writeText(address);
        showToast('Address copied to clipboard', 'success', 2000);
    } catch (error) {
        console.error('Copy failed:', error);
        showToast('Failed to copy address', 'error');
    }
}

async function copyNetworkAddresses(network) {
    const addresses = AppState.derivationResults[network];
    if (!addresses) return;

    const addressList = addresses.map(addr => addr.address).join('\n');

    try {
        await navigator.clipboard.writeText(addressList);
        showToast(`${addresses.length} ${network} addresses copied`, 'success', 2000);
    } catch (error) {
        console.error('Copy failed:', error);
        showToast('Failed to copy addresses', 'error');
    }
}

function expandNetwork(network) {
    // Switch to detailed view and scroll to network
    document.getElementById('viewMode').value = 'detailed';
    displayResults(AppState.derivationResults, AppState.selectedSource);

    // Find and scroll to the network
    setTimeout(() => {
        const networkElement = document.querySelector(`[data-network="${network}"]`);
        if (networkElement) {
            networkElement.scrollIntoView({ behavior: 'smooth', block: 'center' });
        }
    }, 100);
}

function showAddressDetails(network, index) {
    const addresses = AppState.derivationResults[network];
    if (!addresses || !addresses[index]) return;

    const addr = addresses[index];
    const networkInfo = getNetworkInfo(network);

    const details = `
        Network: ${networkInfo.name}
        Type: ${addr.address_type}
        Path: ${addr.path}
        Address: ${addr.address}
    `;

    showToast(details, 'info', 5000);
}

// Tooltip System
function initializeTooltips() {
    const tooltipElements = [
        {
            element: '#bip39Passphrase',
            content: 'Official BIP39 standard feature. Adds entropy during address derivation. Supported by hardware wallets. Completely separate from SCypher.'
        },
        {
            element: '#passwordInput',
            content: 'SCypher proprietary method. Creates encrypted seed using XOR + Argon2id. NOT related to BIP39 standard.'
        },
        {
            element: '.seed-source-container',
            content: 'Input = What you typed/imported. Transformed = Processed with SCypher method.'
        },
        {
            element: '.network-grid',
            content: 'Each network will generate the specified number of addresses using standard derivation paths.'
        }
    ];

    tooltipElements.forEach(({ element, content }) => {
        const el = document.querySelector(element);
        if (el) {
            el.addEventListener('mouseenter', (e) => showTooltip(e, content));
            el.addEventListener('mouseleave', hideTooltip);
        }
    });
}

function showTooltip(event, content) {
    const tooltip = document.getElementById('tooltipContainer');
    const tooltipContent = document.getElementById('tooltipContent');

    tooltipContent.textContent = content;

    // Position tooltip
    const rect = event.target.getBoundingClientRect();
    tooltip.style.left = `${rect.left + (rect.width / 2)}px`;
    tooltip.style.top = `${rect.top - 10}px`;
    tooltip.style.transform = 'translateX(-50%) translateY(-100%)';

    tooltip.classList.add('show');
}

function hideTooltip() {
    document.getElementById('tooltipContainer').classList.remove('show');
}

// NEW: Add useSeedForDerivation function (for "Use for Derivation →" button)
function useSeedForDerivation() {
    if (AppState.transformedSeed) {
        // Switch to derive tab and select transformed source
        switchToTab('derive');
        document.querySelector('input[name="seedSource"][value="transformed"]').checked = true;
        AppState.selectedSource = 'transformed';
        updateSeedSourcePreviews();
        updateDerivationButtonState();
        showToast('Switched to derivation tab with transformed seed selected', 'success');
    }
}

// Global exports for HTML onclick handlers
window.editWordAtIndex = editWordAtIndex;
window.deleteWordAtIndex = deleteWordAtIndex;
window.copyToClipboard = copyToClipboard;
window.saveToFile = saveToFile;
window.processAnother = processAnother;
window.useSeedForDerivation = useSeedForDerivation;
window.copyAddress = copyAddress;
window.copyNetworkAddresses = copyNetworkAddresses;
window.expandNetwork = expandNetwork;
window.showAddressDetails = showAddressDetails;
```

## Backend Integration Requirements

### New Tauri Commands Needed

Add these commands to your Rust backend:

```rust
// Add to commands.rs

#[tauri::command]
pub async fn derive_addresses_with_config(
    seed_phrase: String,
    passphrase: Option<String>,
    network_configs: std::collections::HashMap<String, crate::addresses::NetworkConfig>,
) -> Result<crate::addresses::AddressSet, String> {
    crate::addresses::derive_addresses_with_config(&seed_phrase, passphrase.as_deref(), network_configs)
        .map_err(|e| format!("Address derivation failed: {}", e))
}

#[tauri::command]
pub async fn save_addresses_file(
    content: String,
    filename: String,
) -> Result<(), String> {
    use tauri::api::dialog::FileDialogBuilder;

    let path = FileDialogBuilder::new()
        .set_file_name(&filename)
        .save_file()
        .ok_or("File save cancelled")?;

    std::fs::write(path, content)
        .map_err(|e| format!("Failed to save file: {}", e))
}
```

### Update Main Tauri Setup
```rust
// Add to main.rs in tauri::Builder commands
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    derive_addresses_with_config,
    save_addresses_file
])
```

## Complete HTML Structure Update

### Replace Existing HTML Body
```html
<body>
    <div class="container">
        <div class="header">
            <div class="logo">SCypher</div>
            <div class="tagline">XOR-based BIP39 Seed Cipher v3.0</div>
            <button class="secure-mode-toggle" id="secureMode">
                🛡️ Secure Screen: OFF
            </button>
        </div>

        <!-- Tab Navigation -->
        <div class="tab-container">
            <div class="tab-header">
                <button class="tab-button active" data-tab="transform">
                    Transform Seed
                </button>
                <button class="tab-button" data-tab="derive">
                    Derive Addresses <span class="tab-status">(Select Source)</span>
                </button>
            </div>
        </div>

        <!-- Tab 1: Transform Seed (Enhanced Existing) -->
        <div class="tab-content active" id="transformTab">
            <div class="main-content" id="mainContent">
                <div class="warning-banner">
                    <strong>⚠️ Security Notice:</strong> Only process seed phrases you own. Never share encrypted results with untrusted parties.
                </div>

                <div class="section">
                    <div class="section-title">📝 Input Seed Phrase</div>
                    <!-- Existing input system preserved -->
                    <div class="input-controls">
                        <label style="color: #ddd; font-size: 0.9em;">Mode:</label>
                        <select class="word-count-selector auto" id="wordCount">
                            <option value="auto">AUTO (detect)</option>
                            <option value="12">12 words</option>
                            <option value="15">15 words</option>
                            <option value="18">18 words</option>
                            <option value="21">21 words</option>
                            <option value="24">24 words</option>
                        </select>

                        <div class="control-buttons">
                            <button class="small-button generate" id="generateSeed">🎲 Generate New Seed</button>
                            <button class="small-button clear" id="clearInput">🗑️ Clear</button>
                            <button class="small-button" id="browseFile">📁 Browse</button>
                        </div>
                    </div>

                    <div class="advanced-input-area">
                        <div class="word-input-container" id="wordContainer">
                            <div class="words-display" id="wordsDisplay"></div>
                            <input type="text" class="word-input" id="wordInput" autocomplete="off" spellcheck="false">
                            <div class="autocomplete-dropdown" id="autocompleteDropdown"></div>
                        </div>
                        <div class="input-placeholder" id="inputPlaceholder">Start typing or drag & drop a .txt file...</div>
                    </div>

                    <div class="validation-status" id="validationStatus">
                        <span class="status-progress">Ready to input seed phrase • AUTO mode active</span>
                    </div>
                </div>

                <div class="section">
                    <div class="section-title">🔐 SCypher Security Settings</div>
                    <div class="security-section">
                        <div class="info-card">
                            <div class="info-title">SCypher Method: Proprietary XOR + Argon2id transformation</div>
                            <div class="info-desc">Purpose: Create encrypted variant of your seed phrase</div>
                        </div>

                        <div class="input-group">
                            <label class="label">SCypher Password:</label>
                            <div class="password-input">
                                <input type="password" class="input" id="passwordInput" placeholder="Enter a strong password">
                                <button class="password-toggle" id="passwordToggle" type="button">👁️</button>
                            </div>
                        </div>

                        <div class="argon2-display">
                            <div class="param-display">
                                <div class="param-label">Argon2id Iterations</div>
                                <div class="param-value">5</div>
                            </div>
                            <div class="param-display">
                                <div class="param-label">Memory Cost</div>
                                <div class="param-value">128 MB</div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="action-buttons">
                    <button class="button" id="processButton" disabled>⏳ Enter seed phrase and password...</button>
                </div>

                <div class="progress-bar" id="progressBar">
                    <div class="progress-fill" id="progressFill"></div>
                </div>

                <div class="result-area" id="resultArea">
                    <div class="section-title">✅ Transformation Result</div>
                    <div class="result-text" id="resultText"></div>
                    <div class="result-actions">
                        <button class="copy-button" onclick="copyToClipboard()">📋 Copy to Clipboard</button>
                        <button class="copy-button" onclick="saveToFile()">💾 Save to File</button>
                        <button class="copy-button" onclick="useSeedForDerivation()">🔗 Use for Derivation →</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Tab 2: Derive Addresses (New) -->
        <div class="tab-content" id="deriveTab">
            <div class="main-content">
                <!-- Seed Source Selection -->
                <div class="section">
                    <div class="section-title">🔄 Seed Source Selection</div>
                    <div class="seed-source-container">
                        <label class="source-option">
                            <input type="radio" name="seedSource" value="input" checked>
                            <div class="source-preview">
                                <div class="source-title">Input seed phrase (<span id="inputWordCount">0</span> words)</div>
                                <div class="source-text" id="inputSeedPreview">No seed phrase entered</div>
                            </div>
                        </label>

                        <label class="source-option" id="transformedSourceOption" disabled>
                            <input type="radio" name="seedSource" value="transformed" disabled>
                            <div class="source-preview">
                                <div class="source-title">SCypher-transformed seed ✅ Available</div>
                                <div class="source-text" id="transformedSeedPreview">Transform a seed first</div>
                            </div>
                        </label>
                    </div>
                </div>

                <!-- BIP39 Settings -->
                <div class="section">
                    <div class="section-title">🌐 BIP39 Standard Settings</div>
                    <div class="bip39-section">
                        <div class="info-card">
                            <div class="info-title">BIP39 Standard: Official Bitcoin Improvement Proposal</div>
                            <div class="info-desc">Purpose: Additional entropy during address derivation</div>
                        </div>
                        <div class="input-group">
                            <label class="label">BIP39 Passphrase (optional):</label>
                            <div class="password-input">
                                <input type="password" class="input" id="bip39Passphrase"
                                       placeholder="Enter BIP39 passphrase (leave empty for standard derivation)">
                                <button class="password-toggle" type="button" onclick="togglePasswordVisibility('bip39Passphrase', this)">👁️</button>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Network Selection -->
                <div class="section">
                    <div class="section-title">🌍 Select Networks</div>
                    <div class="network-grid">
                        <button class="network-button" data-network="bitcoin" data-color="#F7931A">
                            <span class="network-icon">₿</span>
                            <span class="network-name">Bitcoin</span>
                        </button>
                        <button class="network-button" data-network="ethereum" data-color="#627EEA">
                            <span class="network-icon">Ξ</span>
                            <span class="network-name">Ethereum</span>
                        </button>
                        <button class="network-button" data-network="ergo" data-color="#FF5722">
                            <span class="network-icon">⚡</span>
                            <span class="network-name">Ergo</span>
                        </button>
                        <button class="network-button" data-network="tron" data-color="#FF0013">
                            <span class="network-icon">🔺</span>
                            <span class="network-name">TRON</span>
                        </button>
                        <button class="network-button" data-network="bsc" data-color="#F0B90B">
                            <span class="network-icon">⚪</span>
                            <span class="network-name">BSC</span>
                        </button>
                        <button class="network-button" data-network="polygon" data-color="#8247E5">
                            <span class="network-icon">◆</span>
                            <span class="network-name">Polygon</span>
                        </button>
                        <button class="network-button" data-network="cardano" data-color="#0033AD">
                            <span class="network-icon">₳</span>
                            <span class="network-name">Cardano</span>
                        </button>
                        <button class="network-button" data-network="dogecoin" data-color="#C2A633">
                            <span class="network-icon">Ð</span>
                            <span class="network-name">Dogecoin</span>
                        </button>
                        <button class="network-button" data-network="litecoin" data-color="#BFBBBB">
                            <span class="network-icon">Ł</span>
                            <span class="network-name">Litecoin</span>
                        </button>
                        <button class="network-button" data-network="solana" data-color="#00FFA3">
                            <span class="network-icon">◉</span>
                            <span class="network-name">Solana</span>
                        </button>
                    </div>

                    <div class="network-controls">
                        <button class="control-button select-all">🔥 Select All</button>
                        <button class="control-button clear-all">❌ Clear All</button>
                    </div>

                    <div class="bip39-warning" id="bip39Warning" style="display: none;">
                        <strong>⚠️ Notice:</strong> Cardano & Solana will derive WITHOUT BIP39 passphrase (not supported by these networks)
                    </div>
                </div>

                <!-- Address Count -->
                <div class="section">
                    <div class="section-title">⚙️ Address Count</div>
                    <div class="count-config">
                        <label class="label">Generate addresses per network:</label>
                        <div class="count-input-group">
                            <input type="number" class="count-input" id="addressCount"
                                   value="1" min="1" max="100" placeholder="1">
                            <span class="count-range">(1-100)</span>
                        </div>
                    </div>
                </div>

                <!-- Derive Button and Progress -->
                <div class="action-section">
                    <button class="button derive-button" id="deriveButton" disabled>
                        🚀 Derive Addresses
                    </button>

                    <div class="progress-section" id="progressSection" style="display: none;">
                        <div class="progress-bar">
                            <div class="progress-fill" id="deriveProgressFill"></div>
                        </div>
                        <div class="progress-text" id="progressText">Preparing derivation...</div>
                        <button class="cancel-button" id="cancelDerivation">❌ Cancel Derivation</button>
                    </div>
                </div>

                <!-- Results Display -->
                <div class="results-section" id="resultsSection" style="display: none;">
                    <div class="section-title">📍 Derivation Results</div>

                    <div class="results-header">
                        <div class="results-info" id="resultsInfo">
                            Derived from: <strong id="sourceIndicator">Input seed phrase</strong>
                        </div>
                        <div class="view-controls">
                            <select class="view-selector" id="viewMode">
                                <option value="detailed">Detailed View</option>
                                <option value="compact">Compact View</option>
                            </select>
                            <select class="export-selector" id="exportFormat">
                                <option value="">Export As...</option>
                                <option value="json">JSON</option>
                                <option value="csv">CSV</option>
                                <option value="txt">TXT</option>
                            </select>
                        </div>
                    </div>

                    <div class="results-content" id="resultsContent"></div>
                </div>
            </div>
        </div>

        <!-- Status Bar -->
        <div class="status-bar">
            <div class="status-text" id="statusText">Ready • AUTO mode • BIP39 validation enabled • Argon2id: 5 iterations, 128MB</div>
            <div class="security-indicator">
                <span>🛡️</span>
                <span id="securityStatus">Secure mode ready</span>
            </div>
        </div>
    </div>

    <!-- Toast Container -->
    <div class="toast-container" id="toastContainer"></div>

    <!-- Tooltip Container -->
    <div id="tooltipContainer" class="tooltip-container">
        <div class="tooltip-content" id="tooltipContent"></div>
        <div class="tooltip-arrow" id="tooltipArrow"></div>
    </div>

    <!-- Enhanced JavaScript with all new functionality -->
    <script>
        // Enhanced existing variables
        let currentWords = [];
        let currentMode = 'auto';
        let editingIndex = -1;
        let highlightedIndex = -1;
        let secureMode = false;
        let bip39WordList = [];
        let invoke = null;
        let tauriAvailable = false;

        // NEW: Global App State
        const AppState = {
            inputSeed: null,
            inputSeedWordCount: 0,
            transformedSeed: null,
            scypherPassword: null,
            selectedSource: 'input',
            bip39Passphrase: '',
            selectedNetworks: [],
            addressCount: 1,
            derivationResults: {},
            processing: {
                active: false,
                type: null,
                progress: 0,
                current: '',
                cancelable: false
            }
        };

        // ALL EXISTING JAVASCRIPT FUNCTIONS PRESERVED
        // ALL NEW JAVASCRIPT FUNCTIONS ADDED
        // (Insert all the JavaScript code from previous sections here)

    </script>
</body>
```

## Error Handling Specifications

### Network-Specific Error Messages
```javascript
const NetworkErrorMessages = {
    'bitcoin': 'Bitcoin address derivation failed. Check seed phrase validity.',
    'ethereum': 'Ethereum address derivation failed. Verify seed format.',
    'cardano': 'Cardano derivation failed. Note: BIP39 passphrase not supported.',
    'solana': 'Solana derivation failed. Note: BIP39 passphrase not supported.',
    'ergo': 'Ergo address derivation failed. Check ErgoTree compatibility.',
    'tron': 'TRON address derivation failed. Verify network connectivity.',
    'bsc': 'BSC address derivation failed. Same as Ethereum format.',
    'polygon': 'Polygon address derivation failed. Same as Ethereum format.',
    'dogecoin': 'Dogecoin address derivation failed. Check coin type.',
    'litecoin': 'Litecoin address derivation failed. Check coin type.'
};
```

### Graceful Degradation
- If a single network fails, continue with others
- Display partial results with error indicators
- Allow re-attempt for failed networks
- Maintain UI responsiveness during failures

## Performance Considerations

### Optimization Points
1. **Lazy loading** - Only load network data when needed
2. **Chunked processing** - Process networks in batches
3. **UI updates** - Throttle progress updates to 100ms intervals
4. **Memory management** - Clear large result sets when switching sources
5. **Event debouncing** - Debounce address count input changes

### Resource Management
- Monitor memory usage for large address sets
- Implement cleanup on tab switches
- Optimize DOM updates for large result sets
- Use DocumentFragment for bulk DOM operations

## Security Considerations

### Data Handling
- Never log sensitive seed data to console
- Clear password fields on tab switch
- Implement secure memory clearing where possible
- Validate all user inputs before processing

### UI Security
- Disable context menus on sensitive elements
- Implement copy protection for critical data
- Use secure clipboard API where available
- Clear sensitive data from DOM when not needed

## Final Implementation Checklist

### Phase 1: Basic Structure
- [ ] Add tab HTML structure
- [ ] Add tab CSS styles
- [ ] Implement basic tab switching
- [ ] Add seed source selection UI

### Phase 2: Network Selection
- [ ] Add network grid HTML
- [ ] Implement network button styling
- [ ] Add network selection logic
- [ ] Implement BIP39 warning system

### Phase 3: Derivation Logic
- [ ] Add address count input
- [ ] Implement derivation process
- [ ] Add progress tracking
- [ ] Implement cancellation

### Phase 4: Results Display
- [ ] Add results HTML structure
- [ ] Implement detailed view
- [ ] Implement compact view
- [ ] Add export functionality

### Phase 5: Integration
- [ ] Enhance existing functions
- [ ] Add state synchronization
- [ ] Implement memory management
- [ ] Add error handling

### Phase 6: Polish
- [ ] Add tooltips
- [ ] Fix placeholder issue
- [ ] Add responsive design
- [ ] Final testing

This completes the comprehensive implementation guide for adding multi-blockchain address derivation functionality to SCypher GUI while preserving all existing functionality and maintaining professional UX standards.
