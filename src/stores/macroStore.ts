import { createStore } from "solid-js/store";
import { invoke } from "@tauri-apps/api/core";

export interface MacroSummary {
  id: string;
  name: string;
  description: string;
  step_count: number;
  created_at: string;
  triggers: string[];
}

export interface ScheduledJob {
  macro_id: string;
  cron: string;
  job_uuid: string;
}

interface MacroState {
  macros: MacroSummary[];
  scheduled: ScheduledJob[];
  isRecording: boolean;
}

const [macroState, setMacroState] = createStore<MacroState>({
  macros: [],
  scheduled: [],
  isRecording: false,
});

export { macroState };

export const macroActions = {
  async init() {
    await this.fetchMacros();
    await this.fetchScheduled();
    const isRec = await invoke<boolean>("is_recording");
    setMacroState("isRecording", isRec);
  },

  async fetchMacros() {
    try {
      const macros = await invoke<MacroSummary[]>("list_macros");
      setMacroState("macros", macros);
    } catch (e) {
      console.error("Failed to list macros", e);
    }
  },

  async fetchScheduled() {
    try {
      const scheduled = await invoke<ScheduledJob[]>("list_scheduled_macros");
      setMacroState("scheduled", scheduled);
    } catch (e) {
      console.error("Failed to list scheduled jobs", e);
    }
  },

  async startRecording() {
    try {
      await invoke("start_recording");
      setMacroState("isRecording", true);
    } catch (e) {
      console.error("Failed to start recording", e);
    }
  },

  async stopRecording(name: string) {
    try {
      await invoke("stop_recording", { name });
      setMacroState("isRecording", false);
      await this.fetchMacros();
    } catch (e) {
      console.error("Failed to stop recording", e);
      throw e; // rethrow for UI feedback
    }
  },

  async cancelRecording() {
    try {
      await invoke("cancel_recording");
      setMacroState("isRecording", false);
    } catch (e) {
      console.error("Failed to cancel recording", e);
    }
  },

  async deleteMacro(id: string) {
    try {
      await invoke("delete_macro", { macroId: id });
      await this.fetchMacros();
      // If it was scheduled, also refresh schedule
      await this.fetchScheduled();
    } catch (e) {
      console.error("Failed to delete macro", e);
    }
  },

  async runMacro(id: string) {
    try {
      await invoke("run_macro_cmd", { macroId: id });
    } catch (e) {
      console.error("Failed to run macro", e);
    }
  },

  async scheduleMacro(id: string, cron: string) {
    try {
      await invoke("schedule_macro", { macroId: id, cron });
      await this.fetchScheduled();
    } catch (e) {
      console.error("Failed to schedule macro", e);
      throw e;
    }
  },

  async unscheduleMacro(id: string) {
    try {
      await invoke("unschedule_macro", { macroId: id });
      await this.fetchScheduled();
    } catch (e) {
      console.error("Failed to unschedule macro", e);
    }
  },
};
