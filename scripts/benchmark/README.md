# Benchmark

The idea of this script is to run a benchmark against Blockfrost too see if we
are missing any optimizations on our side.

# Prerequisites

* API key for [Blockfrost](https://blockfrost.io/).
* Create a Blockfrost Port on Demeter.
* `$ pip install requirements.txt`

## Running the script

```sh
python benchmark.py \
    --network preview \
    --blockfrost-project-id YOUR_BLOCKFROST_PROJECT_ID \
    --demeter-authorized-url YOUR_DEMETER_AUTHORIZED_URL \
    --repetitions 3
```

When this finished executing, it will drop a `data.csv` file with the
information needed to show the results. To show the results again, without
running the benchmark, you can run:

```sh
python benchmark --no-build
```
