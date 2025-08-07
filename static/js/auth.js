// Authentication functionality
const Auth = {
    token: localStorage.getItem('auth_token') || null,
    user: JSON.parse(localStorage.getItem('user_profile') || 'null'),

    async login(email, password) {
        try {
            const response = await fetch('/api/auth/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email, password })
            });

            if (response.ok) {
                const data = await response.json();
                this.setAuth(data.token, data.user);
                return { success: true, user: data.user };
            } else {
                const error = await response.text();
                return { success: false, error };
            }
        } catch (error) {
            return { success: false, error: 'Login failed' };
        }
    },

    async register(email, password) {
        try {
            const response = await fetch('/api/auth/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email, password })
            });

            if (response.ok) {
                const data = await response.json();
                this.setAuth(data.token, data.user);
                return { success: true, user: data.user };
            } else {
                const error = await response.text();
                return { success: false, error };
            }
        } catch (error) {
            return { success: false, error: 'Registration failed' };
        }
    },

    async getProfile() {
        if (!this.token) return null;

        try {
            const response = await fetch('/api/auth/profile', {
                headers: { 'Authorization': `Bearer ${this.token}` }
            });

            if (response.ok) {
                const user = await response.json();
                this.user = user;
                localStorage.setItem('user_profile', JSON.stringify(user));
                return user;
            } else {
                this.logout();
                return null;
            }
        } catch (error) {
            return null;
        }
    },

    setAuth(token, user) {
        this.token = token;
        this.user = user;
        localStorage.setItem('auth_token', token);
        localStorage.setItem('user_profile', JSON.stringify(user));
    },

    logout() {
        this.token = null;
        this.user = null;
        localStorage.removeItem('auth_token');
        localStorage.removeItem('user_profile');
    },

    isAuthenticated() {
        return !!this.token;
    },

    getAuthHeaders() {
        return this.token ? { 'Authorization': `Bearer ${this.token}` } : {};
    }
};

// Login/Register Component
const LoginForm = {
    view() {
        return m('.auth-form', [
            m('.auth-tabs', [
                m('button.auth-tab', {
                    class: window.AppState.authMode === 'login' ? 'active' : '',
                    onclick: () => window.AppState.authMode = 'login'
                }, 'Login'),
                m('button.auth-tab', {
                    class: window.AppState.authMode === 'register' ? 'active' : '',
                    onclick: () => window.AppState.authMode = 'register'
                }, 'Register')
            ]),

            m('form.auth-form-content', {
                onsubmit: async (e) => {
                    e.preventDefault();
                    await this.handleSubmit();
                }
            }, [
                m('input.auth-input', {
                    type: 'email',
                    placeholder: 'Email',
                    value: window.AppState.authEmail || '',
                    oninput: (e) => window.AppState.authEmail = e.target.value,
                    required: true
                }),
                m('input.auth-input', {
                    type: 'password',
                    placeholder: 'Password',
                    value: window.AppState.authPassword || '',
                    oninput: (e) => window.AppState.authPassword = e.target.value,
                    required: true
                }),
                m('button.btn.btn-primary', {
                    type: 'submit',
                    disabled: window.AppState.authLoading
                }, window.AppState.authLoading ? 'Please wait...' : 
                   (window.AppState.authMode === 'login' ? 'Login' : 'Register'))
            ])
        ]);
    },

    async handleSubmit() {
        window.AppState.authLoading = true;
        m.redraw();

        const email = window.AppState.authEmail;
        const password = window.AppState.authPassword;

        let result;
        if (window.AppState.authMode === 'login') {
            result = await Auth.login(email, password);
        } else {
            result = await Auth.register(email, password);
        }

        if (result.success) {
            window.Utils.showToast(`Welcome ${result.user.email}!`);
            window.AppState.showAuth = false;
            window.AppState.currentUser = result.user;
            // Refresh backend list to show enabled backends
            window.AppState.backends = await window.API.getBackends();
        } else {
            window.Utils.showToast(result.error, 'error');
        }

        window.AppState.authLoading = false;
        window.AppState.authEmail = '';
        window.AppState.authPassword = '';
        m.redraw();
    }
};

// User Profile Component
const UserProfile = {
    async refreshProfile() {
        // Refresh user profile to get updated usage stats
        if (Auth.token) {
            const response = await fetch('/api/auth/profile', {
                headers: Auth.getAuthHeaders()
            });
            if (response.ok) {
                const data = await response.json();
                window.AppState.currentUser = data;
                m.redraw();
            }
        }
    },

    view() {
        const user = window.AppState.currentUser;
        if (!user) return null;

        return m('.user-profile', [
            m('.profile-header', [
                m('h3', `Welcome, ${user.email}`),
                m('span.plan-badge', { class: `plan-${user.subscription_plan}` }, 
                  user.subscription_plan.toUpperCase())
            ]),
            m('.usage-stats', [
                m('.stat', [
                    m('.stat-value', user.translations_used_today || 0),
                    m('.stat-label', 'Used Today')
                ]),
                m('.stat', [
                    m('.stat-value', user.translations_remaining || 0),
                    m('.stat-label', 'Remaining')
                ])
            ]),
            m('.profile-actions', [
                m('button.btn.btn-secondary', {
                    onclick: () => window.AppState.showPlans = true
                }, user.subscription_plan === 'free' ? 'Upgrade Plan' : 'Change Plan'),
                m('button.btn.btn-outline', {
                    onclick: () => {
                        Auth.logout();
                        window.AppState.currentUser = null;
                        window.Utils.showToast('Logged out successfully');
                        m.redraw();
                    }
                }, 'Logout')
            ])
        ]);
    }
};

// Export authentication components
window.Auth = Auth;
window.AuthComponents = {
    LoginForm,
    UserProfile
};