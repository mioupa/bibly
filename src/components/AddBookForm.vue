<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Genre, NewBook, Book } from '../types';

const emit = defineEmits<{
  (e: 'book-added', book: Book): void
  (e: 'close'): void
}>();

const genres = ref<Genre[]>([]);
const form = ref<NewBook>({
  title: '',
  genre_id: -1,
  isbn: '',
  author: '',
  publisher: '',
  price: undefined,
  c_code: '',        // ← 追加
  is_read: 0,
});
const submitting = ref(false);
const errorMsg = ref('');

onMounted(async () => {
  try {
    genres.value = await invoke<Genre[]>('get_genres');
    if (genres.value.length && form.value.genre_id === -1) {
      form.value.genre_id = genres.value[0].id;
    }
  } catch (e) {
    errorMsg.value = 'ジャンル取得に失敗しました';
  }
});

async function submit() {
  if (!form.value.title.trim() || form.value.genre_id === -1) return;
  submitting.value = true;
  errorMsg.value = '';
  try {
    const payload: NewBook = {
      ...form.value,
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
    form.value.c_code = ''; // 追加
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
        <select v-model="form.genre_id" required>
          <option v-for="g in genres" :key="g.id" :value="g.id">{{ g.name }}</option>
        </select>
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
        <button type="submit" :disabled="submitting">追加</button>
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
}
.actions {
  display: flex;
  gap: 12px;
  align-items: center;
  flex: 1 1 100%;
  margin-top: 4px;
}
button {
  padding: 4px 14px;
  cursor: pointer;
}
.error {
  color: #d00;
  font-size: 12px;
}
</style>