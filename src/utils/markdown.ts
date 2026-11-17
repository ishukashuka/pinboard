import { marked } from 'marked';
import DOMPurify from 'dompurify';
import { evaluateFormula } from './calculator';

export { evaluateFormula };

// Global counter for task items during a single parse pass
let taskCounter = 0;

/**
 * Configure marked instance with custom extensions
 */
const customMarked = new marked.Renderer();

// Custom list item renderer to inject clickable data-task-index for checklists
customMarked.listitem = function (token) {
  if (token.task) {
    const taskIndex = taskCounter++;
    const isChecked = token.checked;
    return `<li class="task-list-item">
      <label class="task-checkbox-label" data-task-index="${taskIndex}">
        <input type="checkbox" class="task-checkbox" data-task-index="${taskIndex}" ${isChecked ? 'checked' : ''} />
        <span class="task-text ${isChecked ? 'is-done' : ''}">${token.text}</span>
      </label>
    </li>\n`;
  }
  return `<li>${token.text}</li>\n`;
};

// Custom link renderer to always open in new tab safely
customMarked.link = function (token) {
  return `<a href="${token.href}" title="${token.title || ''}" target="_blank" rel="noopener noreferrer">${token.text}</a>`;
};

marked.use({
  renderer: customMarked,
  gfm: true,
  breaks: true,
  extensions: [
    {
      name: 'inlineCalculator',
      level: 'block',
      start(src) {
        const match = src.match(/^[ \t]*=[ \t]*\S/m);
        return match ? match.index : undefined;
      },
      tokenizer(src) {
        const match = /^[ \t]*=([ \t]*[^\n\r]+)(?:\n|$)/.exec(src);
        if (match) {
          const rawExpr = match[1].trim();
          return {
            type: 'inlineCalculator',
            raw: match[0],
            expression: rawExpr,
          };
        }
      },
      renderer(token: any) {
        const expr = token.expression || '';
        const res = evaluateFormula(expr);
        const safeExpr = escapeHtml(expr);

        if (res.success) {
          return `<div class="calc-widget" title="Formula: = ${safeExpr}" data-formula="= ${safeExpr}">
            <span class="calc-pill">
              <span class="calc-badge">=</span>
              <span class="calc-result">${escapeHtml(res.formatted || String(res.result))}</span>
            </span>
            <span class="calc-formula">= ${safeExpr}</span>
          </div>\n`;
        } else {
          return `<div class="calc-widget is-error" title="Formula: = ${safeExpr} (${escapeHtml(res.error || 'Error')})" data-formula="= ${safeExpr}">
            <span class="calc-pill is-error">
              <span class="calc-badge">=</span>
              <span class="calc-error">${escapeHtml(res.error || 'Error')}</span>
            </span>
            <span class="calc-formula">= ${safeExpr}</span>
          </div>\n`;
        }
      },
    },
  ],
});

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/**
 * Render Markdown content to safe HTML.
 */
export function renderMarkdown(content: string): string {
  if (!content || !content.trim()) return '';

  taskCounter = 0; // Reset task counter for this document render
  const rawHtml = marked.parse(content) as string;

  const sanitizeOptions = {
    ALLOWED_TAGS: [
      'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
      'p', 'a', 'strong', 'b', 'em', 'i', 'del', 's', 'strike',
      'ul', 'ol', 'li', 'blockquote',
      'pre', 'code', 'table', 'thead', 'tbody', 'tr', 'th', 'td',
      'hr', 'div', 'span', 'input', 'label', 'br'
    ],
    ALLOWED_ATTR: [
      'class', 'style', 'type', 'checked', 'disabled',
      'data-task-index', 'data-formula',
      'href', 'target', 'rel', 'title'
    ],
    ADD_ATTR: ['target', 'rel'],
  };

  if (typeof window !== 'undefined') {
    const purify = (DOMPurify as any).sanitize ? DOMPurify : (DOMPurify as any)(window);
    return purify.sanitize(rawHtml, sanitizeOptions);
  }

  // Fallback for non-browser environments
  return rawHtml
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
    .replace(/on\w+="[^"]*"/gi, '');
}

/**
 * Toggles a checklist item in raw markdown by task index (0-based).
 */
export function toggleChecklistItem(markdown: string, targetIndex: number): string {
  const lines = markdown.split('\n');
  let currentIndex = 0;

  const updatedLines = lines.map((line) => {
    // Check if line matches a checklist item: - [ ] or - [x] or * [ ] or 1. [ ]
    const match = line.match(/^([ \t]*[-*+]|\d+\.)[ \t]+\[([ xX])\](.*)$/);
    if (match) {
      if (currentIndex === targetIndex) {
        const prefix = match[1];
        const isChecked = match[2].toLowerCase() === 'x';
        const rest = match[3];
        const newBox = isChecked ? '[ ]' : '[x]';
        currentIndex++;
        return `${prefix} ${newBox}${rest}`;
      }
      currentIndex++;
    }
    return line;
  });

  return updatedLines.join('\n');
}
