# FeedTap

A modern desktop RSS feed reader built with Tauri and Vue that provides real-time feed updates with a clean, professional interface.

## What is FeedTap?

FeedTap is a desktop application designed for staying up-to-date with your favorite news sources and blogs. It aggregates RSS and Atom feeds from multiple sources and presents them in a unified, real-time stream. Perfect for cryptocurrency news, tech blogs, journalism, and any RSS-compatible content.

**Key Benefits:**
- **Real-time updates** - See new articles as they're published
- **Unified interface** - All your feeds in one place
- **Desktop native** - Fast, responsive, and works offline
- **Privacy-focused** - Everything runs locally on your machine
- **Customizable** - Add any RSS/Atom feed and configure update intervals

## Features

### Core Functionality
- **Live feed aggregation** from multiple RSS/Atom sources
- **Real-time WebSocket updates** for instant article notifications
- **Smart deduplication** prevents duplicate articles
- **Full-text preview** for articles (click "Show preview" on any item)
- **Configurable polling intervals** per source
- **Search across all articles** by title and content
- **Source filtering** to focus on specific publications

### User Interface
- **Modern sidebar navigation** with collapsible panels
- **Card-based article layout** with rich metadata
- **Compact and comfortable view modes**
- **Live activity logs** showing system status
- **Dark and light themes** with system preference detection
- **Smooth animations** and responsive design
- **Professional typography** and consistent spacing

### Technical Features
- **Desktop application** built with Tauri (Rust + Web technologies)
- **Vue 3 frontend** with modern JavaScript
- **Local WebSocket server** for real-time communication
- **Persistent storage** for source configurations
- **Automatic background updates** with efficient polling
- **Cross-platform** support (Windows, macOS, Linux)

## Quick Start

### Prerequisites
- Node.js (v18 or higher)
- Rust toolchain (latest stable)
- Windows, macOS, or Linux desktop environment

### Installation
```bash
# Clone the repository
git clone <repository-url>
cd feedtap

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

The application will start with:
- Frontend development server at http://localhost:1420
- Backend Tauri app with WebSocket server on port 8787
- Hot reload for both frontend and backend changes

### Building for Production
```bash
npm run tauri build
```

## Usage Guide

### Getting Started
1. **Launch FeedTap** - The app starts with several cryptocurrency news sources pre-configured
2. **Browse articles** - New articles appear automatically in the main feed
3. **Preview content** - Click "Show preview" under any article to see the full text
4. **Add sources** - Click "Manage Sources" to add your own RSS feeds
5. **Filter by source** - Click any source name in the sidebar to filter articles

### Managing Feed Sources
- Click **"Manage Sources"** in the sidebar or header
- Add new RSS/Atom feeds with custom names and update intervals
- Remove sources you no longer need
- Adjust polling frequency per source (from 1 minute to several hours)

### Navigation and Views
- **Sidebar navigation** - Switch between feed view and source management
- **Search functionality** - Use the search bar to find specific articles
- **Compact mode** - Toggle for a denser article list
- **Activity logs** - View real-time system activity and feed updates

## Architecture Overview

FeedTap is built with modern web technologies wrapped in a native desktop application:

### Frontend (Vue 3 + Vite)
- **Vue 3 Composition API** for reactive user interface
- **Modern CSS** with custom properties and responsive design
- **WebSocket client** for real-time feed updates
- **Component-based architecture** for maintainable code

### Backend (Rust + Tauri)
- **Tauri framework** for native desktop integration
- **RSS/Atom feed parsing** with multiple source support
- **WebSocket server** for real-time communication
- **Background polling** with configurable intervals
- **Local data storage** for source configurations

### Key Components
- **Feed Aggregator** - Polls RSS sources and deduplicates articles
- **WebSocket Server** - Broadcasts new articles to the frontend
- **Article Preview** - Fetches and sanitizes full article content
- **Source Manager** - Handles RSS feed configuration and storage

## WebSocket API

The application runs a local WebSocket server for real-time updates:

**Connection**: `ws://127.0.0.1:8787/stream`

### Message Types
```json
// Server hello message
{ "type": "hello", "payload": { "server_version": "0.1.0" } }

// New article
{ 
  "type": "item", 
  "payload": {
    "id": "unique-hash",
    "title": "Article Title",
    "link": "https://example.com/article",
    "summary": "Article description...",
    "published_at": "2024-01-01T12:00:00Z",
    "source": { "id": "source-id", "name": "Source Name" }
  }
}

// Activity log
{ 
  "type": "log", 
  "payload": {
    "level": "info",
    "message": "Fetched 5 new articles from TechCrunch",
    "timestamp": 1640995200000
  }
}

// Heartbeat (every 30 seconds)
{ "type": "heartbeat" }
```

## Preview API

FeedTap includes an HTTP endpoint for fetching article previews:

**Endpoint**: `GET http://127.0.0.1:8787/preview?url=<article-url>`

**Response**:
```json
{
  "url": "https://example.com/article",
  "content_preview": "Sanitized article content..."
}
```

This endpoint:
- Fetches the full article content
- Extracts the main text content
- Sanitizes HTML and removes scripts
- Caches results for performance
- Handles CryptoPanic and other aggregator sites

## Extending FeedTap

### Adding Custom Feed Sources
1. Open the application and click "Manage Sources"
2. Click "Add Source" and enter:
   - **Name**: Display name for the feed
   - **URL**: RSS or Atom feed URL
   - **Interval**: How often to check for updates (in milliseconds)
3. Save and the feed will start polling automatically

### Adding New Feed Parsers
For developers wanting to add custom feed types:

1. Create a new parser in `src-tauri/src/taps/`
2. Implement the `Tap` trait:
```rust
#[async_trait::async_trait]
impl Tap for CustomTap {
    async fn fetch(&self, src: &SourceConfig) -> Result<Vec<NewsItem>> {
        // Custom parsing logic here
    }
}
```
3. Register in `src-tauri/src/taps/mod.rs`
4. Update the UI dropdown in `SourceManager.vue`

### Customizing the Interface
The design uses CSS custom properties for easy theming:

```css
:root {
  --accent-primary: #007aff;     /* Primary accent color */
  --bg-primary: #fafafa;         /* Main background */
  --text-primary: #1d1d1f;       /* Primary text */
  --border-light: #e5e5e7;       /* Light borders */
  /* See src/assets/theme.css for all variables */
}
```

## Default Sources

FeedTap comes pre-configured with several cryptocurrency news sources:

- **CryptoPanic** - Aggregated crypto news
- **CoinDesk** - Bitcoin and blockchain news  
- **Cointelegraph** - Cryptocurrency news and analysis
- **Decrypt** - Web3 and crypto journalism
- **The Block** - Institutional crypto news
- **Bitcoin Magazine** - Bitcoin-focused content
- **And several others**

You can remove these and add your own sources for any topic (tech news, blogs, sports, etc.).

## Troubleshooting

### Common Issues

**Application won't start:**
- Ensure Node.js and Rust toolchain are installed
- Try `npm install` to reinstall dependencies
- Check that port 8787 is not already in use

**Feeds not updating:**
- Check your internet connection
- Verify RSS/Atom URLs are valid and accessible
- Some feeds may have rate limiting or access restrictions

**Preview not working:**
- Some websites block automated content fetching
- Preview works best with standard article formats
- CryptoPanic links are specially handled to extract source content

**Performance issues:**
- Reduce polling frequency for feeds you check less often
- Use compact view mode for better performance with many articles
- Clear browser cache if using development mode

## License

MIT License - see LICENSE file for details.
