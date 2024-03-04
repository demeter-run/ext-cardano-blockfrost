locals {
  postgres_secret_name = "dbsync-postgres-creds"
}

resource "kubernetes_namespace" "namespace" {
  metadata {
    name = var.namespace
  }
}

resource "kubernetes_secret" "blockfrost" {
  metadata {
    namespace = var.namespace
    name      = local.postgres_secret_name
  }

  data = {
    username = var.dbsync_creds.username
    password = var.dbsync_creds.password
  }

  type = "Opaque"
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

module "blockfrost_instances" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = var.instances
  source     = "./instance"

  namespace          = var.namespace
  network            = each.value.network
  salt               = each.value.salt
  dbsync_secret_name = local.postgres_secret_name
  image_tag          = each.value.image_tag
  dbsync_host        = each.value.dbsync_host
  dbsync_database    = each.value.dbsync_database
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
  network   = each.value
}

