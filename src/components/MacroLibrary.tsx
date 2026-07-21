import { Component, createSignal, onMount, For, Show } from "solid-js";
import { macroState, macroActions } from "../stores/macroStore";
import "./MacroLibrary.css";

interface MacroLibraryProps {
  onClose: () => void;
}

export const MacroLibrary: Component<MacroLibraryProps> = (props) => {
  const [macroName, setMacroName] = createSignal("");
  const [scheduleCron, setScheduleCron] = createSignal("");
  const [activeScheduleId, setActiveScheduleId] = createSignal<string | null>(null);

  onMount(() => {
    macroActions.fetchMacros();
    macroActions.fetchScheduled();
  });

  const handleStopRecording = async () => {
    if (!macroName()) return;
    try {
      await macroActions.stopRecording(macroName());
      setMacroName("");
    } catch {
      // Error handled by store, could show toast here
    }
  };

  return (
    <div class="macro-library-overlay" onClick={props.onClose}>
      <div class="macro-library-modal" onClick={(e) => e.stopPropagation()}>
        <div class="macro-library-header">
          <h2>Macro Library (Vibe Automations)</h2>
          <button class="macro-library-close" onClick={props.onClose}>
            ✕
          </button>
        </div>

        <div class="macro-library-content">
          {/* Recording Controls */}
          <div class="macro-recording-controls">
            <Show
              when={macroState.isRecording}
              fallback={
                <button class="btn-primary" onClick={() => macroActions.startRecording()}>
                  ● Start Recording
                </button>
              }
            >
              <div class="macro-save-form">
                <div class="schedule-badge">● RECORDING</div>
                <input
                  type="text"
                  class="macro-input"
                  placeholder="Name your macro..."
                  value={macroName()}
                  onInput={(e) => setMacroName(e.currentTarget.value)}
                />
                <button class="btn-primary" onClick={handleStopRecording} disabled={!macroName()}>
                  Save Macro
                </button>
                <button class="btn-danger" onClick={() => macroActions.cancelRecording()}>
                  Cancel
                </button>
              </div>
            </Show>
          </div>

          {/* Macro List */}
          <div class="macro-list">
            <Show when={macroState.macros.length > 0} fallback={<div class="empty-state">No macros saved yet. Start recording to create one.</div>}>
              <For each={macroState.macros}>
                {(macro) => {
                  const schedule = () => macroState.scheduled.find((s) => s.macro_id === macro.id);
                  const isScheduling = () => activeScheduleId() === macro.id;

                  return (
                    <div class="macro-item">
                      <div class="macro-item-header">
                        <div>
                          <div class="macro-title">{macro.name}</div>
                          <div class="macro-meta">
                            {macro.step_count} steps • ID: {macro.id.slice(0, 8)}
                          </div>
                        </div>
                        <div class="macro-actions">
                          <Show when={schedule()}>
                            <div class="schedule-badge">⏱ {schedule()?.cron}</div>
                            <button class="btn-secondary" onClick={() => macroActions.unscheduleMacro(macro.id)}>
                              Unschedule
                            </button>
                          </Show>

                          <Show when={!schedule()}>
                            <Show
                              when={isScheduling()}
                              fallback={
                                <button class="btn-secondary" onClick={() => setActiveScheduleId(macro.id)}>
                                  Schedule
                                </button>
                              }
                            >
                              <div class="macro-schedule-form">
                                <input
                                  type="text"
                                  placeholder="* * * * * *"
                                  value={scheduleCron()}
                                  onInput={(e) => setScheduleCron(e.currentTarget.value)}
                                />
                                <button
                                  class="btn-primary"
                                  onClick={async () => {
                                    if (scheduleCron()) {
                                      await macroActions.scheduleMacro(macro.id, scheduleCron());
                                      setActiveScheduleId(null);
                                      setScheduleCron("");
                                    }
                                  }}
                                >
                                  Set
                                </button>
                                <button class="btn-secondary" onClick={() => setActiveScheduleId(null)}>
                                  Cancel
                                </button>
                              </div>
                            </Show>
                          </Show>

                          <button class="btn-primary" onClick={() => macroActions.runMacro(macro.id)}>
                            ▶ Play
                          </button>
                          <button class="btn-danger" onClick={() => macroActions.deleteMacro(macro.id)}>
                            Delete
                          </button>
                        </div>
                      </div>
                    </div>
                  );
                }}
              </For>
            </Show>
          </div>
        </div>
      </div>
    </div>
  );
};
