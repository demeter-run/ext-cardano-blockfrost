variable "namespace" {
  type = string
}

variable "api_key_salt" {
  type = string
}

variable "ingress_class" {
  type = string
}

variable "dns_zone" {
  type = string
}

variable "operator_image_tag" {
  type = string
}


variable "metrics_delay" {
  description = "The inverval for polling metrics data (in seconds)"
  default     = "30"
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
