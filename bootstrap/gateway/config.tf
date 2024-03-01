resource "kubernetes_config_map" "key-to-header-plugin" {
  metadata {
    namespace = var.namespace
    name      = "kong-plugin-key-to-header"
  }

  data = {
    "schema.lua"  = "${file("${path.module}/key-to-header/schema.lua")}"
    "handler.lua" = "${file("${path.module}/key-to-header/handler.lua")}"
  }
}
