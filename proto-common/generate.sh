#/usr/bin/env sh

# set -eo pipefail
PROTO_FOLDER=$(dirname "$0")
echo $PROTO_FOLDER

echo "Generating Rust proto code"
buf generate --template buf.gen.rust.yaml