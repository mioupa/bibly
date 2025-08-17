<!-- src/components/GenreList.vue -->
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

function toggleEditMode() {
  isEditMode.value = !isEditMode.value;
}

async function confirmDeleteGenre(genre: Genre) {
  try {
    const count = await invoke<number>('get_book_count_by_genre', { genreId: genre.id });
    const confirmMsg = `このジャンルには${count}冊が登録されています。本当に削除しますか？\n削除された書籍のジャンルは未分類になります。`;
    if (window.confirm(confirmMsg)) {
      await invoke('delete_genre', { genreId: genre.id });
      // UIから削除
      const index = genreList.value.findIndex(g => g.id === genre.id);
      if (index !== -1) {
        genreList.value.splice(index, 1);
        emit('genre-deleted', genre.id);
      }
    }
  } catch (e) {
    console.error('Failed to delete genre:', e);
    alert('ジャンルの削除に失敗しました。');
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
        <button v-if="isEditMode" @click.stop="confirmDeleteGenre(genre)" class="delete-btn">
          🗑️
        </button>
      </li>
    </ul>
  </div>
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
</style>
