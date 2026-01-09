// Attention state utilities for red card filtering
// Single source of truth for attention_state logic

import type { TaskWithAttemptStatus } from 'shared/types';
import type { KanbanColumnItem } from '@/components/tasks/TaskKanbanBoard';

/**
 * Get the attention_state from a task
 */
export function getAttentionState(
    task: TaskWithAttemptStatus
): string | null | undefined {
    return task.attention_state;
}

/**
 * Check if a task is a "red card" (needs_input)
 */
export function isRedCard(task: TaskWithAttemptStatus): boolean {
    return getAttentionState(task) === 'needs_input';
}

/**
 * Check if a task is a "risk card"
 */
export function isRiskCard(task: TaskWithAttemptStatus): boolean {
    return getAttentionState(task) === 'risk';
}

/**
 * Check if a KanbanColumnItem contains a red card
 */
export function isRedCardItem(item: KanbanColumnItem): boolean {
    if (item.type === 'task') {
        return isRedCard(item.task);
    }
    // Shared tasks don't have attention_state in MVP-1
    return false;
}

/**
 * Count red cards in a collection
 */
export function countRedCards(tasks: TaskWithAttemptStatus[]): number {
    return tasks.filter(isRedCard).length;
}
