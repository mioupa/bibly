<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Genre, NewBook, Book } from '../types';

const emit = defineEmits<{
  (e: 'book-added', book: Book): void
  (e: 'close'): void
}>();

const genres = ref<Genre[]>([]);
const genreName = ref(''); // ← 入力可能なジャンル名を保持
const form = ref<NewBook>({
  title: '',
  genre_id: -1,
  isbn: '',
  author: '',
  publisher: '',
  price: undefined,
  c_code: '',
  is_read: 0,
});
const submitting = ref(false);
const errorMsg = ref('');

onMounted(async () => {
  try {
    genres.value = await invoke<Genre[]>('get_genres');
    // 新規登録時はデフォルトで空欄にする（ユーザーが選択または入力）
    // 以前の実装で先頭ジャンルを自動セットしていた処理を削除しました
  } catch (e) {
    errorMsg.value = 'ジャンル取得に失敗しました';
  }
});

// ジャンル名からIDを確実に取得。無ければ add_genre を呼んで作成して返す
async function ensureGenreId(name: string): Promise<number> {
  const found = genres.value.find(g => g.name === name);
  if (found) return found.id;
  const newGenre = await invoke<Genre>('add_genre', { name });
  genres.value.push(newGenre);
  return newGenre.id;
}

async function submit() {
  if (!form.value.title.trim() || !genreName.value.trim()) return;
  submitting.value = true;
  errorMsg.value = '';
  try {
    const genreId = await ensureGenreId(genreName.value.trim());
    const payload: NewBook = {
      ...form.value,
      genre_id: genreId,
      price: form.value.price == null ? undefined : Number(form.value.price),
      c_code: form.value.c_code?.trim() || undefined,
    };
    const book = await invoke<Book>('add_book', { newBook: payload });
    emit('book-added', book);
    // 初期化
    form.value.title = '';
    form.value.isbn = '';
    form.value.author = '';
    form.value.publisher = '';
    form.value.price = undefined;
    form.value.is_read = 0;
    form.value.c_code = '';
    // ジャンルは初期値を維持（先頭に戻す）
    if (genres.value.length) genreName.value = genres.value[0].name;
  } catch (e) {
    errorMsg.value = '登録に失敗しました';
    console.error(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="form-wrapper">
    <div class="form-header">
      <strong>書籍追加</strong>
      <button type="button" class="close-btn" @click="emit('close')" aria-label="閉じる">×</button>
    </div>
    <form class="add-book-form" @submit.prevent="submit">
      <div class="row">
        <label>タイトル<span class="req">*</span></label>
        <input v-model="form.title" required />
      </div>
      <div class="row">
        <label>ジャンル<span class="req">*</span></label>
        <!-- datalist を使って既存ジャンルを候補表示しつつ自由入力を許可 -->
        <input v-model="genreName" list="genre-list" required placeholder="ジャンルを選択または入力" />
        <datalist id="genre-list">
          <option v-for="g in genres" :key="g.id" :value="g.name" />
        </datalist>
      </div>
      <div class="row">
        <label>ISBN</label>
        <input v-model="form.isbn" />
      </div>
      <div class="row">
        <label>著者</label>
        <input v-model="form.author" />
      </div>
      <div class="row">
        <label>出版社</label>
        <input v-model="form.publisher" />
      </div>
      <!-- Cコードを価格の前へ移動 -->
      <div class="row">
        <label>Cコード</label>
        <input v-model="form.c_code" />
      </div>
      <div class="row">
        <label>価格</label>
        <input v-model.number="form.price" type="number" min="0" />
      </div>
      <div class="row">
        <label>読了</label>
        <select v-model="form.is_read">
          <option :value="0">未</option>
          <option :value="1">済</option>
        </select>
      </div>
      <div class="actions">
        <button type="submit" class="btn primary" :disabled="submitting">追加</button>
        <span class="error" v-if="errorMsg">{{ errorMsg }}</span>
      </div>
    </form>
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

.add-book-form {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 16px;
  align-items: flex-end;
  background: #fafafa;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-bottom: 12px;
  font-size: 13px;
}

.add-book-form input,
.add-book-form select {
  border: 1px solid #bbb;
  border-radius: 4px;
  /* ← 角丸を追加 */
  padding: 4px 6px;
  font-size: 13px;
  font-weight: normal;
}

.row {
  display: flex;
  flex-direction: column;
  min-width: 140px;
  flex: 1 1 160px;
}

.row label {
  font-weight: 600;
  margin-bottom: 2px;
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
  /* ← 追加: テキストと揃える */
}

.actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex: 1 1 100%;
  margin-top: 4px;
}

/* 共通ボタンスタイル (BookList / App と揃える) */
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

/* 既存 button セレクタでの padding 指定があれば影響するので削除/調整 */
button {
  padding: 0;
  /* ← 全体指定を無効化して .btn に委譲 (既存で padding 付与していた場合) */
  background: none;
  border: none;
}

.actions .btn {
  /* actions 内で整列 */
}
</style>