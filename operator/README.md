# Blockfrost Operator

This operator will create a key into the CRD to allow Blockfrost to be accessed externally.

## Environment

| Key                        | Value                         |
| ---------------------------| ----------------------------- |
| ADDR                       | 0.0.0.0:5000                  |
| EXTENSION_SUBDOMAIN        | blockfrost-m1                 |
| API_KEY_SALT               | blockfrost-salt               |
| METRICS_DELAY              | 40                            |
| PROMETHEUS_URL             |                               |
| DCU_PER_REQUEST            | preview=5,preprod=5,mainnet=5 |

## Port CRD

To define a new port, a new k8s manifest needs to be created and set the configuration values.

```yml
apiVersion: demeter.run/v1alpha1
kind: BlockfrostPort
metadata:
  name: blockfrost-port-a123ds
  namespace: prj-mainnet-test
spec:
  operatorVersion: "1"
  network: mainnet
  throughputTier: "0"
```

`network`: The Blockfrost network the port will consume.
`throughputTier`: The tier to limit how many requests the port can do. The tiers will be configured in *tiers.toml* on the proxy.

## Commands

To generate the CRD will need to execute crdgen

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
