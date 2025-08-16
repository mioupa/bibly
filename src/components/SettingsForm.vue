```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue';

const emit = defineEmits<{
  (e: 'close'): void
}>();

const apiKey = ref('');
const saving = ref(false);
const errorMsg = ref('');

const STORAGE_KEY = 'googleBooksApiKey';

onMounted(() => {
  apiKey.value = localStorage.getItem(STORAGE_KEY) || '';
});

function save() {
  errorMsg.value = '';
  saving.value = true;
  try {
    // 簡易バリデーション（任意）
    // 空文字は許容（APIキー未設定のまま使う場合があるため）
    localStorage.setItem(STORAGE_KEY, apiKey.value.trim());
    emit('close');
  } catch (e) {
    console.error(e);
    errorMsg.value = '保存に失敗しました';
  } finally {
    saving.value = false;
  }
}

function cancel() {
  emit('close');
}
</script>

<template>
  <div class="form-wrapper">
    <div class="form-header">
      <strong>設定</strong>
      <button type="button" class="close-btn" @click="cancel" aria-label="閉じる">×</button>
    </div>

    <div class="settings-body" style="padding:12px;">
      <div class="row" style="margin-bottom:10px;">
        <label style="display:block;font-weight:600;margin-bottom:6px;">Google Books API キー</label>
        <input v-model="apiKey" placeholder="API キーを入力（未入力可）" style="width:100%;padding:6px;border:1px solid #bbb;border-radius:4px;" />
        <p style="margin:8px 0 0;font-size:12px;color:#666;">将来的に自動ISBN検索で使用します。APIキーなしでも手動入力は可能です。</p>
      </div>

      <div class="actions" style="display:flex;gap:12px;align-items:center;">
        <button class="btn primary" @click="save" :disabled="saving">保存</button>
        <button class="btn" @click="cancel" :disabled="saving">キャンセル</button>
        <span class="error" v-if="errorMsg" style="color:#d00;">{{ errorMsg }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.form-wrapper {
  padding: 4px 4px 12px;
}
.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 10px 2px;
  border-bottom: 1px solid #eee;
  font-size: 14px;
}
.close-btn {
  border: none;
  background: transparent;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}
.close-btn:hover {
  background: #eee;
}
.actions .btn {
  padding: 6px 12px;
}
.error {
  margin-left: 8px;
  font-size: 13px;
}
</style>
```