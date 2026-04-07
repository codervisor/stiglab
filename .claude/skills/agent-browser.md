# agent-browser

Integration skill for using agent-browser (by Vercel Labs) as a browser
automation tool within Claude Code sessions.

## When to use

TRIGGER when: user asks to interact with a web page, test a web UI, take
screenshots, or automate browser actions.

DO NOT TRIGGER when: user is writing code that imports agent-browser as a
library (it's a CLI tool, not a library).

## What is agent-browser

agent-browser is a native Rust CLI for browser automation via Chrome DevTools
Protocol. It's designed for AI agents — semantic selectors, accessibility
snapshots, and batch operations.

Package: `agent-browser` (npm)
Repo: github.com/vercel-labs/agent-browser

## Installation

```bash
# Install the package (already in devDependencies)
pnpm add -D agent-browser

# Install Chrome browser
npx agent-browser install
# On Linux with system deps:
npx agent-browser install --with-deps
```

## Core commands

```bash
# Navigation
npx agent-browser open <url>
npx agent-browser goto <url>
npx agent-browser back
npx agent-browser forward
npx agent-browser reload

# Page inspection
npx agent-browser snapshot          # accessibility tree
npx agent-browser snapshot -i       # interactive elements with @refs
npx agent-browser screenshot        # capture screenshot
npx agent-browser get title
npx agent-browser get url
npx agent-browser get text [@ref]
npx agent-browser get html [@ref]

# Interaction
npx agent-browser click @ref
npx agent-browser dblclick @ref
npx agent-browser fill @ref "value"
npx agent-browser type @ref "text"
npx agent-browser select @ref "option"
npx agent-browser check @ref
npx agent-browser uncheck @ref
npx agent-browser hover @ref

# Waiting
npx agent-browser wait --text "content"
npx agent-browser wait --url "pattern"
npx agent-browser wait --timeout 5000

# JavaScript execution
npx agent-browser evaluate "document.title"

# Session management
npx agent-browser close
```

## Workflow pattern

1. `open` — navigate to the target URL
2. `snapshot -i` — see interactive elements with semantic refs
3. `click` / `fill` / `select` — interact using refs
4. `snapshot` or `screenshot` — verify result
5. Repeat as needed
6. `close` — end the session

## Tips

- Use `snapshot -i` (interactive) to get element refs (@e1, @e2, etc.)
- Refs are stable within a page state but change after navigation
- Use `wait` commands before assertions to handle async rendering
- Screenshots go to stdout as base64 by default; use `--output` for files
- For headless mode (CI), agent-browser runs headless by default
