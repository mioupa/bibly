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
  genre_id?: number; // ← 追加
}

// Genreの型もついでにこちらに移動しておくと、さらに管理しやすくなります
export interface Genre {
  id: number;
  name: string;
}

export interface NewBook {
  title: string;
  genre_id: number;
  isbn?: string;
  author?: string;
  publisher?: string;
  price?: number;
  c_code?: string;      // ← 追加
  is_read?: number; // 省略時 0
}

// 既存書籍更新用
export interface UpdateBook {
  id: number;
  title: string;
  isbn?: string;
  author?: string;
  publisher?: string;
  price?: number;
  c_code?: string;
  is_read: number;
  genre_id?: number; // ← 追加
}
