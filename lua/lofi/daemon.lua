--- Daemon spawn and management for lofi.nvim
--- Handles starting, stopping, and monitoring the lofi-daemon process.
local M = {}

--- @class lofi.DaemonConfig
--- @field daemon_path string|nil Path to lofi-daemon binary (auto-detected if nil)
--- @field model_path string|nil Path to ONNX models (uses default if nil)
--- @field device string Device selection: "auto", "cpu", "cuda", "metal"
--- @field threads number|nil CPU threads (nil = auto-detect)

--- @class lofi.DaemonState
--- @field job_id number|nil Neovim job ID for the daemon process
--- @field stdin any|nil Stdin handle for sending requests
--- @field config lofi.DaemonConfig|nil Configuration passed at setup

--- State tracking for the daemon process
--- @type lofi.DaemonState
local state = {
  job_id = nil,
  stdin = nil,
  config = nil,
}

--- Default configuration
--- @type lofi.DaemonConfig
local defaults = {
  daemon_path = nil,
  model_path = nil,
  device = "auto",
  threads = nil,
}

--- Find the daemon binary path
--- @return string|nil path to daemon binary or nil if not found
local function find_daemon_binary()
  -- Check if custom path is configured
  if state.config and state.config.daemon_path then
    if vim.fn.executable(state.config.daemon_path) == 1 then
      return state.config.daemon_path
    end
  end

  -- Check relative to plugin directory (development)
  local plugin_root = vim.fn.fnamemodify(debug.getinfo(1, "S").source:sub(2), ":h:h:h")
  local dev_path = plugin_root .. "/daemon/target/release/lofi-daemon"
  if vim.fn.executable(dev_path) == 1 then
    return dev_path
  end

  -- Check in PATH
  if vim.fn.executable("lofi-daemon") == 1 then
    return "lofi-daemon"
  end

  return nil
end

--- Build environment variables for daemon process
--- @return table environment variables
local function build_env()
  local env = vim.fn.environ()

  if state.config then
    if state.config.model_path then
      env.LOFI_MODEL_PATH = state.config.model_path
    end
    if state.config.device then
      env.LOFI_DEVICE = state.config.device
    end
    if state.config.threads then
      env.LOFI_THREADS = tostring(state.config.threads)
    end
  end

  return env
end

--- Initialize daemon module with configuration
--- @param opts table|nil configuration options
function M.setup(opts)
  state.config = vim.tbl_deep_extend("force", {}, defaults, opts or {})
end

--- Check if daemon is currently running
--- @return boolean true if daemon process is active
function M.is_running()
  return state.job_id ~= nil and vim.fn.jobwait({ state.job_id }, 0)[1] == -1
end

--- Start the daemon process
--- @param callbacks table callback functions: on_stdout, on_stderr, on_exit
--- @return boolean success true if daemon started successfully
function M.start_daemon(callbacks)
  if M.is_running() then
    return true
  end

  local daemon_path = find_daemon_binary()
  if not daemon_path then
    vim.notify("[lofi] Daemon binary not found. Build with: cd daemon && cargo build --release", vim.log.levels.ERROR)
    return false
  end

  callbacks = callbacks or {}
  local function process_stdout(data)
    if not data then
      return
    end
    -- Neovim splits stdout by newlines, each element is a line
    for _, line in ipairs(data) do
      if line ~= "" and callbacks.on_stdout then
        callbacks.on_stdout(line)
      end
    end
  end

  local function process_stderr(data)
    if not data then
      return
    end
    for _, line in ipairs(data) do
      if line ~= "" and callbacks.on_stderr then
        callbacks.on_stderr(line)
      end
    end
  end

  state.job_id = vim.fn.jobstart({ daemon_path, "--daemon" }, {
    env = build_env(),
    on_stdout = function(_, data)
      vim.schedule(function()
        process_stdout(data)
      end)
    end,
    on_stderr = function(_, data)
      vim.schedule(function()
        process_stderr(data)
      end)
    end,
    on_exit = function(_, exit_code)
      vim.schedule(function()
        state.job_id = nil
        state.stdin = nil
        if callbacks.on_exit then
          callbacks.on_exit(exit_code)
        end
      end)
    end,
    stdin = "pipe",
  })

  if state.job_id <= 0 then
    vim.notify("[lofi] Failed to start daemon process", vim.log.levels.ERROR)
    state.job_id = nil
    return false
  end

  return true
end

--- Stop the daemon process
--- @param force boolean|nil if true, send SIGKILL instead of closing stdin
function M.stop_daemon(force)
  if not M.is_running() then
    return
  end

  local job_id = state.job_id --[[@as integer]]
  state.job_id = nil

  if force then
    vim.fn.jobstop(job_id)
  else
    -- Graceful shutdown by closing stdin (daemon exits on EOF)
    vim.fn.chanclose(job_id, "stdin")
  end
end

--- Send data to daemon stdin
--- @param data string data to send (should be newline-terminated JSON)
--- @return boolean success true if data was sent
function M.send(data)
  if not M.is_running() then
    return false
  end

  local result = vim.fn.chansend(state.job_id, data)
  return result > 0
end

--- Get the current job ID (for testing/debugging)
--- @return number|nil job ID or nil if not running
function M.get_job_id()
  return state.job_id
end

return M
