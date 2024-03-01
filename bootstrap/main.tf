resource "kubernetes_namespace" "namespace" {
  metadata {
    name = var.namespace
  }
}

module "blockfrost_v1_feature" {
  depends_on         = [kubernetes_namespace.namespace]
  source             = "./feature"
  namespace          = var.namespace
  operator_image_tag = var.operator_image_tag
  metrics_delay      = var.metrics_delay
  ingress_class      = var.ingress_class
  dns_zone           = var.dns_zone
  api_key_salt       = var.api_key_salt
  dcu_per_request    = var.dcu_per_request
}

module "blockfrost_v1_gateway" {
  depends_on     = [kubernetes_namespace.namespace]
  source         = "./gateway"
  namespace      = var.namespace
  replicas       = var.gateway_replicas
  dns_zone       = var.dns_zone
  networks       = var.networks
  extension_name = var.extension_name
}

module "blockfrost_configs" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = { for network in var.networks : "${network}" => network }

  source    = "./configs"
  namespace = var.namespace
  network   = each.value
}

module "blockfrost_instances" {
  depends_on = [kubernetes_namespace.namespace, module.blockfrost_configs]
  for_each   = var.instances
  source     = "./instance"

  namespace          = var.namespace
  network            = each.value.network
  salt               = each.value.salt
  dbsync_secret_name = var.dbsync_secret_name
  image_tag          = each.value.image_tag
  dbsync_host        = var.dbsync_host
  replicas           = coalesce(each.value.replicas, 1)
  resources = coalesce(each.value.resources, {
    limits : {
      cpu : "200m"
      memory : "400Mi"
    }
    requests : {
      cpu : "200m"
      memory : "400Mi"
    }
  })
}

module "blockfrost_services" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = { for network in var.networks : "${network}" => network }
  source     = "./service"

  namespace = var.namespace
  network   = each.value.network
}

