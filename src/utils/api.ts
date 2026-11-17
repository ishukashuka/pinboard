import { invoke } from '@tauri-apps/api/core';
import type { BoardData, BoardObject } from '../types';

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
      // Fallback for browser mode
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
      const saved = localStorage.getItem('pinboard_fallback_board');
      if (saved) {
        return JSON.parse(saved);
      }
      return {
        id: 'default',
        name: 'Main Board',
        pan_x: 0,
        pan_y: 0,
        zoom: 1,
        objects: [],
      };
    }
  },

  async saveBoard(
    boardId: string,
    panX: number,
    panY: number,
    zoom: number,
    objects: BoardObject[]
  ): Promise<void> {
    try {
      await invoke('save_board', {
        boardId,
        panX,
        panY,
        zoom,
        objects,
      });
    } catch (err) {
      console.warn('Tauri save_board failed, saving to localStorage:', err);
      localStorage.setItem(
        'pinboard_fallback_board',
        JSON.stringify({ id: boardId, name: 'Main Board', pan_x: panX, pan_y: panY, zoom, objects })
      );
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
