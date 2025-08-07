# AI Translation Service 🌐

A modern web application for translating text and documents using multiple AI backends, built with Rust (Actix-web) and Mithril.js.

## Features

- **Multi-backend AI Support**: Switch between different AI services
  - 🦙 **Ollama** (local models)
  - 🔗 **llama.cpp** (local inference)  
  - 🤖 **OpenAI GPT** (API)
  - 🧠 **Google Gemini** (API)
  - 🌍 **DeepL** (translation service)

- **Document Translation**: Upload and translate images/documents
  - 📄 Support for JPG, PNG, PDF, DOCX
  - ✅ **Real OCR text extraction** (Tesseract v5.5 integrated)
  - 📋 Driver's license translation example

- **Modern UI**: Clean, responsive dashboard
  - 🎨 Gradient design with dark/light themes
  - 📱 Mobile-friendly interface
  - 🔧 Real-time backend configuration

## Quick Start

### 1. Setup Environment

Copy the `.env` file and configure your API keys:

```bash
cp .env .env.local
# Edit .env.local with your API keys
```

### 2. Install Dependencies

**For Tesseract OCR (optional but recommended):**
```bash
# Windows (with Chocolatey) - Run as Administrator
choco install tesseract

# macOS
brew install tesseract

# Ubuntu/Debian
sudo apt-get install tesseract-ocr libtesseract-dev
```

**✅ Tesseract is Now Fully Integrated!**
- Real OCR functionality is enabled and working
- Uses system Tesseract v5.5 command-line interface
- Automatic fallback if OCR fails
- Supports multiple languages via TESSERACT_DEFAULT_LANG env var

### 3. Run the Application

```bash
cargo run
```

Visit: **http://localhost:3000**

## Configuration

### Environment Variables

```env
# OpenAI Configuration
OPENAI_API_KEY=your_openai_api_key_here
OPENAI_MODEL=gpt-3.5-turbo
OPENAI_ENDPOINT=https://api.openai.com/v1

# Google Gemini Configuration
GEMINI_API_KEY=your_gemini_api_key_here
GEMINI_MODEL=gemini-pro

# DeepL Configuration
DEEPL_API_KEY=your_deepl_api_key_here
DEEPL_ENDPOINT=https://api-free.deepl.com/v2/translate

# Ollama Configuration (local)
OLLAMA_ENDPOINT=http://localhost:11434
OLLAMA_MODEL=llama2

# llama.cpp Configuration (local)
LLAMA_CPP_ENDPOINT=http://localhost:8080
LLAMA_CPP_MODEL_PATH=/path/to/your/model.bin

# Server Configuration
SERVER_PORT=3000
SERVER_HOST=127.0.0.1
DATABASE_PATH=translation_service.db

# Tesseract Configuration
TESSERACT_DATA_PATH=
TESSERACT_DEFAULT_LANG=eng
```

### Backend Setup

1. **Ollama**: Install from [ollama.ai](https://ollama.ai)
   ```bash
   ollama pull llama2
   ollama serve
   ```

2. **llama.cpp**: Build and run server mode
   ```bash
   ./server -m model.bin -c 2048 --host 0.0.0.0 --port 8080
   ```

## Architecture

### Backend (Rust)
- **Actix-web**: HTTP server and routing
- **Sled**: Embedded database for configurations
- **Tesseract**: OCR text extraction
- **Reqwest**: HTTP client for AI APIs

### Frontend (Mithril.js)
Modular JavaScript architecture:
- `state.js`: Application state management
- `api.js`: API communication layer
- `utils.js`: Utility functions and constants
- `components.js`: Reusable UI components
- `translation.js`: Translation interface logic
- `dashboard.js`: Main dashboard component

## API Endpoints

- `GET /` - Dashboard UI
- `GET /api/backends` - List all AI backends
- `POST /api/backends` - Save backend configuration
- `POST /api/translate` - Translate text
- `POST /api/upload` - Upload and translate document

## Translation Strategy

### For Images (Driver's License Example):
1. **OCR Extraction**: Tesseract v5.5 extracts text from uploaded image
2. **Language Detection**: Auto-detect source language or use specified language
3. **AI Translation**: Use selected backend (GPT, Gemini, DeepL, etc.) for translation
4. **Result Display**: Show original extracted text and translated text

### OCR Implementation:
- **System Command Approach**: Uses `tesseract` binary directly for maximum compatibility
- **Smart Fallback**: If OCR fails, provides helpful placeholder text
- **Multi-language Support**: Supports all Tesseract language packs (eng, deu, fra, etc.)
- **High Performance**: Optimized with PSM 6 (single uniform block of text) for documents

### Best Practices:
- **DeepL**: Best translation quality for European languages
- **GPT-4/Gemini**: Best for context-aware translation
- **Ollama**: Privacy-focused local translation
- **Tesseract**: Reliable OCR for printed text

## Development

### Project Structure
```
src/
├── main.rs          # Server setup and routing
├── database.rs      # Sled database operations
├── ai_backends.rs   # AI service integrations
├── translation.rs   # Translation logic
└── ocr.rs          # Image text extraction

static/
├── index.html      # Main HTML template
├── style.css       # UI styles
├── app.js          # Legacy monolithic JS
└── js/            # Modular JavaScript
    ├── state.js
    ├── api.js
    ├── utils.js
    ├── components.js
    ├── translation.js
    └── dashboard.js
```

### Building
```bash
# Development
cargo run

# Release
cargo build --release

# Run tests
cargo test
```

### Adding New AI Backends

1. Add configuration in `database.rs`
2. Implement client in `ai_backends.rs`
3. Update frontend backend list

## Troubleshooting

### Tesseract Issues
- **Windows**: Ensure Tesseract is in PATH
- **macOS/Linux**: Install language data packages
- **Fallback**: App works without Tesseract (placeholder text)

### API Key Issues
- Check `.env` file configuration
- Verify API key validity and quotas
- Enable backends in UI settings

### Port Conflicts
- Change `SERVER_PORT` in `.env`
- Update bind address in `main.rs`

## License

MIT License - see LICENSE file for details

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

---

**Built with ❤️ using Rust and Mithril.js**