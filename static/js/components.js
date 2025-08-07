// Components
const Toast = {
    view() {
        if (!window.AppState.toast) return null;
        return m('.toast', { class: window.AppState.toast.type }, window.AppState.toast.message);
    }
};

const BackendCard = {
    view(vnode) {
        const backend = vnode.attrs.backend;
        const isSelected = window.AppState.selectedBackend && window.AppState.selectedBackend.id === backend.id;
        
        // Get backend icon based on type and id
        const getBackendIcon = (backend) => {
            switch(backend.id) {
                case 'openai': return '🤖';
                case 'gemini': return '💎';
                case 'mistral': return '🌪️';
                case 'anthropic': return '🧠';
                case 'ollama': return '🦙';
                case 'llama_cpp': return '🚀';
                case 'deepl': return '🌍';
                case 'ratchet': return '⚡';
                case 'kalosm': return '🔮';
                default: return '🤖';
            }
        };
        
        const getCapabilities = (backend) => {
            const caps = [];
            if (backend.backend_type === 'local') caps.push('Local');
            if (backend.backend_type === 'api') caps.push('Cloud');
            if (backend.backend_type === 'translation') caps.push('Specialized');
            if (backend.id === 'gemini' || backend.id === 'openai') caps.push('Vision');
            return caps.join(' • ');
        };
        
        return m('.backend-item', {
            class: `${isSelected ? 'selected' : ''} ${!backend.enabled ? 'disabled' : ''}`,
            onclick: backend.enabled ? () => {
                window.AppState.selectedBackend = backend;
            } : null
        }, [
            m('.backend-header', [
                m('.backend-icon', getBackendIcon(backend)),
                m('.backend-info', [
                    m('h3', backend.name),
                    m('p.backend-type', getCapabilities(backend)),
                    m('p.backend-id', backend.id)
                ])
            ]),
            m('.backend-status', [
                m('.status-dot', { class: backend.enabled ? 'enabled' : 'disabled' }),
                m('span', backend.enabled ? 'Ready' : 'Configure'),
                isSelected ? m('.selected-indicator', '✓') : null
            ])
        ]);
    }
};

const BackendSettings = {
    view(vnode) {
        const backend = vnode.attrs.backend;
        if (!backend) return null;

        return m('.settings-panel', [
            m('h3', `${backend.name} Settings`),
            Object.entries(backend.config).map(([key, value]) =>
                m('.setting-group', [
                    m('label', key.replace('_', ' ').toUpperCase()),
                    m('input.setting-input', {
                        type: key.includes('key') ? 'password' : 'text',
                        value: value,
                        placeholder: `Enter ${key}...`,
                        oninput: (e) => {
                            backend.config[key] = e.target.value;
                        }
                    })
                ])
            ),
            m('button.btn.btn-primary', {
                onclick: async () => {
                    const saved = await window.API.saveBackend(backend);
                    if (saved) {
                        window.Utils.showToast('Backend settings saved!');
                        // Update the backend in state
                        const index = window.AppState.backends.findIndex(b => b.id === backend.id);
                        if (index >= 0) {
                            window.AppState.backends[index] = { ...backend };
                            window.AppState.selectedBackend = { ...backend };
                        }
                    } else {
                        window.Utils.showToast('Failed to save settings', 'error');
                    }
                }
            }, 'Save Settings')
        ]);
    }
};

const FileUpload = {
    view() {
        return m('.upload-area', {
            onclick: () => document.getElementById('fileInput').click(),
            ondragover: (e) => {
                e.preventDefault();
                e.target.classList.add('dragover');
            },
            ondragleave: (e) => {
                e.target.classList.remove('dragover');
            },
            ondrop: (e) => {
                e.preventDefault();
                e.target.classList.remove('dragover');
                const files = e.dataTransfer.files;
                if (files.length > 0) {
                    window.AppState.uploadedFile = files[0];
                    m.redraw();
                }
            }
        }, [
            m('.upload-icon', '📄'),
            m('.upload-text', window.AppState.uploadedFile ? window.AppState.uploadedFile.name : 'Drop your image or document here'),
            m('.upload-subtext', 'Supports: JPG, PNG, PDF, DOCX'),
            m('input#fileInput.file-input', {
                type: 'file',
                accept: 'image/*,.pdf,.docx,.txt',
                onchange: (e) => {
                    if (e.target.files.length > 0) {
                        window.AppState.uploadedFile = e.target.files[0];
                    }
                }
            })
        ]);
    }
};

// Export components
window.Components = {
    Toast,
    BackendCard,
    BackendSettings,
    FileUpload
};