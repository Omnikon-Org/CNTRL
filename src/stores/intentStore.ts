import { createStore } from 'solid-js/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface Step {
  NavigateTo?: string;
  ClickElement?: string;
  TypeText?: [string, string];
  ReadPage?: null;
  RunScript?: string;
  WaitFor?: number;
  ReportResult?: string;
}

export type TaskStatus = 'Queued' | 'Running' | 'Done' | 'Failed' | 'Cancelled';

export interface TaskResult {
  task_id: string;
  status: TaskStatus;
  steps: Step[];
  failed_at: number | null;
  message: string;
  data: string | null;
}

export const [intentState, setIntentState] = createStore({
  isCommandBarOpen: false,
  isExecuting: false,
  currentTask: null as TaskResult | null,
  currentStepIndex: -1,
  history: [] as string[],
});

export const intentActions = {
  toggleCommandBar() {
    setIntentState('isCommandBarOpen', !intentState.isCommandBarOpen);
  },

  closeCommandBar() {
    setIntentState('isCommandBarOpen', false);
  },

  async executeIntent(input: string) {
    if (!input.trim()) return;
    
    setIntentState('isExecuting', true);
    setIntentState('history', (prev) => [input, ...prev]);
    
    try {
      const result: TaskResult = await invoke('execute_intent', { input });
      setIntentState('currentTask', result);
    } catch (e) {
      console.error("Execution failed:", e);
      setIntentState('currentTask', {
        task_id: "error",
        status: "Failed",
        steps: [],
        failed_at: null,
        message: String(e),
        data: null
      });
    } finally {
      setIntentState('isExecuting', false);
    }
  }
};

// Listen for progress events
void listen('task-progress', (event: unknown) => {
  const payload = (event as { payload: [string, number, Step] }).payload;
  if (intentState.currentTask?.task_id === payload[0]) {
    setIntentState('currentStepIndex', payload[1]);
  }
});
