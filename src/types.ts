// src/types.ts

// Bookの型をここに一元管理する
export interface Book {
  id: number;
  isbn?: string;
  title: string;
  author?: string;
  publisher?: string;
  price?: number;
  c_code?: string;
  is_read: number;
  genre_id?: number | null; // 'null' を追加
  audience?: string | null;
  form?: string | null;
  content?: string | null;
}

// Genreの型もついでにこちらに移動しておくと、さらに管理しやすくなります
export interface Genre {
  id: number;
  name: string;
}

export interface NewBook {
  title: string;
  genre_id: number | null;
  isbn?: string;
  author?: string;
  publisher?: string;
  price?: number;
  c_code?: string;      // ← 追加
  is_read?: number; // 省略時 0
}

export interface UpdateBook {
  id: number;
  title: string;
  isbn?: string;
  author?: string;
  publisher?: string;
  price?: number;
  c_code?: string;
  is_read: number;
  genre_id?: number | null; // 'null' を追加
}

export interface CCodeInterpretation {
  c_code: string;
  audience?: string | null;
  form?: string | null;
  content?: string | null;
}

// APIから取得した書籍情報を表す型
export interface BookInfoFromApi {
  title: string;
  author: string;
  publisher: string;
}
