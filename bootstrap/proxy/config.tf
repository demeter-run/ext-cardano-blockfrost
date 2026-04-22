// numbers here should consider number of proxy replicas
locals {
  tiers = [
    {
      "name" = "0",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(86400 / var.replicas)
        }
      ]
    },
    {
      "name" = "1",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(20 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(1700000 / var.replicas)
        }
      ]
    },
    {
      "name" = "2",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(100 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(8600000 / var.replicas)
        }
      ]
    },
    {
      "name" = "3",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(800 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(69120000 / var.replicas)
        }
      ]
    }
  ]

  configmap_name = var.environment != null ? "proxy-${var.environment}-config" : "proxy-config"
  // Final dot to avoid external dns resolution
  routing_backends = {
    blockfrost = {
      template           = "blockfrost-{network}.${var.namespace}.svc.cluster.local.:3000"
      supported_networks = [] // All networks
    }
    dolos = {
      template           = "internal-{network}-minibf.${var.dolos_dns}.:3000"
      supported_networks = ["cardano-mainnet", "cardano-preprod", "cardano-preview"]
    }
    submitapi = {
      template           = "submitapi-{network}.${var.submitapi_dns}.:8090"
      supported_networks = []
    }
  }
}

resource "kubernetes_config_map" "proxy" {
  metadata {
    namespace = var.namespace
    name      = local.configmap_name
  }

  data = {
    "tiers.toml"       = "${templatefile("${path.module}/proxy-config.toml.tftpl", { tiers = local.tiers })}"
    "cache_rules.toml" = file("${path.module}/cache_rules.toml")
    "routing.toml" = "${templatefile("${path.module}/routing.toml.tftpl", {
      default_backend = "blockfrost"
      backends        = local.routing_backends
      routes          = var.routing_routes
    })}"
  }
}
