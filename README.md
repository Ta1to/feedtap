# FeedTap 2.0 🚀

A modern Tauri + Vue desktop application that provides a live RSS feed reader with real-time updates via WebSocket streaming.

## ✨ Features

### 🎨 **Modern UI/UX Design**
- **Sleek sidebar navigation** with collapsible panels
- **Smart source filtering** with real-time counters  
- **Modern card-based feed layout** with smooth animations
- **Professional activity logs panel** with color-coded events
- **Responsive design** that works on all screen sizes
- **Dark/Light theme** with system preference detection
- **Glassmorphism effects** with backdrop blur
- **Micro-interactions** for enhanced user experience

### ⚡ **Core Functionality**
- **Real-time feed updates** via local WebSocket (ws://127.0.0.1:8787/stream)
- **Configurable RSS/Atom sources** with custom intervals
- **Smart deduplication** of feed items
- **Search functionality** across titles and summaries
- **Compact/comfortable view modes**
- **Live connection status** indicator
- **Fresh item highlighting** for new articles

### 🛠️ **Technical Excellence**
- Built with **Tauri 2.0** + **Vue 3** + **Vite**
- **Modern CSS** with CSS custom properties and modern layout
- **TypeScript-ready** architecture
- **Accessibility-focused** with proper ARIA labels and keyboard navigation
- **Performance-optimized** with efficient rendering and memory management

## 🎯 User Experience Improvements

### Before (v1.0):
- Basic functional interface
- Cluttered header with too many controls
- Simple list layout without visual hierarchy
- Basic logs display
- No source organization
- Limited visual feedback

### After (v2.0):
- **Sidebar-first navigation** similar to modern apps (Discord, Notion, Linear)
- **Clean header** focused on search and essential actions
- **Card-based feed** with rich metadata and visual cues
- **Professional logs panel** with categorized, color-coded entries
- **Smart source management** with grouping and filtering
- **Rich visual feedback** with animations and state indicators
- **Empty states** with helpful guidance
- **Loading states** and error handling

## 🚀 Quick Start

### Prerequisites
- **Node.js** (v18 or higher)
- **Rust toolchain** (latest stable)
- **Windows/macOS/Linux** desktop environment

### Development Setup
```powershell
# Clone and install dependencies
git clone <repository-url>
cd feedtap
npm install

# Start development server with hot-reload
npm run tauri dev
```

The application will start with:
- **Frontend**: Vite dev server at http://localhost:1420
- **Backend**: Tauri app with WebSocket server on port 8787
- **Hot reload**: Both frontend and backend changes trigger automatic reloads

### Production Build
```powershell
npm run tauri build
```

## 🎮 Usage Guide

### 📱 **Navigation**
- **Sidebar**: Navigate between Feed, Sources, and Settings
- **Collapse/Expand**: Click the toggle button in the sidebar header
- **Source Filtering**: Click on any source in the sidebar to filter articles
- **Search**: Use the global search in the header (⌘K shortcut)

### 📰 **Managing Feeds**
1. Click **"Manage Sources"** in sidebar or header actions
2. Click **"Add Source"** to configure a new RSS feed
3. Enter source details (name, URL, update interval)
4. Sources are automatically saved and begin polling immediately

### 🔧 **View Options**
- **Compact Mode**: Dense list view for more articles per screen
- **Comfortable Mode**: Spacious cards with full summaries
- **Activity Logs**: Toggle to view real-time system activity
- **Auto-refresh**: Sources update automatically based on their intervals

## 📡 WebSocket API

The application exposes a local WebSocket server for real-time feed updates:

**Connection**: `ws://127.0.0.1:8787/stream`

### Message Types
```json
// Server greeting
{ "type": "hello", "payload": { "server_version": "0.1.0" } }

// New feed item
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

// Server heartbeat (every 15s)
{ "type": "heartbeat" }

// Error notification
{ "type": "error", "payload": { "message": "Error description" } }
```

## 🛠️ Development Architecture

### Frontend (Vue 3 + Vite)
- **`src/App.vue`**: Main application shell with layout management
- **`src/components/`**: Modular UI components
  - `Sidebar.vue`: Navigation and source filtering
  - `HeaderBar.vue`: Search and global actions
  - `FeedList.vue`: Article display with animations
  - `LogPanel.vue`: Real-time activity monitoring
  - `SourceManager.vue`: Feed configuration modal
- **`src/assets/theme.css`**: Modern design system with CSS custom properties
- **`src/lib/websocket.js`**: WebSocket client with reconnection logic

### Backend (Rust + Tauri)
- **`src-tauri/src/lib.rs`**: Application entry point and Tauri commands
- **`src-tauri/src/aggregator.rs`**: RSS feed polling and deduplication engine
- **`src-tauri/src/ws.rs`**: WebSocket server for real-time updates
- **`src-tauri/src/storage.rs`**: Persistent source configuration
- **`src-tauri/src/taps/`**: Pluggable feed parsers (RSS/Atom)
- **`src-tauri/src/types.rs`**: Shared data structures

### Design System
- **CSS Custom Properties**: Consistent theming with light/dark mode
- **Typography Scale**: Harmonious font sizing and spacing
- **Color Palette**: Carefully selected colors for accessibility
- **Component Library**: Reusable UI elements (buttons, cards, badges)
- **Animation System**: Smooth transitions and micro-interactions
- **Responsive Breakpoints**: Mobile-first responsive design

## 🔌 Extending FeedTap

### Adding New Feed Types
1. Create a new parser in `src-tauri/src/taps/`
2. Implement the `Tap` trait with custom fetch logic
3. Register the new type in `make_tap()` function
4. Update the UI dropdown in `SourceManager.vue`

### Custom Styling
The design system uses CSS custom properties for easy theming:
```css
:root {
  --accent-primary: #007aff;    /* Change primary color */
  --bg-primary: #fafafa;        /* Adjust background */
  --text-primary: #1d1d1f;      /* Modify text color */
  /* ... see src/assets/theme.css for all variables */
}
```

## 📝 License
MIT License - see LICENSE file for details
