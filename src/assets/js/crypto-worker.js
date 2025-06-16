/**
 * SCypher Crypto Worker v3.0
 * Ejecuta operaciones criptográficas sin bloquear el hilo principal
 */

// Importar Tauri API en el worker
importScripts('https://unpkg.com/@tauri-apps/api@1/dist/bundle.umd.js');

// Worker state
let isProcessing = false;
let currentJobId = null;

// Listen for messages from main thread
self.onmessage = async function(e) {
    const { type, jobId, data } = e.data;

    console.log(`🔧 Worker received: ${type} (job: ${jobId})`);

    switch (type) {
        case 'TRANSFORM_SEED':
            await handleTransformSeed(jobId, data);
            break;

        case 'CANCEL':
            handleCancel(jobId);
            break;

        default:
            sendError(jobId, `Unknown command: ${type}`);
    }
};

/**
 * Handle seed transformation with progress updates
 */
async function handleTransformSeed(jobId, { phrase, password, iterations, memoryCost }) {
    if (isProcessing) {
        sendError(jobId, 'Another operation is already in progress');
        return;
    }

    isProcessing = true;
    currentJobId = jobId;

    try {
        console.log('🔐 Worker starting Argon2id transformation...');

        // Send initial progress
        sendProgress(jobId, 0, 'Initializing cryptographic process...');

        // Small delay to ensure progress is sent
        await new Promise(resolve => setTimeout(resolve, 100));

        // Send mid-process progress
        sendProgress(jobId, 25, 'Preparing Argon2id parameters...');

        // Another small delay for visual feedback
        await new Promise(resolve => setTimeout(resolve, 150));

        // Update progress before the heavy computation
        sendProgress(jobId, 50, 'Executing Argon2id (3 iterations, 64MB)...');

        const startTime = Date.now();

        // Execute the actual Tauri command
        // This is the blocking operation, but it's in a worker so UI stays responsive
        const result = await window.__TAURI__.tauri.invoke('transform_seed_phrase', {
            phrase: phrase,
            password: password,
            iterations: iterations,
            memoryCost: memoryCost
        });

        const processingTime = Date.now() - startTime;
        console.log(`🔐 Worker completed Argon2id in: ${processingTime}ms`);

        // Quick progress update
        sendProgress(jobId, 90, 'Finalizing transformation...');

        // Small delay for smooth UX
        await new Promise(resolve => setTimeout(resolve, 200));

        // Send completion
        sendProgress(jobId, 100, `Transformation completed in ${processingTime}ms`);

        // Send final result
        sendSuccess(jobId, {
            result: result,
            processingTime: processingTime
        });

    } catch (error) {
        console.error('🔐 Worker transformation error:', error);
        sendError(jobId, error.message || 'Transformation failed');
    } finally {
        isProcessing = false;
        currentJobId = null;
    }
}

/**
 * Handle cancellation request
 */
function handleCancel(jobId) {
    if (currentJobId === jobId && isProcessing) {
        console.log('🔐 Worker cancelling operation...');
        isProcessing = false;
        currentJobId = null;
        sendError(jobId, 'Operation cancelled by user');
    }
}

/**
 * Send progress update to main thread
 */
function sendProgress(jobId, progress, message) {
    self.postMessage({
        type: 'PROGRESS',
        jobId: jobId,
        data: {
            progress: progress,
            message: message
        }
    });
}

/**
 * Send success result to main thread
 */
function sendSuccess(jobId, result) {
    self.postMessage({
        type: 'SUCCESS',
        jobId: jobId,
        data: result
    });
}

/**
 * Send error to main thread
 */
function sendError(jobId, error) {
    self.postMessage({
        type: 'ERROR',
        jobId: jobId,
        data: {
            error: error
        }
    });
}

// Worker ready
console.log('🔧 SCypher Crypto Worker v3.0 initialized and ready');
self.postMessage({
    type: 'READY',
    data: {
        message: 'Crypto worker initialized successfully'
    }
});
