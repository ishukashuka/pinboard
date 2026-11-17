export type ObjectKind = 'text' | 'image' | 'url' | 'code' | 'diagram_node' | 'icon';

export type DiagramNodeType =
  | 'router'
  | 'switch'
  | 'server'
  | 'olt'
  | 'onu'
  | 'ap'
  | 'client_pc'
  | 'client_phone'
  | 'cloud';

export type AnchorPosition = 'top' | 'right' | 'bottom' | 'left';

export interface ObjectMeta {
  original_filename?: string;
  mime_type?: string;
  natural_width?: number;
  natural_height?: number;
  icon_id?: string;
  icon_category?: string;
  icon_label?: string;
  color?: string;
  font_size?: number;
  font_family?: 'mono' | 'sans';
  word_wrap?: boolean;
  node_type?: DiagramNodeType;
  subtitle?: string; // IP address or description for diagram nodes
  tags?: string[]; // e.g. ["#game", "#design", "#todo"]
  language?: string; // for code snippets
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
  content: string; // Markdown text, asset URL or base64 data, URL, or code
  meta?: ObjectMeta;
  created_at?: number;
  updated_at?: number;
}

export interface BoardConnector {
  id: string;
  board_id?: string;
  from_id?: string;
  from_anchor?: AnchorPosition;
  from_x: number;
  from_y: number;
  to_id?: string;
  to_anchor?: AnchorPosition;
  to_x: number;
  to_y: number;
  line_type: 'curved' | 'orthogonal' | 'straight';
  arrow_start: 'none' | 'arrow';
  arrow_end: 'none' | 'arrow';
  stroke_style: 'solid' | 'dashed' | 'dotted';
  color: string;
  stroke_width: number;
  label?: string;
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

export interface BoardInfo {
  id: string;
  name: string;
  pan_x: number;
  pan_y: number;
  zoom: number;
  updated_at: number;
}

export interface BoardData {
  board: BoardInfo;
  objects: BoardObject[];
  connectors: BoardConnector[];
}

export type ActiveTool =
  | 'select'
  | 'hand'
  | 'marquee'
  | 'text'
  | 'image'
  | 'link'
  | 'connector'
  | 'diagram';
