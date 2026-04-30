// TeamFlow - Main JavaScript
const { invoke } = window.__TAURI__.core;

// State
let currentTeam = null;
let currentProject = null;
let currentView = 'board';
let teams = [];
let projects = [];
let columns = [];
let tasks = [];
let users = [];
let tags = [];
let currentTask = null;
let timerInterval = null;
let timerStart = null;
let voiceRecognition = null;

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    await loadTeams();
    await loadTags();
    await checkOneDrive();
    initVoice();
    setupDragAndDrop();
});

// ============ API Calls ============

async function loadTeams() {
    try {
        teams = await invoke('get_teams');
        renderTeams();
        if (teams.length > 0) {
            selectTeam(teams[0].id);
        }
    } catch (e) {
        console.error('Load teams error:', e);
    }
}

async function loadProjects(teamId) {
    try {
        projects = await invoke('get_projects', { teamId });
        renderProjects();
        if (projects.length > 0) {
            selectProject(projects[0].id);
        }
    } catch (e) {
        console.error('Load projects error:', e);
    }
}

async function loadColumns(projectId) {
    try {
        columns = await invoke('get_columns', { projectId });
        renderBoard();
    } catch (e) {
        console.error('Load columns error:', e);
    }
}

async function loadTasks(projectId) {
    try {
        tasks = await invoke('get_tasks', { projectId });
        renderBoard();
        renderList();
        renderTimeline();
    } catch (e) {
        console.error('Load tasks error:', e);
    }
}

async function loadUsers(teamId) {
    try {
        users = await invoke('get_users', { teamId });
    } catch (e) {
        console.error('Load users error:', e);
    }
}

async function loadTags() {
    try {
        tags = await invoke('get_tags');
    } catch (e) {
        console.error('Load tags error:', e);
    }
}

async function checkOneDrive() {
    try {
        const path = await invoke('get_onedrive_status');
        if (path) {
            document.getElementById('onedrive-path').textContent = `OneDrive path: ${path}`;
            document.querySelector('.sync-status').classList.add('synced');
            document.querySelector('.sync-status span:last-child').textContent = 'OneDrive connected';
        }
    } catch (e) {
        console.error('OneDrive status:', e);
    }
}

// ============ Render Functions ============

function renderTeams() {
    const container = document.getElementById('teams-list');
    container.innerHTML = teams.map(team => `
        <div class="team-item ${currentTeam?.id === team.id ? 'active' : ''}" onclick="selectTeam('${team.id}')">
            <div class="team-color" style="background: ${team.color}"></div>
            <span class="team-name">${team.name}</span>
        </div>
    `).join('');
}

function renderProjects() {
    const container = document.getElementById('projects-list');
    container.innerHTML = projects.map(project => `
        <div class="project-item ${currentProject?.id === project.id ? 'active' : ''}" onclick="selectProject('${project.id}')">
            <div class="project-color" style="background: ${project.color}"></div>
            <span class="project-name">${project.name}</span>
        </div
    `).join('');
}

function renderBoard() {
    const container = document.getElementById('kanban-container');
    
    const columnsWithTasks = columns.map(col => {
        const colTasks = tasks.filter(t => t.column_id === col.id).sort((a, b) => a.position - b.position);
        return { ...col, tasks: colTasks };
    });
    
    container.innerHTML = columnsWithTasks.map(col => `
        <div class="kanban-column" data-column-id="${col.id}">
            <div class="column-header">
                <span class="column-title">
                    <span style="color: ${col.color || '#fff'}">●</span>
                    ${col.name}
                    <span class="column-count">${col.tasks.length}</span>
                </span>
                <div class="column-actions">
                    <button class="column-action-btn" onclick="showTaskModal('${col.id}')">+</button>
                </div>
            </div>
            <div class="column-body" data-column-id="${col.id}">
                ${col.tasks.map(task => renderTaskCard(task)).join('')}
            </div>
        </div>
    `).join('');
    
    setupDragAndDrop();
}

function renderTaskCard(task) {
    const dueDate = task.due_date ? new Date(task.due_date) : null;
    const now = new Date();
    let dueClass = '';
    
    if (dueDate) {
        const diffDays = Math.ceil((dueDate - now) / (1000 * 60 * 60 * 24));
        if (diffDays < 0) dueClass = 'overdue';
        else if (diffDays <= 2) dueClass = 'due-soon';
    }
    
    const priorityClass = task.priority !== 'low' ? `priority-${task.priority}` : '';
    const completedClass = task.completed_at ? 'completed' : '';
    
    return `
        <div class="task-card ${priorityClass} ${completedClass}" 
             data-task-id="${task.id}" 
             onclick="openTask('${task.id}')">
            <div class="task-title">${task.title}</div>
            <div class="task-meta">
                ${dueDate ? `<span class="task-due ${dueClass}">📅 ${formatDate(task.due_date)}</span>` : ''}
            </div>
            <div class="task-tags">
                ${task.tags ? task.tags.map(t => `<span class="task-tag" style="background: ${t.color}">${t.name}</span>`).join('') : ''}
            </div>
            <div class="task-progress">
                ${task.subtasks?.length ? `${task.subtasks.filter(s => s.completed).length}/${task.subtasks.length} subtasks` : ''}
            </div>
        </div>
    `;
}

function renderList() {
    const tbody = document.getElementById('task-list-body');
    tbody.innerHTML = tasks.map(task => `
        <tr onclick="openTask('${task.id}')" style="cursor: pointer;">
            <td>${task.title}</td>
            <td>${task.assignees?.map(a => a.name).join(', ') || '-'}</td>
            <td>${task.due_date ? formatDate(task.due_date) : '-'}</td>
            <td><span class="priority-badge ${task.priority}">${task.priority}</span></td>
            <td>${task.completed_at ? '✅ Done' : '📋 In Progress'}</td>
        </tr>
    `).join('');
}

function renderTimeline() {
    const container = document.getElementById('timeline-body');
    const today = new Date();
    
    container.innerHTML = tasks.filter(t => t.start_date || t.due_date).map(task => {
        const start = task.start_date ? new Date(task.start_date) : today;
        const end = task.due_date ? new Date(task.due_date) : start;
        const days = Math.ceil((end - start) / (1000 * 60 * 60 * 24)) || 1;
        
        return `
            <div class="timeline-row">
                <div class="timeline-task-name">${task.title}</div>
                <div class="timeline-bar" style="width: ${days * 40}px; background: ${getPriorityColor(task.priority)}"></div>
            </div>
        `;
    }).join('');
}

// ============ Selection Functions ============

function selectTeam(teamId) {
    currentTeam = teams.find(t => t.id === teamId);
    renderTeams();
    loadProjects(teamId);
    loadUsers(teamId);
}

function selectProject(projectId) {
    currentProject = projects.find(p => p.id === projectId);
    document.getElementById('current-project-name').textContent = currentProject?.name || 'Select a Project';
    renderProjects();
    loadColumns(projectId);
    loadTasks(projectId);
}

// ============ Modal Functions ============

function showTeamModal() {
    document.getElementById('team-modal').classList.add('active');
}

function showProjectModal() {
    if (!currentTeam) {
        alert('Please select or create a team first');
        return;
    }
    document.getElementById('project-modal').classList.add('active');
}

function showColumnModal() {
    if (!currentProject) {
        alert('Please select or create a project first');
        return;
    }
    document.getElementById('column-modal').classList.add('active');
}

function showTaskModal(columnId = null) {
    if (!currentProject) {
        alert('Please select or create a project first');
        return;
    }
    window.selectedColumnId = columnId || columns[0]?.id;
    document.getElementById('task-create-modal').classList.add('active');
}

function closeModal(modalId) {
    document.getElementById(modalId).classList.remove('active');
}

function showSettings() {
    document.getElementById('settings-modal').classList.add('active');
}

// ============ CRUD Functions ============

async function createTeam() {
    const name = document.getElementById('team-name').value;
    const color = document.getElementById('team-color').value;
    
    if (!name) return alert('Team name required');
    
    try {
        const team = await invoke('create_team', { name, color });
        teams.push(team);
        renderTeams();
        selectTeam(team.id);
        closeModal('team-modal');
        document.getElementById('team-name').value = '';
    } catch (e) {
        console.error('Create team error:', e);
    }
}

async function createProject() {
    const name = document.getElementById('project-name').value;
    const description = document.getElementById('project-description').value;
    const color = document.getElementById('project-color').value;
    
    if (!name || !currentTeam) return alert('Project name and team required');
    
    try {
        const project = await invoke('create_project', { 
            teamId: currentTeam.id, 
            name, 
            description, 
            color 
        });
        projects.push(project);
        renderProjects();
        selectProject(project.id);
        closeModal('project-modal');
    } catch (e) {
        console.error('Create project error:', e);
    }
}

async function createColumn() {
    const name = document.getElementById('column-name').value;
    const color = document.getElementById('column-color').value;
    
    if (!name || !currentProject) return;
    
    try {
        const col = await invoke('create_column', { 
            projectId: currentProject.id, 
            name, 
            color 
        });
        columns.push(col);
        renderBoard();
        closeModal('column-modal');
    } catch (e) {
        console.error('Create column error:', e);
    }
}

async function createTask() {
    const title = document.getElementById('new-task-title').value;
    const description = document.getElementById('new-task-description').value;
    const priority = document.getElementById('new-task-priority').value;
    const dueDate = document.getElementById('new-task-due-date').value || null;
    
    if (!title || !currentProject || !window.selectedColumnId) return;
    
    try {
        const task = await invoke('create_task', { 
            projectId: currentProject.id, 
            columnId: window.selectedColumnId, 
            title, 
            description, 
            priority, 
            dueDate 
        });
        tasks.push(task);
        renderBoard();
        closeModal('task-create-modal');
    } catch (e) {
        console.error('Create task error:', e);
    }
}

// ============ Task Detail Functions ============

async function openTask(taskId) {
    try {
        currentTask = await invoke('get_task', { id: taskId });
        document.getElementById('task-modal').classList.add('active');
        
        // Populate fields
        document.getElementById('task-title-input').value = currentTask.task.title;
        document.getElementById('task-description').value = currentTask.task.description || '';
        document.getElementById('task-status').value = currentTask.task.column_id;
        document.getElementById('task-priority').value = currentTask.task.priority;
        document.getElementById('task-due-date').value = currentTask.task.due_date?.split('T')[0] || '';
        document.getElementById('task-start-date').value = currentTask.task.start_date?.split('T')[0] || '';
        document.getElementById('task-estimated-hours').value = currentTask.task.estimated_hours || 0;
        
        renderSubtasks();
        renderMilestones();
        renderComments();
        renderTimeEntries();
        renderAssignees();
        renderTags();
    } catch (e) {
        console.error('Open task error:', e);
    }
}

function closeTaskModal() {
    document.getElementById('task-modal').classList.remove('active');
    currentTask = null;
}

async function saveTask() {
    if (!currentTask) return;
    
    const updates = {
        id: currentTask.task.id,
        title: document.getElementById('task-title-input').value,
        description: document.getElementById('task-description').value,
        priority: document.getElementById('task-priority').value,
        dueDate: document.getElementById('task-due-date').value || null,
        startDate: document.getElementById('task-start-date').value || null,
        estimatedHours: parseFloat(document.getElementById('task-estimated-hours').value) || 0,
        recurring: null,
        recurringInterval: null
    };
    
    try {
        await invoke('update_task', updates);
        await loadTasks(currentProject.id);
    } catch (e) {
        console.error('Save task error:', e);
    }
}

// Subtasks
function renderSubtasks() {
    const container = document.getElementById('subtasks-list');
    container.innerHTML = currentTask.subtasks.map(st => `
        <div class="subtask-item ${st.completed ? 'completed' : ''}">
            <input type="checkbox" ${st.completed ? 'checked' : ''} onchange="toggleSubtask('${st.id}')" />
            <span class="subtask-title">${st.title}</span>
            <button class="remove-btn" onclick="deleteSubtask('${st.id}')">×</button>
        </div>
    `).join('');
}

async function addSubtask() {
    const title = document.getElementById('new-subtask').value;
    if (!title || !currentTask) return;
    
    await invoke('create_subtask', { taskId: currentTask.task.id, title });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderSubtasks();
    document.getElementById('new-subtask').value = '';
}

async function toggleSubtask(id) {
    await invoke('toggle_subtask', { id });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderSubtasks();
}

async function deleteSubtask(id) {
    await invoke('delete_subtask', { id });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderSubtasks();
}

// Milestones
function renderMilestones() {
    const container = document.getElementById('milestones-list');
    container.innerHTML = currentTask.milestones.map(m => `
        <div class="milestone-item ${m.completed ? 'completed' : ''}">
            <input type="checkbox" ${m.completed ? 'checked' : ''} onchange="toggleMilestone('${m.id}')" />
            <span class="milestone-title">${m.title}</span>
            ${m.due_date ? `<span style="font-size:11px;color:var(--text-muted)">${formatDate(m.due_date)}</span>` : ''}
            <button class="remove-btn" onclick="deleteMilestone('${m.id}')">×</button>
        </div>
    `).join('');
}

async function addMilestone() {
    const title = document.getElementById('new-milestone').value;
    const dueDate = document.getElementById('milestone-date').value || null;
    if (!title || !currentTask) return;
    
    await invoke('create_milestone', { taskId: currentTask.task.id, title, dueDate });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderMilestones();
    document.getElementById('new-milestone').value = '';
}

async function toggleMilestone(id) {
    await invoke('toggle_milestone', { id });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderMilestones();
}

async function deleteMilestone(id) {
    await invoke('delete_milestone', { id });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderMilestones();
}

// Comments
function renderComments() {
    const container = document.getElementById('comments-list');
    container.innerHTML = currentTask.comments.map(c => `
        <div class="comment-item">
            <div class="comment-header">
                <span class="comment-author">${c.user_name}</span>
                <span class="comment-date">${formatDate(c.created_at)}</span>
            </div>
            <div class="comment-content">${c.content}</div>
        </div>
    `).join('');
}

async function addComment() {
    const content = document.getElementById('new-comment').value;
    if (!content || !currentTask || !users[0]) return;
    
    await invoke('create_comment', { 
        taskId: currentTask.task.id, 
        userId: users[0].id, 
        content 
    });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderComments();
    document.getElementById('new-comment').value = '';
}

// Time Tracking
function renderTimeEntries() {
    const container = document.getElementById('time-entries');
    container.innerHTML = currentTask.time_entries?.map(te => `
        <div class="time-entry">
            <span>${formatDate(te.start_time)}</span>
            <span>${formatDuration(te.duration)}</span>
        </div>
    `).join('') || '';
}

async function startTimer() {
    if (!currentTask || !users[0]) return;
    
    const btn = document.getElementById('btn-timer-start');
    
    if (btn.classList.contains('running')) {
        // Stop timer
        await invoke('stop_timer', { entryId: timerEntryId });
        clearInterval(timerInterval);
        btn.classList.remove('running');
        btn.textContent = '▶ Start Timer';
    } else {
        // Start timer
        timerEntryId = await invoke('start_timer', { 
            taskId: currentTask.task.id, 
            userId: users[0].id 
        });
        timerStart = Date.now();
        timerInterval = setInterval(updateTimer, 1000);
        btn.classList.add('running');
        btn.textContent = '⏹ Stop Timer';
    }
}

function updateTimer() {
    const elapsed = Math.floor((Date.now() - timerStart) / 1000);
    document.getElementById('timer-display').textContent = formatDuration(elapsed);
}

// Assignees
function renderAssignees() {
    const container = document.getElementById('task-assignees');
    container.innerHTML = currentTask.assignees.map(a => `
        <span class="assignee-tag">
            ${a.name}
            <button class="remove-btn" onclick="unassignUser('${a.id}')">×</button>
        </span>
    `).join('');
    
    // Populate add assignee dropdown
    const select = document.getElementById('add-assignee');
    select.innerHTML = '<option value="">Add assignee...</option>' + 
        users.filter(u => !currentTask.assignees.some(a => a.id === u.id))
            .map(u => `<option value="${u.id}">${u.name}</option>`).join('');
    select.onchange = async (e) => {
        if (e.target.value) {
            await invoke('assign_user', { taskId: currentTask.task.id, userId: e.target.value });
            currentTask = await invoke('get_task', { id: currentTask.task.id });
            renderAssignees();
        }
    };
}

async function unassignUser(userId) {
    await invoke('unassign_user', { taskId: currentTask.task.id, userId: userId });
    currentTask = await invoke('get_task', { id: currentTask.task.id });
    renderAssignees();
}

// Tags
function renderTags() {
    const container = document.getElementById('task-tags');
    container.innerHTML = currentTask.tags.map(t => `
        <span class="task-tag" style="background: ${t.color}">${t.name}</span>
    `).join('');
}

// ============ View Functions ============

function setView(view) {
    currentView = view;
    document.querySelectorAll('.view-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.view === view);
    });
    
    ['board-view', 'list-view', 'timeline-view', 'gantt-view'].forEach(id => {
        document.getElementById(id).style.display = 'none';
    });
    document.getElementById(`${view}-view`).style.display = 'block';
}

function toggleSidebar() {
    document.getElementById('sidebar').classList.toggle('collapsed');
}

function timelineZoom(zoom) {
    // Timeline zoom implementation
}

// ============ Sync Functions ============

async function syncFromOneDrive() {
    try {
        const result = await invoke('sync_from_onedrive');
        alert(result);
        await loadTeams();
    } catch (e) {
        alert('Sync failed: ' + e);
    }
}

async function syncToOneDrive() {
    try {
        const result = await invoke('sync_to_onedrive');
        alert(result);
        document.querySelector('.sync-status').classList.add('synced');
    } catch (e) {
        alert('Sync failed: ' + e);
    }
}

async function syncData() {
    await syncToOneDrive();
}

// ============ Voice Functions ============

function initVoice() {
    if ('webkitSpeechRecognition' in window) {
        voiceRecognition = new webkitSpeechRecognition();
        voiceRecognition.continuous = false;
        voiceRecognition.interimResults = false;
        
        voiceRecognition.onstart = () => {
            document.getElementById('btn-mic').classList.add('listening');
            document.getElementById('voice-status').textContent = 'Listening...';
        };
        
        voiceRecognition.onresult = (event) => {
            const transcript = event.results[0][0].transcript;
            document.getElementById('voice-input').value = transcript;
        };
        
        voiceRecognition.onend = () => {
            document.getElementById('btn-mic').classList.remove('listening');
            document.getElementById('voice-status').textContent = 'Click microphone to speak';
        };
    }
}

function toggleVoice() {
    const panel = document.getElementById('voice-panel');
    panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
}

function startVoice() {
    if (voiceRecognition) {
        voiceRecognition.start();
    }
}

async function sendVoiceCommand() {
    const command = document.getElementById('voice-input').value;
    if (!command || !users[0]) return;
    
    try {
        const result = await invoke('process_voice_command', { command, userId: users[0].id });
        alert(result);
        document.getElementById('voice-input').value = '';
        if (currentProject) await loadTasks(currentProject.id);
    } catch (e) {
        alert('Command error: ' + e);
    }
}

// ============ Drag and Drop ============

function setupDragAndDrop() {
    const columns = document.querySelectorAll('.column-body');
    
    columns.forEach(col => {
        col.addEventListener('dragover', (e) => {
            e.preventDefault();
            col.parentElement.classList.add('drag-over');
        });
        
        col.addEventListener('dragleave', () => {
            col.parentElement.classList.remove('drag-over');
        });
        
        col.addEventListener('drop', async (e) => {
            e.preventDefault();
            col.parentElement.classList.remove('drag-over');
            
            const taskId = e.dataTransfer.getData('text/plain');
            const columnId = col.dataset.columnId;
            
            if (taskId && columnId) {
                await invoke('move_task', { id: taskId, columnId, position: 0 });
                await loadTasks(currentProject.id);
            }
        });
    });
    
    const cards = document.querySelectorAll('.task-card');
    cards.forEach(card => {
        card.addEventListener('dragstart', (e) => {
            e.dataTransfer.setData('text/plain', card.dataset.taskId);
            card.classList.add('dragging');
        });
        
        card.addEventListener('dragend', () => {
            card.classList.remove('dragging');
        });
    });
}

// ============ Utilities ============

function formatDate(dateStr) {
    if (!dateStr) return '';
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function formatDuration(seconds) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

function getPriorityColor(priority) {
    const colors = { low: '#95a5a6', medium: '#ffc107', high: '#ff4757', urgent: '#e94560' };
    return colors[priority] || colors.medium;
}

function filterBy(filter) {
    // Implement filters
    console.log('Filter:', filter);
}

// Auto-save on task changes
document.getElementById('task-title-input')?.addEventListener('blur', saveTask);
document.getElementById('task-description')?.addEventListener('blur', saveTask);
document.getElementById('task-priority')?.addEventListener('change', saveTask);
document.getElementById('task-due-date')?.addEventListener('change', saveTask);
document.getElementById('task-start-date')?.addEventListener('change', saveTask);
document.getElementById('task-estimated-hours')?.addEventListener('change', saveTask);