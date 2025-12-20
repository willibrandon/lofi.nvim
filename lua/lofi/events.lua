--- Event system for lofi.nvim
--- Provides pub/sub event handling for generation lifecycle events.
local M = {}

--- Event names
M.EVENTS = {
  GENERATION_START = "generation_start",
  GENERATION_PROGRESS = "generation_progress",
  GENERATION_COMPLETE = "generation_complete",
  GENERATION_ERROR = "generation_error",
  DOWNLOAD_PROGRESS = "download_progress",
}

--- Registered event handlers
--- @type table<string, function[]>
local handlers = {}

--- Register an event handler
--- @param event string event name from M.EVENTS
--- @param callback function handler receiving event data
--- @return function unsubscribe function to remove the handler
function M.on(event, callback)
  if not handlers[event] then
    handlers[event] = {}
  end
  table.insert(handlers[event], callback)

  -- Return unsubscribe function
  return function()
    M.off(event, callback)
  end
end

--- Remove an event handler
--- @param event string event name
--- @param callback function handler to remove
function M.off(event, callback)
  if not handlers[event] then
    return
  end

  for i, handler in ipairs(handlers[event]) do
    if handler == callback then
      table.remove(handlers[event], i)
      return
    end
  end
end

--- Emit an event to all registered handlers
--- @param event string event name
--- @param data table event data
function M.emit(event, data)
  if not handlers[event] then
    return
  end

  for _, handler in ipairs(handlers[event]) do
    local ok, err = pcall(handler, data)
    if not ok then
      vim.notify(string.format("[lofi] Event handler error for %s: %s", event, err), vim.log.levels.WARN)
    end
  end
end

--- Clear all handlers for an event
--- @param event string|nil event name, or nil to clear all
function M.clear(event)
  if event then
    handlers[event] = nil
  else
    handlers = {}
  end
end

--- Get count of handlers for an event
--- @param event string event name
--- @return number handler count
function M.handler_count(event)
  if not handlers[event] then
    return 0
  end
  return #handlers[event]
end

return M
