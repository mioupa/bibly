<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Genre, NewBook, Book, BookInfoFromApi } from '../types';

const emit = defineEmits<{
  (e: 'book-added', book: Book): void
  (e: 'genre-added', genre: Genre): void
  (e: 'close'): void
}>();

const genres = ref<Genre[]>([]);
const genreName = ref('');
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
const searching = ref(false); // API検索中の状態
const errorMsg = ref('');
const mode = ref<'auto' | 'manual'>('auto');
const isbnInput = ref('');
const isDropdownOpen = ref(false);
const isbnInputEl = ref<HTMLInputElement | null>(null); // DOM要素への参照

// JANコード入力用ポップアップの状態
const showJanPopup = ref(false);
const janCode = ref('');
const janErrorMsg = ref('');
const tempBookInfo = ref<BookInfoFromApi | null>(null); // NDLからの情報を一時保持
const janInputEl = ref<HTMLInputElement | null>(null); // JANコード入力欄のDOM参照

const GOOGLE_API_KEY_STORAGE = 'googleBooksApiKey';
const RAKUTEN_APP_ID_STORAGE = 'rakutenApplicationId';
const PROVIDER_STORAGE = 'bookInfoApiProvider';

// JANコードポップアップが表示されたら入力欄にフォーカスを当てる
watch(showJanPopup, async (isShown) => {
  if (isShown) {
    await nextTick(); // DOMの更新を待つ
    janInputEl.value?.focus();
  }
});

onMounted(async () => {
  try {
    genres.value = await invoke<Genre[]>('get_genres');
    if (genres.value.length > 0) {
      // genreName.value = genres.value[0].name; // デフォルトジャンルを設定
    }
  } catch (e) {
    errorMsg.value = 'ジャンル取得に失敗しました';
  }
  // モーダルが開いたらISBN入力欄にフォーカスを当てる
  isbnInputEl.value?.focus();
});

function toggleDropdown() {
  isDropdownOpen.value = !isDropdownOpen.value;
}

function closeDropdown() {
  // blurイベントで即座に閉じると、リスト内の項目をクリックできないため、少し遅延させる
  setTimeout(() => {
    isDropdownOpen.value = false;
  }, 200);
}

function selectGenre(genre: Genre) {
  genreName.value = genre.name;
  isDropdownOpen.value = false;
}

async function ensureGenreId(name: string): Promise<number> {
  const found = genres.value.find(g => g.name === name);
  if (found) return found.id;
  const newGenre = await invoke<Genre>('add_genre', { name });
  genres.value.push(newGenre);
  emit('genre-added', newGenre); // 新ジャンル追加を通知
  return newGenre.id;
}

async function submit() {
  // タイトルが空の場合は何もしない
  if (!form.value.title.trim()) return;

  submitting.value = true;
  errorMsg.value = '';

  try {
    // ジャンル名が空欄の場合は「未分類」として扱う
    const genreToEnsure = genreName.value.trim() || '未分類';
    const genreId = await ensureGenreId(genreToEnsure);

    const payload: NewBook = {
      ...form.value,
      genre_id: genreId,
      price: form.value.price == null ? undefined : Number(form.value.price),
      c_code: form.value.c_code?.trim() || undefined,
    };

    const book = await invoke<Book>('add_book', { newBook: payload });
    emit('book-added', book);

    // フォームをリセット
    form.value.title = '';
    form.value.isbn = '';
    form.value.author = '';
    form.value.publisher = '';
    form.value.price = undefined;
    form.value.is_read = 0;
    form.value.c_code = '';
    genreName.value = ''; // ジャンル入力もリセット
    isbnInput.value = ''; // ISBN入力欄もリセット
    mode.value = 'auto'; // 自動入力タブに戻す

    // 自動入力タブのISBN入力欄にフォーカスを戻す
    await nextTick();
    isbnInputEl.value?.focus();

  } catch (e) {
    errorMsg.value = '登録に失敗しました';
    console.error(e);
  } finally {
    submitting.value = false;
  }
}

// EnterキーでISBN検索を実行するハンドラ
function onIsbnEnter(event: KeyboardEvent) {
  // IME変換中のEnterキー操作では何もしない
  if (event.isComposing) {
    return;
  }
  // Enterキーによるフォームのデフォルト送信を確実に防ぐ
  event.preventDefault();
  // 検索を実行
  searchByIsbn();
}

// 選択されたAPIで書籍情報を検索
async function searchByIsbn() {
  errorMsg.value = '';
  if (!isbnInput.value.trim()) {
    errorMsg.value = 'ISBNを入力してください';
    return;
  }
  searching.value = true;
  try {
    const provider = localStorage.getItem(PROVIDER_STORAGE) || 'ndl';
    let result: BookInfoFromApi;
    if (provider === 'google') {
      const apiKey = localStorage.getItem(GOOGLE_API_KEY_STORAGE) || '';
      result = await invoke<BookInfoFromApi>('fetch_book_info_from_google_books', {
        isbn: isbnInput.value.trim(),
        apiKey,
      });
    } else if (provider === 'rakuten') {
      const applicationId = localStorage.getItem(RAKUTEN_APP_ID_STORAGE) || '';
      result = await invoke<BookInfoFromApi>('fetch_book_info_from_rakuten', {
        isbn: isbnInput.value.trim(),
        applicationId,
      });
    } else {
      result = await invoke<BookInfoFromApi>('fetch_book_info_from_ndl', {
        isbn: isbnInput.value.trim(),
      });
    }
    // 取得した情報を一時保持
    tempBookInfo.value = result;
    // JANコード入力ポップアップを表示
    showJanPopup.value = true;
    janCode.value = '';
    janErrorMsg.value = '';
  } catch (e) {
    errorMsg.value = `書籍情報の取得に失敗しました: ${e}`;
    console.error(e);
  } finally {
    searching.value = false;
  }
}

// JANコードを処理してフォームに反映
function processJanCode() {
  janErrorMsg.value = '';
  if (!tempBookInfo.value) return;

  const code = janCode.value.trim();
  // JANコードが未入力の場合はスキップと同じ
  if (!code) {
    skipJanCode();
    return;
  }

  // 書籍JANコード(2段目)のフォーマットを検証
  if (!/^192\d{10}$/.test(code)) {
    janErrorMsg.value = '不正なJANコードです。2段目のバーコード（192で始まる13桁）を入力してください。';
    return;
  }

  const cCode = code.substring(3, 7);
  const price = parseInt(code.substring(7, 12), 10);

  // フォームにすべての情報をセット
  form.value.title = tempBookInfo.value.title;
  form.value.author = tempBookInfo.value.author;
  form.value.publisher = tempBookInfo.value.publisher;
  form.value.isbn = isbnInput.value.trim();
  form.value.c_code = cCode;
  form.value.price = price;

  // ポップアップを閉じて手動入力タブに切り替え
  showJanPopup.value = false;
  mode.value = 'manual';
}

// JANコード入力をスキップしてフォームに反映
function skipJanCode() {
  if (!tempBookInfo.value) return;

  // NDLの情報だけをフォームにセット
  form.value.title = tempBookInfo.value.title;
  form.value.author = tempBookInfo.value.author;
  form.value.publisher = tempBookInfo.value.publisher;
  form.value.isbn = isbnInput.value.trim();

  // ポップアップを閉じて手動入力タブに切り替え
  showJanPopup.value = false;
  mode.value = 'manual';
}
</script>

<template>
  <div class="form-wrapper">
    <div class="form-header">
      <strong>書籍追加</strong>
      <button type="button" class="close-btn" @click="emit('close')" aria-label="閉じる">×</button>
    </div>

    <!-- タブ -->
    <div class="tab-bar" role="tablist" aria-label="入力切替">
      <button type="button" class="tab" :class="{ active: mode === 'auto' }" @click="mode = 'auto'" role="tab"
        :aria-selected="mode === 'auto'">
        自動入力 (ISBN)
      </button>
      <button type="button" class="tab" :class="{ active: mode === 'manual' }" @click="mode = 'manual'" role="tab"
        :aria-selected="mode === 'manual'">
        手動入力
      </button>
    </div>

    <!-- 自動入力パネル -->
    <div v-if="mode === 'auto'" class="auto-panel">
      <form class="add-book-form" @submit.prevent>
        <div class="row">
          <label>ISBN<span class="req">*</span></label>
          <input ref="isbnInputEl" v-model="isbnInput" @keydown.enter="onIsbnEnter" placeholder="ISBNを入力" />
        </div>
        <div class="actions">
          <button type="button" class="btn primary" @click="searchByIsbn" :disabled="searching || !isbnInput.trim()">
            {{ searching ? '検索中...' : '検索して追加' }}
          </button>
          <button type="button" class="btn" @click="emit('close')" :disabled="submitting">キャンセル</button>
          <span class="error" v-if="errorMsg">{{ errorMsg }}</span>
        </div>
      </form>
    </div>

    <!-- 手動入力パネル（既存フォームを移動） -->
    <div v-else class="manual-panel">
      <form class="add-book-form" @submit.prevent="submit">
        <div class="row">
          <label>タイトル<span class="req">*</span></label>
          <input v-model="form.title" required />
        </div>
        <div class="row">
          <label>ジャンル<span class="req">*</span></label>
          <div class="genre-input-wrapper">
            <input v-model="genreName" @blur="closeDropdown"
              placeholder="ジャンルを選択または入力" />
            <button type="button" class="dropdown-arrow" @click.stop="toggleDropdown" tabindex="-1">▼</button>
            <div v-if="isDropdownOpen" class="dropdown-list">
              <ul>
                <li v-for="g in genres" :key="g.id" @mousedown.prevent="selectGenre(g)">
                  {{ g.name }}
                </li>
              </ul>
            </div>
          </div>
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
          <button type="button" class="btn" @click="emit('close')" :disabled="submitting">キャンセル</button>
          <span class="error" v-if="errorMsg">{{ errorMsg }}</span>
        </div>
      </form>
    </div>
  </div>

  <!-- JANコード入力モーダル -->
  <teleport to="body">
    <transition name="fade">
      <div v-if="showJanPopup" class="jan-overlay">
        <div class="jan-modal" role="dialog" aria-modal="true">
          <div class="jan-header">
            <strong>JANコード入力</strong>
          </div>
          <div class="jan-body">
            <p>書籍の2段目のバーコード（192で始まる13桁の数字）を入力すると、Cコードと価格が自動入力されます。</p>
            <input ref="janInputEl" v-model="janCode" @keydown.enter.prevent="processJanCode" placeholder="例: 1920193008001" />
            <span class="error" v-if="janErrorMsg">{{ janErrorMsg }}</span>
          </div>
          <div class="jan-actions">
            <button type="button" class="btn primary" @click="processJanCode">確定</button>
            <button type="button" class="btn" @click="skipJanCode">スキップ</button>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
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

.genre-input-wrapper {
  position: relative;
  width: 100%;
}

.genre-input-wrapper input {
  width: 100%;
  box-sizing: border-box;
}

.dropdown-arrow {
  position: absolute;
  top: 1px;
  right: 1px;
  bottom: 1px;
  width: 28px;
  padding: 0;
  border: none;
  background: #f0f0f0;
  cursor: pointer;
  border-top-right-radius: 3px;
  border-bottom-right-radius: 3px;
  border-left: 1px solid #bbb;
  font-size: 10px;
  color: #333;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dropdown-arrow:hover {
  background: #e8e8e8;
}

.dropdown-list {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background: #fff;
  border: 1px solid #bbb;
  border-top: none;
  border-radius: 0 0 4px 4px;
  max-height: 160px;
  overflow-y: auto;
  z-index: 10;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

.dropdown-list ul {
  list-style: none;
  margin: 0;
  padding: 0;
}

.dropdown-list li {
  padding: 8px 12px;
  cursor: pointer;
  font-size: 13px;
}

.dropdown-list li:hover {
  background: #f0f0f0;
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

/* タブバー */
.tab-bar {
  display: flex;
  gap: 6px;
  padding: 8px 6px 0;
  margin-bottom: 0;
  align-items: flex-end;
  /* 下線 (コンテンツとの境界) */
  border-bottom: 1px solid #e6e6e6;
  background: transparent;
}

/* タブ本体 */
.tab {
  appearance: none;
  -webkit-appearance: none;
  border: 1px solid transparent;
  border-bottom: 1px solid transparent;
  background: transparent;
  padding: 8px 12px;
  border-top-left-radius: 6px;
  border-top-right-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  color: #444;
  transition: background .12s, color .12s, box-shadow .12s, border-color .12s;
  margin-bottom: -1px;
  /* 下線と重ねる */
}

/* 非アクティブ時のホバー */
.tab:not(.active):hover {
  background: #f6f7f8;
}

/* アクティブタブ: 上に浮いたカード風 */
.tab.active {
  background: #fff;
  border-color: #ddd;
  border-bottom-color: #fff;
  /* コンテンツとつながって見えるようにする */
  color: #111;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.06);
  font-weight: 600;
}

/* タブ用パネル（タブ下のコンテンツをカード風に） */
.auto-panel,
.manual-panel {
  padding: 12px;
  background: #fff;
  border: 1px solid #ddd;
  border-top: none;
  /* タブとつながるようにする */
  border-bottom-left-radius: 6px;
  border-bottom-right-radius: 6px;
  margin-top: 0;
}

/* JANコード入力モーダル */
.jan-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1100;
  /* AddBookFormモーダル(1000)より手前に */
}

.jan-modal {
  background: #fff;
  border-radius: 6px;
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.25);
  width: 480px;
  max-width: 90%;
  animation: popup .18s ease;
}

.jan-header {
  padding: 10px 14px;
  border-bottom: 1px solid #eee;
  font-size: 14px;
  font-weight: 600;
}

.jan-body {
  padding: 16px;
  font-size: 13px;
}

.jan-body p {
  margin: 0 0 12px;
  line-height: 1.6;
}

.jan-body input {
  width: 100%;
  padding: 6px 8px;
  border: 1px solid #bbb;
  border-radius: 4px;
  font-size: 14px;
  box-sizing: border-box;
}

.jan-body .error {
  display: block;
  color: #d00;
  font-size: 12px;
  margin-top: 8px;
}

.jan-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 12px 16px;
  background-color: #f7f7f7;
  border-top: 1px solid #eee;
  border-bottom-left-radius: 6px;
  border-bottom-right-radius: 6px;
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