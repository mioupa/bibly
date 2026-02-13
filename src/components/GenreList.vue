<!-- src/components/GenreList.vue -->
<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Genre } from '../types';
import ConfirmModal from './ConfirmModal.vue';

// 親コンポーネントにイベントを通知するための`defineEmits`
const emit = defineEmits<{
  (e: 'genre-selected', id: number): void,
  (e: 'genre-deleted', id: number): void,
}>();

const selectedGenreId = ref<number>(-1); // 初期選択は「すべて表示」

// ジャンルのリストを保持するためのリアクティブな変数
const genreList = ref<Genre[]>([]);
const isEditMode = ref(false);

// 削除確認モーダルの状態
const showDeleteConfirm = ref(false);
const deletingGenre = ref<Genre | null>(null);
const bookCount = ref(0);
const deleteSubmitting = ref(false);
const deleteError = ref('');

function toggleEditMode() {
  isEditMode.value = !isEditMode.value;
}

async function requestDelete(genre: Genre) {
  try {
    bookCount.value = await invoke<number>('get_book_count_by_genre', { genreId: genre.id });
    deletingGenre.value = genre;
    showDeleteConfirm.value = true;
  } catch (e) {
    console.error('Failed to get book count:', e);
    alert('書籍数の取得に失敗しました。');
  }
}

async function handleDeleteConfirm() {
  if (!deletingGenre.value) return;

  deleteSubmitting.value = true;
  deleteError.value = '';
  try {
    await invoke('delete_genre', { id: deletingGenre.value.id });
    emit('genre-deleted', deletingGenre.value.id);
    // ジャンル削除で「未分類」が新規作成される可能性があるため再取得する
    await fetchGenres();
    // 削除完了後は編集モードを終了して通常状態へ戻す
    isEditMode.value = false;
    handleDeleteCancel();
  } catch (e) {
    console.error('Failed to delete genre:', e);
    deleteError.value = '削除に失敗しました。'
  } finally {
    deleteSubmitting.value = false;
  }
}

function handleDeleteCancel() {
  showDeleteConfirm.value = false;
  deletingGenre.value = null;
  bookCount.value = 0;
  deleteSubmitting.value = false;
  deleteError.value = '';
}

// ジャンルがクリックされたときに呼ばれる関数
function selectGenre(genreId: number) {
  if (isEditMode.value) return;

  if (selectedGenreId.value !== genreId) {
    selectedGenreId.value = genreId;
  }
  emit('genre-selected', selectedGenreId.value);
}

// ESCキーで選択解除
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    if (selectedGenreId.value !== -1) {
      selectedGenreId.value = -1;
      emit('genre-selected', -1);
    }
  }
}

async function fetchGenres() {
  try {
    const nextGenres = await invoke<Genre[]>('get_genres');
    genreList.value = nextGenres;
    if (selectedGenreId.value !== -1 && !nextGenres.some((g) => g.id === selectedGenreId.value)) {
      selectedGenreId.value = -1;
      emit('genre-selected', -1);
    }
  } catch (e) {
    console.error('Failed to fetch genres:', e);
  }
}

// コンポーネントがマウントされたら、データを取得する
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  fetchGenres();
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// 親から呼び出せるように、新しいジャンルを追加するメソッドを公開
function addNewGenre(genre: Genre) {
  genreList.value.push(genre);
}
defineExpose({
  addNewGenre,
  fetchGenres
});
</script>

<template>
  <div class="genre-list-container">
    <div class="list-header">
      <h2>ジャンル一覧</h2>
      <button @click="toggleEditMode" class="edit-btn">
        {{ isEditMode ? '完了' : '編集' }}
      </button>
    </div>
    <ul>
      <!-- 「すべて表示」の項目を固定で追加 -->
      <li @click="selectGenre(-1)" :class="{ 'item-disabled': isEditMode, 'selected': selectedGenreId === -1 }">すべて表示</li>
      
      <!-- genreListの内容を元にリストを動的に描画する -->
      <li v-for="genre in genreList" :key="genre.id" @click="selectGenre(genre.id)" :class="{ 'item-disabled': isEditMode, 'selected': selectedGenreId === genre.id }">
        <span>{{ genre.name }}</span>
        <button v-if="isEditMode && genre.name !== '未分類'" @click.stop="requestDelete(genre)" class="delete-btn">
          🗑️
        </button>
      </li>
    </ul>

    <ConfirmModal
      :show="showDeleteConfirm"
      title="ジャンル削除の確認"
      :submitting="deleteSubmitting"
      @confirm="handleDeleteConfirm"
      @cancel="handleDeleteCancel"
    >
      <div v-if="deletingGenre">
        <p>
          「<strong>{{ deletingGenre.name }}</strong>」を削除しますか？<br>
          このジャンルには <strong>{{ bookCount }}</strong> 冊の書籍が登録されています。<br>
          削除後、これらの書籍は「未分類」になります。
        </p>
        <span class="error" v-if="deleteError">{{ deleteError }}</span>
      </div>
    </ConfirmModal>
  </div>

</template>

<style scoped>
.genre-list-container {
  padding: 1em;
  height: 100%;
  box-sizing: border-box;
  overflow-y: auto;
  background-color: #f0f0f0;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5em;
}

h2 {
  margin: 0;
  font-size: 1.2em;
}

.edit-btn {
  background: none;
  border: 1px solid #ccc;
  border-radius: 4px;
  padding: 2px 8px;
  font-size: 13px;
  cursor: pointer;
  transition: background .12s, color .12s;
}

.edit-btn:hover {
  background: #e0e0e0;
}

ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

li {
  display: flex;
  justify-content: space-between;
  align-items: center;
  cursor: pointer;
  padding: 6px 4px;
  border-radius: 4px;
  transition: background .12s;
}

li:hover {
  background: #e0e0e0;
}

li.item-disabled {
  cursor: default;
  color: #888;
}

li.item-disabled:hover {
  background: none;
  font-weight: normal;
}

li.selected {
  background-color: #dcebff;
  font-weight: 600;
}

li.selected:hover {
  background-color: #caddff;
}

.delete-btn {
  background: none;
  border: none;
  cursor: pointer;
  padding: 0 4px;
  font-size: 16px;
  opacity: 0.6;
  line-height: 1;
  transition: opacity .12s, color .12s;
}

.delete-btn:hover {
  opacity: 1;
  color: #d32f2f;
}

.error {
    color: #d32f2f;
    font-size: 13px;
    margin-top: 8px;
}
</style>
