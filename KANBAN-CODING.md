# Kanban-Coding

> **面向 Agentic 开发的看板运行时**  
> Fork of [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) | Based on v0.0.146

---

## 为什么要做这个项目？

### 问题背景

当前 AI 编码代理（Claude Code、Cursor、Gemini CLI 等）的典型使用模式是：

1. **人类下发任务** → Agent 自主执行 → 人类等待结果
2. Agent 运行中如果遇到问题，**没有好的通知机制**告诉人类"我卡住了"
3. 多任务并行时，人类需要**频繁轮询**所有任务的状态
4. 长任务容易**目标漂移**，Agent 可能偏离原始意图

**核心痛点**：Agent 与人类之间缺乏 **结构化的双向通信协议**。

### 设计理念

Kanban-Coding 的设计吸收了多个项目的思想：

#### 1. Vibe Kanban（基础）

[Vibe Kanban](https://github.com/BloopAI/vibe-kanban) 提供了优秀的多 Agent 看板界面：
- 任务/Attempt/Workspace 的数据模型
- 多 Agent 切换（Claude Code、Gemini、Codex 等）
- Worktree 隔离执行

我们在此基础上扩展双向通信能力。

#### 2. Plan-with-Files（核心思想）

受 [kodu-ai/plan-with-files](https://github.com/kodu-ai/plan-with-files) 启发：

> **文件作为 Agent 的外部记忆**  
> Agent 将计划和进度写入文件，而非仅保存在 context window 中

**关键收益**：
- **可追溯性**：所有决策和进度都有文件记录
- **抗遗忘**：长任务中 Agent 可以通过读取文件"回忆"之前的计划
- **人类可干预**：人类可以直接编辑计划文件来纠偏
- **多 Agent 协作**：不同 Agent 可以通过读写同一组文件传递信息

我们实现了 **三文件骨架**：
```
.kanban_agent/
├── task_plan.md     # 计划 + checkbox checklist
├── notes.md         # 开发笔记
└── deliverable.md   # 交付物摘要
```

#### 3. Kanban Agent Protocol (KAP)

我们定义了 `state.json` 作为 Agent 与看板之间的**通信协议**：

```json
{
  "attention_state": "needs_input",
  "needs_input": true,
  "loop": {
    "status": "blocked",
    "reason": "需要人类确认 API 设计"
  }
}
```

**信号链路**：
```
Agent 写入 state.json → KAP Watcher → DB attention_state → 前端红卡 UI
```

这让 Agent 能够 **主动向人类请求关注**，而不是等待人类轮询。

#### 4. Loop State Machine（规划中）

借鉴 Claude Code 的 "agentic loop" 概念，规划中的功能：
- Agent 可以暂停自己（`loop.status = "blocked"`）
- 人类解除阻塞后自动恢复
- 支持 `max_iterations` 限制防止无限循环

---

## 核心功能

### 🔴 红卡解阻塞 (MVP-1) ✅

**问题**：10+ 任务并行时，哪个任务卡住了？

**解决方案**：

| 组件 | 功能 |
|------|------|
| `attention_state` 字段 | 任务状态：`normal` / `needs_input` / `risk` |
| `KapWatcher` | 监听 `state.json` → 自动更新 DB |
| 红边样式 | `needs_input` 任务显示红色边框 |
| BoardFilter | "只看红卡"开关 + 计数 badge |

**效果**：打开看板 → 点击"只看红卡" → 立即定位所有阻塞任务。

### 📝 三文件任务骨架 (MVP-2) ⏳

**问题**：长任务（数小时/数天）容易目标漂移。

**解决方案**：

每个 workspace 创建时自动生成：

```
.kanban_agent/
├── state.json       # KAP 状态通信
├── task_plan.md     # 计划（含 checkbox checklist）
├── notes.md         # 开发笔记
└── deliverable.md   # 交付物摘要
```

**工作流**：
1. Agent 在 `task_plan.md` 中写下步骤清单
2. Agent 完成每步后勾选对应 checkbox
3. 人类可以随时查看进度、编辑计划纠偏
4. 任务完成后 `deliverable.md` 记录最终交付物

### 🔄 Loop Runner (MVP-3) 📋

**规划中**：
- Agent 可自主暂停 (`loop.status = "blocked"`)
- 人类输入后自动恢复 (`loop.status = "running"`)
- 防止无限循环 (`max_iterations`)

### 🔌 MCP 扩展 (MVP-4) 📋

**规划中**：
- 外部 Agent 可通过 MCP 协议读写 KAP 文件
- 支持跨看板/跨项目的 Agent 协作

---

## 项目结构

### 后端扩展

```
crates/services/src/services/
├── kap_watcher.rs      # 文件监听 → DB 状态更新
├── kap_initializer.rs  # 三文件幂等生成
└── mod.rs

crates/local-deployment/src/
└── container.rs        # workspace 创建时调用 KapInitializer

crates/db/src/models/
└── task.rs             # attention_state 字段
```

### 前端扩展

```
frontend/src/
├── lib/attention.ts           # isRedCard / countRedCards
├── components/
│   ├── BoardFilter.tsx        # 只看红卡过滤器
│   └── tasks/TaskCard.tsx     # 红边/橙边样式
└── pages/ProjectTasks.tsx     # BoardFilter 集成
```

---

## 开发状态

| 里程碑 | 状态 | 提交 |
|-------|------|-----|
| MVP-1 红卡解阻塞 | ✅ | 7 commits |
| MVP-2A 三文件生成 | ✅ 后端 | `0e3190d1` |
| MVP-2B checkbox 进度 | ⏳ | - |
| MVP-3 Loop Runner | 📋 | - |
| MVP-4 MCP 扩展 | 📋 | - |

---

## 快速开始

```bash
# 安装依赖
pnpm i

# 启动开发服务器
pnpm run dev

# 验证红卡功能
mkdir -p /path/to/worktree/.kanban_agent
echo '{"needs_input": true}' > /path/to/worktree/.kanban_agent/state.json
# 观察看板：任务应显示红边
```

---

## 上游同步

```bash
git fetch origin
git rebase origin/main
# 解决冲突后
pnpm run generate-types   # 重新生成 TS 类型
cargo test --workspace    # 验证测试
```

---

## 参考项目

- [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) - 基础看板系统
- [plan-with-files](https://github.com/kodu-ai/plan-with-files) - 文件作为外部记忆
- [Ralph Wiggum Loop](https://www.geoffreylitt.com/2025/04/08/how-i-vibe-code) - "Keep trying until it works" 策略，启发 Loop Runner 设计
- [InfiAgents](https://github.com/TeamZaobi/infiagents) - 多 Agent 协作架构，启发 MCP 扩展设计
- [Claude Code](https://claude.ai/code) - Agentic Loop 概念

---

*Maintained by TeamZaobi | 基于 Vibe Kanban v0.0.146*
