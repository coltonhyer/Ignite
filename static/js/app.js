import { encrypt, decrypt, exportKey, importKey } from './crypto.js';
import { buildShareUrl, parseShareUrl } from './url.mjs';

// DOM Elements
const views = {
    create: document.getElementById('view-create'),
    share: document.getElementById('view-share'),
    readPrompt: document.getElementById('view-read-prompt'),
    readResult: document.getElementById('view-read-result'),
    loading: document.getElementById('loading')
};

// Create View Elements
const secretText = document.getElementById('secret-text');
const charCount = document.getElementById('char-count');
const ttlSelect = document.getElementById('ttl-select');
const btnCreate = document.getElementById('btn-create');
const createError = document.getElementById('create-error');

// Share View Elements
const shareLink = document.getElementById('share-link');
const btnCopy = document.getElementById('btn-copy');

// Read View Elements
const btnReveal = document.getElementById('btn-reveal');
const secretContent = document.getElementById('secret-content');
const readError = document.getElementById('read-error');
const loadingText = document.getElementById('loading-text');

// Max payload size from constraints (10KB base64 encoded means plaintext max is lower, let's restrict text length securely)
const MAX_BYTES = 7000; // conservative limit for UTF-8 text before encryption & base64

// Initialize the app based on URL
function init() {
    const parsed = parseShareUrl();
    if (parsed) {
        showView('readPrompt');
        setupReadView(parsed.id, parsed.key);
    } else {
        showView('create');
        setupCreateView();
    }
}

// View Management
function showView(viewName) {
    Object.values(views).forEach(el => el.classList.add('hidden'));
    if (views[viewName]) {
        if (viewName === 'loading') {
            views[viewName].classList.remove('hidden');
            views[viewName].classList.add('flex');
        } else {
            views[viewName].classList.remove('hidden');
            views[viewName].classList.add('flex');
        }
    }
}

function showError(el, message) {
    el.textContent = message;
    el.classList.remove('hidden');
}

function hideError(el) {
    el.classList.add('hidden');
}

// Create Flow
function setupCreateView() {
    secretText.addEventListener('input', () => {
        const bytes = new Blob([secretText.value]).size;
        charCount.textContent = bytes;
        if (bytes > MAX_BYTES) {
            charCount.classList.add('text-red-500');
            btnCreate.disabled = true;
            btnCreate.classList.add('opacity-50', 'cursor-not-allowed');
        } else {
            charCount.classList.remove('text-red-500');
            btnCreate.disabled = false;
            btnCreate.classList.remove('opacity-50', 'cursor-not-allowed');
        }
    });

    btnCreate.addEventListener('click', async () => {
        const plaintext = secretText.value;
        if (!plaintext.trim()) {
            showError(createError, 'Please enter a secret to share.');
            return;
        }

        const bytes = new Blob([plaintext]).size;
        if (bytes > MAX_BYTES) {
            showError(createError, `Secret is too long. Max size is ${MAX_BYTES} bytes.`);
            return;
        }

        const ttlSeconds = parseInt(ttlSelect.value, 10);
        
        hideError(createError);
        loadingText.textContent = 'Encrypting & uploading...';
        showView('loading');

        try {
            // 1. Encrypt client-side
            const { ciphertext, nonce, key } = await encrypt(plaintext);
            
            // 2. Export key
            const base64urlKey = await exportKey(key);

            // 3. Send to server
            const response = await fetch('/api/secrets', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    ciphertext,
                    nonce,
                    ttl_seconds: ttlSeconds
                })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                throw new Error(errData.error || 'Failed to create secret. Server returned ' + response.status);
            }

            const data = await response.json();
            
            // 4. Construct share URL
            const urlPath = buildShareUrl(data.id, base64urlKey);
            const fullUrl = window.location.origin + urlPath;
            
            // 5. Show share view
            shareLink.value = fullUrl;
            showView('share');
            
            // Focus and select the link
            shareLink.select();
            
        } catch (error) {
            console.error('Creation error:', error);
            showError(createError, error.message);
            showView('create');
        }
    });

    btnCopy.addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(shareLink.value);
            const originalIcon = btnCopy.innerHTML;
            btnCopy.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-green-500" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" /></svg>';
            setTimeout(() => {
                btnCopy.innerHTML = originalIcon;
            }, 2000);
        } catch (err) {
            console.error('Failed to copy', err);
        }
    });
}

// Read Flow
function setupReadView(id, keyString) {
    btnReveal.addEventListener('click', async () => {
        hideError(readError);
        loadingText.textContent = 'Fetching & decrypting...';
        showView('loading');

        try {
            // 1. Validate and import key before burn request
            let key;
            try {
                key = await importKey(keyString);
            } catch {
                throw new Error('The decryption key in this link is invalid or malformed.');
            }

            // 2. Fetch from server (this burns the secret)
            const response = await fetch(`/api/secrets/${id}`, {
                method: 'DELETE'
            });

            if (response.status === 410) {
                throw new Error('This secret has already been viewed and destroyed, or it has expired.');
            }

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                throw new Error(errData.error || 'Failed to retrieve secret.');
            }

            const data = await response.json();

            // 3. Decrypt client-side
            const plaintext = await decrypt(data.ciphertext, data.nonce, key);

            // 4. Show result
            secretContent.textContent = plaintext;
            
            // Remove the hash from the URL so it's not accidentally shared or saved in history
            history.replaceState(null, '', window.location.pathname);
            
            showView('readResult');

        } catch (error) {
            console.error('Decryption/Fetch error:', error);
            showError(readError, error.message);
            showView('readPrompt');
        }
    });
}

// Start the app
document.addEventListener('DOMContentLoaded', init);
