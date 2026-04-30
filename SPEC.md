# TeamFlow - Task Tracker Application Specification

## 1. Project Overview

**Project Name:** TeamFlow
**Type:** Desktop Application (Tauri)
**Core Functionality:** Asana-like task tracker with Kanban boards, Gantt charts, and team collaboration, backed by SQLite with OneDrive sync.
**Target Users:** Teams of 5-10 members who want a free, self-hosted alternative to Asana.

---

## 2. Architecture

### Server Mode (Mac Mini)
- SQLite database stored locally at `~/Library/Application Support/TeamFlow/data.db`
- Runs as HTTP server on local network (default port 3847)
- WebSocket for real-time updates to connected clients
- Lock file mechanism to track active users

### Client Mode
- Connect to server via local network IP
- Fallback to local cache if server unavailable
- OneDrive sync folder for backup/access

### Data Sync
- SQLite + lock file approach
- When server is in use, show "Another user is currently editing" warning
- Sync to OneDrive on app close

---

## 3. UI/UX Specification

### Window Structure
- **Main Window**: 1200x800 minimum, resizable
- **Sidebar**: 260px fixed width, collapsible
- **Content Area**: Flexible, fills remaining space

### Color Palette
```css
--bg-primary: #1a1a2e;        /* Deep navy background */
--bg-secondary: #16213e;    /* Slightly lighter navy */
--bg-tertiary: #0f3460;      /* Card backgrounds */
--accent-primary: #e94560;  /* Coral red accent */
--accent-secondary: #533483; /* Purple accent */
--text-primary: #eaeaea;     /* Primary text */
--text-secondary: #a0a0a0;  /* Secondary text */
--success: #00d9a5;         /* Green for completed */
--warning: #ffc107;         /* Yellow for in progress */
--danger: #ff4757;          /* Red for overdue */
--info: #3498db;            /* Blue for info */
```

### Typography
- **Font Family**: 'Inter', -apple-system, system-ui, sans-serif
- **Headings**: 24px (H1), 20px (H2), 16px (H3), weight 600
- **Body**: 14px, weight 400
- **Small**: 12px, weight 400

### Spacing System
- Base unit: 4px
- Padding: 8px, 12px, 16px, 24px
- Margins: 4px, 8px, 16px, 24px, 32px
- Border radius: 4px (small), 8px (medium), 12px (large)

### Visual Effects
- Box shadows: `0 2px 8px rgba(0,0,0,0.3)` for cards
- Transitions: 200ms ease for hover states
- Subtle gradient headers: linear-gradient(135deg, #1a1a2e, #16213e)

---

## 4. Component Specification

### Sidebar Navigation
- Logo/App name at top
- Project list (expandable)
- Team switcher dropdown
- Quick filters: My Tasks, Due Today, Overdue
- Settings button at bottom
- Collapse toggle

### Project View
- Project header with name, description, color tag
- View toggle: Board / List / Timeline / Gantt
- Progress bar showing completion %
- Member avatars (max 5 shown, +N for overflow)

### Kanban Board
- Columns: To Do, In Progress, In Review, Done (customizable)
- Drag & drop cards between columns
- Column header with count badge
- Add new column button

### Task Card
- Title (bold, truncated at 2 lines)
- Tags (colored pills, max 3 shown)
- Due date with color coding (red if overdue)
- Assignee avatar
- Subtask progress indicator (e.g., 2/5)
- Priority indicator (colored left border)
- Attachment icon if has attachments
- Comment count icon

### Task Detail Modal
- Full task information
- Description (rich text)
- Subtasks list with checkboxes
- Progress milestones (checklist style)
- Time tracking (start/stop button, total time display)
- Attachments section
- Comments thread
- Activity log
- Due date picker
- Recurring task settings

### Gantt/Timeline View
- Horizontal scrollable timeline
- Task bars showing duration
- Milestone diamonds
- Dependency arrows
- Zoom controls: Day/Week/Month
- Today line indicator

### Team Management
- Team list in sidebar
- Add team modal
- Member management (add/remove)
- Role assignment (Admin, Member, Guest)

### Assignee View
- List view grouped by assignee
- Filter by status, due date
- Quick actions

---

## 5. Database Schema

### Tables

```sql
-- Teams
CREATE TABLE teams (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    created_at TEXT,
    updated_at TEXT
);

-- Users (team members)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    team_id TEXT REFERENCES teams(id),
    name TEXT NOT NULL,
    email TEXT,
    avatar TEXT,
    role TEXT DEFAULT 'member',
    created_at TEXT
);

-- Projects
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    team_id TEXT REFERENCES teams(id),
    name TEXT NOT NULL,
    description TEXT,
    color TEXT,
    status TEXT DEFAULT 'active',
    created_at TEXT,
    updated_at TEXT
);

-- Project members
CREATE TABLE project_members (
    project_id TEXT REFERENCES projects(id),
    user_id TEXT REFERENCES users(id),
    role TEXT DEFAULT 'member',
    PRIMARY KEY (project_id, user_id)
);

-- Columns (for Kanban)
CREATE TABLE columns (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    name TEXT NOT NULL,
    position INTEGER,
    color TEXT
);

-- Tasks
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    column_id TEXT REFERENCES columns(id),
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT DEFAULT 'medium',
    due_date TEXT,
    start_date TEXT,
    completed_at TEXT,
    estimated_hours REAL,
    actual_hours REAL,
    recurring TEXT,
    position INTEGER,
    created_by TEXT REFERENCES users(id),
    created_at TEXT,
    updated_at TEXT
);

-- Task assignees
CREATE TABLE task_assignees (
    task_id TEXT REFERENCES tasks(id),
    user_id TEXT REFERENCES users(id),
    PRIMARY KEY (task_id, user_id)
);

-- Subtasks
CREATE TABLE subtasks (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    title TEXT NOT NULL,
    completed INTEGER DEFAULT 0,
    position INTEGER
);

-- Milestones (progress steps per task)
CREATE TABLE milestones (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    title TEXT NOT NULL,
    completed INTEGER DEFAULT 0,
    position INTEGER,
    due_date TEXT
);

-- Tags
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT
);

-- Task tags
CREATE TABLE task_tags (
    task_id TEXT REFERENCES tasks(id),
    tag_id TEXT REFERENCES tags(id),
    PRIMARY KEY (task_id, tag_id)
);

-- Comments
CREATE TABLE comments (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    user_id TEXT REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TEXT
);

-- Attachments
CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    filename TEXT NOT NULL,
    filepath TEXT,
    file_type TEXT,
    file_size INTEGER,
    uploaded_by TEXT REFERENCES users(id),
    created_at TEXT
);

-- Time entries
CREATE TABLE time_entries (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    user_id TEXT REFERENCES users(id),
    start_time TEXT NOT NULL,
    end_time TEXT,
    duration INTEGER
);

-- Activity log
CREATE TABLE activities (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id),
    user_id TEXT REFERENCES users(id),
    action TEXT NOT NULL,
    details TEXT,
    created_at TEXT
);

-- App settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- Lock file for multi-user
CREATE TABLE active_sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    ip_address TEXT,
    last_ping TEXT
);
```

---

## 6. API Endpoints (Server)

### Projects
- `GET /api/projects` - List all projects
- `POST /api/projects` - Create project
- `PUT /api/projects/:id` - Update project
- `DELETE /api/projects/:id` - Delete project

### Tasks
- `GET /api/tasks?project_id=` - List tasks
- `POST /api/tasks` - Create task
- `PUT /api/tasks/:id` - Update task
- `DELETE /api/tasks/:id` - Delete task
- `POST /api/tasks/:id/move` - Move task to column

### Columns
- `GET /api/columns?project_id=` - List columns
- `POST /api/columns` - Create column
- `PUT /api/columns/:id` - Update column
- `DELETE /api/columns/:id` - Delete column

### Users & Teams
- `GET /api/teams` - List teams
- `POST /api/teams` - Create team
- `GET /api/users` - List users (for team)
- `POST /api/users` - Add user

### Subtasks & Milestones
- `POST /api/tasks/:id/subtasks` - Add subtask
- `PUT /api/subtasks/:id` - Update subtask
- `POST /api/tasks/:id/milestones` - Add milestone
- `PUT /api/milestones/:id` - Update milestone

### Time Tracking
- `POST /api/time/start` - Start timer
- `POST /api/time/stop` - Stop timer

### Sync
- `GET /api/sync/status` - Check lock status
- `POST /api/sync/lock` - Acquire lock
- `POST /api/sync/unlock` - Release lock

---

## 7. Phases

### Phase 1 - Core Features (MVP)
- Projects CRUD
- Kanban board with drag & drop
- Task creation with title, description, due date
- Subtasks support
- Basic columns (To Do, In Progress, Done)
- Teams & user management
- Assignee view
- Server mode with local network access
- Lock mechanism

### Phase 2 - Advanced Views
- Gantt/Timeline view
- Time tracking with start/stop
- Recurring tasks
- Progress milestones per task
- Rich task detail modal
- Multiple assignees
- Tags

### Phase 3 - Integrations & Voice
- Web Speech API voice input
- Microsoft Teams webhook notifications
- WhatsApp webhook notifications
- OneDrive sync folder

---

## 8. Voice Commands

Supported voice commands (browser Web Speech API):
- "Create task [title]"
- "Add subtask to [task name]"
- "Mark [task name] as done"
- "When is [task name] due?"
- "Show my tasks"
- "Open project [name]"

---

## 9. Acceptance Criteria

### Phase 1
- [ ] App launches without errors
- [ ] Can create new project
- [ ] Kanban board displays columns and tasks
- [ ] Can drag tasks between columns
- [ ] Can create task with title and assignee
- [ ] Server mode works on local network
- [ ] Lock indicator shows when server is in use
- [ ] Multiple clients can connect

### Phase 2
- [ ] Gantt view displays task timeline
- [ ] Time tracking starts/stops correctly
- [ ] Recurring tasks generate new instances
- [ ] Milestones can be checked off

### Phase 3
- [ ] Voice input creates tasks
- [ ] Teams webhook sends notifications
- [ ] WhatsApp webhook sends notifications
- [ ] OneDrive sync works

---

## 10. Tech Stack

- **Backend**: Tauri (Rust) + SQLite (rusqlite)
- **Frontend**: Vanilla HTML/CSS/JS (for simplicity)
- **Real-time**: WebSocket (tokio-tungstenite)
- **Voice**: Web Speech API
- **Build**: Cargo + npm