#!/bin/sh
# Fails when the workspace Cargo.lock resolves a trust-store crate below the
# floor that drops the compromised eMudhra roots. Older lines that predate the
# floor only warn. Reads the lockfile only; no network.
set -eu

lock="${1:-$(dirname "$0")/../Cargo.lock}"

if [ ! -f "$lock" ]; then
    echo "check-trust-floor: $lock not found" >&2
    exit 2
fi

awk '
function below(v, floor,    a, b, n, i) {
    sub(/[-+].*/, "", v)
    n = split(v, a, ".")
    split(floor, b, ".")
    for (i = 1; i <= 3; i++) {
        if ((a[i] + 0) < (b[i] + 0)) return 1
        if ((a[i] + 0) > (b[i] + 0)) return 0
    }
    return 0
}
function check(name, version) {
    if (name == "webpki-roots" || name == "webpki-root-certs") {
        if (version ~ /^0\./) {
            printf "warning: %s %s predates the 1.x trust floor\n", name, version
        } else if (version ~ /^1\./ && below(version, "1.0.10")) {
            printf "error: %s %s is below 1.0.10\n", name, version
            failed = 1
        }
    } else if (name == "rustls-webpki") {
        if (version ~ /^0\.(101|102)\./) {
            printf "warning: %s %s predates the 0.103 trust floor\n", name, version
        } else if (version ~ /^0\.103\./ && below(version, "0.103.16")) {
            printf "error: %s %s is below 0.103.16\n", name, version
            failed = 1
        }
    }
}
/^\[\[package\]\]/ { name = ""; version = ""; next }
/^name = / { name = $3; gsub(/"/, "", name); next }
/^version = / {
    version = $3; gsub(/"/, "", version)
    if (name != "") check(name, version)
    next
}
END { exit failed }
' "$lock" || {
    echo "check-trust-floor: trust-store crates below the floor in $lock" >&2
    exit 1
}

echo "check-trust-floor: ok"
