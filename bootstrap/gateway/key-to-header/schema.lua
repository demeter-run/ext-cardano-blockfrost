local PLUGIN_NAME = "key-to-header"

local schema = {
    name = PLUGIN_NAME,
    fields = {{
        config = {
            type = "record",
            fields = {{
                header_key = {
                    type = "string",
                    required = false,
                    default = "dmtr-api-key"
                }
            }}
        }
    }}
}

return schema