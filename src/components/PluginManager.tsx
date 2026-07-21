import { Component, For, Show, createSignal, onMount } from "solid-js";
import "./PluginManager.css";

// unused interface removed

interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description: string | null;
  entrypoint: string;
  permissions: string[];
}

const IconPlug = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M12 22v-5" />
    <path d="M9 8V2" />
    <path d="M15 8V2" />
    <path d="M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />
  </svg>
);

const IconShield = () => (
  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2-1 4-2 7-2 2.94 0 4.96 1 6.94 2a1 1 0 0 1 1.06 1v7Z" />
  </svg>
);

export const PluginManager: Component = () => {
  const [plugins, setPlugins] = createSignal<PluginManifest[]>([]);
  const [isLoading, setIsLoading] = createSignal(true);

  onMount(() => {
    // In a real app, we would invoke a Tauri command to list loaded plugins
    // For the UI stub integration, we simulate fetching installed plugins.
    setTimeout(() => {
      setPlugins([
        {
          id: "com.vibe.example-plugin",
          name: "Vibe Example Plugin",
          version: "1.0.0",
          description: "An example plugin to demonstrate the WASM Sandbox boundaries.",
          entrypoint: "module.wasm",
          permissions: ["IntentExecution", "NetworkAccess"]
        }
      ]);
      setIsLoading(false);
    }, 500);
  });

  return (
    <section class="sp-card" aria-labelledby="plugin-heading">
      <div class="sp-card-header">
        <span class="sp-card-icon"><IconPlug /></span>
        <h2 class="sp-card-title" id="plugin-heading">Plugin SDK (WASM Sandbox)</h2>
      </div>
      <p class="sp-hint" style="margin-bottom: 1rem;">
        Unprivileged WASM plugins can extend CNTRL safely. Plugins run in a secure sandbox without arbitrary filesystem or shell access.
      </p>

      <div class="pm-plugin-list">
        <Show when={!isLoading()} fallback={<p class="sp-hint">Loading plugins...</p>}>
          <Show when={plugins().length > 0} fallback={<p class="sp-hint">No plugins installed.</p>}>
            <For each={plugins()}>
              {(plugin) => (
                <div class="pm-plugin-item">
                  <div class="pm-plugin-header">
                    <h3 class="pm-plugin-name">{plugin.name}</h3>
                    <span class="pm-plugin-version">v{plugin.version}</span>
                  </div>
                  <p class="pm-plugin-desc">{plugin.description || "No description provided."}</p>
                  
                  <div class="pm-plugin-permissions">
                    <div class="pm-permissions-title">
                      <IconShield />
                      <span>Requested Permissions</span>
                    </div>
                    <ul class="pm-permissions-list">
                      <For each={plugin.permissions}>
                        {(perm) => (
                          <li class="pm-permission-badge">{perm}</li>
                        )}
                      </For>
                    </ul>
                  </div>
                </div>
              )}
            </For>
          </Show>
        </Show>
      </div>

      <div class="sp-row" style="margin-top: 1rem;">
        <button class="sp-btn sp-btn-secondary" disabled>
          Install Plugin (.vibe-plugin)
        </button>
      </div>
    </section>
  );
};
