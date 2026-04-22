# Blockfrost Proxy

This proxy will allow Blockfrost to be accessed externally.

## Environment

| Key                    | Value                   |
| ---------------------- | ----------------------- |
| PROXY_ADDR             | 0.0.0.0:5000            |
| PROXY_NAMESPACE        |                         |
| PROMETHEUS_ADDR        | 0.0.0.0:9090            |
| SSL_CRT_PATH           | /localhost.crt          |
| SSL_KEY_PATH           | /localhost.key          |
| PROXY_TIERS_PATH       | path of tiers toml file |
| CACHE_RULES_PATH       | path for cache rules    |
| CACHE_DB_PATH          | path for cache db       |
| ROUTING_CONFIG_PATH    | path for routing rules  |
| ROUTING_POLL_INTERVAL  | routing reload seconds  |
| HEALTH_ENDPOINT        | /dmtr_health            |
| READINESS_ENDPOINT     | /ready                  |
| GRACE_PERIOD_SECONDS   | 30                      |
| GRACEFUL_SHUTDOWN_TIMEOUT_SECONDS | 5           |

## Rate limit

To define rate limits, it's necessary to create a file with the limiters available that the ports can use. The request limit of each tier can be configured using `s = second`, `m = minute`, `h = hour` and `d = day` eg: `5s` bucket of 5 seconds.

```toml
[[tiers]]
name = "tier0"
[[tiers.rates]]
interval = "1s"
limit = 1
[[tiers.rates]]
interval = "1m"
limit = 10
[[tiers.rates]]
interval = "1h"
limit = 100
[[tiers.rates]]
interval = "1d"
limit = 1000

[[tiers]]
name = "tier1"
[[tiers.rates]]
interval = "5s"
limit = 10
```

after configuring, the file path must be set at the env `PROXY_TIERS_PATH`.


## Caching

To define caching for the different endpoints, it is necessary to add a TOML
file that includes the different rules for different endpoints. This TOML
should contain a list of `rules`, with each rule containing a duration in
seconds and a regex to match the endpoint. The list will be evaluated in order.

```toml
[[rules]]
endpoint = "/blocks/latest"
duration = 300
[[rules]]
endpoint = "/epochs/latest"
duration = 300
```

After configuring, the file path must be set on the env `CACHE_RULES_PATH`.

## Routing

Routing rules live in a separate TOML file (pointed to by `ROUTING_CONFIG_PATH`)
and are hot reloaded. Routes are matched using matchit-style paths with
`{param}` segments. The default backend and per-backend settings are defined in
the routing config. Each backend declares its instance template and the networks
it supports. Templates should include the full backend host and port with
`{network}` as the only variable.

```toml
default_backend = "blockfrost"

[backends.blockfrost]
template = "blockfrost-{network}.svc.cluster.local:3000"
supported_networks = []

[backends.dolos]
template = "internal-{network}-minibf.svc.cluster.local:50051"
supported_networks = ["cardano-mainnet", "cardano-preprod", "cardano-preview"]

[backends.submitapi]
template = "submitapi-{network}.svc.cluster.local:8090"
supported_networks = ["cardano-mainnet", "cardano-preprod", "cardano-preview"]

[[routes]]
path = "/blocks/{hash}"
backend = "dolos"

[[routes]]
path = "/tx/submit"
backend = "submitapi"
```

Template variables:

- `{network}`: consumer network

Backend settings:

- `template`: backend host and port template
- `supported_networks`: list of networks this backend can serve; an empty list means all networks

The routing file is reloaded every `ROUTING_POLL_INTERVAL` seconds. If this env is not set, it defaults to 2 seconds.

## Commands

To generate the CRD will need to execute `crdgen`

```bash
cargo run --bin=crdgen
```

and execute the operator

```bash
cargo run
```

## Metrics

to collect metrics for Prometheus, an HTTP API will enable the route /metrics.

```
/metrics
```
