<script lang="ts">
  // NotificationStack — правая боковая панель уведомлений.
  // Максимум 3 одновременно, автоудаление через 5 секунд.

  import StatusIcon from './StatusIcon.svelte';

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

<div class="notification-stack" aria-live="polite" aria-relevant="additions text">
  {#if visible.length > 0}
    {#each visible as n (n.id)}
      <div class="notification {n.type.toLowerCase()}">
        <StatusIcon
          kind={n.type === 'SUCCESS' ? 'check' : 'cross'}
          label={n.type === 'SUCCESS' ? 'Success' : 'Warning'}
        />
        <span class="notification-msg">{n.message}</span>
      </div>
    {/each}
  {/if}
</div>

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
  .notification.info { background: var(--color-surface-raised); border: 1px solid var(--color-border); color: var(--text); box-shadow: var(--shadow-elevated); }
  .notification.warning { background: color-mix(in srgb, var(--color-warning) 14%, var(--color-surface-raised)); border: 1px solid var(--color-warning); color: var(--text); box-shadow: var(--shadow-elevated); }
  .notification.success { background: color-mix(in srgb, var(--color-success) 14%, var(--color-surface-raised)); border: 1px solid var(--color-success); color: var(--text); box-shadow: var(--shadow-elevated); }
  .notification-msg { font-size: 0.75rem; }
  @keyframes slide-in { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }
</style>
