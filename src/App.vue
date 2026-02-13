<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import GenreList from './components/GenreList.vue';
import BookList from './components/BookList.vue';
import AddBookForm from './components/AddBookForm.vue';
import SettingsForm from './components/SettingsForm.vue';
import ConfirmModal from './components/ConfirmModal.vue';
import type { Book, Genre } from './types';

const genreListRef = ref<InstanceType<typeof GenreList> | null>(null);
const bookListRef = ref<InstanceType<typeof BookList> | null>(null);
const bookList = ref<Book[]>([]);
const showAddForm = ref(false);
const showSettings = ref(false);
const selectedIds = ref<number[]>([]);

const showBulkDeleteConfirm = ref(false);
const bulkDeleteSubmitting = ref(false);

const sidebarWidth = ref(200);
const isResizing = ref(false);
const minWidth = 120;
const maxWidth = 500;

function onMouseDownHandle(e: MouseEvent) {
  e.preventDefault();
  isResizing.value = true;
}

function onMouseMove(e: MouseEvent) {
  if (!isResizing.value) return;
  const newWidth = Math.min(maxWidth, Math.max(minWidth, e.clientX));
  sidebarWidth.value = newWidth;
}

function stopResize() {
  if (isResizing.value) {
    isResizing.value = false;
  }
}

onMounted(() => {
  window.addEventListener('mousemove', onMouseMove);
  window.addEventListener('mouseup', stopResize);
});

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', onMouseMove);
  window.removeEventListener('mouseup', stopResize);
});

async function fetchAllBooks() {
  try {
    bookList.value = await invoke('get_all_books');
  } catch (e) {
    console.error('Failed to fetch all books:', e);
  }
}

async function handleGenreSelected(genreId: number) {
  if (genreId === -1) {
    await fetchAllBooks();
    return;
  }
  try {
    bookList.value = await invoke('get_books_by_genre', { genreId });
  } catch (e) {
    console.error('Failed to fetch books:', e);
  }
}

function toggleAddForm() {
  showAddForm.value = !showAddForm.value;
}

function closeAddForm() {
  showAddForm.value = false;
}

function toggleSettings() {
  showSettings.value = !showSettings.value;
}

function closeSettings() {
  showSettings.value = false;
}

function handleBookAdded(book: Book) {
  bookList.value.unshift(book);
  // 「未分類」が新しく作られた可能性があるので、ジャンルリストを更新する
  genreListRef.value?.fetchGenres();
}

function handleGenreAdded(genre: Genre) {
  genreListRef.value?.addNewGenre(genre);
}

function handleBookDeleted(id: number) {
  bookList.value = bookList.value.filter(book => book.id !== id);
  checkAndCleanEmptyGenres();
}

function handleGenreDeleted(genreId: number) {
  // 書籍リスト内の該当ジャンルをnullにする
  bookList.value.forEach(book => {
    if (book.genre_id === genreId) {
      book.genre_id = null;
    }
  });
  // BookListコンポーネントが持つジャンルリストからも削除する
  bookListRef.value?.removeGenreFromList(genreId);
}

function onSelectionChange(newSelectedIds: number[]) {
  selectedIds.value = newSelectedIds;
}

function requestBulkDelete() {
  if (selectedIds.value.length === 0) return;
  showBulkDeleteConfirm.value = true;
}

async function handleBulkDeleteConfirm() {
  if (bulkDeleteSubmitting.value) return;
  bulkDeleteSubmitting.value = true;
  try {
    await bookListRef.value?.deleteSelectedBooks();
  } catch (e) {
    console.error("一括削除でエラーが発生しました", e);
  } finally {
    bulkDeleteSubmitting.value = false;
    showBulkDeleteConfirm.value = false;
  }
}

function handleBulkDeleteCancel() {
  showBulkDeleteConfirm.value = false;
}

function onBooksDeleted(deletedIds: number[]) {
  bookList.value = bookList.value.filter(book => !deletedIds.includes(book.id));
  selectedIds.value = [];
  checkAndCleanEmptyGenres();
}

let cleanupEmptyGenresPromise: Promise<void> | null = null;
function checkAndCleanEmptyGenres(): Promise<void> {
  if (cleanupEmptyGenresPromise) {
    return cleanupEmptyGenresPromise;
  }

  cleanupEmptyGenresPromise = (async () => {
    try {
      const genres = await invoke<Genre[]>('get_genres');
      const counts = await Promise.all(
        genres.map(async (genre) => ({
          id: genre.id,
          count: await invoke<number>('get_book_count_by_genre', { genreId: genre.id }),
        }))
      );

      const emptyGenreIds = counts.filter((item) => item.count === 0).map((item) => item.id);
      if (emptyGenreIds.length === 0) {
        return;
      }

      const results = await Promise.allSettled(
        emptyGenreIds.map((id) => invoke('delete_genre', { id }))
      );

      const deletedIds: number[] = [];
      results.forEach((result, index) => {
        if (result.status === 'fulfilled') {
          deletedIds.push(emptyGenreIds[index]);
        } else {
          console.error(`Failed to delete empty genre: ${emptyGenreIds[index]}`, result.reason);
        }
      });

      if (deletedIds.length > 0) {
        deletedIds.forEach((id) => handleGenreDeleted(id));
        genreListRef.value?.fetchGenres();
      }
    } catch (e) {
      console.error('Failed to check and clean empty genres:', e);
    } finally {
      cleanupEmptyGenresPromise = null;
    }
  })();

  return cleanupEmptyGenresPromise;
}

function handleBookUpdated() {
  checkAndCleanEmptyGenres();
}

onMounted(() => {
  fetchAllBooks();
  checkAndCleanEmptyGenres();
});
</script>

<template>
  <main class="main-container" :class="{ 'resizing': isResizing }">
    <div class="sidebar" :style="{ width: sidebarWidth + 'px' }">
      <GenreList ref="genreListRef" @genre-selected="handleGenreSelected" @genre-deleted="handleGenreDeleted" />
    </div>
    <div class="resize-handle" @mousedown="onMouseDownHandle" :class="{ active: isResizing }"
      aria-label="Resize sidebar" />
    <div class="content">
      <div class="toolbar">
        <div class="toolbar-left">
          <button v-if="selectedIds.length === 0" class="btn primary" @click="toggleAddForm">
            新規＋
          </button>
          <button v-else class="btn danger" @click="requestBulkDelete">
            削除 ({{ selectedIds.length }})
          </button>
        </div>
        <div class="toolbar-right">
          <button class="btn" @click="toggleSettings" aria-label="設定">設定</button>
        </div>
      </div>

      <!-- モーダル (Teleport で body 直下へ) -->
      <teleport to="body">
        <transition name="fade">
          <div v-if="showAddForm" class="modal-overlay" @click.self="closeAddForm">
            <div class="modal" role="dialog" aria-modal="true">
              <AddBookForm @book-added="handleBookAdded" @genre-added="handleGenreAdded" @close="closeAddForm" />
            </div>
          </div>
        </transition>
      </teleport>

      <teleport to="body">
        <transition name="fade">
          <div v-if="showSettings" class="modal-overlay" @click.self="closeSettings">
            <div class="modal" role="dialog" aria-modal="true">
              <SettingsForm @close="closeSettings" />
            </div>
          </div>
        </transition>
      </teleport>

      <ConfirmModal
        :show="showBulkDeleteConfirm"
        title="削除の確認"
        :submitting="bulkDeleteSubmitting"
        @confirm="handleBulkDeleteConfirm"
        @cancel="handleBulkDeleteCancel"
      >
        <p><strong>{{ selectedIds.length }}</strong>件の書籍を本当に削除しますか？<br>この操作は取り消せません。</p>
      </ConfirmModal>

      <BookList ref="bookListRef" :books="bookList" @book-deleted="handleBookDeleted" @genre-added="handleGenreAdded"
        @selection-change="onSelectionChange" @books-deleted="onBooksDeleted" @book-updated="handleBookUpdated" />
    </div>
  </main>
</template>

<style scoped>
.main-container {
  display: flex;
  flex-direction: row;
  height: 100vh;
  overflow: hidden;
}

/* サイドバー */
.sidebar {
  display: flex;
  flex-direction: column;
  min-width: 120px;
  max-width: 500px;
  border-right: 1px solid #ddd;
  background: #f0f0f0;
  box-sizing: border-box;
  overflow: hidden;
  /* 幅固定: 右側のテーブルが広がっても縮まない */
  flex: 0 0 auto;
  flex-shrink: 0;
}

/* リサイズハンドル */
.resize-handle {
  width: 4px;
  cursor: col-resize;
  background: #e0e0e0;
  transition: background 0.15s;
  user-select: none;
}

.resize-handle:hover,
.resize-handle.active {
  background: #bdbdbd;
}

/* コンテンツエリア */
.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-bottom: 1px solid #ddd;
  background: #fafafa;
}

/* 新規: 左右コンテナと右寄せ */
.toolbar-left {
  display: flex;
  gap: 8px;
}

.toolbar-right {
  margin-left: auto;
  /* これで設定ボタンを右端に寄せる */
  display: flex;
  gap: 8px;
}

/* 統一ボタンスタイル (.btn) */
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

/* モーダル */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding-top: 60px;
  z-index: 1000;
}

.modal {
  background: #fff;
  border-radius: 6px;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.25);
  min-width: 640px;
  max-width: 860px;
  width: 70%;
  animation: popup .18s ease;
}

/* 簡易フェード */
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

/* ドラッグ中にテキスト選択抑制 */
.resizing {
  user-select: none;
}
</style>
