#!/bin/bash

# Read environment variables from metadata.json
export GL_SERVER_DATA_PATH="$HOME/greenlight/.gltestserver"
export GL_CA_CRT=$(jq -r '.ca_crt_path' ./metadata.json)
export GL_NOBODY_CRT=$(jq -r '.nobody_crt_path' ./metadata.json)
export GL_NOBODY_KEY=$(jq -r '.nobody_key_path' ./metadata.json)
export GL_SCHEDULER_GRPC_URI=$(jq -r '.scheduler_grpc_uri' ./metadata.json)
export GL_BITCOIN_RPC_URI=$(jq -r '.bitcoind_rpc_uri' ./metadata.json)
export GL_GRPC_WEB_PROXY_URI=$(jq -r '.grpc_web_proxy_uri' ./metadata.json)
export GL_GRPC_PORT=$(echo "$GL_GRPC_WEB_PROXY_URI" | sed -E 's/.*:([0-9]+)$/\1/')
export LSP_LIGHTNING_DIR=/tmp/.lightning

# Extract values using parameter expansion and regular expressions
RPC_USER=$(echo $GL_BITCOIN_RPC_URI | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\1|')
RPC_PASS=$(echo $GL_BITCOIN_RPC_URI | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\2|')
BITCOIN_HOST=$(echo $GL_BITCOIN_RPC_URI | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\3|')
BITCOIN_PORT=$(echo $GL_BITCOIN_RPC_URI | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\4|')
WALLET_NAME="testwallet"
NODE_PUBKEY_1="0279da9a93e50b008a7ba6bd25355fb7132f5015b790a05ee9f41bc9fbdeb30d19"
NODE_PUBKEY_2="036fc50c7183d47baf1b80f312f1ea53b25c117183456b788c05f91b3b4507c8d3"
LSP_NODE_PUBKEY="0208d70da220f05b859512fd805365af6a6b8583fa3ff10fbd00b6b7fbb29acd32"

# Function to handle Ctrl+C
trap 'echo "Stopping terminals"; pkill -f "gnome-terminal"; exit' SIGINT

# Function to register and schedule a node
run_scheduler_for_node() {
  local node=$1
  mkdir -p $GL_SERVER_DATA_PATH/$node && printf $2 > $GL_SERVER_DATA_PATH/$node/hsm_secret
  gnome-terminal --title="Scheduler $node" -- bash -c "cargo run --bin glcli scheduler register --network=regtest --data-dir=$GL_SERVER_DATA_PATH/$node; 
  cargo run --bin glcli scheduler schedule --verbose --network=regtest --data-dir=$GL_SERVER_DATA_PATH/$node; 
  echo 'Closing terminal...'; exit"
}

# Function to start the signer for a node
run_signer_for_node() {
  local node=$1
  gnome-terminal --title="Signer $node" -- bash -c "cargo run --bin glcli signer run --verbose --network=regtest --data-dir=$GL_SERVER_DATA_PATH/$node; 
  echo 'Closing terminal...'; exit"
}

# Function to start the local LSP node
run_local_lsp_node() {
  mkdir -p $LSP_LIGHTNING_DIR/regtest && printf '\x9f\xaa\x67\x08\x67\x05\x8c\x9d\xcc\x1f\xea\xd9\x9c\xe4\x91\xe2\x85\x95\x6d\xdb\x66\xa8\xed\x05\x85\xf3\x2a\x77\x0e\x1d\x14\xa6' > $LSP_LIGHTNING_DIR/regtest/hsm_secret
  gnome-terminal --title="Local LSP node" -- bash -c "
  lightningd --network=regtest --log-level=debug --lightning-dir=$LSP_LIGHTNING_DIR --bitcoin-rpcconnect=$BITCOIN_HOST --bitcoin-rpcport=$BITCOIN_PORT --bitcoin-datadir=$GL_SERVER_DATA_PATH/gl-testserver --bitcoin-rpcuser=$RPC_USER --bitcoin-rpcpassword=$RPC_PASS --alias=CLNLocalLSPNode --bind-addr=0.0.0.0:9735 --bind-addr=ws:0.0.0.0:5019 --announce-addr=0.0.0.0:9735 --addr=localhost:7171 --grpc-port=9737 --clnrest-port=3010 --clnrest-protocol=http;
  echo 'Closing terminal...'; exit"
}

wait_for_local_funds_confirmation() {
  IS_CONFIRMED=false
  while ! $IS_CONFIRMED; do
    if lightning-cli --network=regtest --lightning-dir="$LSP_LIGHTNING_DIR" listfunds | jq -e '.outputs[] | select(.status == "confirmed" and .reserved == false)' > /dev/null; then
      IS_CONFIRMED=true
    else
      echo "Waiting for funds to confirm on LSP..."
      sleep 1
    fi
  done
}

wait_for_gl_funds_confirmation() {
  cd "./examples/javascript"
  while ! node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "ListFunds" "Listfunds" "Listfunds" "{}" | grep -q "outputs"; do
    echo "Waiting for funds to confirm on gl node 1..."
    sleep 1
  done
  while ! node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "ListFunds" "Listfunds" "Listfunds" "{}" | grep -q "outputs"; do
    echo "Waiting for funds to confirm on gl node 2..."
    sleep 1
  done
  cd "../.."
}

run_gl_node_operations() {
  cd "./examples/javascript"
  CONNECT_PAYLOAD='{"id": "'"$LSP_NODE_PUBKEY"'", "host": "127.0.0.1", "port": 9735}'
  echo "Node-1 + LSP"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "ConnectPeer" "Connect" "Connect" "$CONNECT_PAYLOAD"
  echo "Node-2 + LSP"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "ConnectPeer" "Connect" "Connect" "$CONNECT_PAYLOAD"
  BUFFER_ID=$(echo -n "$LSP_NODE_PUBKEY" | xxd -r -p | base64)
  FUND_PAYLOAD='{"id": "'"$BUFFER_ID"'", "amount": {"amount": {"msat": 11000000000}}}'
  echo "Node-1 -> LSP"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "FundChannel" "Fundchannel" "Fundchannel" "$FUND_PAYLOAD"
  echo "Node-2 -> LSP"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "FundChannel" "Fundchannel" "Fundchannel" "$FUND_PAYLOAD"
  cd "../.."
}

run_node_transactions() {
  cd "./examples/javascript"
  echo "Create invoice 1"
  INVOICE_1_PAYLOAD=$(jq -nc --arg msat "120000000" --arg desc "Learning Bitcoin Book" --arg label "bookinvat$(date +%s)" \
    '{"amount_msat": {"amount": {"msat": ($msat | tonumber)}}, "description": $desc, "label": $label}')
  INVOICE_1_RESPONSE=$(node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "Invoice" "Invoice" "Invoice" "$INVOICE_1_PAYLOAD")
  BOLT11_1=$(echo "$INVOICE_1_RESPONSE" | jq -r '.bolt11')
  echo "Invoice 1: $BOLT11_1"
  echo "Create invoice 2"
  INVOICE_2_PAYLOAD=$(jq -nc --arg msat "5000000" --arg desc "My coffee" --arg label "coffeeinvat$(date +%s)" \
    '{"amount_msat": {"amount": {"msat": ($msat | tonumber)}}, "description": $desc, "label": $label}')
  INVOICE_2_RESPONSE=$(node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "Invoice" "Invoice" "Invoice" "$INVOICE_2_PAYLOAD")
  BOLT11_2=$(echo "$INVOICE_2_RESPONSE" | jq -r '.bolt11')
  echo "Invoice 2: $BOLT11_2"
  echo "Pay invoice 1"
  PAY_PAYLOAD_1='{"bolt11": "'"$BOLT11_1"'"}'
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "Pay" "Pay" "Pay" "$PAY_PAYLOAD_1"
  echo "Pay invoice 2"
  PAY_PAYLOAD_2='{"bolt11": "'"$BOLT11_2"'"}'
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "Pay" "Pay" "Pay" "$PAY_PAYLOAD_2"
  cd "../.."
}

rm -rf $GL_SERVER_DATA_PATH/gl-testserver/regtest/$WALLET_NAME && rm -rf $LSP_LIGHTNING_DIR

# Setup node-1
echo "Scheduler for node-1"
run_scheduler_for_node "node-1" '\x0f\x29\xa7\x60\xea\xa1\xa3\xba\xbb\x7b\xa2\x5b\x7a\x82\xd1\x0b\x38\x55\xb9\xd2\xc6\x39\xb2\x79\xa9\xa0\x8d\x9b\xd1\xe6\x67\x9d'
echo "Signer for node-1"
run_signer_for_node "node-1"

# Setup node-2
echo "Scheduler for node-2"
run_scheduler_for_node "node-2" '\x5d\x74\x25\x7e\x03\xb0\xee\x2f\xf4\x7a\x06\x97\x3a\xd9\x14\xf1\x7c\x61\xec\xac\x20\xb1\xf9\x3b\xb2\x33\x98\xc9\x40\x86\xad\x67'
echo "Signer for node-2"
run_signer_for_node "node-2"

echo "Running local LSP node"
run_local_lsp_node

# Send Bitcoin to Node-1, Node-2 and LSP
echo "Creating bitcoin wallet"
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS createwallet $WALLET_NAME > /dev/null
echo "Generating 101 blocks"
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME -generate 101 > /dev/null
echo "Sending bitcoin to LSP node"
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME sendtoaddress "bcrt1qaew9v5m5q8cjjuh9mdruuujykzxrcufma28x82" 1 > /dev/null
echo "Sending bitcoin to node-1"
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME sendtoaddress "bcrt1q0mlp2u676wv9rgz5e6nrrq2vc76rllxcazcldy" 1 > /dev/null
echo "Sending bitcoin to node-2"
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME sendtoaddress "bcrt1q68n5mqlkf0l877chhpa2w2zxlug343kn0p8c6r" 1 > /dev/null
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME -generate 1 > /dev/null
wait_for_gl_funds_confirmation
wait_for_local_funds_confirmation

# Connect and fund channels Node-1 -> LSP & Node-2 -> LSP
run_gl_node_operations
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME -generate 6 > /dev/null
wait_for_gl_funds_confirmation

echo "LSP -> Node-1"
lightning-cli --network=regtest --lightning-dir=$LSP_LIGHTNING_DIR fundchannel $NODE_PUBKEY_1 12000000
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME -generate 6 > /dev/null
wait_for_local_funds_confirmation

echo "LSP -> Node-2"
lightning-cli --network=regtest --lightning-dir=$LSP_LIGHTNING_DIR fundchannel $NODE_PUBKEY_2 12000000
bitcoin-cli -rpcconnect=$BITCOIN_HOST -rpcport=$BITCOIN_PORT -rpcuser=$RPC_USER -rpcpassword=$RPC_PASS -rpcwallet=$WALLET_NAME -generate 6 > /dev/null
wait_for_local_funds_confirmation

# Print LSP node status with channels and outputs before testing transactions
lightning-cli --network=regtest --lightning-dir=$LSP_LIGHTNING_DIR listfunds

# Create invoice and pay
run_node_transactions

# Keep the script running to listen for Ctrl+C
wait
