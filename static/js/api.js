// API functions
const api = {
    async getBackends() {
        try {
            const response = await fetch('/api/backends');
            return await response.json();
        } catch (error) {
            console.error('Failed to fetch backends:', error);
            return [];
        }
    },

    async saveBackend(backend) {
        try {
            const response = await fetch('/api/backends', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(backend)
            });
            return response.ok;
        } catch (error) {
            console.error('Failed to save backend:', error);
            return false;
        }
    },

    async translateText(text, sourceLanguage, targetLanguage, backendId) {
        try {
            const response = await fetch('/api/translate', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    text,
                    source_lang: sourceLanguage === 'auto' ? null : sourceLanguage,
                    target_lang: targetLanguage,
                    backend_id: backendId
                })
            });
            return await response.json();
        } catch (error) {
            console.error('Translation failed:', error);
            throw error;
        }
    },

    async uploadAndTranslate(file, sourceLanguage, targetLanguage, backendId) {
        try {
            const formData = new FormData();
            formData.append('file', file);
            formData.append('source_lang', sourceLanguage);
            formData.append('target_lang', targetLanguage);
            formData.append('backend_id', backendId);

            const response = await fetch('/api/upload', {
                method: 'POST',
                body: formData
            });
            
            if (!response.ok) {
                throw new Error(`Upload failed: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('Upload and translate failed:', error);
            throw error;
        }
    },

    async exportTranslatedImage(originalFile, translatedText, targetLanguage) {
        try {
            console.log('Export request - File:', originalFile, 'Text:', translatedText);
            
            if (!originalFile) {
                throw new Error('No original file provided');
            }
            
            const formData = new FormData();
            formData.append('original_image', originalFile);
            formData.append('translated_text', translatedText);
            formData.append('target_lang', targetLanguage);
            formData.append('export_type', 'overlay'); // Can be 'overlay' or 'sidebyside'

            console.log('Sending export request...');
            const response = await fetch('/api/export', {
                method: 'POST',
                body: formData
            });
            
            console.log('Export response status:', response.status, response.statusText);
            
            if (!response.ok) {
                const errorText = await response.text();
                console.error('Export error response:', errorText);
                throw new Error(`Export failed: ${response.statusText} - ${errorText}`);
            }
            
            const blob = await response.blob();
            console.log('Export successful, blob size:', blob.size);
            return blob;
        } catch (error) {
            console.error('Image export failed:', error);
            throw error;
        }
    }
};

// Export API for other modules
window.API = api;