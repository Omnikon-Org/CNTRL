# Make a tiny UI change

A short walk-through for first-time contributors: pick a small, visible piece
of the UI, change it, see it render, and open a PR. Use this as a template
for your first contribution to CNTRL.

## Where the frontend code lives

- `src/App.tsx`, `src/index.tsx` — app entry points.
- `src/components/*.tsx` — UI components (`UrlBar.tsx`, `TabBar.tsx`,
  `CommandBar.tsx`, `SettingsPage.tsx`, `WebView.tsx`, `Icons.tsx`).
- `src/components/*.css` — per-component styles.
- `src/styles/tokens.css` — shared CSS custom properties (colors, fonts)
  used across components.
- `src/stores/` — Solid stores/state used by the components.

Good tiny-change targets: a button's color via `tokens.css`, a `title` /
`aria-label` string, placeholder text, or spacing on a component's CSS file.

## 1. Run the dev server

```bash
npm install
npm run dev        # SolidJS frontend only, fastest for UI iteration
# or
npm run tauri dev   # full app in the native Tauri window
```

`npm run dev` starts Vite and prints a local URL (e.g.
`http://localhost:1420`) — open it in a browser to see the UI update live as
you edit files.

## 2. Make the change

Example used in this walk-through: the "Open in External Browser" button in
`src/components/UrlBar.tsx` had a `title` (mouse hover tooltip) but no
`aria-label`, unlike its sibling nav buttons, so screen readers announced it
only as "Open" with no context. The fix is a single attribute on one line:

```diff
- <button class="nav-btn" onClick={handleOpenExternal} title="Open in External Browser">
+ <button class="nav-btn" onClick={handleOpenExternal} title="Open in External Browser" aria-label="Open in external browser">
    <span>Open</span>
  </button>
```

Keep your own change scoped the same way: one file, one clear improvement.

## 3. Take a before/after screenshot

With the dev server running, open the app and screenshot the affected area
before and after your change (browser DevTools → inspect element, or your
OS screenshot tool). Save both images — you'll attach them to the PR.

## 4. Branch, commit, and open a PR

```bash
git checkout -b fix/short-description   # or docs/, feat/ — see README Branching Model
git add <changed files>
git commit -m "fix: add aria-label to external browser button"
git push -u origin fix/short-description
```

Then open a PR against `Omnikon-Org/CNTRL:main`:

```bash
gh pr create -R Omnikon-Org/CNTRL \
  --title "fix: add aria-label to external browser button" \
  --body "Adds a missing aria-label so the button matches its sibling nav buttons. Fixes #<issue-number>."
```

(Or use the "Compare & pull request" button on GitHub after pushing.)

## 5. Include screenshots in the PR description

Drag and drop the before/after images into the PR description text box on
GitHub — it uploads them and inserts Markdown image links automatically.
Label each one, for example:

```markdown
**Before:**
![before](https://github.com/user-attachments/assets/...)

**After:**
![after](https://github.com/user-attachments/assets/...)
```

That's it — a tiny, verifiable UI change, documented and shipped as a PR.
