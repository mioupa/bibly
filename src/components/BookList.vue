<!-- src/components/BookList.vue -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Book, UpdateBook, Genre } from '../types';

const props = defineProps<{
  books: Book[]
}>();

const visibleColumns = ref({
  isbn: true,
  author: true,
  publisher: true,
  c_code: true,       // ← 追加
  price: true,
  is_read: true,
});

const columnWidths = ref<Record<string, number>>({
  title: 240,
  isbn: 130,
  author: 180,
  publisher: 180,
  c_code: 90,         // ← 追加
  price: 90,
  is_read: 70,
});

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
  if (editSubmitting.value) return;
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
    closeEdit();
  } catch (e) {
    console.error(e);
    editError.value = '更新に失敗しました';
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
</script>

<template>
  <div class="book-list-container" :class="{ resizing: isResizing }">
    <h2>書籍一覧</h2>

    <div class="column-selector">
      <strong>表示項目:</strong>
      <label><input type="checkbox" v-model="visibleColumns.isbn"> ISBN</label>
      <label><input type="checkbox" v-model="visibleColumns.author"> 著者</label>
      <label><input type="checkbox" v-model="visibleColumns.publisher"> 出版社</label>
      <label><input type="checkbox" v-model="visibleColumns.c_code"> Cコード</label> <!-- 追加 -->
      <label><input type="checkbox" v-model="visibleColumns.price"> 値段</label>
      <label><input type="checkbox" v-model="visibleColumns.is_read"> 読了</label>
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
            <td class="title-cell" @click="openEdit(book)"><span class="title-link">{{ book.title }}</span></td>
            <td v-if="visibleColumns.isbn">{{ book.isbn }}</td>
            <td v-if="visibleColumns.author">{{ book.author }}</td>
            <td v-if="visibleColumns.publisher">{{ book.publisher }}</td>
            <td v-if="visibleColumns.c_code">{{ book.c_code }}</td> <!-- 追加 -->
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
  <teleport to="body">
    <transition name="fade">
      <div v-if="editing" class="edit-overlay" @click.self="closeEdit">
        <div class="edit-modal" role="dialog" aria-modal="true">
          <div class="edit-header">
            <strong>書籍編集</strong>
            <button type="button" class="close-btn" @click="closeEdit" :disabled="editSubmitting">×</button>
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
              <button type="submit" :disabled="editSubmitting">保存</button>
              <button type="button" @click="closeEdit" :disabled="editSubmitting">キャンセル</button>
              <span class="error" v-if="editError">{{ editError }}</span>
            </div>
          </form>
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

.column-selector {
  margin-bottom: 1em;
  display: flex;
  gap: 1em;
  align-items: center;
  flex-wrap: wrap;
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
  padding: 4px 6px;
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
  line-height: 1.3;
}

/* タイトルリンク風 */
.title-cell {
  padding: 0;
}

.title-link {
  display: inline-block;
  width: 100%;
  padding: 4px 6px;
  color: #1565c0;
  text-decoration: underline;
  cursor: pointer;
}

.title-link:hover {
  color: #0d47a1;
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
  width: 660px;
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
  padding: 12px 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
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
  gap: 12px;
  align-items: center;
}

.actions button {
  padding: 4px 14px;
  cursor: pointer;
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
</style>
