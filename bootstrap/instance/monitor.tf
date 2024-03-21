resource "kubernetes_manifest" "instance_monitor" {
  manifest = {
    apiVersion = "monitoring.coreos.com/v1"
    kind       = "PodMonitor"
    metadata = {
      labels = {
        "app.kubernetes.io/component" = "o11y"
        "app.kubernetes.io/part-of"   = "demeter"
      }
      name      = "blockfrost_instance"
      namespace = var.namespace
    }
    spec = {
      selector = {
        matchLabels = {
          "demeter.run/kind"            = "blockfrost_instance"
          "cardano.demeter.run/network" = var.network
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
