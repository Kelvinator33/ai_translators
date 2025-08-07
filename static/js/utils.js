// Utility functions
const utils = {
    showToast(message, type = 'success') {
        window.AppState.toast = { message, type };
        m.redraw();
        setTimeout(() => {
            window.AppState.toast = null;
            m.redraw();
        }, 3000);
    },

    // Backend availability based on subscription plans
    getAvailableBackends(backends, userPlan = 'free') {
        return backends.filter(backend => {
            if (!backend.enabled) return false;
            
            // Backend availability by subscription plan
            const backendPlans = {
                'ollama': ['free', 'basic', 'pro', 'enterprise'], // Local is available to all
                'deepl': ['basic', 'pro', 'enterprise'], // Professional translation
                'openai': ['pro', 'enterprise'], // Premium AI
                'gemini': ['pro', 'enterprise'], // Premium AI
                'mistral': ['pro', 'enterprise'], // Premium AI
                'anthropic': ['enterprise'], // Enterprise only
                'llama_cpp': ['free', 'basic', 'pro', 'enterprise'], // Local is available to all
                'ratchet': ['pro', 'enterprise'], // Local GPU
                'kalosm': ['pro', 'enterprise'], // Local ML
            };

            const allowedPlans = backendPlans[backend.id] || ['free'];
            return allowedPlans.includes(userPlan);
        });
    },

    getPlanBackendInfo(userPlan = 'free') {
        const planInfo = {
            'free': {
                name: 'Free',
                backends: ['Ollama (Local)', 'llama.cpp (Local)'],
                description: 'Local AI models only'
            },
            'basic': {
                name: 'Basic',
                backends: ['Ollama', 'llama.cpp', 'DeepL API'],
                description: 'Local models + Professional translation'
            },
            'pro': {
                name: 'Pro', 
                backends: ['All Local', 'DeepL', 'OpenAI GPT', 'Google Gemini', 'Mistral'],
                description: 'Premium cloud AI models'
            },
            'enterprise': {
                name: 'Enterprise',
                backends: ['All Available Models'],
                description: 'Full access to all AI backends'
            }
        };
        
        return planInfo[userPlan] || planInfo['free'];
    },

    formatBackendType(type) {
        const types = {
            'local': 'Local',
            'api': 'API',
            'translation': 'Translation Service'
        };
        return types[type] || type;
    },

    languages: {
        'auto': 'Auto-detect',
        'en': 'English',
        'es': 'Spanish',
        'fr': 'French',
        'de': 'German',
        'it': 'Italian',
        'pt': 'Portuguese',
        'ru': 'Russian',
        'ja': 'Japanese',
        'ko': 'Korean',
        'zh': 'Chinese',
        'ar': 'Arabic',
        'hi': 'Hindi',
        'tr': 'Turkish',
        'nl': 'Dutch',
        'sv': 'Swedish',
        'pl': 'Polish',
        'da': 'Danish',
        'no': 'Norwegian',
        'fi': 'Finnish'
    }
};

// Export utils for other modules
window.Utils = utils;