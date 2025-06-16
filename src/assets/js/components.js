/**
 * SCypher GUI v3.0 - Component Management
 * Handles dynamic content loading and component interactions
 */

// Component state variables
let currentWords = [];
let currentMode = 'auto';
let editingIndex = -1;
let highlightedIndex = -1;
let secureMode = false;
let bip39WordList = [];

/**
 * Template Component Loader
 */
class TemplateLoader {
    constructor() {
        this.cache = new Map();
    }

    async loadTemplate(path) {
        if (this.cache.has(path)) {
            return this.cache.get(path);
        }

        try {
            const template = await loadTemplate(path);
            if (template) {
                this.cache.set(path, template);
            }
            return template;
        } catch (error) {
            console.error(`Failed to load template ${path}:`, error);
            return null;
        }
    }

    async loadComponent(containerId, templatePath) {
        const container = document.getElementById(containerId);
        if (!container) {
            console.error(`Container ${containerId} not found`);
            return false;
        }

        const template = await this.loadTemplate(templatePath);
        if (!template) {
            console.error(`Failed to load template ${templatePath}`);
            return false;
        }

        container.innerHTML = template;
        return true;
    }
}

const templateLoader = new TemplateLoader();

/**
 * Tab Management System
 */
class TabManager {
    constructor() {
        this.activeTab = 'transform';
        this.tabsLoaded = new Set();
    }

    async initializeTabs() {
        const tabButtons = document.querySelectorAll('.tab-button');
        const tabContents = document.querySelectorAll('.tab-content');

        tabButtons.forEach(button => {
            button.addEventListener('click', async (e) => {
                if (e.target.classList.contains('disabled')) return;
                if (AppState.processing.active) return;

                const targetTab = e.target.dataset.tab;
                await this.switchToTab(targetTab);
            });
        });

        // Load initial tab content
        await this.loadTabContent('transform');
    }

    async loadTabContent(tabName) {
        if (this.tabsLoaded.has(tabName)) {
            return true;
        }

        const templatePath = `assets/templates/${tabName}-tab.html`;
        const containerId = `${tabName}Tab`;

        const success = await templateLoader.loadComponent(containerId, templatePath);
        if (success) {
            this.tabsLoaded.add(tabName);

            // Initialize tab-specific functionality
            if (tabName === 'transform') {
                this.initializeTransformTab();
            } else if (tabName === 'derive') {
                await this.initializeDeriveTab();
            }
        }

        return success;
    }

    async switchToTab(tabName) {
        // Load tab content if not already loaded
        await this.loadTabContent(tabName);

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

        this.activeTab = tabName;
        this.updateTabState(tabName);
    }

    updateTabState(tabName) {
        if (tabName === 'derive') {
            updateSeedSourcePreviews();
            updateDerivationButtonState();
            updateBip39Warning();
        }
    }

    initializeTransformTab() {
        // Word input events
        const wordInput = document.getElementById('wordInput');
        if (wordInput) {
            wordInput.addEventListener('input', handleWordInput);
            wordInput.addEventListener('keydown', handleKeyDown);
            wordInput.addEventListener('blur', handleInputBlur);
        }

        // Container events
        const container = document.getElementById('wordContainer');
        if (container) {
            container.addEventListener('click', focusWordInput);
            this.setupDragAndDropVisuals(container);
        }

        // Control events
        const wordCount = document.getElementById('wordCount');
        if (wordCount) {
            wordCount.addEventListener('change', handleModeChange);
        }

        const generateSeed = document.getElementById('generateSeed');
        if (generateSeed) {
            generateSeed.addEventListener('click', generateNewSeed);
        }

        const clearInput = document.getElementById('clearInput');
        if (clearInput) {
            clearInput.addEventListener('click', clearAllWords);
        }

        const browseFile = document.getElementById('browseFile');
        if (browseFile) {
            browseFile.addEventListener('click', handleModernBrowse);
        }

        // Security events
        const passwordToggle = document.getElementById('passwordToggle');
        if (passwordToggle) {
            passwordToggle.addEventListener('click', () =>
                togglePasswordVisibility('passwordInput', passwordToggle));
        }

        const passwordInput = document.getElementById('passwordInput');
        if (passwordInput) {
            passwordInput.addEventListener('input', () => {
                updatePasswordStrength();
                updateProcessButtonState();
            });
        }

        const processButton = document.getElementById('processButton');
        if (processButton) {
            processButton.addEventListener('click', processSeed);
        }

        // Autocomplete events
        const autocompleteDropdown = document.getElementById('autocompleteDropdown');
        if (autocompleteDropdown) {
            autocompleteDropdown.addEventListener('click', handleAutocompleteClick);
        }
    }

    async initializeDeriveTab() {
        // Initialize network grid with SVG icons
        await initializeNetworkGrid();

        // Control buttons
        const selectAll = document.querySelector('.select-all');
        if (selectAll) {
            selectAll.addEventListener('click', selectAllNetworks);
        }

        const clearAll = document.querySelector('.clear-all');
        if (clearAll) {
            clearAll.addEventListener('click', clearAllNetworks);
        }

        // Address count input - VERSIÓN ARREGLADA
        const addressCountInput = document.getElementById('addressCount');
        if (addressCountInput) {
            // Evento para cuando se hace click (seleccionar todo el texto)
            addressCountInput.addEventListener('click', function() {
                this.select(); // Selecciona todo el texto al hacer click
            });

            // Evento para cuando se hace focus (seleccionar todo el texto)
            addressCountInput.addEventListener('focus', function() {
                this.select(); // Selecciona todo el texto al obtener focus
            });

            // Evento para validar input mientras se escribe
            addressCountInput.addEventListener('input', function() {
                let value = this.value;

                // Remover caracteres no numéricos
                value = value.replace(/[^0-9]/g, '');

                // Si está vacío, no hacer nada (permitir campo vacío temporalmente)
                if (value === '') {
                    this.value = '';
                    return;
                }

                // Convertir a número y validar rango
                let numValue = parseInt(value);
                if (isNaN(numValue) || numValue < 1) {
                    numValue = 1;
                } else if (numValue > 100) {
                    numValue = 100;
                }

                // Actualizar el valor y el estado
                this.value = numValue;
                AppState.addressCount = numValue;
                updateDerivationButtonState();
            });

            // Evento para cuando se pierde el focus (asegurar valor válido)
            addressCountInput.addEventListener('blur', function() {
                if (this.value === '' || parseInt(this.value) < 1) {
                    this.value = '1'; // Autocompletar con 1 si está vacío o inválido
                    AppState.addressCount = 1;
                    updateDerivationButtonState();
                }
            });

            // Evento para teclas especiales
            addressCountInput.addEventListener('keydown', function(e) {
                // Permitir: backspace, delete, tab, escape, enter
                if ([8, 9, 27, 13, 46].indexOf(e.keyCode) !== -1 ||
                    // Permitir: Ctrl+A, Ctrl+C, Ctrl+V, Ctrl+X
                    (e.keyCode === 65 && e.ctrlKey === true) ||
                    (e.keyCode === 67 && e.ctrlKey === true) ||
                    (e.keyCode === 86 && e.ctrlKey === true) ||
                    (e.keyCode === 88 && e.ctrlKey === true) ||
                    // Permitir: home, end, left, right
                    (e.keyCode >= 35 && e.keyCode <= 39)) {
                    return;
                }

                // Asegurar que solo se permitan números (0-9)
                if ((e.shiftKey || (e.keyCode < 48 || e.keyCode > 57)) && (e.keyCode < 96 || e.keyCode > 105)) {
                    e.preventDefault();
                }
            });

            // Establecer valor inicial
            addressCountInput.value = '1';
            AppState.addressCount = 1;
        }

        // Source selection
        document.querySelectorAll('input[name="seedSource"]').forEach(radio => {
            radio.addEventListener('change', function() {
                AppState.selectedSource = this.value;
                updateDerivationButtonState();
                updateTabHeaders();
            });
        });

        // BIP39 passphrase
        const bip39Passphrase = document.getElementById('bip39Passphrase');
        if (bip39Passphrase) {
            bip39Passphrase.addEventListener('input', function() {
                AppState.bip39Passphrase = this.value;
                updateBip39Warning();
            });
        }

        // Derive button
        const deriveButton = document.getElementById('deriveButton');
        if (deriveButton) {
            deriveButton.addEventListener('click', deriveAddresses);
        }

        // Cancel button
        const cancelDerivation = document.getElementById('cancelDerivation');
        if (cancelDerivation) {
            cancelDerivation.addEventListener('click', cancelDerivationProcess);
        }

        // View mode selector
        const viewMode = document.getElementById('viewMode');
        if (viewMode) {
            viewMode.addEventListener('change', function() {
                if (Object.keys(AppState.derivationResults).length > 0) {
                    displayResults(AppState.derivationResults, AppState.selectedSource);
                }
            });
        }

        // Export selector
        const exportFormat = document.getElementById('exportFormat');
        if (exportFormat) {
            exportFormat.addEventListener('change', function() {
                const format = this.value;
                if (format) {
                    exportResults(AppState.derivationResults, AppState.selectedSource, format);
                    this.value = '';
                }
            });
        }
    }

    setupDragAndDropVisuals(container) {
        container.addEventListener('dragover', function(e) {
            e.preventDefault();
            e.stopPropagation();
            e.dataTransfer.dropEffect = 'copy';
            e.currentTarget.classList.add('drag-over');
        });

        container.addEventListener('dragenter', function(e) {
            e.preventDefault();
            e.stopPropagation();
        });

        container.addEventListener('dragleave', function(e) {
            e.preventDefault();
            e.stopPropagation();
            e.currentTarget.classList.remove('drag-over');
        });
    }

    updateTabHeaders() {
        const transformTab = document.querySelector('[data-tab="transform"]');
        const deriveTab = document.querySelector('[data-tab="derive"]');

        if (transformTab) {
            if (AppState.transformedSeed) {
                transformTab.innerHTML = 'Transform Seed <span class="tab-status">✅</span>';
            } else {
                transformTab.innerHTML = 'Transform Seed';
            }
        }

        if (deriveTab) {
            const sourceType = AppState.selectedSource === 'input' ? 'Input' : 'Transformed';
            deriveTab.innerHTML = `Derive Addresses <span class="tab-status">(${sourceType})</span>`;
        }
    }
}

const tabManager = new TabManager();

/**
 * Word Input Management
 */
function handleWordInput(e) {
    const value = e.target.value.trim();

    if (value.includes(' ')) {
        const words = value.split(/\s+/).filter(word => word.length > 0);
        if (words.length > 1) {
            for (const word of words) {
                if (validateSingleWord(word, bip39WordList)) {
                    addWord(word);
                }
            }
            e.target.value = '';
            hideAutocomplete();
            return;
        }
        if (words.length === 1 && validateSingleWord(words[0], bip39WordList)) {
            addWord(words[0]);
            e.target.value = '';
            hideAutocomplete();
            return;
        }
    }

    if (value.length > 0) {
        showAutocomplete(value);
    } else {
        hideAutocomplete();
    }

    updateContainerState();
}

function handleKeyDown(e) {
    const input = e.target;
    const dropdown = document.getElementById('autocompleteDropdown');
    const items = dropdown.querySelectorAll('.autocomplete-item');

    switch (e.key) {
        case 'Enter':
            e.preventDefault();
            if (highlightedIndex >= 0 && items[highlightedIndex]) {
                selectAutocompleteItem(items[highlightedIndex]);
            } else if (input.value.trim()) {
                const word = input.value.trim();
                if (validateSingleWord(word, bip39WordList)) {
                    addWord(word);
                    input.value = '';
                    hideAutocomplete();
                }
            }
            break;

        case 'Tab':
            if (items.length === 1) {
                e.preventDefault();
                selectAutocompleteItem(items[0]);
            }
            break;

        case 'ArrowDown':
            e.preventDefault();
            if (items.length > 0) {
                highlightedIndex = Math.min(highlightedIndex + 1, items.length - 1);
                updateAutocompleteHighlight(items);
            }
            break;

        case 'ArrowUp':
            e.preventDefault();
            if (items.length > 0) {
                highlightedIndex = Math.max(highlightedIndex - 1, 0);
                updateAutocompleteHighlight(items);
            }
            break;

        case 'Escape':
            hideAutocomplete();
            break;

        case 'Backspace':
            if (input.value === '' && currentWords.length > 0) {
                e.preventDefault();
                editLastWord();
            }
            break;
    }
}

function handleInputBlur() {
    setTimeout(() => {
        if (!document.querySelector('.autocomplete-dropdown:hover')) {
            hideAutocomplete();
        }
    }, 150);
}

/**
 * Autocomplete Functions
 */
function showAutocomplete(query) {
    const dropdown = document.getElementById('autocompleteDropdown');
    if (!dropdown) return;

    const matches = bip39WordList.filter(word =>
        word.toLowerCase().startsWith(query.toLowerCase())
    ).slice(0, 8);

    if (matches.length === 0) {
        hideAutocomplete();
        return;
    }

    dropdown.innerHTML = matches.map((word, index) => `
        <div class="autocomplete-item" data-word="${word}" data-index="${index}">
            <span class="word-text">${word}</span>
            <span class="word-hint">${word.length} chars</span>
        </div>
    `).join('');

    highlightedIndex = -1;
    dropdown.classList.add('show');
}

function hideAutocomplete() {
    const dropdown = document.getElementById('autocompleteDropdown');
    if (dropdown) {
        dropdown.classList.remove('show');
    }
    highlightedIndex = -1;
}

function updateAutocompleteHighlight(items) {
    items.forEach((item, index) => {
        item.classList.toggle('highlighted', index === highlightedIndex);
    });
}

function handleAutocompleteClick(e) {
    const item = e.target.closest('.autocomplete-item');
    if (item) {
        selectAutocompleteItem(item);
    }
}

function selectAutocompleteItem(item) {
    const word = item.dataset.word;
    addWord(word);
    document.getElementById('wordInput').value = '';
    hideAutocomplete();
}

/**
 * Word Management Functions
 */
function addWord(word) {
    if (editingIndex >= 0) {
        currentWords[editingIndex] = word.toLowerCase();
        editingIndex = -1;
    } else {
        currentWords.push(word.toLowerCase());
    }
    renderWords();
    updateValidationStatus();
    updateProcessButtonState();
    focusWordInput();
}

function editLastWord() {
    if (currentWords.length > 0) {
        const lastWord = currentWords[currentWords.length - 1];
        currentWords.pop();
        document.getElementById('wordInput').value = lastWord;
        editingIndex = -1;
        renderWords();
        updateValidationStatus();
        updateProcessButtonState();
        focusWordInput();
    }
}

function editWordAtIndex(index) {
    if (index >= 0 && index < currentWords.length) {
        const word = currentWords[index];
        document.getElementById('wordInput').value = word;
        editingIndex = index;
        focusWordInput();
    }
}

function deleteWordAtIndex(index) {
    if (index >= 0 && index < currentWords.length) {
        currentWords.splice(index, 1);
        renderWords();
        updateValidationStatus();
        updateProcessButtonState();
        focusWordInput();
    }
}

function renderWords() {
    const display = document.getElementById('wordsDisplay');
    if (!display) return;

    display.innerHTML = currentWords.map((word, index) => `
        <div class="word-tag ${editingIndex === index ? 'editing' : ''}"
             onclick="editWordAtIndex(${index})"
             data-index="${index}">
            <span class="word-number">${index + 1}</span>
            <span>${word}</span>
            <span class="delete-word" onclick="event.stopPropagation(); deleteWordAtIndex(${index})">×</span>
        </div>
    `).join('');

    updateContainerState();
}

function updateContainerState() {
    const container = document.getElementById('wordContainer');
    const placeholder = document.getElementById('inputPlaceholder');
    const wordInput = document.getElementById('wordInput');

    if (!container || !placeholder) return;

    const hasContent = currentWords.length > 0 || (wordInput && wordInput.value.length > 0);
    container.classList.toggle('has-content', hasContent);

    if (currentWords.length > 0) {
        placeholder.style.opacity = '0';
        placeholder.style.transform = 'translateY(-70%)';
    } else if (!hasContent) {
        placeholder.style.opacity = '1';
        placeholder.style.transform = 'translateY(-50%)';
    }
}

function focusWordInput() {
    setTimeout(() => {
        const wordInput = document.getElementById('wordInput');
        if (wordInput) {
            wordInput.focus();
        }
    }, 0);
}

/**
 * Validation Functions
 */
async function validateCurrentPhrase() {
    const wordCount = currentWords.length;

    if (currentMode === 'auto') {
        const validCounts = [12, 15, 18, 21, 24];

        if (wordCount === 0) {
            return { valid: false, status: 'empty', message: 'Ready to input seed phrase • AUTO mode active' };
        }

        if (!validCounts.includes(wordCount)) {
            return {
                valid: false,
                status: 'progress',
                message: `${wordCount} words entered • Continue to reach 12, 15, 18, 21, or 24 words`
            };
        }

        const phrase = currentWords.join(' ');
        try {
            const validation = await invoke('validate_seed_phrase', { phrase });
            return {
                valid: validation.valid,
                status: validation.status,
                message: validation.message
            };
        } catch (error) {
            return {
                valid: false,
                status: 'invalid',
                message: `Validation error: ${error}`
            };
        }
    }

    const targetCount = parseInt(currentMode);
    if (wordCount < targetCount) {
        return {
            valid: false,
            status: 'progress',
            message: `${wordCount}/${targetCount} words entered`
        };
    }

    if (wordCount > targetCount) {
        return {
            valid: false,
            status: 'invalid',
            message: `Too many words: ${wordCount}/${targetCount}`
        };
    }

    const phrase = currentWords.join(' ');
    try {
        const validation = await invoke('validate_seed_phrase', { phrase });
        return {
            valid: validation.valid,
            status: validation.status,
            message: validation.message
        };
    } catch (error) {
        return {
            valid: false,
            status: 'invalid',
            message: `Validation error: ${error}`
        };
    }
}

async function updateValidationStatus() {
    const container = document.getElementById('wordContainer');
    const statusElement = document.getElementById('validationStatus');

    if (!container || !statusElement) return;

    const validation = await validateCurrentPhrase();

    container.className = 'word-input-container';
    if (validation.status === 'valid') {
        container.classList.add('valid');
    } else if (validation.status === 'invalid') {
        container.classList.add('invalid');
    }

    if (validation.valid) {
        AppState.inputSeed = currentWords.join(' ');
        AppState.inputSeedWordCount = currentWords.length;
    } else {
        AppState.inputSeed = null;
        AppState.inputSeedWordCount = 0;
    }

    updateSeedSourcePreviews();
    tabManager.updateTabHeaders();

    const statusClass = validation.status === 'valid' ? 'status-valid' :
                       validation.status === 'invalid' ? 'status-invalid' :
                       validation.status === 'progress' ? 'status-progress' : 'status-progress';

    statusElement.innerHTML = `<span class="${statusClass}">${validation.message}</span>`;

    return validation;
}

function updateProcessButtonState() {
    validateCurrentPhrase().then(validation => {
        const passwordInput = document.getElementById('passwordInput');
        const processButton = document.getElementById('processButton');

        if (!passwordInput || !processButton) return;

        const hasValidSeed = validation.valid;
        const hasPassword = passwordInput.value.length >= 8;

        processButton.disabled = !(hasValidSeed && hasPassword);

        if (hasValidSeed && hasPassword) {
            processButton.textContent = '🔄 Process Seed Phrase';
        } else if (!hasValidSeed) {
            processButton.textContent = '⏳ Complete valid seed phrase...';
        } else {
            processButton.textContent = '⏳ Enter password (min 8 chars)...';
        }
    });
}

/**
 * Control Functions
 */
function handleModeChange(e) {
    const newMode = e.target.value;
    const selector = e.target;

    if (newMode === 'auto') {
        selector.classList.add('auto');
        currentMode = 'auto';
    } else {
        selector.classList.remove('auto');

        if (currentWords.length > 0) {
            currentWords = [];
            editingIndex = -1;
            const wordInput = document.getElementById('wordInput');
            if (wordInput) {
                wordInput.value = '';
            }
            renderWords();
            showToast(`Cleared words - switched to ${newMode}-word mode`, 'info', 3000);
        }

        currentMode = newMode;
    }

    updateValidationStatus();
    updateProcessButtonState();
    focusWordInput();
}

async function generateNewSeed() {
    const targetCount = currentMode === 'auto' ? 12 : parseInt(currentMode);

    try {
        console.log(`🎲 Generating ${targetCount}-word seed phrase...`);
        showToast('Generating new seed phrase...', 'info', 2000);

        const generatedPhrase = await invoke('generate_seed_phrase', {
            wordCount: targetCount
        });

        currentWords = generatedPhrase.split(' ');
        editingIndex = -1;
        renderWords();
        await updateValidationStatus();
        updateProcessButtonState();
        focusWordInput();

        showToast(`Generated ${targetCount}-word seed phrase`, 'success');

        const statusText = document.getElementById('statusText');
        if (statusText) {
            statusText.textContent = `Ready • Generated ${targetCount} words • BIP39 validation enabled • Argon2id: 3 iterations, 64MB`;
        }

    } catch (error) {
        console.error('❌ Generation error:', error);
        showToast(`Failed to generate seed: ${error}`, 'error');
    }
}

function clearAllWords() {
    if (!handleNewSeedInput()) {
        return;
    }

    currentWords = [];
    editingIndex = -1;

    const wordInput = document.getElementById('wordInput');
    const passwordInput = document.getElementById('passwordInput');
    const resultArea = document.getElementById('resultArea');
    const resultText = document.getElementById('resultText');

    if (wordInput) wordInput.value = '';
    if (passwordInput) passwordInput.value = '';
    if (resultArea) resultArea.classList.remove('show');
    if (resultText) resultText.textContent = '';

    renderWords();
    hideAutocomplete();
    updateValidationStatus();
    updateProcessButtonState();
    focusWordInput();

    const statusText = document.getElementById('statusText');
    if (statusText) {
        statusText.textContent = 'Ready • AUTO mode • BIP39 validation enabled • Argon2id: 3 iterations, 64MB';
    }
}

/**
 * Password Functions
 */
function updatePasswordStrength() {
    const passwordInput = document.getElementById('passwordInput');
    if (!passwordInput) return;

    const password = passwordInput.value;
    const strengthData = evaluatePasswordStrength(password);

    let indicator = document.getElementById('passwordStrengthIndicator');
    if (!indicator) {
        indicator = document.createElement('div');
        indicator.id = 'passwordStrengthIndicator';
        indicator.style.cssText = `
            margin-top: 8px;
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 0.8em;
            transition: all 0.3s ease;
        `;
        passwordInput.parentElement.appendChild(indicator);
    }

    if (password.length === 0) {
        indicator.style.opacity = '0';
        return;
    }

    indicator.style.opacity = '1';
    indicator.innerHTML = `
        <div style="
            width: 100px;
            height: 4px;
            background: rgba(60, 60, 60, 0.5);
            border-radius: 2px;
            overflow: hidden;
        ">
            <div style="
                width: ${(strengthData.strength / 4) * 100}%;
                height: 100%;
                background: ${strengthData.color};
                transition: all 0.3s ease;
            "></div>
        </div>
        <span style="color: ${strengthData.color}; font-weight: 500;">
            ${strengthData.label}
        </span>
    `;
}

/**
 * Inicializa la grid de redes con iconos SVG reales
 */
async function initializeNetworkGrid() {
    const networkGrid = document.getElementById('networkGrid');
    if (!networkGrid) return;

    const networks = [
        { id: 'bitcoin', color: '#F7931A' },
        { id: 'ethereum', color: '#627EEA' },
        { id: 'ergo', color: '#FF5722' },
        { id: 'tron', color: '#FF0013' },
        { id: 'bsc', color: '#F0B90B' },
        { id: 'polygon', color: '#8247E5' },
        { id: 'cardano', color: '#0033AD' },
        { id: 'dogecoin', color: '#C2A633' },
        { id: 'litecoin', color: '#BFBBBB' },
        { id: 'solana', color: '#00FFA3' }
    ];

    // Limpiar grid existente
    networkGrid.innerHTML = '';

    // Crear botones de red con iconos SVG
    for (const network of networks) {
        const networkInfo = getNetworkInfo(network.id);
        const button = document.createElement('button');

        button.className = 'network-button';
        button.dataset.network = network.id;
        button.dataset.color = network.color;

        // Configurar propiedades CSS
        button.style.setProperty('--network-color', network.color);
        button.style.setProperty('--network-color-rgb', hexToRgb(network.color));

        // Crear estructura interna
        const iconContainer = document.createElement('span');
        iconContainer.className = 'network-icon';

        const nameSpan = document.createElement('span');
        nameSpan.className = 'network-name';
        nameSpan.textContent = networkInfo.name;

        // Intentar cargar icono SVG
        try {
            const iconElement = await createNetworkIcon(network.id, 'network-icon-svg');
            iconContainer.appendChild(iconElement);
        } catch (error) {
            console.warn(`Failed to load icon for ${network.id}, using fallback`);
            iconContainer.textContent = networkInfo.fallbackIcon;
        }

        button.appendChild(iconContainer);
        button.appendChild(nameSpan);

        // Agregar event listener
        button.addEventListener('click', function() {
            toggleNetworkSelection(network.id, this);
        });

        networkGrid.appendChild(button);
    }
}

/**
 * Función faltante para derivación
 */
function useSeedForDerivation() {
    if (AppState && AppState.transformedSeed) {
        // Switch to derive tab and select transformed source
        if (typeof tabManager !== 'undefined') {
            tabManager.switchToTab('derive');
            setTimeout(() => {
                const transformedRadio = document.querySelector('input[name="seedSource"][value="transformed"]');
                if (transformedRadio) {
                    transformedRadio.checked = true;
                    if (typeof AppState !== 'undefined') {
                        AppState.selectedSource = 'transformed';
                    }
                    if (typeof updateSeedSourcePreviews === 'function') {
                        updateSeedSourcePreviews();
                    }
                    if (typeof updateDerivationButtonState === 'function') {
                        updateDerivationButtonState();
                    }
                    if (typeof showToast === 'function') {
                        showToast('Switched to derivation tab with transformed seed selected', 'success');
                    }
                }
            }, 500);
        }
    }
}

/**
 * Función helper para verificar si una nueva entrada de seed está ocurriendo
 */
function handleNewSeedInput() {
    // Siempre permitir limpiar - función de seguridad
    return true;
}
