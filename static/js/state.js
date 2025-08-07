// State Management
const state = {
    // Translation state
    backends: [],
    selectedBackend: null,
    sourceLanguage: 'auto',
    targetLanguage: 'en',
    inputText: '',
    translatedText: '',
    isLoading: false,
    isExporting: false,
    showSettings: false,
    uploadedFile: null,
    inputMode: 'text', // 'text' or 'file'
    toast: null,
    
    // Authentication state
    currentUser: null,
    showAuth: false,
    authMode: 'login', // 'login' or 'register'
    authEmail: '',
    authPassword: '',
    authLoading: false,
    
    // Payment state
    showPlans: false,
    showPaymentMethods: false,
    plans: [],
    selectedPlan: null,
    paymentLoading: false,
    
    // Export state
    showExportOptions: false,
    exportFormat: 'overlay', // 'overlay' or 'sidebyside'
};

// Export state for other modules
window.AppState = state;