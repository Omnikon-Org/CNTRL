import { describe, it, expect, vi } from "vitest";
import { eventBus } from "./events";

describe("EventBus", () => {
  it("should subscribe to and emit events with payloads", () => {
    const callback = vi.fn();
    eventBus.on("TAB_OPEN_NEW", callback);

    eventBus.emit("TAB_OPEN_NEW", { url: "https://example.com" });

    expect(callback).toHaveBeenCalledTimes(1);
    expect(callback).toHaveBeenCalledWith({ url: "https://example.com" });

    eventBus.off("TAB_OPEN_NEW", callback);
  });

  it("should subscribe to and emit void events", () => {
    const callback = vi.fn();
    eventBus.on("TAB_CLOSE_ACTIVE", callback);

    eventBus.emit("TAB_CLOSE_ACTIVE");

    expect(callback).toHaveBeenCalledTimes(1);
    expect(callback).toHaveBeenCalledWith(undefined);

    eventBus.off("TAB_CLOSE_ACTIVE", callback);
  });

  it("should unsubscribe correctly", () => {
    const callback = vi.fn();
    eventBus.on("TAB_REOPEN_LAST", callback);
    eventBus.off("TAB_REOPEN_LAST", callback);

    eventBus.emit("TAB_REOPEN_LAST");

    expect(callback).not.toHaveBeenCalled();
  });
  
  it("should handle multiple listeners", () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();
    
    eventBus.on("TAB_CLOSE_ACTIVE", callback1);
    eventBus.on("TAB_CLOSE_ACTIVE", callback2);
    
    eventBus.emit("TAB_CLOSE_ACTIVE");
    
    expect(callback1).toHaveBeenCalledTimes(1);
    expect(callback2).toHaveBeenCalledTimes(1);
    
    eventBus.off("TAB_CLOSE_ACTIVE", callback1);
    eventBus.off("TAB_CLOSE_ACTIVE", callback2);
  });
});
