--- JSON-RPC client for lofi.nvim
--- Handles JSON-RPC 2.0 request/response and notification handling.
local M = {}

local daemon = require("lofi.daemon")

--- State for RPC client
local state = {
  request_id = 0,          -- Incrementing request ID
  pending = {},            -- Map of request ID -> callback
  notification_handler = nil,  -- Handler for incoming notifications
  initialized = false,     -- True if RPC started the daemon
}

--- Generate next request ID
--- @return number unique request ID
local function next_id()
  state.request_id = state.request_id + 1
  return state.request_id
end

--- Parse a JSON-RPC message from a line
--- @param line string JSON string
--- @return table|nil parsed message or nil on error
local function parse_message(line)
  local ok, msg = pcall(vim.json.decode, line)
  if not ok or type(msg) ~= "table" then
    return nil
  end
  return msg
end

--- Handle incoming message from daemon stdout
--- @param line string JSON-RPC message line
local function handle_message(line)
  local msg = parse_message(line)
  if not msg then
    return
  end

  -- Check if this is a response (has id)
  if msg.id ~= nil then
    local callback = state.pending[msg.id]
    if callback then
      state.pending[msg.id] = nil
      if msg.error then
        callback(msg.error, nil)
      else
        callback(nil, msg.result)
      end
    end
    return
  end

  -- Check if this is a notification (has method, no id)
  if msg.method and state.notification_handler then
    state.notification_handler(msg.method, msg.params)
  end
end

--- Initialize RPC module and connect to daemon
--- @param notification_handler function|nil handler for notifications: (method, params)
--- @return boolean success true if daemon is running/started
function M.init(notification_handler)
  state.notification_handler = notification_handler

  -- Already initialized with proper callbacks
  if state.initialized and daemon.is_running() then
    return true
  end

  -- Restart daemon if running without proper callbacks
  if daemon.is_running() then
    daemon.stop_daemon(true)
  end

  local ok = daemon.start_daemon({
    on_stdout = handle_message,
    on_exit = function(exit_code)
      state.initialized = false
      for id, callback in pairs(state.pending) do
        callback({ code = -32000, message = "Daemon exited", data = { exit_code = exit_code } }, nil)
        state.pending[id] = nil
      end
    end,
  })

  state.initialized = ok
  return ok
end

--- Send a JSON-RPC request to the daemon
--- @param method string RPC method name
--- @param params table|nil method parameters
--- @param callback function callback receiving (error, result)
--- @return number|nil request ID or nil if send failed
function M.send_request(method, params, callback)
  if not daemon.is_running() then
    if callback then
      vim.schedule(function()
        callback({ code = -32000, message = "Daemon not running" }, nil)
      end)
    end
    return nil
  end

  local id = next_id()
  local request = {
    jsonrpc = "2.0",
    id = id,
    method = method,
    params = params or {},
  }

  local json = vim.json.encode(request) .. "\n"
  local ok = daemon.send(json)

  if not ok then
    if callback then
      vim.schedule(function()
        callback({ code = -32000, message = "Failed to send request" }, nil)
      end)
    end
    return nil
  end

  if callback then
    state.pending[id] = callback
  end

  return id
end

--- Set notification handler
--- @param handler function handler receiving (method, params)
function M.set_notification_handler(handler)
  state.notification_handler = handler
end

--- Check if there are pending requests
--- @return boolean true if requests are pending
function M.has_pending()
  return next(state.pending) ~= nil
end

--- Cancel all pending requests
function M.cancel_all()
  for id, callback in pairs(state.pending) do
    callback({ code = -32000, message = "Request cancelled" }, nil)
    state.pending[id] = nil
  end
end

--- Shutdown RPC client and daemon
--- @param force boolean|nil force kill daemon
function M.shutdown(force)
  M.cancel_all()
  daemon.stop_daemon(force)
end

return M
