<script setup lang="ts">
import { ref, onMounted } from 'vue';
import {
  getSavedFontName,
  saveAndApplyFontName,
} from '../fontSettings';

type ApiProvider = 'ndl' | 'google' | 'rakuten';
type SettingsTab = 'api' | 'appearance';

const emit = defineEmits<{
  (e: 'close'): void
}>();

const googleApiKey = ref('');
const rakutenAppId = ref('');
const apiPriority = ref<ApiProvider[]>(['ndl', 'google', 'rakuten']);
const draggedIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);
const fontName = ref('');
const activeTab = ref<SettingsTab>('api');
const saving = ref(false);
const errorMsg = ref('');

const GOOGLE_API_KEY_STORAGE = 'googleBooksApiKey';
const RAKUTEN_APP_ID_STORAGE = 'rakutenApplicationId';
const API_PRIORITY_STORAGE = 'bookInfoApiPriority';
const LEGACY_PROVIDER_STORAGE = 'bookInfoApiProvider';

const ALL_API_PROVIDERS: ApiProvider[] = ['ndl', 'google', 'rakuten'];
const PROVIDER_LABELS: Record<ApiProvider, string> = {
  ndl: 'NDL',
  google: 'Google Books',
  rakuten: 'Rakuten Books',
};

function normalizePriority(list: ApiProvider[]): ApiProvider[] {
  const unique = list.filter((provider, index, arr) => arr.indexOf(provider) === index);
  const missing = ALL_API_PROVIDERS.filter(provider => !unique.includes(provider));
  return [...unique, ...missing];
}

function parseStoredPriority(raw: string | null): ApiProvider[] | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return null;
    const valid = parsed.filter((value): value is ApiProvider => ALL_API_PROVIDERS.includes(value));
    if (valid.length === 0) return null;
    return normalizePriority(valid);
  } catch {
    return null;
  }
}

function getLegacyPriority(legacyProvider: string | null): ApiProvider[] {
  if (!legacyProvider || !ALL_API_PROVIDERS.includes(legacyProvider as ApiProvider)) {
    return [...ALL_API_PROVIDERS];
  }
  const selected = legacyProvider as ApiProvider;
  return [selected, ...ALL_API_PROVIDERS.filter(provider => provider !== selected)];
}

onMounted(() => {
  googleApiKey.value = localStorage.getItem(GOOGLE_API_KEY_STORAGE) || '';
  rakutenAppId.value = localStorage.getItem(RAKUTEN_APP_ID_STORAGE) || '';
  const storedPriority = parseStoredPriority(localStorage.getItem(API_PRIORITY_STORAGE));
  apiPriority.value = storedPriority || getLegacyPriority(localStorage.getItem(LEGACY_PROVIDER_STORAGE));
  fontName.value = getSavedFontName();
});

function moveProvider(from: number, to: number) {
  if (from === to || from < 0 || to < 0) return;
  if (from >= apiPriority.value.length || to >= apiPriority.value.length) return;
  const next = [...apiPriority.value];
  const [item] = next.splice(from, 1);
  next.splice(to, 0, item);
  apiPriority.value = next;
}

function onDragStart(index: number, event: DragEvent) {
  draggedIndex.value = index;
  dragOverIndex.value = index;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', String(index));
  }
}

function onDragOver(index: number, event: DragEvent) {
  event.preventDefault();
  if (dragOverIndex.value !== index) {
    dragOverIndex.value = index;
  }
}

function onDrop(index: number, event: DragEvent) {
  event.preventDefault();
  if (draggedIndex.value == null) return;
  moveProvider(draggedIndex.value, index);
  draggedIndex.value = null;
  dragOverIndex.value = null;
}

function onDragEnd() {
  draggedIndex.value = null;
  dragOverIndex.value = null;
}

function save() {
  errorMsg.value = '';
  saving.value = true;
  try {
    localStorage.setItem(API_PRIORITY_STORAGE, JSON.stringify(apiPriority.value));
    // 互換性のため先頭要素を旧キーにも保存
    localStorage.setItem(LEGACY_PROVIDER_STORAGE, apiPriority.value[0]);
    localStorage.setItem(GOOGLE_API_KEY_STORAGE, googleApiKey.value.trim());
    localStorage.setItem(RAKUTEN_APP_ID_STORAGE, rakutenAppId.value.trim());
    saveAndApplyFontName(fontName.value);
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

function setActiveTab(tab: SettingsTab) {
  activeTab.value = tab;
}
</script>

<template>
  <div class="form-wrapper">
    <div class="form-header">
      <strong>設定</strong>
      <button type="button" class="close-btn" @click="cancel" aria-label="閉じる">×</button>
    </div>

    <div class="tab-bar" role="tablist" aria-label="設定タブ">
      <button
        type="button"
        class="tab-btn"
        :class="{ active: activeTab === 'api' }"
        role="tab"
        :aria-selected="activeTab === 'api'"
        @click="setActiveTab('api')"
      >
        API設定
      </button>
      <button
        type="button"
        class="tab-btn"
        :class="{ active: activeTab === 'appearance' }"
        role="tab"
        :aria-selected="activeTab === 'appearance'"
        @click="setActiveTab('appearance')"
      >
        表示設定
      </button>
    </div>

    <div class="settings-body">
      <div v-if="activeTab === 'api'">
        <div class="row">
          <label class="row-label">API優先順位（上から順）</label>
          <div class="priority-list">
            <div
              class="priority-item"
              :class="{ 'drag-over': dragOverIndex === index }"
              v-for="(provider, index) in apiPriority"
              :key="provider"
              @dragover="onDragOver(index, $event)"
              @drop="onDrop(index, $event)"
            >
              <span>{{ PROVIDER_LABELS[provider] }}</span>
              <div class="priority-actions">
                <button
                  type="button"
                  class="drag-handle"
                  draggable="true"
                  aria-label="ドラッグして並び替え"
                  title="ドラッグして並び替え"
                  @dragstart="onDragStart(index, $event)"
                  @dragend="onDragEnd"
                >
                  &#9776;
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="row">
          <label class="row-label">Google Books API キー</label>
          <input v-model="googleApiKey" class="text-input" placeholder="API キーを入力（未入力可）" />
          <p class="help-text">未入力でもGoogle Books検索は利用できます。必要に応じてAPIキーを設定してください。</p>
        </div>

        <div class="row">
          <label class="row-label">Rakuten Books アプリケーションID</label>
          <input v-model="rakutenAppId" class="text-input" placeholder="アプリケーションIDを入力" />
        </div>
      </div>

      <div v-else>
        <div class="row">
          <label class="row-label">フォント名（任意）</label>
          <input
            v-model="fontName"
            class="text-input"
            placeholder="例: BIZ UDPGothic / Noto Sans JP / 游明朝"
          />
          <p class="help-text">未入力、または指定フォントが見つからない場合はOS標準フォントにフォールバックします。</p>
        </div>
      </div>

      <div class="actions">
        <button class="btn primary" @click="save" :disabled="saving">保存</button>
        <button class="btn" @click="cancel" :disabled="saving">キャンセル</button>
        <span class="error" v-if="errorMsg">{{ errorMsg }}</span>
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

.tab-bar {
  display: flex;
  gap: 8px;
  padding: 10px 12px 0;
}

.tab-btn {
  border: 1px solid #ccc;
  background: #f5f5f5;
  border-radius: 6px 6px 0 0;
  padding: 6px 12px;
  cursor: pointer;
}

.tab-btn.active {
  background: #fff;
  border-bottom-color: #fff;
  font-weight: 600;
}

.settings-body {
  border-top: 1px solid #ddd;
  margin-top: -1px;
  padding: 12px;
}

.row {
  margin-bottom: 10px;
}

.row-label {
  display: block;
  font-weight: 600;
  margin-bottom: 6px;
}

.text-input {
  width: 100%;
  padding: 6px;
  border: 1px solid #bbb;
  border-radius: 4px;
  background: #fff;
}

.help-text {
  margin: 8px 0 0;
  font-size: 12px;
  color: #666;
}

.actions .btn {
  padding: 6px 12px;
}
.actions {
  display: flex;
  gap: 12px;
  align-items: center;
}
.priority-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.priority-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 8px 10px;
  background: #fafafa;
  transition: background-color 120ms ease;
}
.priority-item.drag-over {
  background: #eef6ff;
  border-color: #8fb8ff;
}
.priority-actions {
  display: flex;
  gap: 6px;
}
.drag-handle {
  border: 1px solid #ccc;
  border-radius: 4px;
  background: #fff;
  color: #666;
  cursor: grab;
  font-size: 16px;
  line-height: 1;
  padding: 4px 8px;
}
.drag-handle:active {
  cursor: grabbing;
}
.error {
  margin-left: 8px;
  font-size: 13px;
  color: #d00;
}
</style>
