variable "namespace" {
  type = string
}

variable "replicas" {
  type = number
}

variable "dns_zone" {
  type = string
}

variable "networks" {
  type = set(string)
}

variable "extension_name" {
  type    = string
  default = "blockfrost-v1"
}

variable "feature_name" {
  type    = string
  default = "blockfrost"
}

variable "service_port" {
  type    = number
  default = 3000
}

resource "helm_release" "ingress" {
  name             = "${var.extension_name}-ingress"
  repository       = "https://charts.konghq.com"
  chart            = "kong"
  create_namespace = false
  namespace        = var.namespace
  version          = "2.34.0"

  set {
    name  = "certificates.enabled"
    value = "false"
  }

  set {
    name  = "proxy.ingress.tls"
    value = "${var.extension_name}-wildcard-tls"
  }

  set {
    name  = "proxy.http.parameters[0]"
    value = "http2"
  }

  set {
    name  = "replicaCount"
    value = var.replicas
  }

  set {
    name  = "env.router_flavor"
    value = "traditional_compatible"
  }

  set {
    name  = "env.plugins"
    value = "bundled\\,key-to-header"
  }

  set {
    name  = "env.allow_debug_header"
    value = "true"
  }

  set {
    name  = "plugins.configMaps[0].name"
    value = "kong-plugin-key-to-header"
  }

  set {
    name  = "plugins.configMaps[0].pluginName"
    value = "key-to-header"
  }

  set {
    name  = "resources.requests.cpu"
    value = "500m"
  }

  set {
    name  = "resources.requests.memory"
    value = "1Gi"
  }

  set {
    name  = "resources.limits.memory"
    value = "1Gi"
  }

  set {
    name  = "env.log_level"
    value = "info"
  }

  set {
    name  = "ingressController.env.feature_gates"
    value = "GatewayAlpha=true"
  }

  set {
    name  = "ingressController.env.gateway_api_controller_name"
    value = "konghq.com/kic-gateway-controller-${var.extension_name}"
  }

  set {
    name  = "proxy.annotations.service\\.beta\\.kubernetes\\.io/aws-load-balancer-nlb-target-type"
    value = "instance"
  }

  set {
    name  = "proxy.annotations.service\\.beta\\.kubernetes\\.io/aws-load-balancer-scheme"
    value = "internet-facing"
  }

  set {
    name  = "proxy.annotations.service\\.beta\\.kubernetes\\.io/aws-load-balancer-type"
    value = "external"
  }

  set {
    name  = "ingressController.ingressClass"
    value = var.extension_name
  }

  set {
    name  = "tolerations[0].effect"
    value = "NoSchedule"
  }

  set {
    name  = "tolerations[0].operator"
    value = "Exists"
  }

  set {
    name  = "tolerations[0].key"
    value = "demeter.run/compute-profile"
  }

  set {
    name  = "tolerations[1].effect"
    value = "NoSchedule"
  }

  set {
    name  = "tolerations[1].operator"
    value = "Exists"
  }

  set {
    name  = "tolerations[1].key"
    value = "demeter.run/compute-arch"
  }

  set {
    name  = "tolerations[2].effect"
    value = "NoSchedule"
  }

  set {
    name  = "tolerations[2].operator"
    value = "Equal"
  }

  set {
    name  = "tolerations[2].key"
    value = "demeter.run/availability-sla"
  }

  set {
    name  = "tolerations[2].value"
    value = "consistent"
  }

  set {
    name  = "serviceMonitor.enabled"
    value = "true"
  }

  set {
    name  = "serviceMonitor.labels.app\\.kubernetes\\.io/component"
    value = "o11y"
  }

  set {
    name  = "serviceMonitor.labels.app\\.kubernetes\\.io/part-of"
    value = "demeter"
  }

}


resource "kubernetes_manifest" "gateway_class" {
  manifest = {
    "apiVersion" = "gateway.networking.k8s.io/v1"
    "kind"       = "GatewayClass"
    "metadata" = {
      "name" = var.extension_name
      "annotations" = {
        "konghq.com/gatewayclass-unmanaged" = "true"
      }
    }
    "spec" = {
      "controllerName" = "konghq.com/kic-gateway-controller-${var.extension_name}"
    }
  }
}

resource "kubernetes_manifest" "gateway" {
  field_manager {
    force_conflicts = true
  }

  manifest = {
    "apiVersion" = "gateway.networking.k8s.io/v1"
    "kind"       = "Gateway"
    "metadata" = {
      "name"      = var.extension_name
      "namespace" = var.namespace
      "annotations" = {
        "cert-manager.io/cluster-issuer" : "letsencrypt"
        "konghq.com/gatewayclass-unmanaged" : "${var.namespace}/${var.extension_name}-ingress-kong-proxy"
      }
    }
    "spec" = {
      "gatewayClassName" = var.extension_name
      "listeners" : [
        {
          "allowedRoutes" : {
            "namespaces" : {
              "from" : "All"
            }
          }
          "name" : "proxy"
          "hostname" : "*.${var.extension_name}.${var.dns_zone}"
          "port" : 80
          "protocol" : "HTTP"
        },
        {
          "allowedRoutes" : {
            "namespaces" : {
              "from" : "All"
            }
          }
          "name" : "https-proxy"
          "hostname" : "*.${var.extension_name}.${var.dns_zone}"
          "port" : 443
          "protocol" : "HTTPS"
          "tls" : {
            "certificateRefs" : [
              {
                "group" : "",
                "kind" : "Secret",
                "name" : "${var.extension_name}-wildcard-tls"
              }
            ]
          }
        },
        {
          "allowedRoutes" : {
            "namespaces" : {
              "from" : "All"
            }
          }
          "name" : "authenticated-mainnet"
          "hostname" : "*.mainnet.${var.extension_name}.${var.dns_zone}"
          "port" : 443
          "protocol" : "HTTPS"
          "tls" : {
            "certificateRefs" : [
              {
                "group" : "",
                "kind" : "Secret",
                "name" : "${var.extension_name}-wildcard-tls"
              }
            ]
          }
        },
        {
          "allowedRoutes" : {
            "namespaces" : {
              "from" : "All"
            }
          }
          "name" : "authenticated-preview"
          "hostname" : "*.preview.${var.extension_name}.${var.dns_zone}"
          "port" : 443
          "protocol" : "HTTPS"
          "tls" : {
            "certificateRefs" : [
              {
                "group" : "",
                "kind" : "Secret",
                "name" : "${var.extension_name}-wildcard-tls"
              }
            ]
          }
        },
        {
          "allowedRoutes" : {
            "namespaces" : {
              "from" : "All"
            }
          }
          "name" : "authenticated-preprod"
          "hostname" : "*.preprod.${var.extension_name}.${var.dns_zone}"
          "port" : 443
          "protocol" : "HTTPS"
          "tls" : {
            "certificateRefs" : [
              {
                "group" : "",
                "kind" : "Secret",
                "name" : "${var.extension_name}-wildcard-tls"
              }
            ]
          }
        }

      ]
    }
  }
}


resource "kubernetes_manifest" "prometheus_plugin" {
  manifest = {
    "apiVersion" = "configuration.konghq.com/v1"
    "kind"       = "KongClusterPlugin"
    "metadata" = {
      "name" = "prometheus-${var.extension_name}"
      "annotations" = {
        "kubernetes.io/ingress.class" = var.extension_name
      }
      "labels" = {
        "global" = "true"
      }
    }
    "config" = {
      "per_consumer"            = true
      "bandwidth_metrics"       = true
      "latency_metrics"         = true
      "status_code_metrics"     = true
      "upstream_health_metrics" = true
    }
    "plugin" = "prometheus"
  }
}

