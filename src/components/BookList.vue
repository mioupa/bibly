<!-- src/components/BookList.vue -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Book, UpdateBook, Genre } from '../types';

const props = defineProps<{
  books: Book[]
}>();

const emit = defineEmits<{
  (e: 'book-deleted', id: number): void
}>();

// localStorageから設定を読み込むヘルパー
function loadFromLocalStorage<T extends object>(key: string, defaultValue: T): T {
  const saved = localStorage.getItem(key);
  if (saved) {
    try {
      // 保存された設定とデフォルト値をマージして、将来のキー追加に備える
      return { ...defaultValue, ...JSON.parse(saved) };
    } catch (e) {
      console.error(`Failed to parse localStorage item: ${key}`, e);
      return defaultValue;
    }
  }
  return defaultValue;
}

const defaultVisibleColumns = {
  isbn: false, // デフォルトで非表示に
  author: true,
  publisher: true,
  c_code: true,
  price: true,
  is_read: true,
};

const visibleColumns = ref(loadFromLocalStorage('bibly_visible_columns', defaultVisibleColumns));

watch(visibleColumns, (newValue) => {
  localStorage.setItem('bibly_visible_columns', JSON.stringify(newValue));
}, { deep: true });

// ▼ 追加: 列メニュー制御
const showColumnMenu = ref(false);
const columnMenuRef = ref<HTMLElement | null>(null);
const columnBtnRef = ref<HTMLElement | null>(null);
const columnOptions = [
  { key: 'isbn', label: 'ISBN' },
  { key: 'author', label: '著者' },
  { key: 'publisher', label: '出版社' },
  { key: 'c_code', label: 'Cコード' },
  { key: 'price', label: '値段' },
  { key: 'is_read', label: '読了' },
];

function toggleColumnMenu() {
  showColumnMenu.value = !showColumnMenu.value;
}

function toggleColumn(key: string) {
  (visibleColumns.value as any)[key] = !(visibleColumns.value as any)[key];
}

function onClickOutside(e: MouseEvent) {
  if (!showColumnMenu.value) return;
  const target = e.target as Node;
  if (
    columnMenuRef.value &&
    columnBtnRef.value &&
    !columnMenuRef.value.contains(target) &&
    !columnBtnRef.value.contains(target)
  ) {
    showColumnMenu.value = false;
  }
}

const defaultColumnWidths: Record<string, number> = {
  title: 240,
  isbn: 130,
  author: 180,
  publisher: 120,
  c_code: 90,
  price: 90,
  is_read: 70,
};

const columnWidths = ref(loadFromLocalStorage('bibly_column_widths', defaultColumnWidths));

watch(columnWidths, (newValue) => {
  localStorage.setItem('bibly_column_widths', JSON.stringify(newValue));
}, { deep: true });

// 可視列の幅合計
const tableWidth = computed(() => {
  let sum = columnWidths.value.title;
  if (visibleColumns.value.isbn) sum += columnWidths.value.isbn;
  if (visibleColumns.value.author) sum += columnWidths.value.author;
  if (visibleColumns.value.publisher) sum += columnWidths.value.publisher;
  if (visibleColumns.value.c_code) sum += columnWidths.value.c_code; // ← 追加
  if (visibleColumns.value.price) sum += columnWidths.value.price;
  if (visibleColumns.value.is_read) sum += columnWidths.value.is_read;
  return sum;
});

const isResizing = ref(false);
const resizingCol = ref<string | null>(null);
let startX = 0;
let startWidth = 0;

function startResize(colKey: string, e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
  resizingCol.value = colKey;
  startX = e.clientX;
  startWidth = columnWidths.value[colKey];
  document.body.classList.add('col-resizing');
}

function onMouseMove(e: MouseEvent) {
  if (!isResizing.value || !resizingCol.value) return;
  const delta = e.clientX - startX;
  const newWidth = Math.max(30, startWidth + delta); // さらに狭くできるよう最小幅=30
  columnWidths.value[resizingCol.value] = newWidth;
}

function onMouseUp() {
  if (isResizing.value) {
    isResizing.value = false;
    resizingCol.value = null;
    document.body.classList.remove('col-resizing');
  }
}

const genres = ref<Genre[]>([]); // ジャンル一覧を取得して編集で使う
const editGenreName = ref(''); // 編集時のジャンル名入力

onMounted(() => {
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', onMouseUp);
  document.addEventListener('click', onClickOutside); // ← 追加
  // ジャンル一覧を先に取っておく
  (async () => {
    try {
      genres.value = await invoke<Genre[]>('get_genres');
    } catch (e) {
      console.error('Failed to fetch genres for edit:', e);
    }
  })();
});

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', onMouseUp);
  document.removeEventListener('click', onClickOutside); // ← 追加
});

// ===== 編集モーダル状態 =====
const editing = ref(false);
const editSubmitting = ref(false);
const editError = ref('');
const editForm = ref<UpdateBook | null>(null);

function openEdit(book: Book) {
  editError.value = '';
  editForm.value = {
    id: book.id,
    title: book.title,
    isbn: book.isbn,
    author: book.author,
    publisher: book.publisher,
    price: book.price,
    c_code: book.c_code,
    is_read: book.is_read,
    genre_id: book.genre_id ?? -1,
  };
  // 編集用の表示名を genre_id から解決
  const g = genres.value.find(x => x.id === (book.genre_id ?? -1));
  editGenreName.value = g ? g.name : '';
  editing.value = true;
}

function closeEdit() {
  editing.value = false;
  editForm.value = null;
}

async function submitEdit() {
  if (!editForm.value) return;
  if (!editForm.value.title.trim()) {
    editError.value = 'タイトルは必須です';
    return;
  }
  editSubmitting.value = true;
  editError.value = '';
  try {
    // ジャンル名からIDを確定
    const resolvedGenreId = await ensureGenreIdForEdit(editGenreName.value.trim() || '未分類');
    const payload = { ...editForm.value, title: editForm.value.title.trim(), genre_id: resolvedGenreId };
    const updated = await invoke<Book>('update_book', { book: payload });
    const idx = props.books.findIndex((b: Book) => b.id === updated.id);
    if (idx !== -1) {
      Object.assign(props.books[idx], updated);
    }
    closeEdit(); // ← これで正常に閉じる
  } catch (e) {
    console.error(e);
    editError.value = '更新に失敗しました';
  } finally {
    editSubmitting.value = false;
  }
}

const showDeleteConfirm = ref(false);

function requestDelete() {
  if (!editForm.value) return;
  editError.value = ''; // エラーメッセージをクリア
  showDeleteConfirm.value = true;
}

async function confirmDelete() {
  if (!editForm.value) return;

  editSubmitting.value = true;
  editError.value = '';
  try {
    await invoke('delete_book', { id: editForm.value.id });
    emit('book-deleted', editForm.value.id);
    showDeleteConfirm.value = false;
    closeEdit();
  } catch (e) {
    console.error(e);
    editError.value = '削除に失敗しました';
    // エラーが発生しても確認モーダルは閉じる
    showDeleteConfirm.value = false;
  } finally {
    editSubmitting.value = false;
  }
}

// 編集時にジャンル名を確保してIDを返す（AddBookForm と同様）
async function ensureGenreIdForEdit(name: string): Promise<number> {
  const found = genres.value.find(g => g.name === name);
  if (found) return found.id;
  const newGenre = await invoke<Genre>('add_genre', { name });
  genres.value.push(newGenre);
  return newGenre.id;
}

defineExpose({
  removeGenreFromList(genreId: number) {
    const index = genres.value.findIndex(g => g.id === genreId);
    if (index !== -1) {
      genres.value.splice(index, 1);
    }
  }
});
</script>

<template>
  <div class="book-list-container" :class="{ resizing: isResizing }">
    <!-- ▼ ヘッダーを変更 -->
    <div class="list-header">
      <h2>書籍一覧</h2>
      <div class="header-actions">
        <button ref="columnBtnRef" type="button" class="btn" @click="toggleColumnMenu" aria-haspopup="true"
          :aria-expanded="showColumnMenu">
          表示項目 ▾
        </button>
        <transition name="fade">
          <div v-if="showColumnMenu" ref="columnMenuRef" class="column-menu" role="menu">
            <ul>
              <li v-for="opt in columnOptions" :key="opt.key" @click.stop="toggleColumn(opt.key)" class="column-item"
                :class="{ active: (visibleColumns as any)[opt.key] }" role="menuitemcheckbox"
                :aria-checked="(visibleColumns as any)[opt.key]">
                <span class="check-area">
                  <span v-if="(visibleColumns as any)[opt.key]" class="check-mark">✓</span>
                </span>
                <span class="label">{{ opt.label }}</span>
              </li>
            </ul>
          </div>
        </transition>
      </div>
    </div>

    <div v-if="books.length > 0" class="table-wrapper">
      <table :style="{ width: tableWidth + 'px' }">
        <colgroup>
          <col :style="{ width: columnWidths.title + 'px' }" />
          <col v-if="visibleColumns.isbn" :style="{ width: columnWidths.isbn + 'px' }" />
          <col v-if="visibleColumns.author" :style="{ width: columnWidths.author + 'px' }" />
          <col v-if="visibleColumns.publisher" :style="{ width: columnWidths.publisher + 'px' }" />
          <col v-if="visibleColumns.c_code" :style="{ width: columnWidths.c_code + 'px' }" /> <!-- 追加 -->
          <col v-if="visibleColumns.price" :style="{ width: columnWidths.price + 'px' }" />
          <col v-if="visibleColumns.is_read" :style="{ width: columnWidths.is_read + 'px' }" />
        </colgroup>
        <thead>
          <tr>
            <th>
              <div class="th-inner">
                タイトル
                <span class="col-resizer" @mousedown="startResize('title', $event)"></span>
              </div>
            </th>
            <th v-if="visibleColumns.isbn">
              <div class="th-inner">
                ISBN
                <span class="col-resizer" @mousedown="startResize('isbn', $event)"></span>
              </div>
            </th>
            <th v-if="visibleColumns.author">
              <div class="th-inner">
                著者
                <span class="col-resizer" @mousedown="startResize('author', $event)"></span>
              </div>
            </th>
            <th v-if="visibleColumns.publisher">
              <div class="th-inner">
                出版社
                <span class="col-resizer" @mousedown="startResize('publisher', $event)"></span>
              </div>
            </th>
            <th v-if="visibleColumns.c_code">
              <div class="th-inner">
                Cコード
                <span class="col-resizer" @mousedown="startResize('c_code', $event)"></span>
              </div>
            </th>
            <th v-if="visibleColumns.price">
              <div class="th-inner">
                値段
                <span class="col-resizer" @mousedown="startResize('price', $event)"></span>
              </div>
            </th>
            <th v-if="visibleColumns.is_read">
              <div class="th-inner">
                読了
                <span class="col-resizer" @mousedown="startResize('is_read', $event)"></span>
              </div>
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="book in books" :key="book.id">
            <td class="title-cell">
              <span class="title-text">{{ book.title }}</span>
              <button type="button" class="edit-inline-btn btn" @click.stop="openEdit(book)" aria-label="編集">編集</button>
            </td>
            <td v-if="visibleColumns.isbn">{{ book.isbn }}</td>
            <td v-if="visibleColumns.author">{{ book.author }}</td>
            <td v-if="visibleColumns.publisher">{{ book.publisher }}</td>
            <td v-if="visibleColumns.c_code">{{ book.c_code }}</td>
            <td v-if="visibleColumns.price">{{ book.price }}</td>
            <td v-if="visibleColumns.is_read">{{ book.is_read === 1 ? '済' : '未' }}</td>
          </tr>
        </tbody>
      </table>
    </div>
    <div v-else>
      <p>表示する書籍がありません。ジャンルを選択してください。</p>
    </div>
  </div>

  <!-- 編集モーダル -->
  <teleport to="body">
    <transition name="fade">
      <div v-if="editing" class="edit-overlay" @click.self="closeEdit">
        <div class="edit-modal" role="dialog" aria-modal="true">
          <div class="edit-header">
            <strong>書籍編集</strong>
            <button type="button" class="btn close-btn" @click="closeEdit" :disabled="editSubmitting">×</button>
          </div>
          <form v-if="editForm" class="edit-form" @submit.prevent="submitEdit">
            <div class="grid">
              <label>
                <span class="label-head">タイトル<span class="req">*</span></span>
                <input v-model="editForm.title" required />
              </label>

              <!-- ここにジャンル入力を追加 -->
              <label>ジャンル
                <input v-model="editGenreName" list="edit-genre-list" />
                <datalist id="edit-genre-list">
                  <option v-for="g in genres" :key="g.id" :value="g.name" />
                </datalist>
              </label>

              <label>ISBN
                <input v-model="editForm.isbn" />
              </label>
              <label>著者
                <input v-model="editForm.author" />
              </label>
              <label>出版社
                <input v-model="editForm.publisher" />
              </label>
              <label>Cコード
                <input v-model="editForm.c_code" />
              </label>
              <label>価格
                <input v-model.number="editForm.price" type="number" min="0" />
              </label>
              <label>読了
                <select v-model="editForm.is_read">
                  <option :value="0">未</option>
                  <option :value="1">済</option>
                </select>
              </label>
            </div>
            <div class="actions">
              <button type="submit" class="btn primary" :disabled="editSubmitting">保存</button>
              <button type="button" class="btn" @click="closeEdit" :disabled="editSubmitting">キャンセル</button>
              <button type="button" class="btn danger" @click="requestDelete" :disabled="editSubmitting"
                style="margin-left: auto;">削除</button>
              <span class="error" v-if="editError">{{ editError }}</span>
            </div>
          </form>
        </div>
      </div>
    </transition>

    <!-- 削除確認モーダル -->
    <transition name="fade">
      <div v-if="showDeleteConfirm" class="overlay">
        <div class="modal" role="dialog" aria-modal="true">
          <div class="modal-header">
            <strong>削除の確認</strong>
          </div>
          <div class="modal-body">
            <p>この書籍を本当に削除しますか？<br>この操作は取り消せません。</p>
          </div>
          <div class="modal-actions">
            <button type="button" class="btn danger" @click="confirmDelete" :disabled="editSubmitting">はい、削除します</button>
            <button type="button" class="btn" @click="showDeleteConfirm = false" :disabled="editSubmitting">キャンセル</button>
            <span class="error" v-if="editError">{{ editError }}</span>
          </div>
        </div>
      </div>
    </transition>

  </teleport>
</template>

<style scoped>
.book-list-container {
  padding: 1em;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
  /* 親 flex からの縮小制約を解除 */
}

.book-list-container.resizing {
  user-select: none;
  cursor: col-resize;
}

.table-wrapper {
  flex-grow: 1;
  overflow: auto;
  /* 横スクロール許可 */
}

/* テーブルは合計幅に固定。余白配分させない */
table {
  table-layout: fixed;
  border-collapse: collapse;
  display: inline-table;
  /* 余白に広がらない */
}

th,
td {
  border: 1px solid #ddd;
  padding: 6px 8px;
  /* 行高さ拡張 */
  text-align: left;
  /* 折り返し可能に変更 */
  white-space: normal;
  /* デフォルト折り返し */
  overflow-wrap: anywhere;
  /* 長いISBN等を強制改行可 */
  word-break: break-word;
  /* 旧ブラウザ対策 */
  overflow: hidden;
  /* 隣列へのはみ出し防止 */
  text-overflow: clip;
  /* 省略記号不要 */
  font-size: 14px;
  line-height: 1.45;
  /* 縦方向余裕 */
}

/* タイトルセル＋インライン編集ボタン */
.title-cell {
  position: relative;
  padding: 8px 8px;
  /* 余白拡大 */
  /* ボタンぶんの余白を確保したい場合は下行のコメントを外す */
  /* padding-right: 56px; */
  overflow: visible;
  /* ボタンの下がクリップされないように */
}

.title-text {
  display: block;
  font-weight: 500;
  color: #222;
  line-height: 1.35;
  overflow-wrap: anywhere;
}

.edit-inline-btn {
  position: absolute;
  top: 50%;
  /* 縦中央 */
  right: 8px;
  transform: translateY(-50%);
  padding: 2px 8px;
  font-size: 12px;
  line-height: 1.1;
  opacity: 0;
  pointer-events: none;
  transition: opacity .12s;
  white-space: nowrap;
  z-index: 1;
  /* テキスト上に重ねる */
  /* 背景を半透明にして下の文字を若干読めるようにしたい場合:
     background: rgba(255,255,255,0.85); */
}

tbody tr:hover .edit-inline-btn,
.title-cell:hover .edit-inline-btn {
  opacity: 1;
  pointer-events: auto;
}

/* 行ホバー時の視認性向上（任意） */
tbody tr:hover {
  background: #eef5ff;
}

/* 編集モーダル */
.edit-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, .35);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 70px;
  z-index: 1200;
}

.edit-modal {
  background: #fff;
  border-radius: 6px;
  box-shadow: 0 6px 24px rgba(0, 0, 0, .25);
  width: 680px; /* 少し広げる */
  max-width: 90%;
  animation: popup .18s ease;
  font-size: 13px;
}

.edit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px 4px;
  border-bottom: 1px solid #eee;
  font-size: 14px;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 18px;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
}

.close-btn:hover {
  background: #eee;
}

.edit-form {
  padding: 16px 20px; /* パディングを調整 */
  display: flex;
  flex-direction: column;
  gap: 16px; /* gapを調整 */
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 12px 16px;
}

.grid label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-weight: 600;
}

.grid input,
.grid select {
  font-weight: normal;
  font-size: 13px;
  padding: 4px 6px;
  border: 1px solid #bbb;
  border-radius: 4px;
}

.req {
  color: #d00;
  margin-left: 4px;
  font-size: 11px;
  font-weight: normal;
  display: inline-block;
  /* ← 追加: アスタリスクを折り返さない */
  white-space: nowrap;
  /* ← 追加 */
  vertical-align: middle;
  /* ← 追加 */
}

.label-head {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
  line-height: 1.2;
}

.actions {
  display: flex;
  gap: 10px; /* ボタン間のgapを少し詰める */
  align-items: center;
  padding: 8px 20px 12px; /* パディングを調整 */
  background-color: #f7f7f7;
  border-top: 1px solid #eee;
  margin: 0 -20px -16px; /* フォームのpaddingに合わせる */
  border-bottom-left-radius: 6px;
  border-bottom-right-radius: 6px;
}

.actions button {
  padding: 4px 14px;
  cursor: pointer;
}

/* ▼ 追加: 共通ボタンスタイル */
.btn {
  padding: 4px 10px;
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

.btn.primary {
  background: #1976d2;
  color: #fff;
  border-color: #1565c0;
}

.btn.primary:hover:not(:disabled) {
  background: #1565c0;
}

.btn.danger {
  background-color: #d32f2f;
  color: white;
  border-color: #c62828;
}

.btn.danger:hover:not(:disabled) {
  background-color: #c62828;
}

.error {
  color: #d00;
  font-size: 12px;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity .18s ease;
}

/* --- 汎用モーダルスタイル --- */
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, .35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1200;
}

.modal {
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
  padding: 8px 14px 12px;
  background-color: #f7f7f7;
  border-top: 1px solid #eee;
  border-bottom-left-radius: 6px;
  border-bottom-right-radius: 6px;
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

tbody tr:nth-child(even) {
  background-color: #f9f9f9;
}

thead {
  background-color: #f2f2f2;
}

th {
  position: relative;
  user-select: none;
  padding: 0;
}

.th-inner {
  position: relative;
  padding: 4px 6px;
  height: 100%;
}

.col-resizer {
  position: absolute;
  top: 0;
  right: 0;
  width: 6px;
  height: 100%;
  cursor: col-resize;
  user-select: none;
  background: transparent;
  transition: background 0.15s;
}

.col-resizer:hover,
.col-resizer:active {
  background: rgba(0, 0, 0, 0.15);
}

.col-resizing * {
  cursor: col-resize !important;
}

/* ヘッダー */
.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: .6em;
  flex-wrap: wrap;
}

.list-header h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  position: relative;
}

/* 列メニュー */
.column-menu {
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  background: #fff;
  border: 1px solid #ccc;
  border-radius: 6px;
  min-width: 160px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, .15);
  z-index: 50;
  padding: 6px 0;
  font-size: 13px;
}

.column-menu ul {
  list-style: none;
  margin: 0;
  padding: 0;
  max-height: 260px;
  overflow-y: auto;
}

.column-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  cursor: pointer;
  user-select: none;
  transition: background .12s;
}

.column-item:hover {
  background: #f5f5f5;
}

.column-item.active .label {
  font-weight: 600;
}

.check-area {
  width: 18px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.check-mark {
  font-size: 12px;
  color: #1565c0;
  font-weight: 600;
  line-height: 1;
}

/* 編集モーダル内の close-btn は .btn を利用するため微調整 */
.close-btn {
  font-size: 16px;
  padding: 2px 6px;
  line-height: 1;
  border: 1px solid transparent;
}

.close-btn:hover {
  background: #eee;
}
</style>
