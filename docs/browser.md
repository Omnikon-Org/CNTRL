# Browser Service

The `BrowserService` (Phase 2) integrates a Playwright-driven headless browser. It operates securely out-of-process, avoiding unsafe iframe usage for rendering target webpages.

## Capabilities
- Open/Close tabs.
- Navigate URLs (automatically injecting `https://` if needed, supporting `cntrl://` internal schemes).
- Execute arbitrary JavaScript within the page context.
- Fallback Chromium rendering for unsupported layout patterns.
