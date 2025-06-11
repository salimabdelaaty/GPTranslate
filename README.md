# ![GPTranslate](img/logo_app.svg) GPTranslate

A fast, modern **AI powered** desktop translation application built with Rust and Tauri that provides instant translation between multiple languages with global hotkey support, system tray integration, and a beautiful user interface. Using either OpenAI or Azure OpenAI, it delivers high-quality translations with minimal latency and cost.

**Note:** This project has been tested on Windows 11 only. It may work on other platforms, but is not guaranteed. If you would like to help with testing on macOS or Linux, please open an issue.

## 📸 Screenshots

### Main Interface

**Light Theme**
![Light Theme](img/main_light.png)

**Dark Theme**
![Dark Theme](img/main_dark.png)

### Features Overview

**Translation in Action**
![Translation](img/translation.png)

**History Management**
![History](img/history.png)

**Settings Panel**
![Settings](img/settings.png)

## ✨ Features

### Core Translation

- **Intelligent Language Detection**: Automatically detects the source language from most supported languages
- **Multi-language Translation**: Seamless translation between dozens of languages with configurable target language
- **Real-time Translation**: Debounced translation as you type (500ms delay)
- **Multiple AI Providers**: Support for both OpenAI and Azure OpenAI APIs
- **Custom Prompts**: Configurable translation prompts with variable support
- **Alternative Target Language**: Fallback when source equals target language

### User Interface

- **Modern UI**: Clean, responsive interface with two-panel layout
- **Theme Support**: Auto, light, and dark themes with system detection
- **Keyboard Shortcuts**:
  - **Configurable Global Hotkey** (default `Ctrl+Alt+C`):
    - When app is **not focused**: Clipboard capture and translation
    - When app **is focused**: Reset detected language
  - `Ctrl+C`: Copy translated text when app is focused
- **Responsive Design**: Scales beautifully across different window sizes

### System Integration

- **System Tray**: Runs quietly in background with context menu
- **Global Hotkey**: Capture clipboard content from anywhere
- **Auto-start**: Optional startup with Windows
- **Single Instance**: Prevents multiple app instances
- **Clipboard Integration**: Seamless read/write clipboard operations

### Data Management

- **Translation History**: Persistent history
- **Configuration**: JSON-based settings in `~/.gptranslate/config.json`
- **Settings GUI**: User-friendly configuration interface

### Developer Features

- **Hot Reload**: Fast development with Vite and Tauri dev server
- **Error Handling**: Comprehensive error handling throughout
- **Logging**: Detailed console logging for debugging
- **Duplicate Prevention**: Smart request deduplication
- **Performance**: Optimized with debouncing and efficient state management

## 💰 Cost Analysis

GPTranslate is designed to be extremely cost-effective with the recommended `gpt-4.1-nano` model:

### Translation Costs (per request)

| Model                          | Input Cost      | Output Cost      | Total per Paragraph\* |
| ------------------------------ | --------------- | ---------------- | --------------------- |
| **gpt-4.1-nano** (recommended) | $0.10/1M tokens | $0.40/1M tokens  | **~$0.00005**         |
| gpt-4.1-mini                   | $0.40/1M tokens | $1.60/1M tokens  | ~$0.00019             |
| gpt-4o-mini                    | $0.15/1M tokens | $0.60/1M tokens  | ~$0.000071            |
| gpt-4.1                        | $2.00/1M tokens | $8.00/1M tokens  | ~$0.00095             |
| gpt-4o                         | $2.50/1M tokens | $10.00/1M tokens | ~$0.00119             |

\*Based on ~100 words (130 tokens input + output)

### Real-World Usage Examples

- **Single paragraph**: Less than $0.0001 with gpt-4.1-nano
- **100 translations/day**: ~$0.005 ($1.50/month)
- **1000 translations/day**: ~$0.05 ($15/month)

The `gpt-4.1-nano` model provides excellent translation quality at virtually no cost for typical usage patterns, making GPTranslate highly economical for both personal and professional use.

## ⬇️ Download Windows Installer

You can download the latest Windows installer from the [Releases](https://github.com/philberndt/GPTranslate/releases/tag/Stable) page.

## 🚀 Quick Start

### Prerequisites

- **Rust**: 1.87+ with stable toolchain
- **Node.js**: 22+ with npm
- **Visual Studio Build Tools**: 2022 (Windows)
- **WebView2**: Runtime (usually pre-installed on Windows 11)

### Installation

1. **Clone the repository**:

   ```bash
   git clone https://github.com/philberndt/gptranslate.git
   cd gptranslate
   ```

2. **Install dependencies**:

   ```bash
   npm install
   ```

3. **Set up your API configuration**:

   - Run the app once to create the config file
   - Add your OpenAI or Azure OpenAI credentials in Settings

4. **Run in development**:

   ```bash
   npm run tauri dev
   ```

## 🔧 Configuration

Settings are automatically created in `~/.gptranslate/config.json`:

```json
{
  "provider": "openai",
  "api_key": "your-api-key-here",
  "model": "gpt-4.1-nano",
  "azure_endpoint": "https://your-resource.openai.azure.com/",
  "azure_deployment_name": "gpt-4.1-nano",
  "target_language": "Norwegian",
  "alternative_target_language": "English",
  "custom_prompt": "Translate accurately while preserving meaning and tone...",
  "theme": "auto",
  "hotkey": "Crtl+Alt+C",
  "auto_start": true,
  "close_to_tray": true
}
```

⚠️ **Note:** The API Keys and Endpoint are stored in plain text. Ensure your system is secure.

### API Setup

#### OpenAI

1. Get an API key from [OpenAI Platform](https://platform.openai.com/api-keys)
2. In Settings, select "OpenAI" as provider
3. Enter your API key and preferred model

#### Azure OpenAI

1. Create an Azure OpenAI resource
2. Deploy a model (recommend `gpt-4.1-nano`), as it is super cheap and fast.
3. In Settings, select "Azure OpenAI" as provider
4. Enter your endpoint and API key

## 🎯 Usage

### Basic Translation

1. **Launch**: Click the system tray icon or use the global hotkey
2. **Type or Paste**: Enter text in the left panel
3. **Auto-translate**: Translation appears in the right panel
4. **Copy**: Click Copy button or press `Ctrl+C` when app is focused

### Global Hotkey Workflow

1. **Copy text anywhere** with `Ctrl+C`
2. **Press global hotkey** (`Ctrl+Alt+C`)
3. **View translation** in app window
4. **Copy result** automatically to clipboard

### Keyboard Shortcuts

- **Configurable Global Hotkey** (default `Ctrl+Alt+C`):
  - **Always**: Reset detected language first
  - When app is **not focused**: Also capture clipboard and translate
  - When app **is focused**: Only reset detected language
- `Ctrl+C`: Copy translated text (when app focused)
- `Esc`: Close settings/history dialogs

## 🔨 Development

### Build Commands

```bash
# Development server with hot reload
npm run tauri dev

# Build for production
npm run tauri build

# Run frontend only
npm run dev

# Type check
npm run check

# Lint code
npm run lint
```

### Project Structure

```text
src/
├── lib/                    # Svelte components
│   ├── AppIcon.svelte     # Application icon
│   ├── Settings.svelte    # Settings dialog
│   └── History.svelte     # Translation history
├── routes/
│   ├── +layout.ts         # Layout configuration
│   └── +page.svelte       # Main application
├── app.html               # HTML template
src-tauri/
├── src/
│   ├── main.rs           # Application entry
│   ├── lib.rs            # Core library
│   ├── config.rs         # Configuration management
│   ├── translation.rs    # Translation service
│   ├── history.rs        # History management
│   └── tray.rs           # System tray
├── icons/                # Application icons
└── capabilities/         # Tauri permissions
```

## 🎨 Customization

### Custom Translation Prompts

Use variables in your prompts:

- `{detected_language}`: Source language
- `{target_language}`: Target language

Example:

```text
Translate from {detected_language} to {target_language}.
Preserve the original tone and cultural context.
```

### Hotkey Customization

Modify the global hotkey in Settings. Supports:

- Modifiers: `Ctrl`, `Alt`, `Shift`, `Cmd` (macOS)
- Keys: Letters, numbers, function keys
- Combinations: `Ctrl+Alt+T`, `Shift+F1`, etc.

## 📦 Building for Distribution

### Windows Installer

```bash
# Build MSI installer
npm run tauri build

# Build NSIS installer
npm run tauri build -- --target nsis
```

Installers are created in `src-tauri/target/release/bundle/`

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes and test thoroughly
4. Commit with clear messages: `git commit -m "Add: feature description"`
5. Push and create a Pull Request

### Code Style

- **Rust**: Use `cargo fmt` and `cargo clippy`
- **TypeScript/Svelte**: Use `npm run lint`
- **Commits**: Follow conventional commits format

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Icons**: [Bootstrap Icons](https://icons.getbootstrap.com/)
- **Translation**: Powered by OpenAI and Azure OpenAI
- **Framework**: Built with [Tauri](https://tauri.app/) and [SvelteKit](https://kit.svelte.dev/)

## 📧 Support

- **Issues**: [GitHub Issues](https://github.com/philberndt/gptranslate/issues)

---

**Technology Stack**: Tauri 2.5+ • Rust • SvelteKit • Vite • TypeScript
