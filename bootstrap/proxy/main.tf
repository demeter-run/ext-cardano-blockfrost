locals {
  name = var.name
  role = "proxy"

  prometheus_port = 9187
  prometheus_addr = "0.0.0.0:${local.prometheus_port}"
  proxy_port      = 8080
  proxy_addr      = "0.0.0.0:${local.proxy_port}"
  # proxy_labels = var.environment != null ? { role = local.role, environment = var.environment } : { role = local.role }
  proxy_labels = var.environment != null ? { role = "${local.role}-${var.environment}" } : { role = local.role }
}

variable "name" {
  type    = string
  default = "proxy"
}

// blue - green
variable "environment" {
  default = null
}

variable "namespace" {
  type = string
}

variable "replicas" {
  type    = number
  default = 1
}

variable "proxy_image_tag" {
  type = string
}

variable "resources" {
  type = object({
    limits = object({
      cpu               = string
      memory            = string
      ephemeral_storage = string
    })
    requests = object({
      cpu               = string
      memory            = string
      ephemeral_storage = string
    })
  })
  default = {
    limits : {
      cpu : "50m",
      memory : "250Mi"
      ephemeral_storage : "4Gi"
    }
    requests : {
      cpu : "50m",
      memory : "250Mi"
      ephemeral_storage : "4Gi"
    }
  }
}

variable "blockfrost_port" {
  type    = number
  default = 3000
}

variable "dolos_enabled" {
  type    = bool
  default = true
}

variable "dolos_port" {
  type    = number
  default = 3001
}

variable "dolos_dns" {
  type    = string
  default = "ext-utxorpc-m1.svc.cluster.local"
}

variable "extension_name" {
  type = string
}

variable "versions" {
  type    = list(string)
  default = ["1"]
}

variable "dns_zone" {
  type    = string
  default = "demeter.run"
}

variable "dns_names" {
  type = list(string)
}

variable "cache_max_size_bytes" {
  type    = number
  default = 3000000000
}

variable "dolos_endpoints" {
  type    = string
  default = "\\/blocks\\/[A-z0-9]+\\/txs\\/?$,\\/blocks\\/[A-z0-9]+\\/?$,\\/addresses\\/[A-z0-9]+\\/utxos(\\?.*)?$"
}

variable "submitapi_enabled" {
  type    = bool
  default = true
}

variable "submitapi_port" {
  type    = number
  default = 8090
}

variable "submitapi_dns" {
  type    = string
  default = "ext-submitapi-m1.svc.cluster.local"
}

variable "submitapi_endpoints" {
  type    = string
  default = "\\/tx\\/submit"
}

variable "tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = [
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-profile"
      operator = "Equal"
      value    = "general-purpose"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/compute-arch"
      operator = "Equal"
      value    = "x86"
    },
    {
      effect   = "NoSchedule"
      key      = "demeter.run/availability-sla"
      operator = "Equal"
      value    = "consistent"
    }
  ]
}
