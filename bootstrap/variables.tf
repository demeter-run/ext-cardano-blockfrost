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

variable "extension_subdomain" {
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
  type = string
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

variable "operator_resources" {
  type = object({
    limits = object({
      cpu    = string
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    limits = {
      cpu    = "50m"
      memory = "512Mi"
    }
    requests = {
      cpu    = "50m"
      memory = "512Mi"
    }
  }
}

variable "metrics_delay" {
  type    = number
  default = 60
}

// Instance 
variable "dbsync_creds" {
  type = object({
    username = string
    password = string
  })
}

// Proxy
variable "proxy_image_tag" {
  type = string
}

variable "proxy_replicas" {
  type    = number
  default = 1
}

variable "proxy_resources" {
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

variable "instances" {
  type = map(object({
    image_tag       = string
    network         = string
    salt            = string
    replicas        = optional(number)
    dbsync_database = string
    dbsync_host     = string
    resources = optional(object({
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
