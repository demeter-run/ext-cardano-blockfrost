// numbers here should consider number of proxy replicas
locals {
  tiers = [
    {
      "name" = "0",
      "rates" = [
        {
          "interval" = "1m",
          "limit"    = floor(5 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(430000 / var.replicas)
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
      "rates" =[
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
      "rates" =[
        {
          "interval" = "1m",
          "limit"    = floor(300 * 60 / var.replicas)
        },
        {
          "interval" = "1d",
          "limit"    = floor(26000000 / var.replicas)
        }
      ] 
    }
  ]
}

resource "kubernetes_config_map" "proxy" {
  metadata {
    namespace = var.namespace
    name      = "proxy-config"
  }

  data = {
    "tiers.toml"       = "${templatefile("${path.module}/proxy-config.toml.tftpl", { tiers = local.tiers })}"
    "cache_rules.toml" = file("${path.module}/cache_rules.toml")
  }
}
