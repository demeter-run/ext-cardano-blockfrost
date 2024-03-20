locals {
  name = "proxy"
  role = "proxy"

  prometheus_port = 9187
  prometheus_addr = "0.0.0.0:${local.prometheus_port}"
  proxy_port      = 8080
  proxy_addr      = "0.0.0.0:${local.proxy_port}"
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
      cpu    = string
      memory = string
      ephemeral_storage = string
    })
    requests = object({
      cpu    = string
      memory = string
      ephemeral_storage = string
    })
  })
  default = {
    limits : {
      cpu : "50m",
      memory : "250Mi"
      ephemeral_storage = "4Gi"
    }
    requests : {
      cpu : "50m",
      memory : "250Mi"
      ephemeral_storage = "4Gi"
    }
  }
}

variable "blockfrost_port" {
  type    = number
  default = 3000
}


variable "extension_name" {
  type = string
}

variable "networks" {
  type = list(string)
  default = ["mainnet", "preprod", "preview"]
}

variable "versions" {
  type = list(string)
  default = ["2"]
}

variable "dns_zone" {
  type    = string
  default = "demeter.run"
}
