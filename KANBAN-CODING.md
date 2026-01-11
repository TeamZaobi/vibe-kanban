# Kanban-Coding 扩展

> **基于 Vibe Kanban 的 Agentic 开发运行时扩展**

本 fork 在 [Vibe Kanban](https://github.com/BloopAI/vibe-kanban) 基础上添加了 **Kanban Agent Protocol (KAP)** 支持，使看板能够与 AI 编码代理进行双向通信。

## 扩展功能

### 🔴 红卡解阻塞 (MVP-1)

**问题**：多任务并行时，被阻塞的任务难以快速定位。

**解决方案**：
- **attention_state 字段**：任务卡片支持 `normal | needs_input | risk | waiting_external` 状态
- **KAP Watcher**：自动监听 `.kanban_agent/state.json`，派生任务阻塞状态
- **红边可视化**：`needs_input` 任务显示红色边框
- **只看红卡过滤器**：快速筛选阻塞任务

```
文件变更 → KAP Watcher → DB attention_state → 前端红卡 UI
```

### 📝 三文件任务骨架 (MVP-2)

**问题**：长任务容易目标漂移、缺乏结构化记录。

**解决方案**：
每个 workspace 自动生成 `.kanban_agent/` 目录：

| 文件 | 用途 |
|-----|------|
| `state.json` | KAP 状态（attention_state、loop status） |
| `task_plan.md` | 计划 + checkbox checklist |
| `notes.md` | 开发笔记 |
| `deliverable.md` | 交付物摘要 |

## 项目结构（扩展部分）

```
crates/services/src/services/
├── kap_watcher.rs      # 文件监听 → DB 状态更新
├── kap_initializer.rs  # 三文件幂等生成
└── mod.rs

frontend/src/
├── lib/attention.ts    # isRedCard / countRedCards
├── components/
│   ├── BoardFilter.tsx # 只看红卡过滤器
│   └── tasks/
│       └── TaskCard.tsx # 红边/橙边样式
```

## 开发状态

| 里程碑 | 状态 | 分支 |
|-------|------|-----|
| MVP-1 红卡解阻塞 | ✅ 完成 | `feature/mvp1-attention-state` |
| MVP-2A 三文件生成 | ✅ 后端完成 | `feature/mvp1-attention-state` |
| MVP-2B checkbox 进度 | ⏳ 待开始 | - |
| MVP-3 Loop Runner | 📋 计划中 | - |
| MVP-4 MCP 扩展 | 📋 计划中 | - |

## 快速验证

1. **红卡功能**：
   ```bash
   # 启动服务
   pnpm run dev
   
   # 在任务 worktree 中创建
   mkdir -p .kanban_agent
   echo '{"needs_input": true}' > .kanban_agent/state.json
   
   # 观察看板：任务卡片应显示红边
   ```

2. **三文件生成**：
   - 创建新任务 → 启动 attempt
   - 检查 worktree 内 `.kanban_agent/` 目录

## 相关规范

- [KAP v0.1 规范](docs/kap_v0.1.md)
- [Loop State Machine](docs/loop_state_machine_v0.1.md)

## 上游同步

本 fork 保持与上游 BloopAI/vibe-kanban 的 rebase 同步：

```bash
git fetch origin
git rebase origin/main
```

---

*Maintained by TeamZaobi - 基于 Vibe Kanban v0.0.146*
