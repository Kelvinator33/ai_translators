// Payment and subscription functionality
const Payments = {
    plans: [],

    async getPlans() {
        try {
            const response = await fetch('/api/plans');
            this.plans = await response.json();
            return this.plans;
        } catch (error) {
            console.error('Failed to fetch plans:', error);
            return [];
        }
    },

    async createPaymentIntent(planId, paymentMethod) {
        try {
            const response = await fetch('/api/payment/intent', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...window.Auth.getAuthHeaders()
                },
                body: JSON.stringify({
                    plan: planId,
                    payment_method: paymentMethod
                })
            });

            if (response.ok) {
                return await response.json();
            } else {
                throw new Error('Failed to create payment intent');
            }
        } catch (error) {
            console.error('Payment creation failed:', error);
            throw error;
        }
    },

    async cancelSubscription() {
        try {
            const response = await fetch('/api/subscription/cancel', {
                method: 'POST',
                headers: window.Auth.getAuthHeaders()
            });

            return response.ok;
        } catch (error) {
            console.error('Subscription cancellation failed:', error);
            return false;
        }
    }
};

// Subscription Plans Component
const SubscriptionPlans = {
    async oninit() {
        window.AppState.plans = await Payments.getPlans();
    },

    view() {
        return m('.plans-container', [
            m('.plans-header', [
                m('h2', 'Choose Your Plan'),
                m('p', 'Select the perfect plan for your translation needs'),
                m('button.close-btn', {
                    onclick: () => window.AppState.showPlans = false
                }, '×')
            ]),

            m('.plans-grid', 
                window.AppState.plans.map(plan => 
                    m(PlanCard, { plan })
                )
            )
        ]);
    }
};

const PlanCard = {
    view(vnode) {
        const plan = vnode.attrs.plan;
        const isCurrentPlan = window.AppState.currentUser && window.AppState.currentUser.subscription_plan === plan.id;
        const isFree = plan.id === 'free';

        return m('.plan-card', { 
            class: `${isCurrentPlan ? 'current-plan' : ''} ${plan.id === 'pro' ? 'featured' : ''}`
        }, [
            plan.id === 'pro' ? m('.plan-badge', 'MOST POPULAR') : null,
            
            m('.plan-header', [
                m('h3', plan.name),
                m('.plan-price', [
                    m('span.currency', '$'),
                    m('span.amount', plan.price.toFixed(0)),
                    isFree ? null : m('span.period', '/month')
                ])
            ]),

            m('.plan-features', 
                plan.features.map(feature => 
                    m('.feature', [
                        m('.feature-icon', '✓'),
                        m('.feature-text', feature)
                    ])
                )
            ),

            m('.plan-action', [
                isCurrentPlan ? 
                    m('button.btn.btn-outline', { disabled: true }, 'Current Plan') :
                    isFree ?
                        m('button.btn.btn-outline', {
                            onclick: () => this.downgradeToPlan(plan.id)
                        }, 'Downgrade') :
                        m('button.btn.btn-primary', {
                            onclick: () => this.selectPlan(plan)
                        }, 'Choose Plan')
            ])
        ]);
    },

    selectPlan(plan) {
        window.AppState.selectedPlan = plan;
        window.AppState.showPaymentMethods = true;
        m.redraw();
    },

    async downgradeToPlan(planId) {
        if (confirm('Are you sure you want to downgrade to the free plan?')) {
            const success = await Payments.cancelSubscription();
            if (success) {
                window.Utils.showToast('Subscription cancelled successfully');
                window.Auth.user.subscription_plan = 'free';
                window.AppState.showPlans = false;
            } else {
                window.Utils.showToast('Failed to cancel subscription', 'error');
            }
            m.redraw();
        }
    }
};

// Payment Methods Component
const PaymentMethods = {
    view() {
        if (!window.AppState.selectedPlan) return null;

        const plan = window.AppState.selectedPlan;
        
        return m('.payment-methods', [
            m('.payment-header', [
                m('h3', `Subscribe to ${plan.name}`),
                m('p', `$${plan.price}/month`),
                m('button.back-btn', {
                    onclick: () => {
                        window.AppState.showPaymentMethods = false;
                        window.AppState.selectedPlan = null;
                    }
                }, '← Back to Plans')
            ]),

            m('.payment-methods-grid', [
                m('.payment-method', {
                    onclick: () => this.processPayment('stripe')
                }, [
                    m('.payment-icon', '💳'),
                    m('.payment-name', 'Credit Card'),
                    m('.payment-description', 'Visa, Mastercard, American Express')
                ]),

                m('.payment-method', {
                    onclick: () => this.processPayment('paypal')
                }, [
                    m('.payment-icon', '🅿️'),
                    m('.payment-name', 'PayPal'),
                    m('.payment-description', 'Pay with your PayPal account')
                ]),

                m('.payment-method', {
                    onclick: () => this.processPayment('revolut')
                }, [
                    m('.payment-icon', '🏦'),
                    m('.payment-name', 'Revolut'),
                    m('.payment-description', 'Pay with Revolut')
                ]),

                m('.payment-method', {
                    onclick: () => this.processPayment('klarna')
                }, [
                    m('.payment-icon', '🛍️'),
                    m('.payment-name', 'Klarna'),
                    m('.payment-description', 'Buy now, pay later')
                ])
            ])
        ]);
    },

    async processPayment(paymentMethod) {
        if (!window.Auth.isAuthenticated()) {
            window.Utils.showToast('Please login to subscribe', 'error');
            return;
        }

        window.AppState.paymentLoading = true;
        m.redraw();

        try {
            const paymentIntent = await Payments.createPaymentIntent(
                window.AppState.selectedPlan.id,
                paymentMethod
            );

            // In a real implementation, you would integrate with the actual payment providers
            // For now, simulate successful payment
            setTimeout(async () => {
                await this.completeSubscriptionUpgrade(window.AppState.selectedPlan.id);
                window.Utils.showToast(`Successfully subscribed to ${window.AppState.selectedPlan.name}!`);
                window.AppState.showPlans = false;
                window.AppState.showPaymentMethods = false;
                window.AppState.selectedPlan = null;
                window.AppState.paymentLoading = false;
                m.redraw();
            }, 2000);

        } catch (error) {
            window.Utils.showToast('Payment failed: ' + error.message, 'error');
            window.AppState.paymentLoading = false;
            m.redraw();
        }
    },

    async completeSubscriptionUpgrade(newPlan) {
        try {
            // Update subscription in backend database
            const response = await fetch('/api/subscription/update', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    ...window.Auth.getAuthHeaders()
                },
                body: JSON.stringify({
                    plan: newPlan
                })
            });

            if (response.ok) {
                // Get updated user profile from backend
                const updatedUser = await response.json();
                window.AppState.currentUser = updatedUser;
                
                // Refresh backends to show newly available ones
                window.AppState.backends = await window.API.getBackends();
                
                // Update selected backend if current one is no longer available
                if (window.AppState.selectedBackend) {
                    const availableBackends = window.Utils.getAvailableBackends(window.AppState.backends, newPlan);
                    if (!availableBackends.find(b => b.id === window.AppState.selectedBackend.id)) {
                        window.AppState.selectedBackend = availableBackends[0] || null;
                    }
                }
                
                console.log('Subscription updated successfully:', updatedUser);
            } else {
                console.error('Failed to update subscription in backend');
                // Fallback to frontend-only update for demo
                if (window.AppState.currentUser) {
                    window.AppState.currentUser.subscription_plan = newPlan;
                    const limits = { 'basic': 100, 'pro': 1000, 'enterprise': 10000 };
                    const newLimit = limits[newPlan] || 10;
                    window.AppState.currentUser.translations_remaining = newLimit - (window.AppState.currentUser.translations_used_today || 0);
                }
            }
        } catch (error) {
            console.error('Error updating subscription:', error);
            // Fallback to frontend-only update for demo
            if (window.AppState.currentUser) {
                window.AppState.currentUser.subscription_plan = newPlan;
                const limits = { 'basic': 100, 'pro': 1000, 'enterprise': 10000 };
                const newLimit = limits[newPlan] || 10;
                window.AppState.currentUser.translations_remaining = newLimit - (window.AppState.currentUser.translations_used_today || 0);
            }
        }
    }
};

// Export payment components
window.Payments = Payments;
window.PaymentComponents = {
    SubscriptionPlans,
    PaymentMethods
};