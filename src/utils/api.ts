import { invoke } from '@tauri-apps/api/core';
import type { BoardConnector, BoardData, BoardInfo, BoardObject } from '../types';

export interface ClipboardPayload {
  kind: 'text' | 'image' | 'empty';
  text?: string;
  image_data?: string;
  width?: number;
  height?: number;
  filename?: string;
}

export const api = {
  async getClipboard(): Promise<ClipboardPayload> {
    try {
      return await invoke<ClipboardPayload>('get_clipboard');
    } catch (err) {
      console.warn('Tauri get_clipboard failed, falling back to navigator.clipboard:', err);
      if (navigator.clipboard && navigator.clipboard.readText) {
        try {
          const text = await navigator.clipboard.readText();
          if (text) {
            return { kind: 'text', text };
          }
        } catch {
          // ignore
        }
      }
      return { kind: 'empty' };
    }
  },

  async copyText(text: string): Promise<void> {
    try {
      await invoke('copy_text', { text });
    } catch (err) {
      console.warn('Tauri copy_text failed, using navigator.clipboard:', err);
      if (navigator.clipboard && navigator.clipboard.writeText) {
        await navigator.clipboard.writeText(text);
      }
    }
  },

  async copyImage(imageData: string): Promise<void> {
    try {
      await invoke('copy_image', { imageData });
    } catch (err) {
      console.error('Tauri copy_image failed:', err);
    }
  },

  async saveImageFromBytes(bytesBase64: string, originalFilename?: string): Promise<ClipboardPayload> {
    return await invoke<ClipboardPayload>('save_image_from_bytes', {
      bytesBase64,
      originalFilename,
    });
  },

  async loadBoard(boardId?: string): Promise<BoardData> {
    try {
      return await invoke<BoardData>('load_board', { boardId });
    } catch (err) {
      console.warn('Tauri load_board failed, using local fallback:', err);
      const saved = localStorage.getItem(`pinboard_${boardId || 'default'}`);
      if (saved) {
        return JSON.parse(saved);
      }
      return {
        board: {
          id: boardId || 'default',
          name: 'Untitled Board',
          pan_x: 0,
          pan_y: 0,
          zoom: 1,
          updated_at: Date.now(),
        },
        objects: [],
        connectors: [],
      };
    }
  },

  async saveBoard(
    boardId: string,
    panX: number,
    panY: number,
    zoom: number,
    objects: BoardObject[],
    connectors: BoardConnector[]
  ): Promise<void> {
    try {
      await invoke('save_board', {
        boardId,
        panX,
        panY,
        zoom,
        objects,
        connectors,
      });
    } catch (err) {
      console.warn('Tauri save_board failed, saving to localStorage:', err);
      localStorage.setItem(
        `pinboard_${boardId}`,
        JSON.stringify({
          board: { id: boardId, name: 'Untitled Board', pan_x: panX, pan_y: panY, zoom, updated_at: Date.now() },
          objects,
          connectors,
        })
      );
    }
  },

  async getBoards(): Promise<BoardInfo[]> {
    try {
      return await invoke<BoardInfo[]>('get_boards');
    } catch (err) {
      console.warn('Tauri get_boards failed:', err);
      return [
        {
          id: 'default',
          name: 'Untitled Board',
          pan_x: 0,
          pan_y: 0,
          zoom: 1,
          updated_at: Date.now(),
        },
      ];
    }
  },

  async createBoard(name: string): Promise<BoardInfo> {
    try {
      return await invoke<BoardInfo>('create_board', { name });
    } catch (err) {
      console.warn('Tauri create_board failed:', err);
      return {
        id: crypto.randomUUID(),
        name,
        pan_x: 0,
        pan_y: 0,
        zoom: 1,
        updated_at: Date.now(),
      };
    }
  },

  async renameBoard(id: string, name: string): Promise<void> {
    try {
      await invoke('rename_board', { id, name });
    } catch (err) {
      console.warn('Tauri rename_board failed:', err);
    }
  },

  async deleteObjects(ids: string[]): Promise<void> {
    try {
      await invoke('delete_objects', { ids });
    } catch (err) {
      console.warn('Tauri delete_objects failed:', err);
    }
  },

  async setAlwaysOnTop(enabled: boolean): Promise<void> {
    try {
      await invoke('set_always_on_top', { enabled });
    } catch (err) {
      console.warn('Tauri set_always_on_top failed:', err);
    }
  },
};
