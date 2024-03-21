variable "namespace" {
  type = string
}

variable "network" {
  type = string

  validation {
    condition     = contains(["mainnet", "preprod", "preview"], var.network)
    error_message = "Invalid network. Allowed values are mainnet, preprod and preview."
  }
}

variable "salt" {
  type = string
}

variable "dbsync_secret_name" {
  type = string
}

variable "image_tag" {
  type = string
}

variable "replicas" {
  type    = number
  default = 1
}

variable "server_port" {
  type    = number
  default = 3000
}

variable "server_debug" {
  type    = bool
  default = false
}

variable "server_prometheus_metrics" {
  type    = bool
  default = true
}

variable "dbsync_host" {
  type = string
}

variable "dbsync_database" {
  type = string
}

variable "dbsync_max_conn" {
  type    = number
  default = 10
}

variable "token_registry_url" {
  type    = string
  default = "https://tokens.cardano.org"
}

variable "resources" {
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
    limits : {
      cpu : "200m"
      memory : "400Mi"
    }
    requests : {
      cpu : "200m"
      memory : "400Mi"
    }
  }
}
