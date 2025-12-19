# lofi.nvim — Design Document

> Local AI-powered lofi beat generation for Neovim

## Overview

`lofi.nvim` is a Neovim plugin that generates ambient lofi beats locally using lightweight AI models. Designed for developers who want non-distracting background music while coding, without leaving the editor or relying on external services.

---

## Goals

- **Zero-latency startup** — lazy load everything; never block init.lua
- **Fully local** — no API keys, no network, no telemetry
- **Async-first** — all inference runs in background jobs
- **Composable** — expose Lua API for scripting and integration
- **Minimal footprint** — single binary backend, pure Lua frontend

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        Neovim                               │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    lofi.nvim (Lua)                    │  │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   │  │
│  │  │ Config  │  │Commands │  │   API   │  │  State  │   │  │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘   │  │
│  │       └───────────┴───────────┴───────────┘          │  │
│  │                        │                              │  │
│  │              ┌─────────▼─────────┐                    │  │
│  │              │   Job Controller  │                    │  │
│  │              │  (vim.loop/libuv) │                    │  │
│  │              └─────────┬─────────┘                    │  │
│  └────────────────────────│──────────────────────────────┘  │
└───────────────────────────│─────────────────────────────────┘
                            │ stdin/stdout (JSON-RPC)
                    ┌───────▼───────┐
                    │  lofi-daemon  │
                    │   (Rust bin)  │
                    │               │
                    │ ┌───────────┐ │
                    │ │ MusicGen  │ │
                    │ │  (ONNX)   │ │
                    │ └───────────┘ │
                    │ ┌───────────┐ │
                    │ │  rodio/   │ │
                    │ │  cpal     │ │
                    │ └───────────┘ │
                    └───────────────┘
```

### Components

| Component | Language | Responsibility |
|-----------|----------|----------------|
| `lua/lofi/*.lua` | Lua | Config, commands, keymaps, state, UI |
| `lofi-daemon` | Rust | Model inference, audio playback, IPC |
| Model weights | ONNX | MusicGen-small quantized (~150MB) |

---

## Backend: `lofi-daemon`

A single static binary handling inference and audio. Communicates via JSON-RPC over stdin/stdout.

### Why Rust?

- Single binary distribution (no Python runtime)
- ONNX Runtime bindings (`ort` crate) for fast CPU/GPU inference
- `rodio`/`cpal` for cross-platform audio
- Small binary size (~5-10MB stripped)

### JSON-RPC Interface

```jsonc
// Request: Generate a new track
{
  "jsonrpc": "2.0",
  "method": "generate",
  "params": {
    "prompt": "lofi hip hop, jazzy piano, rain sounds",
    "duration_sec": 30,
    "seed": 42          // optional, for reproducibility
  },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "track_id": "a1b2c3",
    "duration_sec": 30,
    "path": "/tmp/lofi-nvim/a1b2c3.wav"  // cached
  },
  "id": 1
}

// Request: Playback control
{ "method": "play", "params": { "track_id": "a1b2c3" } }
{ "method": "pause" }
{ "method": "resume" }
{ "method": "stop" }
{ "method": "volume", "params": { "level": 0.5 } }
{ "method": "status" }

// Notification (daemon → nvim)
{
  "jsonrpc": "2.0",
  "method": "progress",
  "params": { "track_id": "a1b2c3", "percent": 45 }
}
```

### Model Selection

**Primary: MusicGen-small (ONNX quantized)**
- ~150MB weights (INT8 quantized)
- Runs on CPU in ~10-20s for 10s of audio
- Optional CUDA/Metal acceleration

**Fallback: Riffusion (if user prefers)**
- Stable Diffusion-based, heavier
- Better for very short loops

---

## Frontend: `lua/lofi/`

```
lua/
└── lofi/
    ├── init.lua          -- setup(), public API
    ├── config.lua        -- defaults, validation, merging
    ├── daemon.lua        -- spawn, IPC, lifecycle
    ├── commands.lua      -- user commands
    ├── state.lua         -- playback state, track cache
    ├── ui.lua            -- statusline component, float window
    └── health.lua        -- :checkhealth lofi
```

### `init.lua` — Public API

```lua
local lofi = require("lofi")

-- Setup (call from init.lua)
lofi.setup({
  -- see config section
})

-- Programmatic API
lofi.generate(opts)       -- async, returns track_id via callback
lofi.play(track_id?)      -- play specific or last generated
lofi.pause()
lofi.resume()
lofi.toggle()             -- pause/resume
lofi.stop()
lofi.volume(level)        -- 0.0 - 1.0
lofi.status()             -- returns state table
lofi.queue(track_id)      -- add to playlist

-- Callbacks / Events
lofi.on("generate_start", callback)
lofi.on("generate_done", callback)
lofi.on("playback_start", callback)
lofi.on("playback_end", callback)
lofi.on("error", callback)
```

### Configuration

```lua
require("lofi").setup({
  -- Daemon
  daemon = {
    bin = nil,                        -- auto-detect or explicit path
    auto_start = true,                -- start on first command
    auto_stop = true,                 -- stop on VimLeave
    log_level = "warn",               -- "debug" | "info" | "warn" | "error"
  },

  -- Model
  model = {
    backend = "musicgen",             -- "musicgen" | "riffusion"
    weights_path = nil,               -- auto-download to stdpath("data")
    device = "cpu",                   -- "cpu" | "cuda" | "metal"
    quantized = true,                 -- use INT8 weights
  },

  -- Generation defaults
  defaults = {
    prompt = "lofi hip hop, chill, mellow piano, vinyl crackle",
    duration_sec = 30,
    seed = nil,                       -- nil = random
  },

  -- Playback
  playback = {
    volume = 0.3,                     -- default volume (0.0 - 1.0)
    loop = true,                      -- loop generated tracks
    crossfade_sec = 2,                -- crossfade between tracks
    cache_dir = vim.fn.stdpath("cache") .. "/lofi",
    max_cache_mb = 500,               -- auto-cleanup old tracks
  },

  -- UI
  ui = {
    notify = true,                    -- use vim.notify for status
    progress = "mini",                -- "mini" | "fidget" | "none"
    statusline = true,                -- expose component
  },

  -- Keymaps (set to false to disable)
  keymaps = {
    toggle = "<leader>ll",
    generate = "<leader>lg",
    volume_up = "<leader>l+",
    volume_down = "<leader>l-",
    prompt = "<leader>lp",            -- open prompt input
  },
})
```

---

## User Commands

```vim
:Lofi                    " Toggle playback (generate if nothing cached)
:Lofi play [track_id]    " Play specific track or resume
:Lofi pause
:Lofi stop
:Lofi generate [prompt]  " Generate new track
:Lofi prompt             " Open input for custom prompt
:Lofi volume [0-100]     " Set volume
:Lofi status             " Show current state
:Lofi list               " Show cached tracks (Telescope if available)
:Lofi clear              " Clear cache
:Lofi log                " Open daemon log in split
```

Command completion via `vim.api.nvim_create_user_command` with `complete` callback.

---

## Async Model

All blocking operations use `vim.loop` (libuv) for async execution:

```lua
-- daemon.lua (simplified)
local M = {}
local uv = vim.loop

function M.spawn()
  local stdin = uv.new_pipe()
  local stdout = uv.new_pipe()
  local stderr = uv.new_pipe()

  M.handle = uv.spawn(M.bin_path, {
    args = {},
    stdio = { stdin, stdout, stderr },
  }, function(code)
    M.on_exit(code)
  end)

  M.stdin = stdin
  M.stdout = stdout

  -- Read stdout for JSON-RPC responses
  stdout:read_start(function(err, data)
    if data then
      vim.schedule(function()
        M.handle_response(vim.json.decode(data))
      end)
    end
  end)
end

function M.request(method, params, callback)
  local id = M.next_id()
  M.pending[id] = callback

  local msg = vim.json.encode({
    jsonrpc = "2.0",
    method = method,
    params = params,
    id = id,
  })

  M.stdin:write(msg .. "\n")
end
```

---

## UI Integration

### Statusline Component

Exposes a function for statusline plugins (lualine, heirline, etc.):

```lua
-- In lualine config:
sections = {
  lualine_x = {
    {
      require("lofi.ui").statusline,
      cond = require("lofi.ui").is_active,
    },
  },
}

-- Output examples:
-- "🎵 lofi ▶ 0:45/1:30"
-- "🎵 lofi ⏸"
-- "🎵 generating 67%"
```

### Progress Reporting

Integrates with common progress plugins:

```lua
-- fidget.nvim integration
local fidget = require("fidget")
lofi.on("generate_start", function(track_id)
  fidget.notify("Generating track...", vim.log.levels.INFO, { key = "lofi" })
end)

-- Also supports mini.notify, nvim-notify, or plain vim.notify
```

### Telescope Extension

```lua
-- Optional telescope picker for cached tracks
require("telescope").load_extension("lofi")

-- Usage
:Telescope lofi tracks
:Telescope lofi prompts  -- saved prompt history
```

---

## Installation

### lazy.nvim

```lua
{
  "username/lofi.nvim",
  dependencies = {
    "nvim-lua/plenary.nvim",  -- optional, for async utils
  },
  build = "./install.sh",      -- downloads daemon + model weights
  cmd = "Lofi",                -- lazy load on command
  keys = {
    { "<leader>ll", "<cmd>Lofi<cr>", desc = "Toggle lofi" },
    { "<leader>lg", "<cmd>Lofi generate<cr>", desc = "Generate lofi" },
  },
  opts = {
    -- your config
  },
}
```

### Manual / packer / vim-plug

```lua
-- packer
use {
  "username/lofi.nvim",
  run = "./install.sh",
  config = function()
    require("lofi").setup()
  end
}
```

### Build Script (`install.sh`)

```bash
#!/bin/bash
set -e

REPO="username/lofi-daemon"
VERSION="v0.1.0"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map arch
case $ARCH in
  x86_64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
esac

# Download daemon binary
BINARY_URL="https://github.com/$REPO/releases/download/$VERSION/lofi-daemon-$OS-$ARCH"
curl -L "$BINARY_URL" -o ./bin/lofi-daemon
chmod +x ./bin/lofi-daemon

# Download model weights (if not cached)
WEIGHTS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/nvim/lofi/models"
mkdir -p "$WEIGHTS_DIR"
if [ ! -f "$WEIGHTS_DIR/musicgen-small-int8.onnx" ]; then
  echo "Downloading model weights (~150MB)..."
  curl -L "https://huggingface.co/.../musicgen-small-int8.onnx" \
    -o "$WEIGHTS_DIR/musicgen-small-int8.onnx"
fi

echo "✓ lofi.nvim installed"
```

---

## Health Check

`:checkhealth lofi` output:

```
lofi: require("lofi.health").check()

lofi.nvim ~
- OK Neovim >= 0.9.0
- OK lofi-daemon binary found: ~/.local/share/nvim/lofi/bin/lofi-daemon
- OK Model weights found: musicgen-small-int8.onnx (147MB)
- OK Audio device available: default
- WARNING CUDA not available, using CPU (generation will be slower)
```

---

## Error Handling

```lua
-- All errors surfaced via vim.notify and callbacks
lofi.on("error", function(err)
  vim.notify("lofi: " .. err.message, vim.log.levels.ERROR)
end)

-- Specific error types
-- { type = "daemon_crash", message = "...", code = 1 }
-- { type = "generation_failed", message = "...", track_id = "..." }
-- { type = "playback_failed", message = "...", device = "..." }
-- { type = "model_load_failed", message = "..." }
```

---

## Future Considerations

- **Presets**: Ship common lofi presets (rainy day, coffee shop, late night)
- **Live mixing**: Adjust parameters during playback (tempo, effects)
- **Integration with pomodoro plugins**: Auto-pause during breaks
- **Remote daemon**: Connect to daemon on another machine
- **DAW export**: Save tracks as stems for further editing

---

## File Structure (Final)

```
lofi.nvim/
├── lua/
│   └── lofi/
│       ├── init.lua
│       ├── config.lua
│       ├── daemon.lua
│       ├── commands.lua
│       ├── state.lua
│       ├── ui.lua
│       ├── health.lua
│       └── telescope/
│           └── lofi.lua
├── plugin/
│   └── lofi.lua          -- autocmds, command registration
├── bin/
│   └── .gitkeep          -- daemon binary installed here
├── doc/
│   └── lofi.txt          -- vimdoc
├── install.sh
├── README.md
├── LICENSE
└── .github/
    └── workflows/
        └── release.yml   -- build daemon for all platforms
```

---

## Summary

| Aspect | Decision |
|--------|----------|
| Backend | Rust binary with ONNX Runtime |
| Model | MusicGen-small INT8 (~150MB) |
| IPC | JSON-RPC over stdin/stdout |
| Async | vim.loop (libuv) |
| Config | Single `setup()` call, sensible defaults |
| UI | Statusline component, vim.notify, optional Telescope |
| Install | Lazy-compatible, single build script |

This design prioritizes the "it just works" experience Neovim users expect: lazy loading, async everything, Lua-native API, and zero external dependencies at runtime.
