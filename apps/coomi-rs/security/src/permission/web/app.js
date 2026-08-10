// Coomi Agent Permission Manager - JavaScript

const API = '';

// === Tab 切换 ===
document.querySelectorAll('.tab').forEach(tab => {
    tab.addEventListener('click', () => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
        tab.classList.add('active');
        document.getElementById(tab.dataset.tab).classList.add('active');
        // 切换时刷新数据
        if (tab.dataset.tab === 'audit') refreshAudit();
        if (tab.dataset.tab === 'tools') refreshTools();
        if (tab.dataset.tab === 'permissions') refreshPermissions();
    });
});

// === API 调用 ===
async function api(path, opts = {}) {
    const res = await fetch(API + path, {
        headers: { 'Content-Type': 'application/json' },
        ...opts,
    });
    if (!res.ok) {
        const err = await res.text();
        throw new Error(err);
    }
    if (res.headers.get('content-type')?.includes('json')) {
        return res.json();
    }
    return null;
}

// === Badge ===
function badge(perm) {
    const cls = perm.toLowerCase();
    return `<span class="badge badge-${cls}">${perm}</span>`;
}

// === 权限管理 ===
let knownTools = [];

async function refreshPermissions() {
    try {
        const data = await api('/permissions');
        knownTools = data.known_tools || [];
        const rules = data.rules || {};

        // 默认模式
        document.getElementById('defaultMode').value = rules.default_mode || 'Ask';

        // 工具选择
        const sel = document.getElementById('toolSelect');
        const currentVal = sel.value;
        sel.innerHTML = '<option value="">-- 选择工具 --</option>';
        knownTools.forEach(t => {
            sel.innerHTML += `<option value="${t.name}">${t.name} - ${t.description}</option>`;
        });
        sel.value = currentVal;

        // 规则表
        renderRules(rules.tools || []);
    } catch (e) {
        console.error('refreshPermissions:', e);
    }
}

function renderRules(rules) {
    const tbody = document.getElementById('rulesBody');
    if (rules.length === 0) {
        tbody.innerHTML = '<tr><td colspan="4" style="text-align:center;color:var(--text-dim)">暂无规则</td></tr>';
        return;
    }
    tbody.innerHTML = rules.map(r => `
        <tr>
            <td><code>${r.tool}</code></td>
            <td>${badge(r.permission)}</td>
            <td>${r.note || '-'}</td>
            <td><button class="btn btn-delete" onclick="deleteRule('${r.tool}')">删除</button></td>
        </tr>
    `).join('');
}

// 添加规则
document.getElementById('addRule').addEventListener('click', async () => {
    const toolSel = document.getElementById('toolSelect').value;
    const custom = document.getElementById('customTool').value.trim();
    const tool = custom || toolSel;
    if (!tool) return alert('请选择或输入工具名');

    const permission = document.getElementById('rulePermission').value;
    const note = document.getElementById('ruleNote').value.trim();

    try {
        await api('/rules', {
            method: 'POST',
            body: JSON.stringify({ tool, permission, note }),
        });
        document.getElementById('customTool').value = '';
        document.getElementById('ruleNote').value = '';
        refreshPermissions();
    } catch (e) {
        alert('添加失败: ' + e.message);
    }
});

// 删除规则
window.deleteRule = async (tool) => {
    if (!confirm(`确定删除 ${tool} 的规则？`)) return;
    try {
        await api(`/rules/${encodeURIComponent(tool)}`, { method: 'DELETE' });
        refreshPermissions();
    } catch (e) {
        alert('删除失败: ' + e.message);
    }
};

// 保存默认模式
document.getElementById('saveDefault').addEventListener('click', async () => {
    const permission = document.getElementById('defaultMode').value;
    try {
        await api('/default', {
            method: 'POST',
            body: JSON.stringify({ permission }),
        });
        alert('已保存');
    } catch (e) {
        alert('保存失败: ' + e.message);
    }
});

// 重置全部
document.getElementById('resetAll').addEventListener('click', async () => {
    if (!confirm('确定重置所有规则为默认值？')) return;
    try {
        await api('/permissions/reset', { method: 'POST' });
        refreshPermissions();
    } catch (e) {
        alert('重置失败: ' + e.message);
    }
});

// === 审计日志 ===
async function refreshAudit() {
    try {
        const stats = await api('/audit/stats');
        document.getElementById('statTotal').textContent = stats.total;
        document.getElementById('statAllowed').textContent = stats.allowed;
        document.getElementById('statDenied').textContent = stats.denied;
        document.getElementById('statAsked').textContent = stats.asked;

        const logs = await api('/audit?limit=50');
        const tbody = document.getElementById('auditBody');
        if (logs.length === 0) {
            tbody.innerHTML = '<tr><td colspan="5" style="text-align:center;color:var(--text-dim)">暂无日志</td></tr>';
            return;
        }
        tbody.innerHTML = logs.reverse().map(l => `
            <tr>
                <td>${l.timestamp}</td>
                <td><code>${l.tool}</code></td>
                <td>${badge(l.decision)}</td>
                <td>${l.rule || '-'}</td>
                <td title="${l.context}">${l.context?.substring(0, 40) || '-'}</td>
            </tr>
        `).join('');
    } catch (e) {
        console.error('refreshAudit:', e);
    }
}

document.getElementById('refreshAudit').addEventListener('click', refreshAudit);
document.getElementById('clearAudit').addEventListener('click', async () => {
    if (!confirm('确定清空审计日志？')) return;
    try {
        await api('/audit', { method: 'DELETE' });
        refreshAudit();
    } catch (e) {
        alert('清空失败: ' + e.message);
    }
});

// === 工具列表 ===
async function refreshTools() {
    try {
        const tools = await api('/tools');
        const tbody = document.getElementById('toolsBody');
        tbody.innerHTML = tools.map(t => `
            <tr>
                <td><code>${t.name}</code></td>
                <td>${t.description}</td>
            </tr>
        `).join('');
    } catch (e) {
        console.error('refreshTools:', e);
    }
}

// === 初始化 ===
refreshPermissions();