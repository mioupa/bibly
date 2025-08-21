<script setup lang="ts">

defineProps<{
  show: boolean;
  title?: string;
  submitting?: boolean;
}>();

defineEmits<{
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();
</script>

<template>
  <transition name="fade">
    <div v-if="show" class="confirm-modal-overlay" @click.self="$emit('cancel')">
      <div class="confirm-modal" role="dialog" aria-modal="true">
        <div class="modal-header">
          <strong>{{ title || '確認' }}</strong>
        </div>
        <div class="modal-body">
          <slot>本当に実行しますか？</slot>
        </div>
        <div class="modal-actions">
          <button type="button" class="btn danger" @click="$emit('confirm')" :disabled="submitting">はい</button>
          <button type="button" class="btn" @click="$emit('cancel')" :disabled="submitting">キャンセル</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.confirm-modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, .35);
  display: grid;
  place-items: center;
  z-index: 1300;
}

.confirm-modal {
  background: #fff;
  border-radius: 6px;
  box-shadow: 0 6px 24px rgba(0, 0, 0, .25);
  width: 420px;
  max-width: 90%;
  animation: popup .18s ease;
}

.modal-header {
  padding: 10px 14px;
  border-bottom: 1px solid #eee;
  font-size: 14px;
  font-weight: 600;
}

.modal-body {
  padding: 16px 14px;
  font-size: 14px;
}

.modal-body p {
  margin: 0;
  line-height: 1.6;
}

.modal-actions {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: flex-end;
  padding: 8px 14px 12px;
  background-color: #f7f7f7;
  border-top: 1px solid #eee;
  border-bottom-left-radius: 6px;
  border-bottom-right-radius: 6px;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity .18s ease;
}

@keyframes popup {
  from {
    transform: translateY(8px);
    opacity: 0;
  }

  to {
    transform: translateY(0);
    opacity: 1;
  }
}

/* 共通ボタンスタイル */
.btn {
  padding: 4px 14px;
  font-size: 13px;
  cursor: pointer;
  border: 1px solid #bbb;
  background: #fff;
  border-radius: 4px;
  line-height: 1.3;
  transition: background .15s, border-color .15s, box-shadow .15s;
}

.btn:hover:not(:disabled) {
  background: #f0f0f0;
}

.btn:disabled {
  opacity: .55;
  cursor: not-allowed;
}

.btn.danger {
  background-color: #d32f2f;
  color: white;
  border-color: #c62828;
}

.btn.danger:hover:not(:disabled) {
  background-color: #c62828;
}
</style>
