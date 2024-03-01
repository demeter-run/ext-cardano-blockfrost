locals {
  role = "operator"
  port = 9946
}

resource "kubernetes_deployment_v1" "operator" {
  wait_for_rollout = false

  metadata {
    namespace = var.namespace
    name      = local.role
    labels = {
      role = local.role
    }
  }

  spec {
    replicas = 1

    selector {
      match_labels = {
        role = local.role
      }
    }

    template {
      metadata {
        labels = {
          role = local.role
        }
      }

      spec {
        container {
          image = "ghcr.io/demeter-run/ext-cardano-blockfrost-operator:${var.operator_image_tag}"
          name  = "main"

          env {
            name  = "PORT"
            value = local.port
          }

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "PROMETHEUS_URL"
            value = "http://prometheus-operated.demeter-system.svc.cluster.local:9090/api/v1"
          }

          env {
            name  = "METRICS_DELAY"
            value = var.metrics_delay
          }

          env {
            name  = "DCU_PER_REQUEST_MAINNET"
            value = var.dcu_per_request["mainnet"]
          }

          env {
            name  = "DCU_PER_REQUEST_PREPROD"
            value = var.dcu_per_request["preprod"]
          }

          env {
            name  = "DCU_PER_REQUEST_PREVIEW"
            value = var.dcu_per_request["preview"]
          }

          env {
            name  = "DCU_PER_REQUEST_SANCHONET"
            value = var.dcu_per_request["sanchonet"]
          }

          env {
            name  = "API_KEY_SALT"
            value = var.api_key_salt
          }

          env {
            name  = "INGRESS_CLASS"
            value = var.ingress_class
          }

          env {
            name  = "DNS_ZONE"
            value = var.dns_zone
          }

          resources {
            limits = {
              memory = "512Mi"
            }
            requests = {
              cpu    = "50m"
              memory = "512Mi"
            }
          }

          port {
            name           = "metrics"
            container_port = local.port
            protocol       = "TCP"
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
