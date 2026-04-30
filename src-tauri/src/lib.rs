use chrono::{DateTime, Utc, NaiveDate};
use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};
use uuid::Uuid;

// ============ Database State ============

pub struct DbState(pub Mutex<Connection>);

fn get_db_path() -> PathBuf {
    let app_data = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("TeamFlow");
    
    fs::create_dir_all(&app_data).ok();
    app_data.join("teamflow.db")
}

fn get_onedrive_path() -> Option<PathBuf> {
    // Try OneDrive locations
    let onedrive_paths: Vec<PathBuf> = vec![
        dirs::document_dir().and_then(|d| d.parent().map(|p| p.join("OneDrive"))),
        std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("OneDrive")),
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("OneDrive")),
    ].into_iter().flatten().collect();
    
    for path in onedrive_paths {
        if path.exists() {
            let teamflow_folder = path.join("TeamFlow");
            fs::create_dir_all(&teamflow_folder).ok();
            return Some(teamflow_folder);
        }
    }
    None
}

fn init_db(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS teams (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT DEFAULT '#e94560',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            team_id TEXT REFERENCES teams(id),
            name TEXT NOT NULL,
            email TEXT,
            avatar TEXT,
            role TEXT DEFAULT 'member',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            team_id TEXT REFERENCES teams(id),
            name TEXT NOT NULL,
            description TEXT,
            color TEXT DEFAULT '#533483',
            status TEXT DEFAULT 'active',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS project_members (
            project_id TEXT REFERENCES projects(id),
            user_id TEXT REFERENCES users(id),
            role TEXT DEFAULT 'member',
            PRIMARY KEY (project_id, user_id)
        );

        CREATE TABLE IF NOT EXISTS columns (
            id TEXT PRIMARY KEY,
            project_id TEXT REFERENCES projects(id),
            name TEXT NOT NULL,
            position INTEGER DEFAULT 0,
            color TEXT
        );

        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            project_id TEXT REFERENCES projects(id),
            column_id TEXT REFERENCES columns(id),
            title TEXT NOT NULL,
            description TEXT,
            priority TEXT DEFAULT 'medium',
            due_date TEXT,
            start_date TEXT,
            completed_at TEXT,
            estimated_hours REAL DEFAULT 0,
            actual_hours REAL DEFAULT 0,
            recurring TEXT,
            recurring_interval INTEGER,
            position INTEGER DEFAULT 0,
            created_by TEXT REFERENCES users(id),
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_assignees (
            task_id TEXT REFERENCES tasks(id),
            user_id TEXT REFERENCES users(id),
            PRIMARY KEY (task_id, user_id)
        );

        CREATE TABLE IF NOT EXISTS subtasks (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id),
            title TEXT NOT NULL,
            completed INTEGER DEFAULT 0,
            position INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS milestones (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id),
            title TEXT NOT NULL,
            completed INTEGER DEFAULT 0,
            position INTEGER DEFAULT 0,
            due_date TEXT
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            color TEXT DEFAULT '#3498db'
        );

        CREATE TABLE IF NOT EXISTS task_tags (
            task_id TEXT REFERENCES tasks(id),
            tag_id TEXT REFERENCES tags(id),
            PRIMARY KEY (task_id, tag_id)
        );

        CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id),
            user_id TEXT REFERENCES users(id),
            content TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS attachments (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id),
            filename TEXT NOT NULL,
            filepath TEXT,
            file_type TEXT,
            file_size INTEGER,
            uploaded_by TEXT REFERENCES users(id),
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS time_entries (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id),
            user_id TEXT REFERENCES users(id),
            start_time TEXT NOT NULL,
            end_time TEXT,
            duration INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS activities (
            id TEXT PRIMARY KEY,
            task_id TEXT REFERENCES tasks(id),
            user_id TEXT REFERENCES users(id),
            action TEXT NOT NULL,
            details TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS active_sessions (
            id TEXT PRIMARY KEY,
            user_id TEXT,
            device_name TEXT,
            last_ping TEXT NOT NULL
        );
    "#)?;

    // Insert default columns for any projects that don't have them
    Ok(())
}

// ============ Data Models ============

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub team_id: String,
    pub name: String,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub role: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub team_id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Column {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub position: i32,
    pub color: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub column_id: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub completed_at: Option<String>,
    pub estimated_hours: f64,
    pub actual_hours: f64,
    pub recurring: Option<String>,
    pub recurring_interval: Option<i32>,
    pub position: i32,
    pub created_by: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Subtask {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub completed: bool,
    pub position: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Milestone {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub completed: bool,
    pub position: i32,
    pub due_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskWithDetails {
    pub task: Task,
    pub assignees: Vec<User>,
    pub subtasks: Vec<Subtask>,
    pub milestones: Vec<Milestone>,
    pub tags: Vec<Tag>,
}

// ============ Tauri Commands ============

// Sync with OneDrive
#[tauri::command]
async fn sync_from_onedrive(state: State<'_, DbState>) -> Result<String, String> {
    let onedrive_path = get_onedrive_path().ok_or("OneDrive folder not found")?;
    let onedrive_db = onedrive_path.join("teamflow.db");
    
    if onedrive_db.exists() {
        let db_path = get_db_path();
        fs::copy(&onedrive_db, &db_path).map_err(|e| e.to_string())?;
        return Ok("Synced from OneDrive".to_string());
    }
    
    Err("No database found on OneDrive".to_string())
}

#[tauri::command]
async fn sync_to_onedrive(state: State<'_, DbState>) -> Result<String, String> {
    let onedrive_path = get_onedrive_path().ok_or("OneDrive folder not found")?;
    let onedrive_db = onedrive_path.join("teamflow.db");
    let db_path = get_db_path();
    
    fs::copy(&db_path, &onedrive_db).map_err(|e| e.to_string())?;
    Ok("Synced to OneDrive".to_string())
}

#[tauri::command]
async fn get_onedrive_status() -> Result<Option<String>, String> {
    Ok(get_onedrive_path().map(|p| p.to_string_lossy().to_string()))
}

// Teams
#[tauri::command]
fn get_teams(state: State<'_, DbState>) -> Result<Vec<Team>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, color, created_at, updated_at FROM teams ORDER BY name").map_err(|e| e.to_string())?;
    
    let teams = stmt.query_map([], |row| {
        Ok(Team {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(teams)
}

#[tauri::command]
fn create_team(state: State<'_, DbState>, name: String, color: String) -> Result<Team, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute("INSERT INTO teams (id, name, color, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)", 
        params![id, name, color, now, now]).map_err(|e| e.to_string())?;
    
    Ok(Team { id, name, color, created_at: now.clone(), updated_at: now })
}

// Users
#[tauri::command]
fn get_users(state: State<'_, DbState>, team_id: String) -> Result<Vec<User>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, team_id, name, email, avatar, role, created_at FROM users WHERE team_id = ?1").map_err(|e| e.to_string())?;
    
    let users = stmt.query_map([&team_id], |row| {
        Ok(User {
            id: row.get(0)?,
            team_id: row.get(1)?,
            name: row.get(2)?,
            email: row.get(3)?,
            avatar: row.get(4)?,
            role: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(users)
}

#[tauri::command]
fn create_user(state: State<'_, DbState>, team_id: String, name: String, email: String) -> Result<User, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute("INSERT INTO users (id, team_id, name, email, role, created_at) VALUES (?1, ?2, ?3, ?4, 'member', ?5)", 
        params![id, team_id, name, email, now]).map_err(|e| e.to_string())?;
    
    Ok(User { id, team_id, name, email: Some(email), avatar: None, role: "member".to_string(), created_at: now })
}

// Projects
#[tauri::command]
fn get_projects(state: State<'_, DbState>, team_id: String) -> Result<Vec<Project>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, team_id, name, description, color, status, created_at, updated_at FROM projects WHERE team_id = ?1 ORDER BY name").map_err(|e| e.to_string())?;
    
    let projects = stmt.query_map([&team_id], |row| {
        Ok(Project {
            id: row.get(0)?,
            team_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            color: row.get(4)?,
            status: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(projects)
}

#[tauri::command]
fn create_project(state: State<'_, DbState>, team_id: String, name: String, description: String, color: String) -> Result<Project, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute("INSERT INTO projects (id, team_id, name, description, color, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7)", 
        params![id, team_id, name, description, color, now, now]).map_err(|e| e.to_string())?;
    
    // Create default columns
    let default_columns = vec![
        ("To Do", 0, "#95a5a6"),
        ("In Progress", 1, "#3498db"),
        ("In Review", 2, "#9b59b6"),
        ("Done", 3, "#27ae60"),
    ];
    
    for (name, pos, color) in default_columns {
        let col_id = Uuid::new_v4().to_string();
        conn.execute("INSERT INTO columns (id, project_id, name, position, color) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![col_id, id, name, pos, color]).map_err(|e| e.to_string())?;
    }
    
    Ok(Project { id, team_id, name, description: Some(description), color, status: "active".to_string(), created_at: now.clone(), updated_at: now })
}

#[tauri::command]
fn update_project(state: State<'_, DbState>, id: String, name: String, description: String, color: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    
    conn.execute("UPDATE projects SET name = ?1, description = ?2, color = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, description, color, now, id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn delete_project(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM tasks WHERE project_id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM columns WHERE project_id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM project_members WHERE project_id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Columns
#[tauri::command]
fn get_columns(state: State<'_, DbState>, project_id: String) -> Result<Vec<Column>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, project_id, name, position, color FROM columns WHERE project_id = ?1 ORDER BY position").map_err(|e| e.to_string())?;
    
    let columns = stmt.query_map([&project_id], |row| {
        Ok(Column {
            id: row.get(0)?,
            project_id: row.get(1)?,
            name: row.get(2)?,
            position: row.get(3)?,
            color: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(columns)
}

#[tauri::command]
fn create_column(state: State<'_, DbState>, project_id: String, name: String, color: String) -> Result<Column, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    
    // Get max position
    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM columns WHERE project_id = ?1",
        [&project_id], |row| row.get(0)
    ).unwrap_or(-1);
    
    conn.execute("INSERT INTO columns (id, project_id, name, position, color) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, project_id, name, max_pos + 1, color]).map_err(|e| e.to_string())?;
    
    Ok(Column { id, project_id, name, position: max_pos + 1, color: Some(color) })
}

#[tauri::command]
fn update_column(state: State<'_, DbState>, id: String, name: String, color: String, position: i32) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute("UPDATE columns SET name = ?1, color = ?2, position = ?3 WHERE id = ?4",
        params![name, color, position, id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn delete_column(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    // Move tasks to first column or delete them
    let first_col: Option<String> = conn.query_row(
        "SELECT id FROM columns WHERE project_id = (SELECT project_id FROM columns WHERE id = ?1) ORDER BY position LIMIT 1",
        [&id], |row| row.get(0)
    ).ok();
    
    if let Some(first) = first_col {
        conn.execute("UPDATE tasks SET column_id = ?1 WHERE column_id = ?2", params![first, id]).ok();
    } else {
        conn.execute("DELETE FROM tasks WHERE column_id = ?1", [&id]).ok();
    }
    
    conn.execute("DELETE FROM columns WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Tasks
#[tauri::command]
fn get_tasks(state: State<'_, DbState>, project_id: String) -> Result<Vec<Task>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, project_id, column_id, title, description, priority, due_date, start_date, completed_at, 
         estimated_hours, actual_hours, recurring, recurring_interval, position, created_by, created_at, updated_at 
         FROM tasks WHERE project_id = ?1 ORDER BY position"
    ).map_err(|e| e.to_string())?;
    
    let tasks = stmt.query_map([&project_id], |row| {
        Ok(Task {
            id: row.get(0)?,
            project_id: row.get(1)?,
            column_id: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            priority: row.get(5)?,
            due_date: row.get(6)?,
            start_date: row.get(7)?,
            completed_at: row.get(8)?,
            estimated_hours: row.get(9)?,
            actual_hours: row.get(10)?,
            recurring: row.get(11)?,
            recurring_interval: row.get(12)?,
            position: row.get(13)?,
            created_by: row.get(14)?,
            created_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(tasks)
}

#[tauri::command]
fn get_task(state: State<'_, DbState>, id: String) -> Result<TaskWithDetails, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    let task: Task = conn.query_row(
        "SELECT id, project_id, column_id, title, description, priority, due_date, start_date, completed_at,
         estimated_hours, actual_hours, recurring, recurring_interval, position, created_by, created_at, updated_at
         FROM tasks WHERE id = ?1", [&id], |row| {
        Ok(Task {
            id: row.get(0)?,
            project_id: row.get(1)?,
            column_id: row.get(2)?,
            title: row.get(3)?,
            description: row.get(4)?,
            priority: row.get(5)?,
            due_date: row.get(6)?,
            start_date: row.get(7)?,
            completed_at: row.get(8)?,
            estimated_hours: row.get(9)?,
            actual_hours: row.get(10)?,
            recurring: row.get(11)?,
            recurring_interval: row.get(12)?,
            position: row.get(13)?,
            created_by: row.get(14)?,
            created_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    }).map_err(|e| e.to_string())?;
    
    // Get assignees
    let mut stmt = conn.prepare(
        "SELECT u.id, u.team_id, u.name, u.email, u.avatar, u.role, u.created_at 
         FROM users u JOIN task_assignees ta ON u.id = ta.user_id WHERE ta.task_id = ?1"
    ).map_err(|e| e.to_string())?;
    
    let assignees = stmt.query_map([&id], |row| {
        Ok(User {
            id: row.get(0)?,
            team_id: row.get(1)?,
            name: row.get(2)?,
            email: row.get(3)?,
            avatar: row.get(4)?,
            role: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // Get subtasks
    let mut stmt = conn.prepare("SELECT id, task_id, title, completed, position FROM subtasks WHERE task_id = ?1 ORDER BY position")
        .map_err(|e| e.to_string())?;
    let subtasks = stmt.query_map([&id], |row| {
        Ok(Subtask {
            id: row.get(0)?,
            task_id: row.get(1)?,
            title: row.get(2)?,
            completed: row.get::<_, i32>(3)? == 1,
            position: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // Get milestones
    let mut stmt = conn.prepare("SELECT id, task_id, title, completed, position, due_date FROM milestones WHERE task_id = ?1 ORDER BY position")
        .map_err(|e| e.to_string())?;
    let milestones = stmt.query_map([&id], |row| {
        Ok(Milestone {
            id: row.get(0)?,
            task_id: row.get(1)?,
            title: row.get(2)?,
            completed: row.get::<_, i32>(3)? == 1,
            position: row.get(4)?,
            due_date: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    // Get tags
    let mut stmt = conn.prepare(
        "SELECT t.id, t.name, t.color FROM tags t JOIN task_tags tt ON t.id = tt.tag_id WHERE tt.task_id = ?1"
    ).map_err(|e| e.to_string())?;
    let tags = stmt.query_map([&id], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(TaskWithDetails { task, assignees, subtasks, milestones, tags })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[tauri::command]
fn create_task(state: State<'_, DbState>, project_id: String, column_id: String, title: String, description: String, priority: String, due_date: Option<String>) -> Result<Task, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    // Get max position in column
    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM tasks WHERE column_id = ?1",
        [&column_id], |row| row.get(0)
    ).unwrap_or(-1);
    
    conn.execute(
        "INSERT INTO tasks (id, project_id, column_id, title, description, priority, due_date, position, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id, project_id, column_id, title, description, priority, due_date, max_pos + 1, now, now]
    ).map_err(|e| e.to_string())?;
    
    Ok(Task {
        id,
        project_id,
        column_id,
        title,
        description: Some(description),
        priority,
        due_date,
        start_date: None,
        completed_at: None,
        estimated_hours: 0.0,
        actual_hours: 0.0,
        recurring: None,
        recurring_interval: None,
        position: max_pos + 1,
        created_by: None,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
fn update_task(state: State<'_, DbState>, id: String, title: String, description: String, priority: String, due_date: Option<String>, start_date: Option<String>, estimated_hours: f64, recurring: Option<String>, recurring_interval: Option<i32>) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        "UPDATE tasks SET title = ?1, description = ?2, priority = ?3, due_date = ?4, start_date = ?5, estimated_hours = ?6, recurring = ?7, recurring_interval = ?8, updated_at = ?9 WHERE id = ?10",
        params![title, description, priority, due_date, start_date, estimated_hours, recurring, recurring_interval, now, id]
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn move_task(state: State<'_, DbState>, id: String, column_id: String, position: i32) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    
    conn.execute("UPDATE tasks SET column_id = ?1, position = ?2, updated_at = ?3 WHERE id = ?4",
        params![column_id, position, now, id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn complete_task(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    
    conn.execute("UPDATE tasks SET completed_at = ?1, updated_at = ?1 WHERE id = ?2",
        params![now, id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn delete_task(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM task_assignees WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM subtasks WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM milestones WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM task_tags WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM comments WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM time_entries WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM activities WHERE task_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM tasks WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Assignees
#[tauri::command]
fn assign_user(state: State<'_, DbState>, task_id: String, user_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute("INSERT OR IGNORE INTO task_assignees (task_id, user_id) VALUES (?1, ?2)",
        params![task_id, user_id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
fn unassign_user(state: State<'_, DbState>, task_id: String, user_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM task_assignees WHERE task_id = ?1 AND user_id = ?2",
        params![task_id, user_id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

// Subtasks
#[tauri::command]
fn get_subtasks(state: State<'_, DbState>, task_id: String) -> Result<Vec<Subtask>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, task_id, title, completed, position FROM subtasks WHERE task_id = ?1 ORDER BY position")
        .map_err(|e| e.to_string())?;
    
    let subtasks = stmt.query_map([&task_id], |row| {
        Ok(Subtask {
            id: row.get(0)?,
            task_id: row.get(1)?,
            title: row.get(2)?,
            completed: row.get::<_, i32>(3)? == 1,
            position: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(subtasks)
}

#[tauri::command]
fn create_subtask(state: State<'_, DbState>, task_id: String, title: String) -> Result<Subtask, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    
    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM subtasks WHERE task_id = ?1",
        [&task_id], |row| row.get(0)
    ).unwrap_or(-1);
    
    conn.execute("INSERT INTO subtasks (id, task_id, title, position) VALUES (?1, ?2, ?3, ?4)",
        params![id, task_id, title, max_pos + 1]).map_err(|e| e.to_string())?;
    
    Ok(Subtask { id, task_id, title, completed: false, position: max_pos + 1 })
}

#[tauri::command]
fn toggle_subtask(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    let completed: i32 = conn.query_row("SELECT completed FROM subtasks WHERE id = ?1", [&id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    
    let new_completed = if completed == 1 { 0 } else { 1 };
    conn.execute("UPDATE subtasks SET completed = ?1 WHERE id = ?2", params![new_completed, id]).map_err(|e| e.to_string())?;
    
    Ok(new_completed == 1)
}

#[tauri::command]
fn delete_subtask(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM subtasks WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

// Milestones
#[tauri::command]
fn get_milestones(state: State<'_, DbState>, task_id: String) -> Result<Vec<Milestone>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, task_id, title, completed, position, due_date FROM milestones WHERE task_id = ?1 ORDER BY position")
        .map_err(|e| e.to_string())?;
    
    let milestones = stmt.query_map([&task_id], |row| {
        Ok(Milestone {
            id: row.get(0)?,
            task_id: row.get(1)?,
            title: row.get(2)?,
            completed: row.get::<_, i32>(3)? == 1,
            position: row.get(4)?,
            due_date: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(milestones)
}

#[tauri::command]
fn create_milestone(state: State<'_, DbState>, task_id: String, title: String, due_date: Option<String>) -> Result<Milestone, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    
    let max_pos: i32 = conn.query_row(
        "SELECT COALESCE(MAX(position), -1) FROM milestones WHERE task_id = ?1",
        [&task_id], |row| row.get(0)
    ).unwrap_or(-1);
    
    conn.execute("INSERT INTO milestones (id, task_id, title, position, due_date) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, task_id, title, max_pos + 1, due_date]).map_err(|e| e.to_string())?;
    
    Ok(Milestone { id, task_id, title, completed: false, position: max_pos + 1, due_date })
}

#[tauri::command]
fn toggle_milestone(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    
    let completed: i32 = conn.query_row("SELECT completed FROM milestones WHERE id = ?1", [&id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    
    let new_completed = if completed == 1 { 0 } else { 1 };
    conn.execute("UPDATE milestones SET completed = ?1 WHERE id = ?2", params![new_completed, id]).map_err(|e| e.to_string())?;
    
    Ok(new_completed == 1)
}

#[tauri::command]
fn delete_milestone(state: State<'_, DbState>, id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM milestones WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

// Time Tracking
#[tauri::command]
fn start_timer(state: State<'_, DbState>, task_id: String, user_id: String) -> Result<String, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    // Stop any running timer first
    conn.execute("UPDATE time_entries SET end_time = ?1, duration = CAST((julianday(?1) - julianday(start_time)) * 86400 AS INTEGER) WHERE task_id = ?2 AND user_id = ?3 AND end_time IS NULL",
        params![now, task_id, user_id]).ok();
    
    conn.execute("INSERT INTO time_entries (id, task_id, user_id, start_time) VALUES (?1, ?2, ?3, ?4)",
        params![id, task_id, user_id, now]).map_err(|e| e.to_string())?;
    
    Ok(id)
}

#[tauri::command]
fn stop_timer(state: State<'_, DbState>, entry_id: String) -> Result<i64, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let now = Utc::now().to_rfc3339();
    
    conn.execute("UPDATE time_entries SET end_time = ?1, duration = CAST((julianday(?1) - julianday(start_time)) * 86400 AS INTEGER) WHERE id = ?2",
        params![now, entry_id]).map_err(|e| e.to_string())?;
    
    let duration: i64 = conn.query_row("SELECT duration FROM time_entries WHERE id = ?1", [&entry_id], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    
    Ok(duration)
}

#[tauri::command]
fn get_time_entries(state: State<'_, DbState>, task_id: String) -> Result<Vec<TimeEntry>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, task_id, user_id, start_time, end_time, duration FROM time_entries WHERE task_id = ?1 ORDER BY start_time DESC")
        .map_err(|e| e.to_string())?;
    
    let entries = stmt.query_map([&task_id], |row| {
        Ok(TimeEntry {
            id: row.get(0)?,
            task_id: row.get(1)?,
            user_id: row.get(2)?,
            start_time: row.get(3)?,
            end_time: row.get(4)?,
            duration: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(entries)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeEntry {
    pub id: String,
    pub task_id: String,
    pub user_id: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration: i64,
}

// Tags
#[tauri::command]
fn get_tags(state: State<'_, DbState>) -> Result<Vec<Tag>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, name, color FROM tags ORDER BY name").map_err(|e| e.to_string())?;
    
    let tags = stmt.query_map([], |row| {
        Ok(Tag {
            id: row.get(0)?,
            name: row.get(1)?,
            color: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(tags)
}

#[tauri::command]
fn create_tag(state: State<'_, DbState>, name: String, color: String) -> Result<Tag, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    
    conn.execute("INSERT INTO tags (id, name, color) VALUES (?1, ?2, ?3)",
        params![id, name, color]).map_err(|e| e.to_string())?;
    
    Ok(Tag { id, name, color })
}

#[tauri::command]
fn add_tag(state: State<'_, DbState>, task_id: String, tag_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?1, ?2)", params![task_id, tag_id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn remove_tag(state: State<'_, DbState>, task_id: String, tag_id: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM task_tags WHERE task_id = ?1 AND tag_id = ?2", params![task_id, tag_id]).map_err(|e| e.to_string())?;
    Ok(())
}

// Comments
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub task_id: String,
    pub user_id: String,
    pub user_name: String,
    pub content: String,
    pub created_at: String,
}

#[tauri::command]
fn get_comments(state: State<'_, DbState>, task_id: String) -> Result<Vec<Comment>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT c.id, c.task_id, c.user_id, u.name, c.content, c.created_at FROM comments c JOIN users u ON c.user_id = u.id WHERE c.task_id = ?1 ORDER BY c.created_at DESC"
    ).map_err(|e| e.to_string())?;
    
    let comments = stmt.query_map([&task_id], |row| {
        Ok(Comment {
            id: row.get(0)?,
            task_id: row.get(1)?,
            user_id: row.get(2)?,
            user_name: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(comments)
}

#[tauri::command]
fn create_comment(state: State<'_, DbState>, task_id: String, user_id: String, content: String) -> Result<Comment, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute("INSERT INTO comments (id, task_id, user_id, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, task_id, user_id, content, now]).map_err(|e| e.to_string())?;
    
    let user_name: String = conn.query_row("SELECT name FROM users WHERE id = ?1", [&user_id], |row| row.get(0))
        .unwrap_or_else(|_| "Unknown".to_string());
    
    Ok(Comment { id, task_id, user_id, user_name, content, created_at: now })
}

// Voice command
#[tauri::command]
fn process_voice_command(state: State<'_, DbState>, command: String, user_id: String) -> Result<String, String> {
    let command_lower = command.to_lowercase();
    
    // Parse simple commands
    if command_lower.starts_with("create task ") {
        let title = command[12..].trim().to_string();
        // Get first project and column
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        
        let project_id: String = conn.query_row(
            "SELECT id FROM projects LIMIT 1", [], |row| row.get(0)
        ).map_err(|_| "No projects found. Create a project first.")?;
        
        let column_id: String = conn.query_row(
            "SELECT id FROM columns WHERE project_id = ?1 ORDER BY position LIMIT 1",
            [&project_id], |row| row.get(0)
        ).map_err(|_| "No columns found")?;
        
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        
        conn.execute(
            "INSERT INTO tasks (id, project_id, column_id, title, position, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
            params![id, project_id, column_id, title, now, now]
        ).map_err(|e| e.to_string())?;
        
        return Ok(format!("Created task: {}", title));
    }
    
    if command_lower.starts_with("mark ") && command_lower.contains(" as done") {
        let task_name = command[5..command_lower.len() - 8].trim();
        let conn = state.0.lock().map_err(|e| e.to_string())?;
        let now = Utc::now().to_rfc3339();
        
        let rows = conn.execute(
            "UPDATE tasks SET completed_at = ?1, updated_at = ?1 WHERE title LIKE ?2",
            params![now, format!("%{}%", task_name)]
        );
        
        if rows.map(|r| r > 0).unwrap_or(false) {
            return Ok(format!("Marked '{}' as done", task_name));
        }
        return Err(format!("Task '{}' not found", task_name));
    }
    
    Err("Unknown command. Try 'create task [title]' or 'mark [task] as done'".to_string())
}

// Activity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Activity {
    pub id: String,
    pub task_id: String,
    pub user_id: String,
    pub user_name: String,
    pub action: String,
    pub details: Option<String>,
    pub created_at: String,
}

#[tauri::command]
fn get_activities(state: State<'_, DbState>, task_id: String) -> Result<Vec<Activity>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT a.id, a.task_id, a.user_id, u.name, a.action, a.details, a.created_at FROM activities a JOIN users u ON a.user_id = u.id WHERE a.task_id = ?1 ORDER BY a.created_at DESC LIMIT 50"
    ).map_err(|e| e.to_string())?;
    
    let activities = stmt.query_map([&task_id], |row| {
        Ok(Activity {
            id: row.get(0)?,
            task_id: row.get(1)?,
            user_id: row.get(2)?,
            user_name: row.get(3)?,
            action: row.get(4)?,
            details: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?
    .filter_map(|r| r.ok())
    .collect();
    
    Ok(activities)
}

fn log_activity(conn: &Connection, task_id: &str, user_id: &str, action: &str, details: Option<&str>) {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    
    conn.execute("INSERT INTO activities (id, task_id, user_id, action, details, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, task_id, user_id, action, details, now]).ok();
}

// Settings
#[tauri::command]
fn get_setting(state: State<'_, DbState>, key: String) -> Result<Option<String>, String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    let value: Option<String> = conn.query_row("SELECT value FROM settings WHERE key = ?1", [&key], |row| row.get(0)).ok();
    Ok(value)
}

#[tauri::command]
fn set_setting(state: State<'_, DbState>, key: String, value: String) -> Result<(), String> {
    let conn = state.0.lock().map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value]).map_err(|e| e.to_string())?;
    Ok(())
}

// ============ App Entry ============

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    let db_path = get_db_path();
    let conn = Connection::open(&db_path).expect("Failed to open database");
    init_db(&conn).expect("Failed to initialize database");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(DbState(Mutex::new(conn)))
        .invoke_handler(tauri::generate_handler![
            // Sync
            sync_from_onedrive,
            sync_to_onedrive,
            get_onedrive_status,
            // Teams
            get_teams,
            create_team,
            // Users
            get_users,
            create_user,
            // Projects
            get_projects,
            create_project,
            update_project,
            delete_project,
            // Columns
            get_columns,
            create_column,
            update_column,
            delete_column,
            // Tasks
            get_tasks,
            get_task,
            create_task,
            update_task,
            move_task,
            complete_task,
            delete_task,
            // Assignees
            assign_user,
            unassign_user,
            // Subtasks
            get_subtasks,
            create_subtask,
            toggle_subtask,
            delete_subtask,
            // Milestones
            get_milestones,
            create_milestone,
            toggle_milestone,
            delete_milestone,
            // Time tracking
            start_timer,
            stop_timer,
            get_time_entries,
            // Tags
            get_tags,
            create_tag,
            add_tag,
            remove_tag,
            // Comments
            get_comments,
            create_comment,
            // Activities
            get_activities,
            // Voice
            process_voice_command,
            // Settings
            get_setting,
            set_setting,
        ])
        .setup(|app| {
            // Get app data directory
            let app_data = dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("TeamFlow");
            
            std::fs::create_dir_all(&app_data).ok();
            
            println!("TeamFlow data directory: {:?}", app_data);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}