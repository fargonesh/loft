import { useState } from 'react';
import {
  CODE_BG, CODE_BORDER, CODE_FONT,
  INLINE_STYLE, COPY_BTN_BASE_STYLE,
  wrapBlockHtml, wrapInlineHtml, parseStyle,
} from './LoftCodeHtml';

function CopyButton({ code }) {
  const [copied, setCopied] = useState(false);
  const copy = () => {
    navigator.clipboard.writeText(code);
    setCopied(true);
    setTimeout(() => setCopied(false), 1800);
  };
  return (
    <button
      onClick={copy}
      style={{
        ...parseStyle(COPY_BTN_BASE_STYLE),
        background: copied ? 'rgba(104,211,145,0.12)' : 'rgba(255,255,255,0.05)',
        borderColor: copied ? '#68d391' : 'rgba(255,255,255,0.1)',
        color: copied ? '#68d391' : '#4a5568',
      }}
    >
      {copied ? 'copied!' : 'copy'}
    </button>
  );
}

function PlainBlock({ code }) {
  return (
    <pre style={{
      background: CODE_BG,
      border: `1px solid ${CODE_BORDER}`,
      borderRadius: 8,
      padding: '16px 20px',
      fontFamily: CODE_FONT,
      fontSize: '0.875em',
      lineHeight: 1.75,
      color: '#abb2bf',
      overflowX: 'auto',
      margin: '20px 0',
      whiteSpace: 'pre',
    }}>
      <code>{code}</code>
    </pre>
  );
}

function PlainInline({ code }) {
  return (
    <code style={{ ...parseStyle(INLINE_STYLE), display: 'inline' }}>
      {code}
    </code>
  );
}

export default function LoftCodeBlock({
  code = '',
  inline = false,
  lang,
  label,
  highlighter,
  noCopy = false,
}) {
  const resolvedLang = lang || 'loft';

  if (inline) {
    if (highlighter) {
      try {
        const html = highlighter.codeToHtml(code, { lang: resolvedLang, theme: 'one-dark-pro' });
        const match = html.match(/<code[^>]*>([\s\S]*)<\/code>/);
        if (match) {
          // wrapInlineHtml produces identical markup to what Docs.jsx injects,
          // wrapped in a neutral span so React has a mounting point.
          return (
            <span dangerouslySetInnerHTML={{ __html: wrapInlineHtml(match[1]) }} />
          );
        }
      } catch (_) { /* fall through */ }
    }
    return <PlainInline code={code} />;
  }

  if (highlighter) {
    try {
      const shikiHtml = highlighter.codeToHtml(code.replace(/\n$/, ''), {
        lang: resolvedLang,
        theme: 'one-dark-pro',
      });

      // wrapBlockHtml generates the shell (label + styled wrapper) without
      // a copy button so we can overlay a real React <CopyButton> as sibling.
      const shellHtml = wrapBlockHtml(shikiHtml, { label, noCopy: true });

      return (
        <div className="not-prose" style={{ position: 'relative' }}>
          {/* Reset any prose-pre / prose-code Tailwind Typography overrides that
              survive not-prose, and strip any stray background on Shiki's pre. */}
          <style>{`
            [data-loft-block] pre { background: transparent !important; padding: 1em 1.25em !important; margin: 0 !important; border: none !important; box-shadow: none !important; }
            [data-loft-block] code { background: transparent !important; padding: 0 !important; border-radius: 0 !important; border: none !important; color: inherit !important; font-size: inherit !important; white-space: inherit !important; }
            [data-loft-inline] { background: rgba(183,148,244,0.1) !important; border: 1px solid rgba(183,148,244,0.2) !important; border-radius: 4px !important; padding: 1px 6px !important; color: #b794f4 !important; white-space: nowrap !important; }
          `}</style>
          {!noCopy && <CopyButton code={code} />}
          <div dangerouslySetInnerHTML={{ __html: shellHtml }} />
        </div>
      );
    } catch (_) { /* fall through */ }
  }

  return (
    <div className="not-prose" style={{ position: 'relative' }}>
      {!noCopy && <CopyButton code={code} />}
      <PlainBlock code={code} />
    </div>
  );
}