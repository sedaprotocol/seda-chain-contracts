WASMVM_VERSION="v1.5.2"

ARCH=$(uname -m)
if [ $ARCH != "aarch64" ]; then
	ARCH="x86_64"
fi

LIBWASMVM_URL=https://github.com/CosmWasm/wasmvm/releases/download/$WASMVM_VERSION/libwasmvm.$ARCH.so
COSMOS_LDS=$HOME/COSMOS_LDS

printf "\n\n\nSETTING UP SHARED LIBRARY\n\n\n\n"
mkdir -p $COSMOS_LDS
curl -LO $LIBWASMVM_URL
mv $(basename $LIBWASMVM_URL) /usr/local/lib