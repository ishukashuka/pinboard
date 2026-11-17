# Pinboard

> **A Keyboard-First Infinite Scratchboard Desktop App**
> PureRef's spatial freedom + Vim-style text editing + seamless clipboard integration + instant search-to-focus + persistent SQLite storage.

Built with **Tauri 2**, **Rust**, **Vue 3**, **TypeScript**, **SQLite**, and **CodeMirror 6 with Vim mode**.

---

## Features

- **Infinite Canvas**: Unlimited 2D canvas with smooth panning (`Space + Drag` or Middle Mouse Drag) and cursor-centered zooming (`Mouse Wheel`).
- **Clipboard-First Flow**:
  - Paste plain text (`Ctrl+V`) to spawn a movable, resizable Markdown text block.
  - Paste images (`Ctrl+V`) or drag-and-drop image files to spawn resizable image objects (with aspect-ratio preservation).
  - Paste URLs to spawn interactive URL cards with instant external browser launch.
- **True System Clipboard Interoperability**:
  - Copy text back to the clipboard with original formatting.
  - Copy images back as actual image binary data (supporting Wayland via `wl-copy` and X11/XWayland via `xclip` and `arboard`).
- **CodeMirror 6 + Vim Editing**:
  - Full Normal, Insert, and Visual modes.
  - Markdown syntax highlighting.
  - Familiar Vim motions and buffer search (`/`).
  - Custom Vim ex commands (`:w`, `:q`, `:wq`).
  - Press `Esc` in Normal mode to exit back to canvas mode.
  - Full keyboard event isolation between canvas mode and text mode.
- **Search-to-Focus (`/`)**:
  - Press `/` anywhere in canvas mode to open the centered search palette.
  - Instant search across text content, titles, image filenames, and URLs.
  - Navigate results with `↑`/`↓` or `Ctrl+N`/`Ctrl+P`.
  - Press `Enter` to close search, smoothly pan/zoom camera to the object, flash a focus highlight ring, and select it.
  - Press `Backspace` (with no object selected) or `Ctrl+O` to jump back to your previous camera position.
- **Robust Local Persistence**:
  - Automated SQLite storage (`pinboard.db`) preserving camera position, objects, dimensions, z-order, and metadata.
  - Managed local asset directory for images (`assets/`).
  - Debounced autosave that ensures nothing is lost.
  - Undo / Redo support (`Ctrl+Z`, `Ctrl+Shift+Z` / `Ctrl+Y`).
- **Minimalist Aesthetic**:
  - Dark neutral interface (`#121214`), subtle canvas dot grid, smooth highlight animations, and a sleek floating HUD.

---

## Keyboard Shortcuts

### Canvas Mode
| Shortcut | Action |
|---|---|
| `/` | Open Search-to-Focus palette |
| `i` or `Enter` | Edit selected text object (enters Text Mode) |
| `n` | Create new text note at canvas center and start editing |
| `Ctrl+V` | Paste text or image from system clipboard at cursor |
| `Ctrl+C` | Copy selected object's content (text or image) to system clipboard |
| `Delete` / `Backspace` | Delete selected objects |
| `h` / `j` / `k` / `l` or Arrows | Nudge selected objects (hold `Shift` for larger steps) |
| `Space + Drag` | Pan canvas |
| Middle Mouse Drag | Pan canvas |
| Mouse Wheel | Zoom centered at cursor |
| `Ctrl+0` | Fit all objects into view |
| `Ctrl+1` | Reset camera (0, 0, 100%) |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` / `Ctrl+Y` | Redo |
| `]` / `[` | Bring forward / Send backward (Z-order) |
| `Backspace` (empty selection) or `Ctrl+O` | Restore camera position prior to search jump |
| `Esc` | Deselect all |

### Text Mode (CodeMirror Vim)
| Action | Shortcut |
|---|---|
| Normal Mode Motions | `h`, `j`, `k`, `l`, `w`, `b`, `0`, `$`, `gg`, `G`, etc. |
| Enter Insert Mode | `i`, `a`, `o`, `I`, `A`, `O` |
| Enter Visual Mode | `v`, `V`, `Ctrl+V` |
| Search within text | `/` |
| Save and Exit | `:w`, `:q`, `:wq` |
| Exit to Canvas Mode | `Esc` (while in Normal mode) |

---

## Hyprland Integration

To summon or toggle Pinboard with a global compositor keybinding in Hyprland, add the following to `~/.config/hypr/hyprland.conf`:

```ini
# Toggle Pinboard with Super+P
bind = $mainMod, P, exec, /data/MyShit/projects/pinboard/src-tauri/target/release/pinboard

# Pinboard window rules (optional floating / centered scratchpad behavior)
windowrulev2 = float, class:^(com.pinboard.desktop)$
windowrulev2 = size 1280 800, class:^(com.pinboard.desktop)$
windowrulev2 = center, class:^(com.pinboard.desktop)$
```

---

## Installation & Distribution

Pinboard provides standalone, self-contained packages for Linux and Windows.

### Linux (x86_64)

#### Portable AppImage (Recommended for Arch, Hyprland, Fedora, Ubuntu)
Download `Pinboard_0.2.4_amd64.AppImage`, make it executable, and run:
```bash
chmod +x Pinboard_0.2.4_amd64.AppImage
./Pinboard_0.2.4_amd64.AppImage
```
The AppImage bundles its own WebKitGTK and runtime dependencies without requiring external library installations.

#### Debian / Ubuntu (.deb)
```bash
sudo dpkg -i pinboard_0.2.4_amd64.deb
# If missing dependencies:
sudo apt-get install -f
```

### Windows (x64)

#### NSIS Installer
Run `Pinboard_0.2.4_x64-setup.exe`. This performs a clean, per-user installation without requiring administrator privileges. If Microsoft Edge WebView2 runtime is not detected on your system, the installer will automatically download and configure the official evergreen bootstrapper.

#### Portable Executable
Run `Pinboard_0.2.4_x64-portable.exe` directly from any folder or USB drive.

---

## Running and Building from Source

### Prerequisites
- **Node.js**: v20 or newer
- **Rust**: stable toolchain (`rustup default stable`)
- **Linux system dependencies** (Ubuntu/Debian):
  ```bash
  sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev squashfs-tools
  ```

### Development Mode
```bash
npm run tauri dev
```

### Production Build
```bash
# Build optimized release binary
npm run tauri build -- --no-bundle

# Build Linux AppImage and .deb bundles
APPIMAGE_EXTRACT_AND_RUN=1 npx tauri build --bundles appimage,deb

# Build Windows NSIS installer (on Windows or CI)
npx tauri build --bundles nsis
```
Built binaries and installers are located in:
`src-tauri/target/release/bundle/`

### Automated Tests
Run unit and integration tests:
```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

---

## CI/CD Release Automation

Pinboard includes a GitHub Actions workflow (`.github/workflows/release.yml`) for automated cross-platform builds:
- Triggers on tag pushes (`v*`) and manual trigger (`workflow_dispatch`).
- Compiles native Linux AppImage and Debian packages on `ubuntu-22.04`.
- Compiles native Windows NSIS installer and standalone portable `.exe` on `windows-latest`.
- Attaches all release packages to workflow artifacts and drafts a new GitHub Release.

---

## Acknowledgments & Inspiration

Pinboard is proudly inspired by:
- **PureRef** — for spatial reference management, effortless image arranging, and distraction-free framing.
- **Excalidraw** — for fluid canvas navigation, tactile feedback, and intuitive connector routing.
