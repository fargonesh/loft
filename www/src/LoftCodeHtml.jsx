export const CODE_BG     = '#13151f';
export const CODE_BORDER = 'rgba(255,255,255,0.09)';
export const CODE_FONT   = "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace";

export const INLINE_STYLE = [
  'background:rgba(183,148,244,0.1)',
  'border:1px solid rgba(183,148,244,0.2)',
  'border-radius:4px',
  'padding:1px 6px',
  `font-family:${CODE_FONT}`,
  'font-size:0.85em',
  'color:#b794f4',
  'vertical-align:baseline',
  'white-space:nowrap',
].join(';');

export const BLOCK_STYLE = [
  `background:${CODE_BG}`,
  `border:1px solid ${CODE_BORDER}`,
  'border-radius:8px',
  'overflow:hidden',
  'box-shadow:0 4px 20px rgba(0,0,0,0.3)',
].join(';');

// Signature / fn header — single line, no shadow
export const SIGNATURE_STYLE = [
  `background:${CODE_BG}`,
  `border:1px solid ${CODE_BORDER}`,
  'border-radius:6px',
  'padding:8px 16px',
  'margin:8px 0 16px',
  'overflow-x:auto',
  `font-family:${CODE_FONT}`,
  'font-size:0.875em',
  'line-height:1.6',
  'display:block',
].join(';');

const LABEL_STYLE = [
  'font-family:system-ui,sans-serif',
  'font-size:10px',
  'letter-spacing:0.1em',
  'font-weight:600',
  'color:#4a5568',
  'text-transform:uppercase',
  'display:block',
  'margin-bottom:6px',
].join(';');

// Shared copy button appearance — used both by the HTML string and by
// LoftCodeBlock's React <CopyButton> so they look identical.
export const COPY_BTN_BASE_STYLE = [
  'position:absolute',
  'top:10px',
  'right:12px',
  'background:rgba(255,255,255,0.05)',
  'border:1px solid rgba(255,255,255,0.1)',
  'border-radius:5px',
  'color:#4a5568',
  'font-size:11px',
  'font-family:system-ui,sans-serif',
  'padding:3px 9px',
  'cursor:pointer',
  'letter-spacing:0.04em',
  'line-height:1.5',
  'transition:all 0.18s',
].join(';');

// ─── CSS reset for doc-generator stylesheet bleed ─────────────────────────────
//
// The doc generator emits a scoped stylesheet containing:
//   .pkg-doc-content code { background-color: #f5f5f5; padding: 2px 6px; ... }
//
// Shiki puts its dark background on the <pre>, not the <code> child, so the
// doc's rule bleeds through and paints a white rectangle inside every dark block.
//
// Fix: inject this reset alongside the scoped CSS in Docs.jsx.
// Selector specificity:
//   doc rule:   .pkg-doc-content code          → (0,1,1)
//   our reset:  .pkg-doc-content [data-loft-block] code → (0,2,1)  ← wins
//   our inline: .pkg-doc-content [data-loft-inline]     → (0,2,0) + !important
export const LOFT_RESET_CSS = `
  /* kill doc-generator code background inside our highlighted blocks */
  .pkg-doc-content [data-loft-block] code,
  .pkg-doc-content [data-loft-block] pre {
    background: transparent !important;
    padding: 0 !important;
    border-radius: 0 !important;
    border: none !important;
    color: inherit !important;
    font-size: inherit !important;
    white-space: inherit !important;
    box-shadow: none !important;
  }
  /* Give the pre proper padding since we stripped Shiki's */
  .pkg-doc-content [data-loft-block] pre {
    padding: 1em 1.25em !important;
    margin: 0 !important;
  }
  /* enforce our inline badge style over the doc-generator's code rule */
  .pkg-doc-content [data-loft-inline] {
    background: rgba(183,148,244,0.1) !important;
    border: 1px solid rgba(183,148,244,0.2) !important;
    border-radius: 4px !important;
    padding: 1px 6px !important;
    font-family: 'JetBrains Mono','Fira Code','Cascadia Code',monospace !important;
    font-size: 0.85em !important;
    color: #b794f4 !important;
    vertical-align: baseline !important;
    white-space: nowrap !important;
  }
`;

export function wrapBlockHtml(shikiHtml, { label = '', noCopy = false } = {}) {
  const labelHtml = label
    ? `<span style="${LABEL_STYLE}">${label}</span>`
    : '';

  const copyBtn = noCopy
    ? ''
    : `<button data-loft-copy-btn style="${COPY_BTN_BASE_STYLE}">copy</button>`;

  // Strip Shiki's inline background-color from <pre> — it's an inline style so it
  // beats any CSS selector. Our wrapper div provides the background instead.
  const cleanHtml = shikiHtml.replace(
    /(<pre\b[^>]*?)\s*background-color\s*:[^;'"]+;?/gi,
    '$1'
  );

  return [
    `<div data-loft-block class="not-prose" style="position:relative;margin:20px 0">`,
      labelHtml,
      `<div style="position:relative">`,
        copyBtn,
        `<div style="${BLOCK_STYLE}">${cleanHtml}</div>`,
      `</div>`,
    `</div>`,
  ].join('');
}

export function wrapInlineHtml(innerCodeHtml) {
  return `<code data-loft-inline style="${INLINE_STYLE}">${innerCodeHtml}</code>`;
}

export function parseStyle(cssText) {
  return Object.fromEntries(
    cssText.split(';')
      .filter(Boolean)
      .map(rule => {
        const idx = rule.indexOf(':');
        const prop = rule.slice(0, idx).trim();
        const val  = rule.slice(idx + 1).trim();
        const camel = prop.replace(/-([a-z])/g, (_, c) => c.toUpperCase());
        return [camel, val];
      })
  );
}