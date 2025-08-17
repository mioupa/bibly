hi<!-- src/components/GenreList.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
// ★★★ types.tsからGenreの型をインポート ★★★
import type { Genre } from '../types';

// 親コンポーネントにイベントを通知するための`defineEmits`
const emit = defineEmits<{
  (e: 'genre-selected', id: number): void,
  (e: 'genre-deleted', id: number): void
}>();

// ジャンルのリストを保持するためのリアクティブな変数
const genreList = ref<Genre[]>([]);
const isEditMode = ref(false);

// 削除確認モーダルの状態
const showDeleteConfirm = ref(false);
const deletingGenre = ref<Genre | null>(null);
const deleteError = ref('');
const deleteSubmitting = ref(false);
const bookCount = ref(0);

function toggleEditMode() {
  isEditMode.value = !isEditMode.value;
}

async function requestDelete(genre: Genre) {
  deleteError.value = '';
  try {
    bookCount.value = await invoke<number>('get_book_count_by_genre', { genreId: genre.id });
    deletingGenre.value = genre;
    showDeleteConfirm.value = true;
  } catch (e) {
    console.error('Failed to get book count:', e);
    alert('書籍数の取得に失敗しました。');
  }
}

async function confirmDelete() {
  if (!deletingGenre.value) return;

  deleteSubmitting.value = true;
  deleteError.value = '';
  try {
    await invoke('delete_genre', { genreId: deletingGenre.value.id });
    const index = genreList.value.findIndex(g => g.id === deletingGenre.value!.id);
    if (index !== -1) {
      genreList.value.splice(index, 1);
      emit('genre-deleted', deletingGenre.value!.id);
    }
    showDeleteConfirm.value = false;
    deletingGenre.value = null;
  } catch (e) {
    console.error('Failed to delete genre:', e);
    deleteError.value = 'ジャンルの削除に失敗しました。';
  } finally {
    deleteSubmitting.value = false;
  }
}

// ジャンルがクリックされたときに呼ばれる関数
function selectGenre(genreId: number) {
  // 'genre-selected'という名前で、選択されたジャンルのIDを親に通知
  emit('genre-selected', genreId);
}

// コンポーネントがマウントされたら、データを取得する
onMounted(async () => {
  try {
    // Rustのget_genresコマンドを呼び出し、結果をgenreListに入れる
    genreList.value = await invoke('get_genres');
  } catch (e) {
    console.error('Failed to fetch genres:', e);
  }
});

// 親から呼び出せるように、新しいジャンルを追加するメソッドを公開
function addNewGenre(genre: Genre) {
  genreList.value.push(genre);
}
defineExpose({
  addNewGenre,
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
      <!-- クリックされたら、特別なIDとして-1を送る -->
      <li @click="!isEditMode && selectGenre(-1)" :class="{ 'item-disabled': isEditMode }">すべて表示</li>
      
      <!-- genreListの内容を元にリストを動的に描画する -->
      <li v-for="genre in genreList" :key="genre.id" @click="!isEditMode && selectGenre(genre.id)" :class="{ 'item-disabled': isEditMode }">
        <span>{{ genre.name }}</span>
        <button v-if="isEditMode" @click.stop="requestDelete(genre)" class="delete-btn">
          🗑️
        </button>
      </li>
    </ul>
  </div>

  <!-- 削除確認モーダル -->
  <teleport to="body">
    <transition name="fade">
      <div v-if="showDeleteConfirm" class="overlay">
        <div class="modal" role="dialog" aria-modal="true">
          <div class="modal-header">
            <strong>ジャンル削除の確認</strong>
          </div>
          <div class="modal-body">
            <p v-if="deletingGenre">
              「<strong>{{ deletingGenre.name }}</strong>」を削除しますか？<br>
              このジャンルには <strong>{{ bookCount }}</strong> 冊の書籍が登録されています。<br>
              削除後、これらの書籍は「未分類」になります。
            </p>
          </div>
          <div class="modal-actions">
            <button type="button" class="btn danger" @click="confirmDelete" :disabled="deleteSubmitting">はい、削除します</button>
            <button type="button" class="btn" @click="showDeleteConfirm = false" :disabled="deleteSubmitting">キャンセル</button>
            <span class="error" v-if="deleteError">{{ deleteError }}</span>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
.genre-list-container {
  padding: 1em;
  /* width は親(App.vue)で制御するため削除 */
  height: 100%;
  box-sizing: border-box;
  overflow-y: auto;
  /* 背景色は親と揃える (必要なら残す) */
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

/* --- 共通スタイルとモーダル --- */
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

.error {
  color: #d00;
  font-size: 12px;
}

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
    transform: scale(0.95);
    opacity: 0;
  }
  to {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
