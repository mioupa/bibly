<!-- src/components/GenreList.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
// ★★★ types.tsからGenreの型をインポート ★★★
import type { Genre } from '../types';

// 親コンポーネントにイベントを通知するための`defineEmits`
const emit = defineEmits(['genre-selected']);

// ジャンルのリストを保持するためのリアクティブな変数
const genreList = ref<Genre[]>([]);

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
</script>

<template>
  <div class="genre-list-container">
    <h2>ジャンル一覧</h2>
    <ul>
      <!-- 「すべて表示」の項目を固定で追加 -->
      <!-- クリックされたら、特別なIDとして-1を送る -->
      <li @click="selectGenre(-1)">すべて表示</li>
      
      <!-- genreListの内容を元にリストを動的に描画する -->
      <li v-for="genre in genreList" :key="genre.id" @click="selectGenre(genre.id)">
        {{ genre.name }}
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

li {
  cursor: pointer;
  padding: 4px 0;
}
li:hover {
  font-weight: bold;
}
</style>
