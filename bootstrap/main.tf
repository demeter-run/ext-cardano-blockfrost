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
  depends_on          = [kubernetes_namespace.namespace]
  source              = "./feature"
  namespace           = var.namespace
  operator_image_tag  = var.operator_image_tag
  metrics_delay       = var.metrics_delay
  extension_subdomain = var.extension_subdomain
  dns_zone            = var.dns_zone
  api_key_salt        = var.api_key_salt
  dcu_per_request     = var.dcu_per_request
  resources           = var.operator_resources
}

module "blockfrost_v1_proxy" {
  depends_on      = [kubernetes_namespace.namespace]
  source          = "./proxy"
  namespace       = var.namespace
  replicas        = var.proxy_replicas
  extension_name  = var.extension_name
  dns_zone        = var.dns_zone
  proxy_image_tag = var.proxy_image_tag
  resources       = var.proxy_resources
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

