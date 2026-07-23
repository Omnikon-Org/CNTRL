# Design System & Aesthetics — CNTRL Browser

CNTRL Browser follows a **Mecha-Industrial Design System**: a sleek, dark-first, precision aesthetic with vibrant cyan/amber accents, subtle glassmorphic surfaces, and crisp typography.

---

## 1. Color Palette & Tokens

All styles are powered by CSS Custom Properties defined in `src/styles/tokens.css`.

### Core Tokens
```css
:root {
  /* Dark Theme Default */
  --color-bg-base: #0a0c10;
  --color-bg-surface: #12161f;
  --color-bg-elevated: #1a202c;
  
  --color-border: #2d3748;
  --color-border-active: #4a5568;
  
  --color-text-primary: #f7fafc;
  --color-text-secondary: #a0aec0;
  --color-text-muted: #718096;
  
  --color-accent: #00f2fe;        /* Cyan Glow */
  --color-accent-hover: #4facfe;  /* Electric Blue */
  --color-accent-amber: #ffb703;  /* Industrial Amber */
  --color-danger: #ff4757;        /* Warning Red */
  --color-success: #2ed573;       /* Mint Green */
}

[data-theme="light"] {
  --color-bg-base: #f8fafc;
  --color-bg-surface: #ffffff;
  --color-bg-elevated: #edf2f7;
  
  --color-border: #e2e8f0;
  --color-border-active: #cbd5e0;
  
  --color-text-primary: #0f172a;
  --color-text-secondary: #475569;
  --color-text-muted: #94a3b8;
  
  --color-accent: #0284c7;
  --color-accent-hover: #0369a1;
}
```

---

## 2. Typography

- **Primary Sans Font**: `Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif`
- **Monospace / Code Font**: `JetBrains Mono, Fira Code, Menlo, Monaco, Consolas, monospace`

### Type Scale
- **Header 1**: `24px` / `1.3` (Bold 700)
- **Header 2**: `18px` / `1.4` (Semi-bold 600)
- **Body Text**: `14px` / `1.5` (Regular 400)
- **Small / Badges**: `12px` / `1.4` (Medium 500)
- **Code / Mono**: `12px` / `1.5` (Monospace 400)

---

## 3. UI Components & Micro-Interactions

1. **Tab Bar (`TabBar.css`)**:
   - Active tabs feature a subtle top border highlight with background elevation.
   - Smooth 150ms cubic-bezier transitions on hover and active state change.
2. **Command Bar Overlay (`CommandBar.css`)**:
   - Fixed centered modal with backdrop blur (`backdrop-filter: blur(12px)`).
   - Live streaming step feed with animated status dots.
3. **Macro Recording Badge**:
   - Floating pulse indicator (`● RECORDING MACRO`) with glowing amber animation.
