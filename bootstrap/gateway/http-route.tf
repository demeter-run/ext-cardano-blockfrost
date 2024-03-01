resource "kubernetes_manifest" "http_route" {
  for_each = var.networks

  manifest = {
    "apiVersion" = "gateway.networking.k8s.io/v1"
    "kind"       = "HTTPRoute"
    "metadata" = {
      "annotations" = {
        "konghq.com/plugins" = "${var.feature_name}-key-to-header, ${var.feature_name}-auth, acl-${var.feature_name}-${each.key}"
      }
      "labels" = {
        "demeter.run/kind"    = "http-route"
        "demeter.run/tenancy" = "proxy"
      }
      "name"      = "${var.feature_name}-${each.key}"
      "namespace" = var.namespace
    }
    "spec" = {
      "hostnames" = [
        "${each.key}.${var.extension_name}.demeter.run",
        "*.${each.key}.${var.extension_name}.demeter.run"
      ]
      "parentRefs" = [
        {
          "group"     = "gateway.networking.k8s.io"
          "kind"      = "Gateway"
          "name"      = var.extension_name
          "namespace" = var.namespace
        },
      ]
      "rules" = [
        {
          "backendRefs" = [
            {
              "group"  = ""
              "kind"   = "Service"
              "name"   = "${var.feature_name}-${each.key}"
              "port"   = var.service_port
              "weight" = 1
            },
          ]
          "matches" = [
            {
              "path" = {
                "type"  = "PathPrefix"
                "value" = "/"
              }
            },
          ]
        },
      ]
    }
  }
}

resource "kubernetes_manifest" "kong_auth_plugin" {
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongPlugin"
    "metadata" = {
      "name"      = "${var.feature_name}-auth"
      "namespace" = var.namespace
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
    }
    "config" = {
      "key_names" = [
        "dmtr-api-key"
      ]
    }
    "plugin" = "key-auth"
  }
}

resource "kubernetes_manifest" "kong_key_to_header_plugin" {
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongPlugin"
    "metadata" = {
      "name"      = "${var.feature_name}-key-to-header"
      "namespace" = var.namespace
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
    }
    config   = {}
    "plugin" = "key-to-header"
  }
}

