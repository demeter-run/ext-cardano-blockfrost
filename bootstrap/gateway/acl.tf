
resource "kubernetes_manifest" "acl_cluster_plugin" {
  for_each = var.networks
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongClusterPlugin"
    "metadata" = {
      "name" = "acl-${var.feature_name}-${each.key}"
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
      "labels" : {
        "global" : "false"
      }
    }
    "config" = {
      "allow" = ["${each.key}"]
    }
    "plugin" = "acl"
  }
}
