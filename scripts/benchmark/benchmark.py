import argparse
import time

import pandas as pd
from blockfrost import BlockFrostApi, ApiError, ApiUrls
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
    parser.add_argument("--demeter-authorized-url", default=None)

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

    blockfrost_api = BlockFrostApi(
        project_id=args.blockfrost_project_id,
        base_url=getattr(ApiUrls, args.network).value,
    )

    if args.demeter_authorized_url is None:
        raise ValueError("Demeter Authorized URL is undefined.")

    demeter_api = BlockFrostApi(base_url=args.demeter_authorized_url)
    demeter_api.api_version = ""  # Ugly workaround, but works

    # Get valid parameters for endpoints
    block = None
    block_previous_hash = None
    while block is None:
        if block_previous_hash is None:
            block = blockfrost_api.block_latest()
        else:
            block = blockfrost_api.block(block_previous_hash)

        block_previous_hash = block.previous_block
        if block.tx_count == 0:
            block = None

    block_hash = block.hash
    epoch_number = block.epoch
    slot_number = block.epoch_slot

    address_object = blockfrost_api.blocks_addresses(block_hash)[0]
    address = address_object.address
    tx_hash = address_object.transactions[0].tx_hash
    stake_address = blockfrost_api.address(address).stake_address

    asset = blockfrost_api.assets()[0].asset
    policy_id = blockfrost_api.asset(asset).policy_id

    pool_id = blockfrost_api.epoch_stakes(epoch_number)[0].pool_id
    label = blockfrost_api.metadata_labels()[0].label
    script_hash = blockfrost_api.scripts()[0].script_hash

    endpoints = [
        # Health
        ("health", {}),
        ("clock", {}),
        # Nutlink
        ("nutlink_address", {"address": NUTLINK_ADDRESS}),
        ("nutlink_address_tickers", {"address": NUTLINK_ADDRESS}),
        ("nutlink_address_ticker", {"address": NUTLINK_ADDRESS, "ticker": TICKER}),
        ("nutlink_ticker", {"ticker": TICKER}),
        # Accounts
        ("accounts", {"stake_address": stake_address}),
        ("account_rewards", {"stake_address": stake_address}),
        ("account_history", {"stake_address": stake_address}),
        ("account_delegations", {"stake_address": stake_address}),
        ("account_registrations", {"stake_address": stake_address}),
        ("account_withdrawals", {"stake_address": stake_address}),
        ("account_mirs", {"stake_address": stake_address}),
        ("account_addresses", {"stake_address": stake_address}),
        ("account_addresses_assets", {"stake_address": stake_address}),
        ("account_addresses_total", {"stake_address": stake_address}),
        # Addresses
        ("address", {"address": address}),
        ("address_extended", {"address": address}),
        ("address_total", {"address": address}),
        ("address_utxos", {"address": address}),
        ("address_utxos_asset", {"address": address, "asset": asset}),
        ("address_transactions", {"address": address}),
        # Assets
        ("assets", {}),
        ("asset", {"asset": asset}),
        ("asset_history", {"asset": asset}),
        ("asset_transactions", {"asset": asset}),
        ("asset_addresses", {"asset": asset}),
        ("assets_policy", {"policy_id": policy_id}),
        # Blocks
        ("block_latest", {}),
        ("block_latest_transactions", {}),
        ("block", {"hash_or_number": block_hash}),
        ("block_slot", {"slot_number": slot_number}),
        (
            "block_epoch_slot",
            {"slot_number": slot_number, "epoch_number": epoch_number},
        ),
        ("blocks_next", {"hash_or_number": block_hash}),
        ("blocks_previous", {"hash_or_number": block_hash}),
        ("block_transactions", {"hash_or_number": block_hash}),
        ("blocks_addresses", {"hash_or_number": block_hash}),
        # Epochs
        ("epoch_latest", {}),
        ("epoch_latest_parameters", {}),
        ("epoch", {"number": epoch_number}),
        ("epochs_next", {"number": epoch_number}),
        ("epochs_previous", {"number": epoch_number}),
        ("epoch_stakes", {"number": epoch_number}),
        ("epoch_pool_stakes", {"number": epoch_number, "pool_id": pool_id}),
        ("epoch_blocks", {"number": epoch_number}),
        ("epoch_pool_blocks", {"number": epoch_number, "pool_id": pool_id}),
        ("epoch_protocol_parameters", {"number": epoch_number}),
        # Ledger
        ("genesis", {}),
        # Metadata
        ("metadata_labels", {}),
        ("metadata_label_json", {"label": label}),
        ("metadata_label_cbor", {"label": label}),
        # Network
        ("network", {}),
        # Pools
        ("pools", {}),
        ("pools_extended", {}),
        ("pools_retired", {}),
        ("pools_retiring", {}),
        ("pool", {"pool_id": pool_id}),
        ("pool_history", {"pool_id": pool_id}),
        ("pool_metadata", {"pool_id": pool_id}),
        ("pool_relays", {"pool_id": pool_id}),
        ("pool_delegators", {"pool_id": pool_id}),
        ("pool_blocks", {"pool_id": pool_id}),
        ("pool_updates", {"pool_id": pool_id}),
        # Transactions
        ("transaction", {"hash": tx_hash}),
        ("transaction_utxos", {"hash": tx_hash}),
        ("transaction_stakes", {"hash": tx_hash}),
        ("transaction_delegations", {"hash": tx_hash}),
        ("transaction_withdrawals", {"hash": tx_hash}),
        ("transaction_mirs", {"hash": tx_hash}),
        ("transaction_pool_updates", {"hash": tx_hash}),
        ("transaction_pool_retires", {"hash": tx_hash}),
        ("transaction_metadata", {"hash": tx_hash}),
        ("transaction_metadata_cbor", {"hash": tx_hash}),
        ("transaction_redeemers", {"hash": tx_hash}),
        # Scripts
        ("scripts", {}),
        ("script", {"script_hash": script_hash}),
        ("script_json", {"script_hash": script_hash}),
        ("script_cbor", {"script_hash": script_hash}),
        ("script_redeemers", {"script_hash": script_hash}),
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
                    lambda: getattr(blockfrost_api, endpoint)(**params)
                )
                row["demeter"] = time_function(
                    lambda: getattr(demeter_api, endpoint)(**params)
                )
                endpoint_rows.append(row)
        except ApiError as e:
            print(f"Error with {endpoint} endpoint: {e}")
        else:
            rows.extend(endpoint_rows)

        # Partial results are valuable
        pd.DataFrame(rows).to_csv("data.csv")
    return pd.DataFrame(rows)


def transform(df):
    # Get offset
    health_endpoint = df[df["endpoint"] == "health"]
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
