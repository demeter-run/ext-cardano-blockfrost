locals {
  name  = "blockfrost-${var.network}-${var.salt}"
  image = "${var.image}:${var.image_tag}"
}

resource "kubernetes_deployment_v1" "blockfrost" {
  wait_for_rollout = false
  metadata {
    name      = local.name
    namespace = var.namespace
    labels = {
      "demeter.run/kind"            = "blockfrost_instance"
      "cardano.demeter.run/network" = var.network
    }
  }

  spec {
    replicas = var.replicas

    selector {
      match_labels = {
        "demeter.run/instance"        = local.name
        "cardano.demeter.run/network" = var.network
        "demeter.run/kind"            = "blockfrost_instance"
      }
    }

    template {
      metadata {
        name = local.name
        labels = {
          "demeter.run/instance"        = local.name
          "cardano.demeter.run/network" = var.network
          "demeter.run/kind"            = "blockfrost_instance"
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
            value = coalesce(var.network_argument, var.network)
          }

          env {
            name  = "BLOCKFROST_CONFIG_TOKEN_REGISTRY_URL"
            value = var.token_registry_url
          }

          env {
            name  = "PGSSLMODE"
            value = "disable"
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
                name = var.dbsync_secret_name
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

