# Deployment Guide

## Architecture Overview

This AI Translation Service uses a split architecture:
- **Frontend**: Static Mithril.js application hosted on Vercel
- **Backend**: Rust API server hosted on Railway/Fly.io

## Deployment Options

### Option 1: Split Architecture (Recommended)

#### Deploy Backend (Railway)

1. **Install Railway CLI**:
   ```bash
   npm install -g @railway/cli
   ```

2. **Login and deploy**:
   ```bash
   railway login
   railway init
   railway up
   ```

3. **Set environment variables** in Railway dashboard:
   ```
   OPENAI_API_KEY=your-key
   GEMINI_API_KEY=your-key
   DEEPL_API_KEY=your-key
   STRIPE_SECRET_KEY=your-key
   JWT_SECRET=your-secret
   ```

4. **Get your backend URL** (e.g., `https://your-app.railway.app`)

#### Deploy Frontend (Vercel)

1. **Update config** in `static/js/config.js`:
   ```javascript
   API_BASE_URL: 'https://your-backend-url.railway.app'
   ```

2. **Install Vercel CLI**:
   ```bash
   npm install -g vercel
   ```

3. **Deploy**:
   ```bash
   vercel --prod
   ```

### Option 2: Full Backend Deployment (Fly.io)

1. **Install Fly CLI**:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Login and deploy**:
   ```bash
   fly auth login
   fly launch
   fly deploy
   ```

3. **Set secrets**:
   ```bash
   fly secrets set OPENAI_API_KEY=your-key
   fly secrets set GEMINI_API_KEY=your-key
   ```

## Environment Variables

Required environment variables:

```bash
# AI API Keys
OPENAI_API_KEY=sk-...
GEMINI_API_KEY=...
DEEPL_API_KEY=...
MISTRAL_API_KEY=...
ANTHROPIC_API_KEY=...

# Payment
STRIPE_SECRET_KEY=sk_...
PAYPAL_CLIENT_ID=...
KLARNA_API_KEY=...

# Auth
JWT_SECRET=your-secret-key

# Database
DATABASE_PATH=./translation_service.db
```

## Files Structure for Deployment

```
├── static/           # Frontend files (for Vercel)
├── src/             # Backend Rust code
├── Cargo.toml       # Rust dependencies
├── Dockerfile       # Container build
├── vercel.json      # Vercel config
├── railway.toml     # Railway config
├── fly.toml         # Fly.io config
└── package.json     # NPM config
```

## Post-Deployment

1. Test API endpoints: `https://your-backend.com/api/backends`
2. Test frontend: `https://your-frontend.vercel.app`
3. Update CORS settings if needed
4. Configure custom domains
5. Set up monitoring and logging

## Troubleshooting

- **CORS Issues**: Check `vercel.json` headers configuration
- **API Connection**: Verify `config.js` has correct backend URL
- **Database**: Ensure persistent storage is configured
- **Environment Variables**: Double-check all required keys are set