// Translation Interface Component
const TranslationInterface = {
    async handleTranslate() {
        if (!window.AppState.selectedBackend) {
            window.Utils.showToast('Please select an AI backend', 'error');
            return;
        }

        window.AppState.isLoading = true;
        m.redraw();

        try {
            let result;
            
            if (window.AppState.uploadedFile) {
                // Upload and translate file
                result = await window.API.uploadAndTranslate(
                    window.AppState.uploadedFile,
                    window.AppState.sourceLanguage,
                    window.AppState.targetLanguage,
                    window.AppState.selectedBackend.id
                );
            } else if (window.AppState.inputText.trim()) {
                // Translate text
                result = await window.API.translateText(
                    window.AppState.inputText,
                    window.AppState.sourceLanguage,
                    window.AppState.targetLanguage,
                    window.AppState.selectedBackend.id
                );
            } else {
                window.Utils.showToast('Please enter text or upload a file', 'error');
                window.AppState.isLoading = false;
                return;
            }

            window.AppState.translatedText = result.translated_text;
            window.Utils.showToast(`Translated using ${result.backend_used}`);
            
            // Refresh user profile to update usage stats
            if (window.AuthComponents.UserProfile.refreshProfile) {
                await window.AuthComponents.UserProfile.refreshProfile();
            }
            
        } catch (error) {
            window.Utils.showToast('Translation failed: ' + error.message, 'error');
        } finally {
            window.AppState.isLoading = false;
            m.redraw();
        }
    },

    async handleExportImage() {
        if (!window.AppState.uploadedFile || !window.AppState.translatedText) {
            window.Utils.showToast('Please upload an image and translate it first', 'error');
            return;
        }

        try {
            window.AppState.isExporting = true;
            m.redraw();

            const blob = await window.API.exportTranslatedImage(
                window.AppState.uploadedFile,
                window.AppState.translatedText,
                window.AppState.targetLanguage
            );

            // Create download link
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `translated_${window.AppState.uploadedFile.name}.png`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);

            window.Utils.showToast('Translated image downloaded!');
        } catch (error) {
            window.Utils.showToast('Export failed: ' + error.message, 'error');
        } finally {
            window.AppState.isExporting = false;
            m.redraw();
        }
    },

    view() {
        const hasImage = window.AppState.uploadedFile && window.AppState.uploadedFile.type.startsWith('image/');
        const hasTranslation = window.AppState.translatedText.trim().length > 0;

        return m('.translation-section', [
            // Language selector
            m('.language-selector', [
                m('.language-group', [
                    m('label', 'From:'),
                    m('select.language-select', {
                        value: window.AppState.sourceLanguage,
                        onchange: (e) => window.AppState.sourceLanguage = e.target.value
                    }, Object.entries(window.Utils.languages).map(([code, name]) =>
                        m('option', { value: code }, name)
                    ))
                ]),
                
                m('button.swap-btn', {
                    onclick: () => {
                        const temp = window.AppState.sourceLanguage;
                        window.AppState.sourceLanguage = window.AppState.targetLanguage;
                        window.AppState.targetLanguage = temp;
                    },
                    title: 'Swap languages'
                }, '⇄'),
                
                m('.language-group', [
                    m('label', 'To:'),
                    m('select.language-select', {
                        value: window.AppState.targetLanguage,
                        onchange: (e) => window.AppState.targetLanguage = e.target.value
                    }, Object.entries(window.Utils.languages).filter(([code]) => code !== 'auto').map(([code, name]) =>
                        m('option', { value: code }, name)
                    ))
                ])
            ]),

            // Input section with tabs
            m('.input-section', [
                m('.input-tabs', [
                    m('button.tab', {
                        class: window.AppState.inputMode === 'text' ? 'active' : '',
                        onclick: () => window.AppState.inputMode = 'text'
                    }, '📝 Text'),
                    m('button.tab', {
                        class: window.AppState.inputMode === 'file' ? 'active' : '',
                        onclick: () => window.AppState.inputMode = 'file'
                    }, '📄 Document')
                ]),

                window.AppState.inputMode === 'file' ? [
                    m(window.Components.FileUpload),
                    hasImage ? m('.image-preview', [
                        m('img', {
                            src: URL.createObjectURL(window.AppState.uploadedFile),
                            alt: 'Uploaded image preview',
                            style: { maxWidth: '100%', maxHeight: '200px' }
                        }),
                        m('.file-info', [
                            m('span', window.AppState.uploadedFile.name),
                            m('button.btn.btn-small', {
                                onclick: () => {
                                    window.AppState.uploadedFile = null;
                                    window.AppState.translatedText = '';
                                }
                            }, '✕ Remove')
                        ])
                    ]) : null
                ] : [
                    m('textarea.text-input', {
                        placeholder: 'Enter text to translate...',
                        value: window.AppState.inputText,
                        oninput: (e) => window.AppState.inputText = e.target.value,
                        rows: 6
                    })
                ]
            ]),

            // Action buttons
            m('.action-buttons', [
                m('button.btn.btn-primary', {
                    onclick: () => this.handleTranslate(),
                    disabled: window.AppState.isLoading || !window.AppState.selectedBackend
                }, window.AppState.isLoading ? m('.loading', [m('.spinner'), 'Translating...']) : '🚀 Translate'),

                hasImage && hasTranslation ? m('button.btn.btn-secondary', {
                    onclick: () => this.handleExportImage(),
                    disabled: window.AppState.isExporting
                }, window.AppState.isExporting ? m('.loading', [m('.spinner'), 'Exporting...']) : '💾 Export Image') : null
            ]),

            // Output section
            hasTranslation ? m('.output-section', [
                m('.output-header', [
                    m('h3', 'Translation Result'),
                    m('button.btn.btn-small', {
                        onclick: () => navigator.clipboard.writeText(window.AppState.translatedText)
                    }, '📋 Copy')
                ]),
                m('textarea.text-output', {
                    readonly: true,
                    value: window.AppState.translatedText,
                    rows: 8
                })
            ]) : m('.output-placeholder', [
                m('div', '🌐'),
                m('p', 'Your translation will appear here')
            ])
        ]);
    }
};

// Export translation interface
window.TranslationInterface = TranslationInterface;