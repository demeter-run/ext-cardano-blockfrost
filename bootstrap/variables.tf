variable "namespace" {
  type = string
}

variable "dns_zone" {
  type    = string
  default = "demeter.run"
}

variable "extension_name" {
  type    = string
  default = "blockfrost-m1"
}

variable "ingress_class" {
  type    = string
  default = "blockfrost-m1"
}

variable "networks" {
  type    = list(string)
  default = ["mainnet", "preprod", "preview"]
}

// Operator
variable "operator_image_tag" {
  type = string
}

variable "api_key_salt" {
  type    = string
  default = "blockfrost-salt"
}

variable "dcu_per_request" {
  type = map(string)
  default = {
    "mainnet"   = "10"
    "preprod"   = "5"
    "preview"   = "5"
    "sanchonet" = "5"
  }
}

variable "metrics_delay" {
  type    = number
  default = 60
}

// Instance 
variable "dbsync_secret_name" {
  type = string
}

variable "dbsync_host" {
  type = string
}

// Gateway
variable "gateway_replicas" {
  type    = number
  default = 1
}

variable "instances" {
  type = map(object({
    image_tag = string
    network   = string
    salt      = string
    replicas  = option(number)
    resources = option(object({
      limits = object({
        cpu    = string
        memory = string
      })
      requests = object({
        cpu    = string
        memory = string
      })
    }))
  }))
}
