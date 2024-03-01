locals {
  rate_limiting_tiers = {
    "0" = {
      "minute" = 10
    },
    "1" = {
      "minute" = 100
    },
    "2" = {
      "minute" = 1000
    },
  }
}

resource "kubernetes_manifest" "rate_limiting_cluster_plugin" {
  for_each = local.rate_limiting_tiers
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongClusterPlugin"
    "metadata" = {
      "name" = "rate-limiting-${var.feature_name}-tier-${each.key}"
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
      "labels" : {
        "global" : "false"
      }
    }
    "config" = {
      "minute" = each.value.minute
      "policy" = "local"
    }
    "plugin" = "rate-limiting"
  }
}
