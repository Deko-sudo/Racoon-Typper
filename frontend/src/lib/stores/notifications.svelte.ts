export type NotificationType = 'SUCCESS' | 'WARNING';

export interface Notification {
  id: number;
  type: NotificationType;
  message: string;
  timestamp: number;
}

export function createNotificationStore(timeoutMs = 5_000) {
  let notifications = $state<Notification[]>([]);
  let nextId = 0;

  function add(type: NotificationType, message: string) {
    const id = ++nextId;
    notifications = [...notifications, { id, type, message, timestamp: Date.now() }];
    setTimeout(() => {
      notifications = notifications.filter((notification) => notification.id !== id);
    }, timeoutMs);
  }

  return {
    get notifications() {
      return notifications;
    },
    add,
  };
}
