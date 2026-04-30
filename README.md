# ⚡ TeamFlow

Asana-Alternative for Teams — Self-Hosted, Free Forever

---

## What is TeamFlow?

TeamFlow is a **desktop task tracker** like Asana, but:
- **No hosting** — runs locally on your device
- **No subscriptions** — 100% free
- **OneDrive sync** — your cloud storage acts as the shared database

Works on: **macOS** (built), **Windows** (build from source)

---

## How It Works

### The Architecture

```
┌─────────────┐     ┌──────────────────┐
│  TeamFlow   │────▶│   SQLite DB      │
│  (App)     │     │ (local file)      │
└─────────────┘     └────────┬─────────┘
                            │
                    ┌───────▼────────┐
                    │  OneDrive     │
                    │  ~/OneDrive/  │
                    │  TeamFlow/   │
                    └───────────────┘
```

1. **App runs** locally on your Mac/Windows
2. **Data saves** to a SQLite file (`teamflow.db`)
3. **OneDrive folder** is shared = team shares data
4. **Pull/Push** to sync with cloud

### Team Sharing

To share tasks with your team:

1. Each person installs TeamFlow on their device
2. Create folder: `mkdir ~/OneDrive/TeamFlow`
3. Share that folder in OneDrive with team members
4. Everyone: Open app → Settings → Pull from OneDrive
5. Now all see the same tasks!

---

## Download & Run

### Option 1: Pre-built (macOS)

```bash
# The app is at:
teamflow/src-tauri/target/release/bundle/macos/TeamFlow.app

# Or open with:
open teamflow/src-tauri/target/release/bundle/macos/TeamFlow.app/Contents/MacOS/tauri-app
```

### Option 2: Build from Source

```bash
# 1. Install dependencies
cd teamflow
npm install

# 2. Build the app
npm run tauri build

# 3. Run it
open src-tauri/target/release/bundle/macos/TeamFlow.app
```

### Option 3: Development Mode

```bash
cd teamflow
npm install
npm run tauri dev
```

---

## OneDrive Sync Setup

### Step 1: Create the sync folder

```bash
mkdir -p ~/OneDrive/TeamFlow
```

### Step 2: Enable auto-sync

In the app:
1. Click ⚙️ **Settings** button
2. Toggle **Auto-sync** to ON
3. Click **Push to OneDrive** to upload your data

### Step 3: For teams

1. Share the folder `~/OneDrive/TeamFlow` in OneDrive
2. Team members: Open Settings → **Pull from OneDrive**
3. Now everyone sees the same tasks!

---

## Features

| Feature | Description |
|---------|------------|
| **Kanban** | Drag & drop tasks between columns |
| **Tasks** | Title, description, priority, due date |
| **Subtasks** | Checkboxes inside tasks |
| **Milestones** | Progress steps per task |
| **Time Tracking** | Start/stop timer per task |
| **Recurring** | Daily/weekly/monthly tasks |
| **Teams** | Multiple teams |
| **Projects** | Group tasks by project |
| **Voice** | "Create task Fix bug" |
| **Tags** | Label tasks |
| **Comments** | Discuss on tasks |
| **OneDrive** | Sync without server |

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Click + Drag | Move task |
| Double-click task | Open details |
| Click column + | Add task |

---

## Troubleshooting

### "OneDrive folder not found"
- Make sure OneDrive is installed
- Create: `mkdir ~/OneDrive/TeamFlow`

### "No tasks showing"
- Create a team first (sidebar)
- Then create a project
- Then add tasks

### Database location
- macOS: `~/Library/Application Support/TeamFlow/teamflow.db`
- OneDrive: `~/OneDrive/TeamFlow/teamflow.db`

---

## Tech Stack

- **Desktop**: [Tauri 2](https://tauri.app)
- **Database**: SQLite
- **Backend**: Rust
- **Frontend**: Vanilla HTML/CSS/JS
- **Sync**: OneDrive

---

## Contributing

```bash
git clone https://github.com/seshakiran/teamflow
cd teamflow
npm install
npm run tauri dev
```

---

## License

MIT — Free forever.
