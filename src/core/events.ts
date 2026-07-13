export type EventMap = {
  TAB_OPEN_NEW: { url: string; isBackground?: boolean };
  TAB_CLOSE_ACTIVE: void;
  TAB_REOPEN_LAST: void;
};

export type EventKey = keyof EventMap;

type EventCallback<T> = (payload: T) => void;

class EventBus {
  private listeners: {
    [K in EventKey]?: Array<EventCallback<EventMap[K]>>;
  } = {};

  /**
   * Subscribe to an event.
   */
  on<K extends EventKey>(event: K, callback: EventCallback<EventMap[K]>) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }
    this.listeners[event]!.push(callback);
  }

  /**
   * Unsubscribe from an event.
   */
  off<K extends EventKey>(event: K, callback: EventCallback<EventMap[K]>) {
    if (!this.listeners[event]) return;
    this.listeners[event] = this.listeners[event]!.filter((cb) => cb !== callback);
  }

  /**
   * Emit an event to all subscribers.
   */
  emit<K extends EventKey>(event: K, ...args: EventMap[K] extends void ? [undefined?] : [EventMap[K]]) {
    if (!this.listeners[event]) return;
    const payload = args[0] as EventMap[K];
    this.listeners[event]!.forEach((cb) => {
      try {
        cb(payload);
      } catch (err) {
        console.error(`Error in event listener for ${event}:`, err);
      }
    });
  }
}

export const eventBus = new EventBus();
