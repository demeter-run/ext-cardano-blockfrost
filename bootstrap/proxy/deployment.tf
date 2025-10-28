resource "kubernetes_deployment_v1" "blockfrost_proxy" {
  wait_for_rollout = false
  depends_on       = [kubernetes_manifest.certificate_cluster_wildcard_tls]

  metadata {
    name      = local.name
    namespace = var.namespace
    labels    = local.proxy_labels
  }
  spec {
    replicas = var.replicas
    selector {
      match_labels = local.proxy_labels
    }
    strategy {
      rolling_update {
        max_surge       = 2
        max_unavailable = 0
      }
    }
    template {
      metadata {
        name   = local.name
        labels = local.proxy_labels
      }
      spec {
        container {
          name              = "main"
          image             = "ghcr.io/demeter-run/ext-cardano-blockfrost-proxy:${var.proxy_image_tag}"
          image_pull_policy = "IfNotPresent"

          resources {
            limits = {
              cpu                 = var.resources.limits.cpu
              memory              = var.resources.limits.memory
              "ephemeral-storage" = var.resources.limits.ephemeral_storage
            }
            requests = {
              cpu                 = var.resources.requests.cpu
              memory              = var.resources.requests.memory
              "ephemeral-storage" = var.resources.requests.ephemeral_storage
            }
          }

          port {
            name           = "metrics"
            container_port = local.prometheus_port
            protocol       = "TCP"
          }

          port {
            name           = "proxy"
            container_port = local.proxy_port
            protocol       = "TCP"
          }

          env {
            name  = "PROXY_NAMESPACE"
            value = var.namespace
          }

          env {
            name  = "PROXY_ADDR"
            value = local.proxy_addr
          }

          env {
            name  = "PROMETHEUS_ADDR"
            value = local.prometheus_addr
          }

          env {
            name  = "BLOCKFROST_PORT"
            value = var.blockfrost_port
          }

          env {
            name  = "BLOCKFROST_DNS"
            value = "${var.namespace}.svc.cluster.local"
          }

          env {
            name  = "DOLOS_ENABLED"
            value = var.dolos_enabled
          }

          env {
            name  = "DOLOS_PORT"
            value = var.dolos_port
          }

          env {
            name  = "DOLOS_DNS"
            value = var.dolos_dns
          }

          env {
            name  = "DEFAULT_BLOCKFROST_VERSION"
            value = "v1"
          }

          env {
            name  = "SSL_CRT_PATH"
            value = "/certs/tls.crt"
          }

          env {
            name  = "SSL_KEY_PATH"
            value = "/certs/tls.key"
          }

          env {
            name  = "PROXY_TIERS_PATH"
            value = "/configs/tiers.toml"
          }

          env {
            name  = "CACHE_RULES_PATH"
            value = "/configs/cache_rules.toml"
          }

          env {
            name  = "CACHE_DB_PATH"
            value = "/cache/cache.redb"
          }

          env {
            name  = "CACHE_MAX_SIZE_BYTES"
            value = var.cache_max_size_bytes
          }

          env {
            name  = "FORBIDDEN_ENDPOINTS"
            value = "/network,/pools/extended,/pools/\\w+$"
          }

          env {
            name  = "DOLOS_ENDPOINTS"
            value = var.dolos_endpoints
          }

          env {
            name  = "SUBMITAPI_ENABLED"
            value = var.submitapi_enabled
          }

          env {
            name  = "SUBMITAPI_PORT"
            value = var.submitapi_port
          }

          env {
            name  = "SUBMITAPI_DNS"
            value = var.submitapi_dns
          }

          volume_mount {
            mount_path = "/certs"
            name       = "certs"
          }

          volume_mount {
            mount_path = "/configs"
            name       = "configs"
          }

          volume_mount {
            name       = "ephemeral"
            mount_path = "/cache"
          }
        }

        volume {
          name = "certs"
          secret {
            secret_name = local.cert_secret_name
          }
        }

        volume {
          name = "configs"
          config_map {
            name = kubernetes_config_map.proxy.metadata.0.name
          }
        }

        volume {
          name = "ephemeral"
          empty_dir {
            size_limit = var.resources.limits.ephemeral_storage
          }
        }

        dynamic "toleration" {
          for_each = var.tolerations
          content {
            effect   = toleration.value.effect
            key      = toleration.value.key
            operator = toleration.value.operator
            value    = toleration.value.value
          }
        }
      }
    }
  }
}
