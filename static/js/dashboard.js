// Main Dashboard Component
const Dashboard = {
    async oninit() {
        // Check authentication status
        if (window.Auth.token) {
            window.AppState.currentUser = await window.Auth.getProfile();
        }
        
        // Load backends on init
        window.AppState.backends = await window.API.getBackends();
        // Select first enabled backend by default
        window.AppState.selectedBackend = window.AppState.backends.find(b => b.enabled);
        
        // Load subscription plans
        window.AppState.plans = await window.Payments.getPlans();
        
        m.redraw();
    },

    view() {
        return m('.container', [
            m(window.Components.Toast),
            
            m('.header', [
                m('.header-content', [
                    m('h1', '🌐 AI Translation Service'),
                    m('p', 'Translate text and documents using various AI backends')
                ]),
                m('.header-actions', [
                    window.Auth.isAuthenticated() ? 
                        m(window.AuthComponents.UserProfile) :
                        m('button.btn.btn-primary', {
                            onclick: () => window.AppState.showAuth = true
                        }, 'Login / Register')
                ])
            ]),

            m('.dashboard', [
                // Left panel - Backend selection
                m('.card', [
                    m('h2', 'AI Backends'),
                    m('.backend-list',
                        window.Utils.getAvailableBackends(
                            window.AppState.backends, 
                            window.AppState.currentUser?.subscription_plan || 'free'
                        ).map(backend =>
                            m(window.Components.BackendCard, { backend })
                        )
                    ),
                    
                    // Show plan limitations if not enterprise
                    window.AppState.currentUser && window.AppState.currentUser.subscription_plan !== 'enterprise' ?
                        m('.plan-limitation', [
                            m('p.limitation-text', `${window.Utils.getPlanBackendInfo(window.AppState.currentUser.subscription_plan).description}`),
                            m('button.btn.btn-outline', {
                                onclick: () => window.AppState.showPlans = true,
                                style: { marginTop: '10px', width: '100%' }
                            }, '⬆️ Upgrade for More Models')
                        ]) : null,
                    
                    // Settings toggle
                    m('button.btn.btn-secondary', {
                        onclick: () => window.AppState.showSettings = !window.AppState.showSettings,
                        style: { marginTop: '20px', width: '100%' }
                    }, window.AppState.showSettings ? 'Hide Settings' : 'Configure Backend')
                ]),

                // Right panel - Translation interface
                m('.card', [
                    m('h2', 'Translation'),
                    m(window.TranslationInterface)
                ])
            ]),

            // Settings panel
            window.AppState.showSettings && window.AppState.selectedBackend ?
                m(window.Components.BackendSettings, { backend: window.AppState.selectedBackend }) : null,
            
            // Authentication modal
            window.AppState.showAuth ? 
                m('.modal-overlay', {
                    onclick: (e) => {
                        if (e.target === e.currentTarget) {
                            window.AppState.showAuth = false;
                        }
                    }
                }, m('.modal', m(window.AuthComponents.LoginForm))) : null,
            
            // Subscription plans modal
            window.AppState.showPlans ? 
                m('.modal-overlay', {
                    onclick: (e) => {
                        if (e.target === e.currentTarget) {
                            window.AppState.showPlans = false;
                        }
                    }
                }, m('.modal.plans-modal', 
                    window.AppState.showPaymentMethods ?
                        m(window.PaymentComponents.PaymentMethods) :
                        m(window.PaymentComponents.SubscriptionPlans)
                )) : null
        ]);
    }
};

// Initialize the application
document.addEventListener('DOMContentLoaded', function() {
    m.mount(document.getElementById('app'), Dashboard);
});