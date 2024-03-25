resource "kubernetes_manifest" "monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = "operator"
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          role = "operator"
        }
      }
      podMetricsEndpoints = [
        {
          port = "metrics",
          path = "/metrics"
        }
      ]
    }
  }
}

resource "kubernetes_manifest" "instance_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = "blockfrost-instance"
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          "demeter.run/kind"            = "blockfrost_instance"
        }
      }
      podMetricsEndpoints = [
        {
          port = "api",
          path = "/prometheus"
        }
      ]
    }
  }
}
