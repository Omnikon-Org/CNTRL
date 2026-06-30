import { getCurrentWindow } from '@tauri-apps/api/window';

/**
 * Custom window control buttons for Windows OS.
 * On macOS, the native traffic lights are used via titleBarStyle: Overlay.
 * This component renders only on Windows.
 */
export function WindowControls() {
  const appWindow = getCurrentWindow();

  return (
    <div class="window-controls">
      <button
        class="wc-btn wc-minimize"
        onClick={() => void appWindow.minimize()}
        aria-label="Minimize"
      >─</button>
      <button
        class="wc-btn wc-maximize"
        onClick={() => void (async () => {
          const maximized = await appWindow.isMaximized();
          if (maximized) {
            await appWindow.unmaximize();
          } else {
            await appWindow.maximize();
          }
        })()}
        aria-label="Maximize"
      >□</button>
      <button
        class="wc-btn wc-close"
        onClick={() => void appWindow.close()}
        aria-label="Close"
      >✕</button>
    </div>
  );
}
