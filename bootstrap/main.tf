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
  depends_on           = [kubernetes_namespace.namespace]
  source               = "./proxy"
  namespace            = var.namespace
  replicas             = var.proxy_blue_replicas
  proxy_image_tag      = var.proxy_blue_image_tag
  extension_name       = var.extension_name
  dns_zone             = var.dns_zone
  resources            = var.proxy_resources
  name                 = "proxy"
  tolerations          = var.proxy_blue_tolerations
  dns_names            = var.dns_names
  cache_max_size_bytes = var.proxy_cache_max_size_bytes
}

module "blockfrost_v1_proxy_green" {
  depends_on           = [kubernetes_namespace.namespace]
  source               = "./proxy"
  namespace            = var.namespace
  replicas             = var.proxy_green_replicas
  proxy_image_tag      = var.proxy_green_image_tag
  extension_name       = var.extension_name
  dns_zone             = var.dns_zone
  resources            = var.proxy_resources
  environment          = "green"
  name                 = "proxy-green"
  tolerations          = var.proxy_green_tolerations
  dns_names            = var.dns_names
  cache_max_size_bytes = var.proxy_cache_max_size_bytes
}

module "blockfrost_instances" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = var.instances
  source     = "./instance"

  namespace          = var.namespace
  network            = each.value.network
  salt               = each.value.salt
  dbsync_secret_name = local.postgres_secret_name
  image              = coalesce(each.value.image, "ghcr.io/demeter-run/ext-cardano-blockfrost-instance")
  image_tag          = each.value.image_tag
  dbsync_host        = each.value.dbsync_host
  dbsync_database    = each.value.dbsync_database
  replicas           = coalesce(each.value.replicas, 1)
  network_argument   = each.value.network_argument
  image_pull_secret  = each.value.image_pull_secret
  dbsync_max_conn    = coalesce(each.value.max_conn, 5)
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
  tolerations = coalesce(each.value.tolerations, [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-profile"
      operator = "Equal"
      value    = "mem-intensive"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-arch"
      operator = "Equal"
      value    = "arm64"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Equal"
      value    = "consistent"
    }

  ])
}

module "blockfrost_services" {
  depends_on = [kubernetes_namespace.namespace]
  for_each   = { for network in var.networks : "${network}" => network }
  source     = "./service"

  namespace = var.namespace
  network   = each.value
}

