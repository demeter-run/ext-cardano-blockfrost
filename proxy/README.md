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
| BLOCKFROST_PORT        |                         |
| BLOCKFROST_DNS         | internal k8s dns        |
| PROXY_TIERS_PATH       | path of tiers toml file |
| CACHE_RULES_PATH       | path for cache rules    |
| CACHE_DB_PATH          | path for cache db       |

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
