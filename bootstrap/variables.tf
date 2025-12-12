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
  default = ["cardano-mainnet", "cardano-preprod", "cardano-preview"]
}

// Operator
variable "operator_image_tag" {
  type = string
}

variable "api_key_salt" {
  type = string
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
      cpu    = "1"
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
variable "dns_names" {
  type = list(string)
}

variable "proxy_cache_max_size_bytes" {
  type    = number
  default = 3000000000
}

variable "proxy_blue_dolos_endpoints" {
  type    = string
  default = "\\/blocks\\/[A-z0-9]+\\/txs\\/?$,\\/blocks\\/[A-z0-9]+\\/?$,\\/addresses\\/[A-z0-9]+\\/utxos(\\?.*)?$"
}

variable "proxy_green_dolos_endpoints" {
  type    = string
  default = "\\/blocks\\/[A-z0-9]+\\/txs\\/?$,\\/blocks\\/[A-z0-9]+\\/?$,\\/addresses\\/[A-z0-9]+\\/utxos(\\?.*)?$"
}

variable "proxy_green_image_tag" {
  type = string
}

variable "proxy_green_replicas" {
  type    = number
  default = 1
}

variable "proxy_blue_image_tag" {
  type = string
}

variable "proxy_blue_replicas" {
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
      cpu : "4",
      memory : "250Mi"
      ephemeral_storage : "4Gi"
    }
    requests : {
      cpu : "100m",
      memory : "250Mi"
      ephemeral_storage : "4Gi"
    }
  }
}

variable "instances" {
  type = map(object({
    image             = optional(string)
    image_tag         = string
    network           = string
    salt              = string
    replicas          = optional(number)
    dbsync_database   = string
    dbsync_host       = string
    network_argument  = optional(string)
    image_pull_secret = optional(string)
    max_conn          = optional(number)
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
    tolerations = optional(list(object({
      effect   = string
      key      = string
      operator = string
      value    = optional(string)
    })))
  }))
}

variable "proxy_blue_tolerations" {
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

variable "proxy_green_tolerations" {
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
