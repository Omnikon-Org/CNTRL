export type EventMap = {
  TAB_OPEN_NEW: { url: string; isBackground?: boolean };
  TAB_CLOSE_ACTIVE: void;
  TAB_REOPEN_LAST: void;
};

export type EventKey = keyof EventMap;

type EventCallback<T> = (payload: T) => void;

class EventBus {
  private listeners: Map<EventKey, Set<EventCallback<any>>> = new Map();

  /**
   * Subscribe to an event.
   */
  on<K extends EventKey>(event: K, callback: EventCallback<EventMap[K]>) {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  /**
   * Unsubscribe from an event.
   */
  off<K extends EventKey>(event: K, callback: EventCallback<EventMap[K]>) {
    const callbacks = this.listeners.get(event);
    if (callbacks) {
      callbacks.delete(callback);
    }
  }

  /**
   * Emit an event to all subscribers.
   */
  emit<K extends EventKey>(event: K, ...args: EventMap[K] extends void ? [undefined?] : [EventMap[K]]) {
    const callbacks = this.listeners.get(event);
    if (!callbacks) return;
    const payload = args[0] as EventMap[K];
    callbacks.forEach((cb) => {
      try {
        cb(payload);
      } catch (err) {
        console.error(`Error in event listener for ${event}:`, err);
      }
    });
  }
}

export const eventBus = new EventBus();
