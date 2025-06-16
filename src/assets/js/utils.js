/**
 * SCypher GUI v3.0 - Utility Functions
 * Contains helper functions for the entire application
 */

// Global state management
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

// Tauri integration utilities
let invoke = null;
let tauriAvailable = false;

/**
 * Initialize Tauri connection
 */
async function initializeTauri() {
    try {
        const { invoke: tauriInvoke } = await import('https://unpkg.com/@tauri-apps/api@1/tauri');
        invoke = tauriInvoke;
        tauriAvailable = true;
        console.log('✅ Tauri backend connected');
        return true;
    } catch (error) {
        console.error('❌ Failed to connect to Tauri backend:', error);
        throw new Error('Tauri backend not available');
    }
}

/**
 * Initialize native Tauri file drop functionality
 */
async function initializeTauriFileDrop() {
    try {
        console.log('🔄 Initializing Tauri native file drop...');
        const { listen } = await import('https://unpkg.com/@tauri-apps/api@1/event');

        const unlisten = await listen('tauri://file-drop', async (event) => {
            console.log('🎯 TAURI NATIVE FILE DROP DETECTED!');

            const container = document.getElementById('wordContainer');
            container.classList.remove('drag-over');

            try {
                if (currentWords.length > 0) {
                    if (!confirm('This will replace current words, continue?')) {
                        return;
                    }
                }

                const droppedFiles = event.payload;
                if (droppedFiles && droppedFiles.length > 0) {
                    const filePath = droppedFiles[0];
                    showToast('Reading dropped file...', 'info', 2000);

                    const content = await invoke('read_seed_file', { path: filePath });
                    const words = content.split(/\s+/).filter(word => word.length > 0);

                    currentWords = words.map(word => word.toLowerCase());
                    editingIndex = -1;
                    renderWords();
                    updateValidationStatus();
                    updateProcessButtonState();

                    showToast(`Loaded ${words.length} words from dropped file`, 'success');
                }
            } catch (error) {
                console.error('❌ Native file drop error:', error);
                showToast(`Failed to read dropped file: ${error}`, 'error');
            }
        });

        window.tauriFileDropUnlisten = unlisten;
        console.log('✅ Tauri native file drop initialized');
    } catch (error) {
        console.error('❌ Failed to initialize Tauri file drop:', error);
    }
}

/**
 * Template loading utility
 */
async function loadTemplate(templatePath) {
    try {
        const response = await fetch(templatePath);
        if (!response.ok) {
            throw new Error(`Failed to load template: ${response.status}`);
        }
        return await response.text();
    } catch (error) {
        console.error('Error loading template:', error);
        return null;
    }
}

/**
 * Toast notification system
 */
function showToast(message, type = 'info', duration = 4000) {
    const container = document.getElementById('toastContainer');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast ${type}`;

    const icons = {
        success: '✅',
        error: '❌',
        warning: '⚠️',
        info: 'ℹ️'
    };

    toast.innerHTML = `
        <span class="toast-icon">${icons[type]}</span>
        <span>${message}</span>
        <button class="toast-close" onclick="removeToast(this.parentElement)">×</button>
    `;

    container.appendChild(toast);
    setTimeout(() => toast.classList.add('show'), 100);
    setTimeout(() => removeToast(toast), duration);
}

function removeToast(toast) {
    if (toast && toast.parentElement) {
        toast.classList.remove('show');
        setTimeout(() => {
            if (toast.parentElement) {
                toast.parentElement.removeChild(toast);
            }
        }, 300);
    }
}

/**
 * Network information utilities
 */
function getNetworkInfo(network) {
    const networkMap = {
        bitcoin: {
            name: 'Bitcoin',
            icon: 'assets/icons/btc.svg',
            color: '#F7931A',
            fallbackIcon: '₿'
        },
        ethereum: {
            name: 'Ethereum',
            icon: 'assets/icons/eth.svg',
            color: '#627EEA',
            fallbackIcon: 'Ξ'
        },
        ergo: {
            name: 'Ergo',
            icon: 'assets/icons/erg.svg',
            color: '#FF5722',
            fallbackIcon: '⚡'
        },
        tron: {
            name: 'TRON',
            icon: 'assets/icons/trx.svg',
            color: '#FF0013',
            fallbackIcon: '🔺'
        },
        bsc: {
            name: 'BSC',
            icon: 'assets/icons/bnb.svg',
            color: '#F0B90B',
            fallbackIcon: '⚪'
        },
        polygon: {
            name: 'Polygon',
            icon: 'assets/icons/matic.svg',
            color: '#8247E5',
            fallbackIcon: '◆'
        },
        cardano: {
            name: 'Cardano',
            icon: 'assets/icons/ada.svg',
            color: '#0033AD',
            fallbackIcon: '₳'
        },
        dogecoin: {
            name: 'Dogecoin',
            icon: 'assets/icons/doge.svg',
            color: '#C2A633',
            fallbackIcon: 'Ð'
        },
        litecoin: {
            name: 'Litecoin',
            icon: 'assets/icons/ltc.svg',
            color: '#BFBBBB',
            fallbackIcon: 'Ł'
        },
        solana: {
            name: 'Solana',
            icon: 'assets/icons/sol.svg',
            color: '#00FFA3',
            fallbackIcon: '◉'
        }
    };

    return networkMap[network] || {
        name: network,
        icon: null,
        color: '#888',
        fallbackIcon: '●'
    };
}

function hexToRgb(hex) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ?
        `${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}` :
        '136, 136, 136';
}

/**
 * Password strength evaluation
 */
function evaluatePasswordStrength(password) {
    if (password.length === 0) {
        return { strength: 0, label: '', color: '' };
    }

    let score = 0;
    const checks = {
        length: password.length >= 12,
        uppercase: /[A-Z]/.test(password),
        lowercase: /[a-z]/.test(password),
        numbers: /\d/.test(password),
        symbols: /[^A-Za-z0-9]/.test(password),
        longLength: password.length >= 16
    };

    if (checks.length) score += 2;
    if (checks.longLength) score += 1;
    if (checks.uppercase) score += 1;
    if (checks.lowercase) score += 1;
    if (checks.numbers) score += 1;
    if (checks.symbols) score += 2;

    if (score < 3) {
        return { strength: 1, label: 'Weak', color: '#ef4444' };
    } else if (score < 6) {
        return { strength: 2, label: 'Fair', color: '#f59e0b' };
    } else if (score < 8) {
        return { strength: 3, label: 'Good', color: '#10b981' };
    } else {
        return { strength: 4, label: 'Strong', color: '#059669' };
    }
}

/**
 * File handling utilities
 */
async function handleModernBrowse() {
    try {
        if (currentWords.length > 0) {
            if (!confirm('This will replace current words, continue?')) {
                return;
            }
        }

        console.log('📂 Opening modern native file dialog...');
        showToast('Opening file dialog...', 'info', 2000);

        const selectedFile = await invoke('open_file_dialog');

        if (selectedFile) {
            const content = await invoke('read_seed_file', { path: selectedFile });
            const words = content.split(/\s+/).filter(word => word.length > 0);

            currentWords = words.map(word => word.toLowerCase());
            editingIndex = -1;
            renderWords();
            updateValidationStatus();
            updateProcessButtonState();

            showToast(`Loaded ${words.length} words from file`, 'success');
        } else {
            showToast('File selection cancelled', 'info', 2000);
        }
    } catch (error) {
        console.error('❌ Modern file dialog error:', error);
        showToast(`Failed to open file dialog: ${error}`, 'error');
    }
}

async function handleModernSave() {
    try {
        const resultText = document.getElementById('resultText').textContent;

        if (!resultText || resultText.trim() === '') {
            showToast('No result to save', 'warning');
            return;
        }

        showToast('Opening save dialog...', 'info', 2000);
        const selectedPath = await invoke('save_file_dialog');

        if (selectedPath) {
            await invoke('save_result_file', {
                content: resultText,
                path: selectedPath
            });
            showToast(`File saved successfully to ${selectedPath}`, 'success');
        } else {
            showToast('Save cancelled', 'info', 2000);
        }
    } catch (error) {
        console.error('❌ Modern save dialog error:', error);
        showToast(`Failed to save file: ${error}`, 'error');
    }
}

/**
 * Export utilities
 */
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

/**
 * Clipboard utilities
 */
async function copyToClipboard() {
    const resultText = document.getElementById('resultText').textContent;
    try {
        await navigator.clipboard.writeText(resultText);
        showToast('Result copied to clipboard', 'success', 2000);
    } catch (error) {
        console.error('Failed to copy to clipboard:', error);
        showToast('Failed to copy result', 'error');
    }
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

/**
 * Password visibility toggle
 */
function togglePasswordVisibility(inputId, button) {
    const input = document.getElementById(inputId);
    if (input.type === 'password') {
        input.type = 'text';
        button.textContent = '🙈';
    } else {
        input.type = 'password';
        button.textContent = '👁️';
    }
}

/**
 * General validation functions
 */
function validateSingleWord(word, bip39WordList) {
    return bip39WordList.includes(word.toLowerCase());
}

function handleNewSeedInput() {
    const hasResults = Object.keys(AppState.derivationResults).length > 0;
    const hasTransformed = AppState.transformedSeed !== null;

    if (hasResults || hasTransformed) {
        const message = hasResults ?
            'This will clear existing derivation results. Continue?' :
            'This will clear the transformed seed. Continue?';

        if (!confirm(message)) {
            return false;
        }
    }

    AppState.derivationResults = {};
    AppState.transformedSeed = null;
    AppState.scypherPassword = null;

    document.getElementById('resultsSection').style.display = 'none';
    return true;
}

/**
 * Global exports for HTML onclick handlers
 */
window.copyToClipboard = copyToClipboard;
window.copyAddress = copyAddress;
window.copyNetworkAddresses = copyNetworkAddresses;
window.togglePasswordVisibility = togglePasswordVisibility;
window.removeToast = removeToast;

// Cache para iconos SVG cargados
const iconCache = new Map();

/**
 * Carga un icono SVG y lo retorna como string
 */
async function loadSVGIcon(iconPath) {
    if (iconCache.has(iconPath)) {
        return iconCache.get(iconPath);
    }

    try {
        const response = await fetch(iconPath);
        if (!response.ok) {
            throw new Error(`Failed to load icon: ${response.status}`);
        }

        const svgContent = await response.text();
        iconCache.set(iconPath, svgContent);
        return svgContent;
    } catch (error) {
        console.warn(`Failed to load SVG icon ${iconPath}:`, error);
        return null;
    }
}

/**
 * Crea un elemento de icono (SVG o fallback)
 */
async function createNetworkIcon(network, className = 'network-icon') {
    const networkInfo = getNetworkInfo(network);

    // Intentar cargar el SVG
    if (networkInfo.icon) {
        const svgContent = await loadSVGIcon(networkInfo.icon);
        if (svgContent) {
            // Crear un contenedor para el SVG
            const iconContainer = document.createElement('span');
            iconContainer.className = className;
            iconContainer.innerHTML = svgContent;

            // Aplicar estilos al SVG
            const svg = iconContainer.querySelector('svg');
            if (svg) {
                svg.style.width = '100%';
                svg.style.height = '100%';
                svg.style.display = 'block';
            }

            return iconContainer;
        }
    }

    // Fallback al emoji/texto
    const iconContainer = document.createElement('span');
    iconContainer.className = className;
    iconContainer.textContent = networkInfo.fallbackIcon;
    return iconContainer;
}
