//! KAP (Kanban Agent Protocol) directory and file initializer.
//!
//! Creates the `.kanban_agent/` directory structure and planning files
//! when a workspace is created. All operations are idempotent - existing
//! files are never overwritten.

use std::path::Path;

use chrono::Utc;
use thiserror::Error;
use tokio::fs;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum KapInitError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// KAP directory and file initializer
pub struct KapInitializer;

impl KapInitializer {
    /// Initialize the `.kanban_agent` directory with planning files.
    /// 
    /// This is idempotent - existing files are never overwritten.
    /// The function creates:
    /// - `.kanban_agent/state.json` (KAP state file)
    /// - `.kanban_agent/task_plan.md`
    /// - `.kanban_agent/notes.md`
    /// - `.kanban_agent/deliverable.md`
    pub async fn initialize(
        worktree_path: &Path,
        task_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), KapInitError> {
        let kap_dir = worktree_path.join(".kanban_agent");

        // Create directory if it doesn't exist
        if !kap_dir.exists() {
            fs::create_dir_all(&kap_dir).await?;
            info!("Created KAP directory: {:?}", kap_dir);
        }

        // Initialize all files (idempotent)
        Self::init_state_json(&kap_dir, task_id, workspace_id).await?;
        Self::init_task_plan(&kap_dir, task_id, workspace_id).await?;
        Self::init_notes(&kap_dir, task_id, workspace_id).await?;
        Self::init_deliverable(&kap_dir, task_id, workspace_id).await?;

        info!(
            "KAP initialization complete for workspace {} in {:?}",
            workspace_id, worktree_path
        );

        Ok(())
    }

    /// Initialize state.json if it doesn't exist
    async fn init_state_json(
        kap_dir: &Path,
        task_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), KapInitError> {
        let path = kap_dir.join("state.json");
        
        if path.exists() {
            debug!("state.json already exists, skipping");
            return Ok(());
        }

        let state = serde_json::json!({
            "schema_version": "1.0",
            "task_id": task_id.to_string(),
            "attempt_id": workspace_id.to_string(),
            "attention_state": "normal",
            "needs_input": false,
            "loop": {
                "status": "idle"
            },
            "created_at": Utc::now().to_rfc3339()
        });

        let content = serde_json::to_string_pretty(&state)?;
        fs::write(&path, content).await?;
        debug!("Created state.json");

        Ok(())
    }

    /// Initialize task_plan.md if it doesn't exist
    async fn init_task_plan(
        kap_dir: &Path,
        task_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), KapInitError> {
        let path = kap_dir.join("task_plan.md");

        if path.exists() {
            debug!("task_plan.md already exists, skipping");
            return Ok(());
        }

        let content = format!(
            r#"---
schema_version: "1.0"
task_id: "{task_id}"
workspace_id: "{workspace_id}"
created_at: "{timestamp}"
---

# 任务计划

## 目标
[在此描述任务目标]

## 步骤清单
- [ ] Step 1: 分析需求
- [ ] Step 2: 设计方案
- [ ] Step 3: 实现代码
- [ ] Step 4: 测试验证
- [ ] Step 5: 代码审查

## 验收标准
[定义完成条件]

## 风险与阻塞
[记录潜在风险和当前阻塞]
"#,
            task_id = task_id,
            workspace_id = workspace_id,
            timestamp = Utc::now().to_rfc3339()
        );

        fs::write(&path, content).await?;
        debug!("Created task_plan.md");

        Ok(())
    }

    /// Initialize notes.md if it doesn't exist
    async fn init_notes(
        kap_dir: &Path,
        task_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), KapInitError> {
        let path = kap_dir.join("notes.md");

        if path.exists() {
            debug!("notes.md already exists, skipping");
            return Ok(());
        }

        let content = format!(
            r#"---
task_id: "{task_id}"
workspace_id: "{workspace_id}"
created_at: "{timestamp}"
---

# 开发笔记

## 进展记录
<!-- 按时间顺序记录重要发现、决策和阻塞 -->

### {date}
- 任务开始

## 技术笔记
<!-- 记录技术细节、代码片段、参考链接 -->

## 待办事项
<!-- 临时待办，完成后移至 task_plan.md 的 checklist -->
"#,
            task_id = task_id,
            workspace_id = workspace_id,
            timestamp = Utc::now().to_rfc3339(),
            date = Utc::now().format("%Y-%m-%d")
        );

        fs::write(&path, content).await?;
        debug!("Created notes.md");

        Ok(())
    }

    /// Initialize deliverable.md if it doesn't exist
    async fn init_deliverable(
        kap_dir: &Path,
        task_id: Uuid,
        workspace_id: Uuid,
    ) -> Result<(), KapInitError> {
        let path = kap_dir.join("deliverable.md");

        if path.exists() {
            debug!("deliverable.md already exists, skipping");
            return Ok(());
        }

        let content = format!(
            r#"---
task_id: "{task_id}"
workspace_id: "{workspace_id}"
created_at: "{timestamp}"
---

# 交付物

## 变更摘要
<!-- 列出本任务的主要变更 -->

### 新增文件
- 

### 修改文件
- 

### 删除文件
- 

## 测试验证
<!-- 描述如何验证变更 -->

### 自动测试
- [ ] 单元测试通过
- [ ] 集成测试通过

### 手动验证
- [ ] 功能测试完成

## 备注
<!-- 其他需要说明的事项 -->
"#,
            task_id = task_id,
            workspace_id = workspace_id,
            timestamp = Utc::now().to_rfc3339()
        );

        fs::write(&path, content).await?;
        debug!("Created deliverable.md");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize_creates_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let worktree_path = temp_dir.path();
        let task_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        KapInitializer::initialize(worktree_path, task_id, workspace_id)
            .await
            .unwrap();

        let kap_dir = worktree_path.join(".kanban_agent");
        assert!(kap_dir.exists());
        assert!(kap_dir.join("state.json").exists());
        assert!(kap_dir.join("task_plan.md").exists());
        assert!(kap_dir.join("notes.md").exists());
        assert!(kap_dir.join("deliverable.md").exists());
    }

    #[tokio::test]
    async fn test_initialize_is_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let worktree_path = temp_dir.path();
        let task_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        // First initialization
        KapInitializer::initialize(worktree_path, task_id, workspace_id)
            .await
            .unwrap();

        // Modify a file
        let plan_path = worktree_path.join(".kanban_agent/task_plan.md");
        let original_content = fs::read_to_string(&plan_path).await.unwrap();
        fs::write(&plan_path, "# Custom content").await.unwrap();

        // Second initialization should NOT overwrite
        KapInitializer::initialize(worktree_path, task_id, workspace_id)
            .await
            .unwrap();

        let content_after = fs::read_to_string(&plan_path).await.unwrap();
        assert_eq!(content_after, "# Custom content");
        assert_ne!(content_after, original_content);
    }

    #[tokio::test]
    async fn test_state_json_contains_correct_ids() {
        let temp_dir = TempDir::new().unwrap();
        let worktree_path = temp_dir.path();
        let task_id = Uuid::new_v4();
        let workspace_id = Uuid::new_v4();

        KapInitializer::initialize(worktree_path, task_id, workspace_id)
            .await
            .unwrap();

        let state_path = worktree_path.join(".kanban_agent/state.json");
        let content = fs::read_to_string(&state_path).await.unwrap();
        let state: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(state["task_id"], task_id.to_string());
        assert_eq!(state["attempt_id"], workspace_id.to_string());
        assert_eq!(state["attention_state"], "normal");
    }
}
