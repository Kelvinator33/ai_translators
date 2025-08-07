// Configuration for different environments
window.Config = {
  // Development
  API_BASE_URL: window.location.hostname === 'localhost' 
    ? 'http://localhost:3000'
    : 'https://your-backend-url.railway.app', // Replace with your backend URL
    
  // Feature flags
  FEATURES: {
    PAYMENT_ENABLED: true,
    OCR_ENABLED: true,
    EXPORT_ENABLED: true
  }
};