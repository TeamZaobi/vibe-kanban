//! KAP (Kanban Agent Protocol) file watcher.
//!
//! Monitors `.kanban_agent/state.json` files in worktrees and updates
//! the Task's `attention_state` field in the database accordingly.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use db::models::task::{AttentionState, Task};
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use serde::Deserialize;
use sqlx::SqlitePool;
use thiserror::Error;
use tokio::sync::mpsc;
use uuid::Uuid;

/// KAP state.json schema
#[derive(Debug, Deserialize)]
pub struct KapStateJson {
    #[serde(default)]
    pub schema_version: String,
    pub task_id: Uuid,
    pub attempt_id: Uuid,
    #[serde(default)]
    pub attention_state: String,
    #[serde(default)]
    pub needs_input: bool,
    #[serde(default)]
    pub loop_status: Option<LoopStatus>,
}

#[derive(Debug, Deserialize, Default)]
pub struct LoopStatus {
    #[serde(default)]
    pub status: String,
}

#[derive(Debug, Error)]
pub enum KapWatcherError {
    #[error(transparent)]
    Notify(#[from] notify::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error("Watcher channel closed")]
    ChannelClosed,
}

/// KAP file watcher service
pub struct KapWatcher {
    pool: SqlitePool,
    worktrees_root: PathBuf,
    debounce_duration: Duration,
}

impl KapWatcher {
    /// Create a new KAP watcher
    pub fn new(pool: SqlitePool, worktrees_root: PathBuf) -> Self {
        Self {
            pool,
            worktrees_root,
            debounce_duration: Duration::from_millis(300),
        }
    }

    /// Create a new KAP watcher with custom debounce duration
    pub fn with_debounce(pool: SqlitePool, worktrees_root: PathBuf, debounce: Duration) -> Self {
        Self {
            pool,
            worktrees_root,
            debounce_duration: debounce,
        }
    }

    /// Start the KAP watcher as a background task
    pub async fn start(self) -> Result<(), KapWatcherError> {
        let (tx, mut rx) = mpsc::channel::<DebounceEventResult>(64);

        // Create the debounced watcher
        let mut debouncer: Debouncer<RecommendedWatcher, RecommendedCache> = new_debouncer(
            self.debounce_duration,
            None,
            move |res: DebounceEventResult| {
                let tx = tx.clone();
                // Use blocking send since we're in a sync context
                if let Err(e) = tx.blocking_send(res) {
                    tracing::warn!("Failed to send watcher event: {}", e);
                }
            },
        )?;

        // Watch the worktrees root recursively
        debouncer.watch(&self.worktrees_root, RecursiveMode::Recursive)?;
        tracing::info!(
            "KAP watcher started monitoring: {:?}",
            self.worktrees_root
        );

        // Keep debouncer alive
        let _debouncer = Arc::new(debouncer);

        // Process events
        while let Some(result) = rx.recv().await {
            match result {
                Ok(events) => {
                    for event in events {
                        // Filter for state.json files in .kanban_agent directories
                        for path in &event.paths {
                            if self.is_kap_state_file(path) {
                                if let Err(e) = self.handle_state_change(path).await {
                                    tracing::warn!(
                                        "Failed to handle KAP state change for {:?}: {}",
                                        path,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
                Err(errors) => {
                    for error in errors {
                        tracing::error!("KAP watcher error: {}", error);
                    }
                }
            }
        }

        Err(KapWatcherError::ChannelClosed)
    }

    /// Check if a path is a KAP state.json file
    fn is_kap_state_file(&self, path: &Path) -> bool {
        is_kap_state_file(path)
    }

    /// Handle a state.json change event
    async fn handle_state_change(&self, path: &Path) -> Result<(), KapWatcherError> {
        // Read and parse the state file
        let content = match tokio::fs::read_to_string(path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // File was deleted, ignore
                tracing::debug!("KAP state file deleted: {:?}", path);
                return Ok(());
            }
            Err(e) => return Err(e.into()),
        };

        let state: KapStateJson = match serde_json::from_str(&content) {
            Ok(s) => s,
            Err(e) => {
                // JSON parse failed, might be a partial write
                // Log and skip, don't propagate error
                tracing::debug!("Failed to parse KAP state.json at {:?}: {}", path, e);
                return Ok(());
            }
        };

        // Derive attention_state from KAP state
        let attention_state = self.derive_attention_state(&state);

        // Update the task in database
        tracing::info!(
            "Updating task {} attention_state to {:?} (needs_input={}, loop_status={:?})",
            state.task_id,
            attention_state,
            state.needs_input,
            state.loop_status.as_ref().map(|l| &l.status)
        );

        Task::update_attention_state(&self.pool, state.task_id, attention_state).await?;

        Ok(())
    }

    /// Derive AttentionState from KAP state
    fn derive_attention_state(&self, state: &KapStateJson) -> AttentionState {
        derive_attention_state_from_kap(state)
    }
}

/// Derive AttentionState from KAP state (standalone function for testing)
pub fn derive_attention_state_from_kap(state: &KapStateJson) -> AttentionState {
    // Rule 1: needs_input=true OR loop.status="blocked" => NeedsInput
    if state.needs_input {
        return AttentionState::NeedsInput;
    }

    if let Some(ref loop_status) = state.loop_status {
        match loop_status.status.as_str() {
            "blocked" => return AttentionState::NeedsInput,
            "failed" => return AttentionState::Risk,
            _ => {}
        }
    }

    // Rule 2: Check attention_state field directly (agent might set it)
    match state.attention_state.as_str() {
        "needs_input" => AttentionState::NeedsInput,
        "risk" => AttentionState::Risk,
        "waiting_external" => AttentionState::WaitingExternal,
        _ => AttentionState::Normal,
    }
}

/// Check if a path is a KAP state.json file (standalone function for testing)
pub fn is_kap_state_file(path: &Path) -> bool {
    // Check if path ends with .kanban_agent/state.json
    if let Some(file_name) = path.file_name() {
        if file_name != "state.json" {
            return false;
        }
    } else {
        return false;
    }

    if let Some(parent) = path.parent() {
        if let Some(dir_name) = parent.file_name() {
            return dir_name == ".kanban_agent";
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_attention_state_needs_input() {
        let state = KapStateJson {
            schema_version: "1.0".to_string(),
            task_id: Uuid::new_v4(),
            attempt_id: Uuid::new_v4(),
            attention_state: "normal".to_string(),
            needs_input: true,
            loop_status: None,
        };

        assert_eq!(
            derive_attention_state_from_kap(&state),
            AttentionState::NeedsInput
        );
    }

    #[test]
    fn test_derive_attention_state_blocked() {
        let state = KapStateJson {
            schema_version: "1.0".to_string(),
            task_id: Uuid::new_v4(),
            attempt_id: Uuid::new_v4(),
            attention_state: "normal".to_string(),
            needs_input: false,
            loop_status: Some(LoopStatus {
                status: "blocked".to_string(),
            }),
        };

        assert_eq!(
            derive_attention_state_from_kap(&state),
            AttentionState::NeedsInput
        );
    }

    #[test]
    fn test_derive_attention_state_failed() {
        let state = KapStateJson {
            schema_version: "1.0".to_string(),
            task_id: Uuid::new_v4(),
            attempt_id: Uuid::new_v4(),
            attention_state: "normal".to_string(),
            needs_input: false,
            loop_status: Some(LoopStatus {
                status: "failed".to_string(),
            }),
        };

        assert_eq!(
            derive_attention_state_from_kap(&state),
            AttentionState::Risk
        );
    }

    #[test]
    fn test_is_kap_state_file() {
        assert!(is_kap_state_file(Path::new(
            "/worktrees/abc123/.kanban_agent/state.json"
        )));
        assert!(!is_kap_state_file(Path::new(
            "/worktrees/abc123/.kanban_agent/questions.json"
        )));
        assert!(!is_kap_state_file(Path::new(
            "/worktrees/abc123/state.json"
        )));
    }

    #[test]
    fn test_derive_attention_state_from_field() {
        let state = KapStateJson {
            schema_version: "1.0".to_string(),
            task_id: Uuid::new_v4(),
            attempt_id: Uuid::new_v4(),
            attention_state: "waiting_external".to_string(),
            needs_input: false,
            loop_status: None,
        };

        assert_eq!(
            derive_attention_state_from_kap(&state),
            AttentionState::WaitingExternal
        );
    }
}

