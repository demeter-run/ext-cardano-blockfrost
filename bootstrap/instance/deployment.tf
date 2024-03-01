locals {
  name  = "blockfrost-${var.network}-${var.salt}"
  image = "ghcr.io/demeter-run/cardano-blockfrost:${var.image_tag}"
}

resource "kubernetes_deployment_v1" "blockfrost" {
  metadata {
    name = local.name
    labels = {
      "demeter.run/kind"            = "blockfrost_instance"
      "cardano.demeter.run/network" = var.network
    }
  }

  spec {
    replicas = spec.replicas

    selector {
      match_labels = {
        "demeter.run/instance" = local.name
      }
    }

    template {
      metadata {
        name = local.name
        labels = {
          "demeter.run/instance" = local.name
        }
      }

      spec {
        restart_policy = "Always"

        security_context {
          fs_group = 1000
        }

        container {
          name              = "main"
          image             = local.image
          image_pull_policy = "IfNotPresent"
          args              = ["yarn", "start"]

          env {
            name  = "BLOCKFROST_CONFIG_SERVER_LISTEN_ADDRESS"
            value = "0.0.0.0"
          }

          env {
            name  = "BLOCKFROST_CONFIG_SERVER_PORT"
            value = var.server_port
          }

          env {
            name  = "BLOCKFROST_CONFIG_SERVER_DEBUG"
            value = var.server_debug
          }

          env {
            name  = "BLOCKFROST_CONFIG_SERVER_PROMETHEUS_METRICS"
            value = var.server_prometheus_metrics
          }

          env {
            name  = "BLOCKFROST_CONFIG_DBSYNC_HOST"
            value = var.dbsync_host
          }

          env {
            name  = "BLOCKFROST_CONFIG_DBSYNC_DATABASE"
            value = var.dbsync_database
          }

          env {
            name  = "BLOCKFROST_CONFIG_DBSYNC_MAX_CONN"
            value = var.dbsync_max_conn
          }

          env {
            name  = "BLOCKFROST_CONFIG_NETWORK"
            value = var.network
          }

          env {
            name  = "BLOCKFROST_CONFIG_TOKEN_REGISTRY_URL"
            value = var.token_registry_url
          }

          env {
            name  = "PGSSLMODE"
            value = "no-verify"
          }

          env {
            name = "BLOCKFROST_CONFIG_DBSYNC_USER"

            value_from {
              secret_key_ref {
                key  = "username"
                name = var.dbsync_secret_name
              }
            }
          }

          env {
            name = "PGPASSWORD"

            value_from {
              secret_key_ref {
                key  = "password"
                name = spec.dbsync_secret_name
              }
            }
          }
          port {
            container_port = var.server_port
            name           = "api"
          }

          resources {
            limits = {
              cpu    = var.resources.limits.cpu
              memory = var.resources.limits.memory
            }
            requests = {
              cpu    = var.resources.requests.cpu
              memory = var.resources.requests.memory
            }
          }
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-profile"
          operator = "Equal"
          value    = "general-purpose"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/compute-arch"
          operator = "Equal"
          value    = "x86"
        }

        toleration {
          effect   = "NoSchedule"
          key      = "demeter.run/availability-sla"
          operator = "Equal"
          value    = "consistent"
        }
      }
    }
  }
}

