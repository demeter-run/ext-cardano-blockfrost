local plugin = {
  PRIORITY = 1251,
  VERSION = "1.0.0"
}

function plugin:access(plugin_conf)
  local host, err = kong.request.get_host()
  if err then
      kong.log.err(err)
      return
  end

  local host_pattern = "(dmtr_[%w%d-]+)%.[%w]+.+"

  local match = string.match(host, host_pattern)

  if match then
      kong.service.request.set_header(plugin_conf.header_key, match)
  end
end

return plugin