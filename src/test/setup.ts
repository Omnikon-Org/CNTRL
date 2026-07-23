import "@testing-library/jest-dom";
import { beforeAll, vi } from "vitest";
class MockResizeObserver {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}
globalThis.ResizeObserver = MockResizeObserver;
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    switch (cmd) {
      case "get_tabs":
        return Promise.resolve([]);
      case "get_api_key_status":
        return Promise.resolve("");
      case "health_check_all":
        return Promise.resolve({});
      case "get_hf_models":
        return Promise.resolve([]);
      case "get_openrouter_free_models":
        return Promise.resolve([]);
      default:
        return Promise.resolve(undefined);
    }
  }),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockImplementation(() => {
    return Promise.resolve(() => {
      
    });
  }),
}));
vi.mock("@tauri-apps/plugin-os", () => ({
  platform: vi.fn().mockResolvedValue("macos"),
}));

vi.mock("@tauri-apps/plugin-shell", () => ({
  open: vi.fn().mockResolvedValue(undefined),
}));

beforeAll(() => {
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    value: {
      invoke: (cmd: string, _args: unknown): Promise<unknown> => {
        switch (cmd) {
          case "get_tabs":
            return Promise.resolve([]);
          case "get_api_key_status":
            return Promise.resolve("");
          case "health_check_all":
            return Promise.resolve({});
          default:
            return Promise.resolve(undefined);
        }
      },
    },
  });
});