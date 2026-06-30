import { Component, createSignal, onMount, onCleanup, Show, For } from 'solid-js';
import { intentState, intentActions } from '../stores/intentStore';
import './CommandBar.css';

export const CommandBar: Component = () => {
  const [input, setInput] = createSignal('');
  // eslint-disable-next-line no-unassigned-vars
  let inputRef: HTMLInputElement | undefined;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape') {
      intentActions.closeCommandBar();
    } else if (e.key === 'Enter') {
      if (input().trim()) {
        void intentActions.executeIntent(input());
        setInput('');
      }
    }
  };

  onMount(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        intentActions.toggleCommandBar();
        if (intentState.isCommandBarOpen) {
          setTimeout(() => inputRef?.focus(), 50);
        }
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    onCleanup(() => window.removeEventListener('keydown', handleGlobalKeyDown));
  });

  return (
    <Show when={intentState.isCommandBarOpen}>
      <div class="command-bar-overlay" onClick={() => intentActions.closeCommandBar()}>
        <div class="command-bar-container" onClick={(e) => e.stopPropagation()}>
          <div class="command-bar-header">
            <input
              ref={inputRef}
              class="command-bar-input"
              type="text"
              placeholder="What do you want to do? (e.g. 'go to reddit', 'mute')"
              value={input()}
              onInput={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              autofocus
            />
            <div class={`status-indicator ${intentState.isExecuting ? 'executing' : ''}`}></div>
          </div>
          
          <Show when={intentState.currentTask}>
            <div class="command-bar-results">
              <div class="task-status">
                Status: <span class={`status-badge ${intentState.currentTask?.status?.toLowerCase() ?? ''}`}>
                  {intentState.currentTask?.status}
                </span>
              </div>
              <div class="task-message">
                {intentState.currentTask?.message}
              </div>
              
              <Show when={intentState.currentTask?.steps && intentState.currentTask.steps.length > 0}>
                <div class="task-steps">
                  <For each={intentState.currentTask?.steps}>
                    {(step, index) => {
                      const isActive = () => index() === intentState.currentStepIndex;
                      const isPast = () => index() < intentState.currentStepIndex;
                      const isString = typeof step === 'string';
                      const stepRecord = step as Record<string, unknown>;
                      const stepName = isString ? step : Object.keys(stepRecord)[0];
                      const stepValue = isString ? null : Object.values(stepRecord)[0];
                      
                      return (
                        <div class={`step-item ${isActive() ? 'active' : ''} ${isPast() ? 'past' : ''}`}>
                          <span class="step-icon">
                            {isPast() ? '✓' : isActive() ? '↻' : '○'}
                          </span>
                          <span class="step-name">{stepName}</span>
                          <Show when={stepValue !== null}>
                            <span class="step-value">{String(stepValue)}</span>
                          </Show>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </Show>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
