<script lang="ts">
  // NotificationStack — правая боковая панель уведомлений.
  // Максимум 3 одновременно, автоудаление через 5 секунд.

  type NotificationType = string;
  interface Notification {
    id: number;
    type: NotificationType;
    message: string;
    timestamp: number;
  }

  let { notifications = [] }: { notifications?: Notification[] } = $props();

  // Keep only last 3
  let visible = $derived(notifications.slice(-3));
</script>

{#if visible.length > 0}
  <div class="notification-stack">
    {#each visible as n (n.id)}
      <div class="notification {n.type.toLowerCase()}">
        <span class="notification-icon">
          {#if n.type === 'INFO'}ℹ️{:else if n.type === 'WARNING'}⚠️{:else}✅{/if}
        </span>
        <span class="notification-msg">{n.message}</span>
      </div>
    {/each}
  </div>
{/if}

<style>
  .notification-stack {
    position: fixed; right: 1rem; top: 1rem; z-index: 100;
    display: flex; flex-direction: column; gap: 0.5rem; max-width: 300px;
  }
  .notification {
    display: flex; gap: 0.5rem; align-items: center;
    padding: 0.75rem 1rem; border-radius: 8px;
    font-size: 0.875rem; animation: slide-in 0.3s ease;
  }
  .notification.info { background: rgba(85,85,85,0.9); color: var(--text); }
  .notification.warning { background: rgba(226,183,20,0.2); border: 1px solid var(--main); color: var(--text); }
  .notification.success { background: rgba(100,200,100,0.15); border: 1px solid #6c8; color: var(--text); }
  .notification-icon { font-size: 1rem; }
  .notification-msg { font-size: 0.75rem; }
  @keyframes slide-in { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
</style>