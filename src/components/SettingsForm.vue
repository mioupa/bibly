<script setup lang="ts">
import { ref, onMounted } from 'vue';

const emit = defineEmits<{
  (e: 'close'): void
}>();

const apiChoice = ref<'ndl' | 'amazon'>('ndl');
const amazonAccessKey = ref('');
const amazonSecretKey = ref('');
const amazonAssociateTag = ref('');
const saving = ref(false);
const errorMsg = ref('');

const STORAGE_API = 'bookInfoApi';
const STORAGE_AMAZON_ACCESS = 'amazonAccessKey';
const STORAGE_AMAZON_SECRET = 'amazonSecretKey';
const STORAGE_AMAZON_ASSOCIATE = 'amazonAssociateTag';

onMounted(() => {
  apiChoice.value = (localStorage.getItem(STORAGE_API) as 'ndl' | 'amazon') || 'ndl';
  amazonAccessKey.value = localStorage.getItem(STORAGE_AMAZON_ACCESS) || '';
  amazonSecretKey.value = localStorage.getItem(STORAGE_AMAZON_SECRET) || '';
  amazonAssociateTag.value = localStorage.getItem(STORAGE_AMAZON_ASSOCIATE) || '';
});

function save() {
  errorMsg.value = '';
  saving.value = true;
  try {
    localStorage.setItem(STORAGE_API, apiChoice.value);
    if (apiChoice.value === 'amazon') {
      localStorage.setItem(STORAGE_AMAZON_ACCESS, amazonAccessKey.value.trim());
      localStorage.setItem(STORAGE_AMAZON_SECRET, amazonSecretKey.value.trim());
      localStorage.setItem(STORAGE_AMAZON_ASSOCIATE, amazonAssociateTag.value.trim());
    }
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
        <label style="display:block;font-weight:600;margin-bottom:6px;">使用するAPI</label>
        <select v-model="apiChoice" style="width:100%;padding:6px;border:1px solid #bbb;border-radius:4px;">
          <option value="ndl">NDL</option>
          <option value="amazon">Amazon</option>
        </select>
      </div>

      <div v-if="apiChoice === 'amazon'" style="margin-bottom:10px;">
        <label style="display:block;font-weight:600;margin-bottom:6px;">Amazon アクセスキーID</label>
        <input v-model="amazonAccessKey" style="width:100%;padding:6px;border:1px solid #bbb;border-radius:4px;" />

        <label style="display:block;font-weight:600;margin:10px 0 6px;">Amazon シークレットキー</label>
        <input v-model="amazonSecretKey" style="width:100%;padding:6px;border:1px solid #bbb;border-radius:4px;" />

        <label style="display:block;font-weight:600;margin:10px 0 6px;">Amazon アソシエイトID</label>
        <input v-model="amazonAssociateTag" style="width:100%;padding:6px;border:1px solid #bbb;border-radius:4px;" />
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
