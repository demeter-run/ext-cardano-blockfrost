import argparse
import time

import pandas as pd
import requests
from rich.console import Console
from rich.table import Table
from rich.text import Text


def get_parser():
    parser = argparse.ArgumentParser(
        description="Benchmark Demeter's Blockfrost RYO against Blockfrost.",
    )

    parser.add_argument(
        "-n", "--network", choices=["mainnet", "preview", "preprod"], default=None
    )
    parser.add_argument("-r", "--repetitions", type=int, default=5)
    parser.add_argument("-t", "--timeout", type=int, default=10)
    parser.add_argument("--blockfrost-project-id", default=None)
    parser.add_argument("--demeter-api-key", default=None)

    parser.add_argument("--no-build", action="store_true")

    return parser


# Only valid on mainnet
NUTLINK_ADDRESS = "addr1v8yczm692pktwlvjfgwucrullt6af0lme7rh97fhfw2fgjc4chr79"
TICKER = "ADABTC"


def time_function(func):
    start = time.time()
    func()
    return time.time() - start


def build(args):
    if args.network is None:
        raise ValueError("Network is undefined.")

    if args.blockfrost_project_id is None:
        raise ValueError("Blockfrost project ID is undefined.")

    if args.demeter_api_key is None:
        raise ValueError("Demeter API key is undefined.")

    def query_demeter(endpoint, params):
        response = requests.get(
            f"https://{args.network}.blockfrost-m1.demeter.run"
            + endpoint.format(**params),
            headers={"dmtr-api-key": args.demeter_api_key},
            timeout=args.timeout,
        )
        response.raise_for_status()

    def query_blockfrost(endpoint, params):
        response = requests.get(
            f"https://cardano-{args.network}.blockfrost.io/api/v0"
            + endpoint.format(**params),
            headers={"project_id": args.blockfrost_project_id},
            timeout=args.timeout,
        )
        response.raise_for_status()
        return response.json()

    # Get valid parameters for endpoints
    block = None
    block_previous_hash = None
    while block is None:
        if block_previous_hash is None:
            block = query_blockfrost("/blocks/latest", {})
        else:
            block = query_blockfrost("/blocks/{hash}", {"hash": block_previous_hash})

        block_previous_hash = block["previous_block"]
        if block["tx_count"] == 0:
            block = None

    block_hash = block["hash"]
    epoch_number = block["epoch"]
    epoch_slot = block["epoch_slot"]
    block_slot = block["slot"]

    address_object = query_blockfrost(f"/blocks/{block_hash}/addresses", {})[0]
    address = address_object["address"]
    tx_hash = address_object["transactions"][0]["tx_hash"]
    stake_address = query_blockfrost(f"/addresses/{address}", {})["stake_address"]

    asset = query_blockfrost(f"/assets", {})[0]["asset"]
    policy_id = query_blockfrost(f"/assets/{asset}", {})["policy_id"]

    pool_id = query_blockfrost(f"/epochs/{epoch_number}/stakes", {})[0]["pool_id"]
    label = query_blockfrost("/metadata/txs/labels", {})[0]["label"]
    script_hash = query_blockfrost("/scripts", {})[0]["script_hash"]

    endpoints = [
        # Health
        ("/health", {}),
        ("/health/clock", {}),
        # Nutlink
        ("/nutlink/{address}", {"address": NUTLINK_ADDRESS}),
        ("/nutlink/{address}/tickers", {"address": NUTLINK_ADDRESS}),
        (
            "/nutlink/{address}/tickets/{ticker}",
            {"address": NUTLINK_ADDRESS, "ticker": TICKER},
        ),
        ("/nutlink/tickers/{ticker}", {"ticker": TICKER}),
        # Accounts
        ("/accounts/{stake_address}", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/rewards", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/history", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/delegations", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/registrations", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/withdrawals", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/mirs", {"stake_address": stake_address}),
        ("/accounts/{stake_address}/addresses", {"stake_address": stake_address}),
        (
            "/accounts/{stake_address}/addresses/assets",
            {"stake_address": stake_address},
        ),
        ("/accounts/{stake_address}/addresses/total", {"stake_address": stake_address}),
        # Addresses
        ("/addresses/{address}", {"address": address}),
        ("/addresses/{address}/extended", {"address": address}),
        ("/addresses/{address}/total", {"address": address}),
        ("/addresses/{address}/utxos", {"address": address}),
        ("/addresses/{address}/utxos/{asset}", {"address": address, "asset": asset}),
        ("/addresses/{address}/txs", {"address": address}),
        # Assets
        ("/assets", {}),
        ("/assets/{asset}", {"asset": asset}),
        ("/assets/{asset}/history", {"asset": asset}),
        ("/assets/{asset}/txs", {"asset": asset}),
        ("/assets/{asset}/addresses", {"asset": asset}),
        ("/assets/policy/{policy_id}", {"policy_id": policy_id}),
        # Blocks
        ("/blocks/latest", {}),
        ("/blocks/latest/txs", {}),
        ("/blocks/{hash_or_number}", {"hash_or_number": block_hash}),
        ("/blocks/slot/{block_slot}", {"block_slot": block_slot}),
        (
            "/blocks/epoch/{epoch_slot}/slot/{block_slot}",
            {"epoch_slot": epoch_slot, "block_slot": block_slot},
        ),
        ("/blocks/{hash_or_number}/next", {"hash_or_number": block_hash}),
        ("/blocks/{hash_or_number}/previous", {"hash_or_number": block_hash}),
        ("/blocks/{hash_or_number}/txs", {"hash_or_number": block_hash}),
        ("/blocks/{hash_or_number}/addresses", {"hash_or_number": block_hash}),
        # Epochs
        ("/epochs/latest", {}),
        ("/epochs/latest/parameters", {}),
        ("/epochs/{number}", {"number": epoch_number}),
        ("/epochs/{number}/next", {"number": epoch_number}),
        ("/epochs/{number}/previous", {"number": epoch_number}),
        ("/epochs/{number}/stakes", {"number": epoch_number}),
        ("/epochs/{number}/blocks", {"number": epoch_number}),
        (
            "/epochs/{number}/stakes/{pool_id}",
            {"number": epoch_number, "pool_id": pool_id},
        ),
        (
            "/epochs/{number}/blocks/{pool_id}",
            {"number": epoch_number, "pool_id": pool_id},
        ),
        ("/epochs/{number}/parameters", {"number": epoch_number}),
        # Ledger
        ("/genesis", {}),
        # Metadata
        ("/metadata/txs/labels", {}),
        ("/metadata/txs/labels/{label}", {"label": label}),
        ("/metadata/txs/labels/{label}/cbor", {"label": label}),
        # Network
        # ("/network", {}),
        # Pools
        ("/pools", {}),
        ("/pools/extended", {}),
        ("/pools/retired", {}),
        ("/pools/retiring", {}),
        ("/pools/{pool_id}", {"pool_id": pool_id}),
        ("/pools/{pool_id}/history", {"pool_id": pool_id}),
        ("/pools/{pool_id}/metadata", {"pool_id": pool_id}),
        ("/pools/{pool_id}/relays", {"pool_id": pool_id}),
        ("/pools/{pool_id}/delegators", {"pool_id": pool_id}),
        ("/pools/{pool_id}/blocks", {"pool_id": pool_id}),
        ("/pools/{pool_id}/updates", {"pool_id": pool_id}),
        # Transactions
        ("/txs/{hash}", {"hash": tx_hash}),
        ("/txs/{hash}/utxos", {"hash": tx_hash}),
        ("/txs/{hash}/stakes", {"hash": tx_hash}),
        ("/txs/{hash}/delegations", {"hash": tx_hash}),
        ("/txs/{hash}/withdrawals", {"hash": tx_hash}),
        ("/txs/{hash}/mirs", {"hash": tx_hash}),
        ("/txs/{hash}/pool_updates", {"hash": tx_hash}),
        ("/txs/{hash}/pool_retires", {"hash": tx_hash}),
        ("/txs/{hash}/metadata", {"hash": tx_hash}),
        ("/txs/{hash}/metadata/cbor", {"hash": tx_hash}),
        ("/txs/{hash}/redeemers", {"hash": tx_hash}),
        # Scripts
        ("/scripts", {}),
        ("/scripts/{script_hash}", {"script_hash": script_hash}),
        ("/scripts/{script_hash}/json", {"script_hash": script_hash}),
        ("/scripts/{script_hash}/cbor", {"script_hash": script_hash}),
        ("/scripts/{script_hash}/redeemers", {"script_hash": script_hash}),
    ]

    rows = []
    for endpoint, params in endpoints:
        print(f"Getting information for {endpoint} endpoint.")

        endpoint_rows = []
        try:
            for _ in range(args.repetitions):
                row = {}
                row["endpoint"] = endpoint
                row["blockfrost"] = time_function(
                    lambda: query_blockfrost(endpoint, params)
                )
                row["demeter"] = time_function(lambda: query_demeter(endpoint, params))
                endpoint_rows.append(row)
        except requests.RequestException as e:
            print(f"Error with {endpoint} endpoint: {e}")
        else:
            rows.extend(endpoint_rows)

        # Partial results are valuable
        pd.DataFrame(rows).to_csv("data.csv")
    return pd.DataFrame(rows)


def transform(df):
    # Get offset
    health_endpoint = df[df["endpoint"] == "/health"]
    offset = health_endpoint["demeter"].mean() - health_endpoint["blockfrost"].mean()
    print("Health endpoint offset: {:.2f}s".format(offset))

    df["demeter"] = df["demeter"] - offset
    df = df.groupby("endpoint").agg(
        {
            "demeter": ["mean", "median", "max"],
            "blockfrost": ["mean", "median", "max"],
        }
    )
    df["difference"] = df["demeter"]["mean"] - df["blockfrost"]["mean"]
    return df.sort_values("difference", ascending=False)


def show(df):
    table = Table(title="Benchmark results")

    table.add_column("Endpoint", justify="right", style="cyan", no_wrap=True)
    table.add_column("Demeter", justify="center")
    table.add_column("Blockfrost", justify="center")
    table.add_column("Difference (mean)", justify="center")
    table.add_column("Difference (median)", justify="center")
    table.add_column("Difference (max)", justify="center")

    for index, row in df.iterrows():
        table.add_row(
            index,
            "mean: {:.2f}s, median: {:.2f}s, max: {:.2f}s".format(
                row["demeter"]["mean"],
                row["demeter"]["median"],
                row["demeter"]["max"],
            ),
            "mean: {:.2f}s, median: {:.2f}s, max: {:.2f}s".format(
                row["blockfrost"]["mean"],
                row["blockfrost"]["median"],
                row["blockfrost"]["max"],
            ),
            Text(
                "{:.2f}".format(row["demeter"]["mean"] - row["blockfrost"]["mean"]),
                style=(
                    "red"
                    if (row["demeter"]["mean"] - row["blockfrost"]["mean"]) > 1
                    else "green"
                ),
            ),
            Text(
                "{:.2f}".format(row["demeter"]["median"] - row["blockfrost"]["median"]),
                style=(
                    "red"
                    if (row["demeter"]["median"] - row["blockfrost"]["median"]) > 1
                    else "green"
                ),
            ),
            Text(
                "{:.2f}".format(row["demeter"]["max"] - row["blockfrost"]["max"]),
                style=(
                    "red"
                    if (row["demeter"]["max"] - row["blockfrost"]["max"]) > 1
                    else "green"
                ),
            ),
        )

    console = Console()
    console.print(table)


def main():
    parser = get_parser()
    args = parser.parse_args()

    if not args.no_build:
        df = build(args)
        df.to_csv("data.csv")

    df = pd.read_csv("data.csv", index_col=0)
    df = transform(df)
    show(df)


if __name__ == "__main__":
    main()
