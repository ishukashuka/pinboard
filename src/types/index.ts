export type ObjectKind = 'text' | 'image' | 'url';

export interface ObjectMeta {
  original_filename?: string;
  mime_type?: string;
  natural_width?: number;
  natural_height?: number;
  font_size?: number;
  font_family?: 'mono' | 'sans';
  word_wrap?: boolean;
  [key: string]: any;
}

export interface BoardObject {
  id: string;
  board_id?: string;
  kind: ObjectKind;
  x: number;
  y: number;
  width: number;
  height: number;
  z_index: number;
  title?: string;
  content: string; // Markdown text, asset URL or base64 data, or raw URL
  meta?: ObjectMeta;
  created_at?: number;
  updated_at?: number;
}

export interface Camera {
  panX: number;
  panY: number;
  zoom: number;
}

export interface SearchResult {
  id: string;
  kind: ObjectKind;
  title: string;
  snippet: string;
  x: number;
  y: number;
  width: number;
  height: number;
  matchField: string;
}

export interface BoardData {
  id: string;
  name: string;
  pan_x: number;
  pan_y: number;
  zoom: number;
  objects: BoardObject[];
}
