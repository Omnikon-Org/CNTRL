import { getCurrentWindow } from '@tauri-apps/api/window';

export function WindowControls() {
  const appWindow = getCurrentWindow();
  return (
    <div class="window-controls">
      <button
        class="wc-btn wc-minimize"
        onClick={() => appWindow.minimize()}
        aria-label="Minimize"
      >─</button>
      <button
        class="wc-btn wc-maximize"
        onClick={async () => {
          const maximized = await appWindow.isMaximized();
          if (maximized) {
            await appWindow.unmaximize();
          } else {
            await appWindow.maximize();
          }
        }}
        aria-label="Maximize"
      >□</button>
      <button
        class="wc-btn wc-close"
        onClick={() => appWindow.close()}
        aria-label="Close"
      >✕</button>
    </div>
  );
}