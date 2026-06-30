import '@testing-library/jest-dom';
import { beforeAll, vi } from 'vitest';

// Mock ResizeObserver
class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}
globalThis.ResizeObserver = MockResizeObserver;

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === 'get_tabs') return Promise.resolve([]);
    return Promise.resolve();
  }),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockImplementation(() => {
    return Promise.resolve(() => {}); // returns unlisten function
  }),
}));

vi.mock('@tauri-apps/plugin-os', () => ({
  platform: vi.fn().mockResolvedValue('macos'),
}));

// Mock Tauri IPC
beforeAll(() => {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    value: {
      invoke: (cmd: string, _args: unknown) => {
        if (cmd === 'get_tabs') return Promise.resolve([]);
        return Promise.resolve();
      }
    }
  });
});
