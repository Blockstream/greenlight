#!/bin/bash
# shellcheck disable=SC2317

# Set AUTO_START to the first argument, default to "yes" if not provided
AUTO_START=${1:-"start"}

# Read environment variables from metadata.json
GL_SERVER_DATA_PATH="$HOME/workspace/greenlight/.gltestserver"
GL_CA_CRT=$(jq -r '.ca_crt_path' ./metadata.json)
GL_NOBODY_CRT=$(jq -r '.nobody_crt_path' ./metadata.json)
GL_NOBODY_KEY=$(jq -r '.nobody_key_path' ./metadata.json)
GL_SCHEDULER_GRPC_URI=$(jq -r '.scheduler_grpc_uri' ./metadata.json)
GL_BITCOIN_RPC_URI=$(jq -r '.bitcoind_rpc_uri' ./metadata.json)
GL_GRPC_WEB_PROXY_URI=$(jq -r '.grpc_web_proxy_uri' ./metadata.json)
GL_GRPC_PORT=$(echo "$GL_GRPC_WEB_PROXY_URI" | sed -E 's/.*:([0-9]+)$/\1/')
LOCAL_LIGHTNING_DIR="/tmp/.lightning"

# Extract values using parameter expansion and regular expressions
RPC_USER=$(echo "$GL_BITCOIN_RPC_URI" | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\1|')
RPC_PASS=$(echo "$GL_BITCOIN_RPC_URI" | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\2|')
BITCOIN_HOST=$(echo "$GL_BITCOIN_RPC_URI" | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\3|')
BITCOIN_PORT=$(echo "$GL_BITCOIN_RPC_URI" | sed -E 's|^http://([^:]+):([^@]+)@([^:]+):([0-9]+)$|\4|')
WALLET_NAME="testwallet"
NODE_PUBKEY_1="0279da9a93e50b008a7ba6bd25355fb7132f5015b790a05ee9f41bc9fbdeb30d19"
NODE_PUBKEY_2="036fc50c7183d47baf1b80f312f1ea53b25c117183456b788c05f91b3b4507c8d3"
LOCAL_NODE_PUBKEY="0208d70da220f05b859512fd805365af6a6b8583fa3ff10fbd00b6b7fbb29acd32"

# Function to handle Ctrl+C
trap 'echo "Stopping terminals"; pkill -f "gnome-terminal"; exit' SIGINT

# Function to print environment variables
print_envs() {
  echo "GL_SERVER_DATA_PATH=$GL_SERVER_DATA_PATH"
  echo "GL_CA_CRT=$GL_CA_CRT"
  echo "GL_NOBODY_CRT=$GL_NOBODY_CRT"
  echo "GL_NOBODY_KEY=$GL_NOBODY_KEY"
  echo "GL_SCHEDULER_GRPC_URI=$GL_SCHEDULER_GRPC_URI"
  echo "GL_BITCOIN_RPC_URI=$GL_BITCOIN_RPC_URI"
  echo "GL_GRPC_WEB_PROXY_URI=$GL_GRPC_WEB_PROXY_URI"
  echo "GL_GRPC_PORT=$GL_GRPC_PORT"
  echo "LOCAL_LIGHTNING_DIR=$LOCAL_LIGHTNING_DIR"
  echo "RPC_USER=$RPC_USER"
  echo "RPC_PASS=$RPC_PASS"
  echo "BITCOIN_HOST=$BITCOIN_HOST"
  echo "BITCOIN_PORT=$BITCOIN_PORT"
  echo "WALLET_NAME=testwallet"
  echo "NODE_PUBKEY_1=0279da9a93e50b008a7ba6bd25355fb7132f5015b790a05ee9f41bc9fbdeb30d19"
  echo "NODE_PUBKEY_2=036fc50c7183d47baf1b80f312f1ea53b25c117183456b788c05f91b3b4507c8d3"
  echo "LOCAL_NODE_PUBKEY=0208d70da220f05b859512fd805365af6a6b8583fa3ff10fbd00b6b7fbb29acd32"
}

# Function to register and schedule a node
run_scheduler_for_gl_node() {
  local node="$1"
  local hsm_secret="$2"
  if [ -z "$node" ] || [ -z "$hsm_secret" ]; then
    echo "Forgot params node/hsm_secret? Exiting..."
  else
    echo "Scheduler for $node"
    rm -rf "${GL_SERVER_DATA_PATH}/${node:?}" && mkdir -p "${GL_SERVER_DATA_PATH}/${node}"
    printf '%b' "$hsm_secret" > "${GL_SERVER_DATA_PATH}/${node}/hsm_secret"
    gnome-terminal --title="Scheduler $node" -- bash -c \
      "GL_CA_CRT=$GL_CA_CRT \
      GL_NOBODY_CRT=$GL_NOBODY_CRT \
      GL_NOBODY_KEY=$GL_NOBODY_KEY \
      GL_SCHEDULER_GRPC_URI=$GL_SCHEDULER_GRPC_URI \
      cargo run --bin glcli scheduler register \
        --network=regtest \
        --data-dir=${GL_SERVER_DATA_PATH}/${node} && sleep 1 && \
      GL_CA_CRT=$GL_CA_CRT \
      GL_NOBODY_CRT=$GL_NOBODY_CRT \
      GL_NOBODY_KEY=$GL_NOBODY_KEY \
      GL_SCHEDULER_GRPC_URI=$GL_SCHEDULER_GRPC_URI \
      cargo run --bin glcli scheduler schedule \
        --verbose \
        --network=regtest \
        --data-dir=${GL_SERVER_DATA_PATH}/${node}; \
      echo 'Press Enter to close this terminal...'; \
      read"
  fi
}

# Function to start the signer for a node
run_signer_for_gl_node() {
  local node="$1"
  if [ -z "$node" ]; then
    echo "Forgot param node? Exiting..."
  else
    echo "Signer for $node"
    gnome-terminal --title="Signer $node" -- bash -c \
    "GL_CA_CRT=$GL_CA_CRT \
    GL_NOBODY_CRT=$GL_NOBODY_CRT \
    GL_NOBODY_KEY=$GL_NOBODY_KEY \
    GL_SCHEDULER_GRPC_URI=$GL_SCHEDULER_GRPC_URI \
    cargo run --bin glcli signer run --verbose --network=regtest --data-dir=${GL_SERVER_DATA_PATH}/${node}; \
    echo 'Press Enter to close this terminal...'; \
    read"
  fi
}

# Function to start the local CLN node
run_local_cln_node() {
  local node="$1"
  local hsm_secret="$2"
  if [ -z "$node" ] || [ -z "$hsm_secret" ]; then
    echo "Forgot params node/hsm_secret? Exiting..."
  else
    echo "Running $node node"
    rm -rf $LOCAL_LIGHTNING_DIR && mkdir -p "$LOCAL_LIGHTNING_DIR/regtest"
    printf '%b' "$hsm_secret" > "$LOCAL_LIGHTNING_DIR/regtest/hsm_secret"
    gnome-terminal --title="Local node" -- bash -c \
      "lightningd --network=regtest --log-level=debug --lightning-dir=$LOCAL_LIGHTNING_DIR --bitcoin-rpcconnect=$BITCOIN_HOST --bitcoin-rpcport=$BITCOIN_PORT --bitcoin-datadir=$GL_SERVER_DATA_PATH/gl-testserver --bitcoin-rpcuser=$RPC_USER --bitcoin-rpcpassword=$RPC_PASS --alias=CLNLocalNode --bind-addr=0.0.0.0:9735 --bind-addr=ws:0.0.0.0:5019 --announce-addr=0.0.0.0:9735 --addr=localhost:7171 --grpc-port=9737 --clnrest-port=3010 --clnrest-protocol=http; \
      echo 'Press Enter to close this terminal...'; \
      read"
  fi
}

connect_and_fund_channels_from_gl_nodes() {
  cd "./examples/javascript" || exit
  CONNECT_PAYLOAD='{"id": "'"$LOCAL_NODE_PUBKEY"'", "host": "127.0.0.1", "port": 9735}'
  echo "gl1 + LOCAL"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "ConnectPeer" "Connect" "Connect" "$CONNECT_PAYLOAD"
  echo "gl2 + LOCAL"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "ConnectPeer" "Connect" "Connect" "$CONNECT_PAYLOAD"
  BUFFER_ID=$(echo -n "$LOCAL_NODE_PUBKEY" | xxd -r -p | base64)
  FUND_PAYLOAD='{"id": "'"$BUFFER_ID"'", "amount": {"amount": {"msat": 11000000000}}}'
  echo "gl1 -> LOCAL"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "FundChannel" "Fundchannel" "Fundchannel" "$FUND_PAYLOAD"
  echo "gl2 -> LOCAL"
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "FundChannel" "Fundchannel" "Fundchannel" "$FUND_PAYLOAD"
  cd "../.." || exit
  mine_blocks 6
}

fund_channels_from_local_node() {
  echo "LOCAL -> gl1"
  lightning-cli --network=regtest --lightning-dir="$LOCAL_LIGHTNING_DIR" fundchannel "$NODE_PUBKEY_1" 12000000
  mine_blocks 6

  echo "LOCAL -> gl2"
  lightning-cli --network=regtest --lightning-dir="$LOCAL_LIGHTNING_DIR" fundchannel "$NODE_PUBKEY_2" 12000000
  mine_blocks 6
}

create_invoice_and_pay() {
  cd "./examples/javascript" || exit
  echo "Create invoice 1"
  INVOICE_1_PAYLOAD=$(jq -nc --arg msat "120000000" --arg desc "Learning Bitcoin" --arg label "bookinvat$(date +%s)" \
    '{"amount_msat": {"amount": {"msat": ($msat | tonumber)}}, "description": $desc, "label": $label}')
  INVOICE_1_RESPONSE=$(node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "Invoice" "Invoice" "Invoice" "$INVOICE_1_PAYLOAD")
  BOLT11_1=$(echo "$INVOICE_1_RESPONSE" | jq -r '.bolt11')
  echo "Invoice 1: $BOLT11_1"
  echo "Create invoice 2"
  INVOICE_2_PAYLOAD=$(jq -nc --arg msat "5000000" --arg desc "Coffee" --arg label "coffeeinvat$(date +%s)" \
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
  echo "Decoded invoice 2"
  DECODE_PAYLOAD='{"string": "'"$BOLT11_2"'"}'
  node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "Decode" "Decode" "Decode" "$DECODE_PAYLOAD"
  cd "../.." || exit
}

start_nodes() {
  run_scheduler_for_gl_node "gl1" '\x0f\x29\xa7\x60\xea\xa1\xa3\xba\xbb\x7b\xa2\x5b\x7a\x82\xd1\x0b\x38\x55\xb9\xd2\xc6\x39\xb2\x79\xa9\xa0\x8d\x9b\xd1\xe6\x67\x9d'
  run_signer_for_gl_node "gl1"

  run_scheduler_for_gl_node "gl2" '\x5d\x74\x25\x7e\x03\xb0\xee\x2f\xf4\x7a\x06\x97\x3a\xd9\x14\xf1\x7c\x61\xec\xac\x20\xb1\xf9\x3b\xb2\x33\x98\xc9\x40\x86\xad\x67'
  run_signer_for_gl_node "gl2"

  run_local_cln_node "local1" '\x9f\xaa\x67\x08\x67\x05\x8c\x9d\xcc\x1f\xea\xd9\x9c\xe4\x91\xe2\x85\x95\x6d\xdb\x66\xa8\xed\x05\x85\xf3\x2a\x77\x0e\x1d\x14\xa6'
}

wait_for_local_funds_confirmation() {
  local is_confirmed=false
  while ! $is_confirmed; do
    if lightning-cli --network=regtest --lightning-dir="$LOCAL_LIGHTNING_DIR" listfunds | jq -e '.outputs[] | select(.status == "confirmed" and .reserved == false)' > /dev/null; then
      is_confirmed=true
    else
      echo "Waiting for funds to confirm on local node..."
      sleep 1
    fi
  done
}

wait_for_gl_funds_confirmation() {
  cd "./examples/javascript" || exit
  while ! node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_1" "ListFunds" "Listfunds" "Listfunds" "{}" | grep -q "outputs"; do
    echo "Waiting for funds to confirm on gl node 1..."
    sleep 1
  done
  while ! node node-operations.js "$GL_GRPC_PORT" "$NODE_PUBKEY_2" "ListFunds" "Listfunds" "Listfunds" "{}" | grep -q "outputs"; do
    echo "Waiting for funds to confirm on gl node 2..."
    sleep 1
  done
  cd "../.." || exit
}

mine_blocks() {
  local number_of_blocks=${1:-"6"}
  bitcoin-cli -rpcconnect="$BITCOIN_HOST" -rpcport="$BITCOIN_PORT" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcwallet="$WALLET_NAME" -generate "$number_of_blocks" > /dev/null
  wait_for_gl_funds_confirmation
  wait_for_local_funds_confirmation
}

fund_nodes() {
  rm -rf "${GL_SERVER_DATA_PATH}/gl-testserver/regtest/${WALLET_NAME}"  
  echo "Creating bitcoin wallet"
  bitcoin-cli -rpcconnect="$BITCOIN_HOST" -rpcport="$BITCOIN_PORT" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" createwallet "$WALLET_NAME" > /dev/null
  echo "Generating 101 blocks"
  bitcoin-cli -rpcconnect="$BITCOIN_HOST" -rpcport="$BITCOIN_PORT" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcwallet="$WALLET_NAME" -generate 101 > /dev/null
  echo "Sending bitcoin to local node"
  bitcoin-cli -rpcconnect="$BITCOIN_HOST" -rpcport="$BITCOIN_PORT" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcwallet="$WALLET_NAME" sendtoaddress "bcrt1qaew9v5m5q8cjjuh9mdruuujykzxrcufma28x82" 1 > /dev/null
  echo "Sending bitcoin to gl1"
  bitcoin-cli -rpcconnect="$BITCOIN_HOST" -rpcport="$BITCOIN_PORT" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcwallet="$WALLET_NAME" sendtoaddress "bcrt1q0mlp2u676wv9rgz5e6nrrq2vc76rllxcazcldy" 1 > /dev/null
  echo "Sending bitcoin to gl2"
  bitcoin-cli -rpcconnect="$BITCOIN_HOST" -rpcport="$BITCOIN_PORT" -rpcuser="$RPC_USER" -rpcpassword="$RPC_PASS" -rpcwallet="$WALLET_NAME" sendtoaddress "bcrt1q68n5mqlkf0l877chhpa2w2zxlug343kn0p8c6r" 1 > /dev/null
  mine_blocks 1
}

if [ "$AUTO_START" = "start" ]; then
  print_envs
  start_nodes
  fund_nodes
  connect_and_fund_channels_from_gl_nodes
  fund_channels_from_local_node
  create_invoice_and_pay
else
  echo "List of functions: "
  echo "print_envs: Prints all environment variables used in the script."
  echo "start_nodes: Starts all nodes (GL nodes and the local CLN node)."
  echo "fund_nodes: Creates a Bitcoin wallet, generates blocks, and sends funds to nodes."
  echo "connect_and_fund_channels_from_gl_nodes: Connects and funds channels between Greenlight (GL) nodes and the local CLN node."
  echo "fund_channels_from_local_node: Funds channels from the local CLN node to GL nodes."
  echo "create_invoice_and_pay: Creates invoices and simulates payments between nodes."
  echo "run_scheduler_for_gl_node: Registers and schedules a Greenlight (GL) node."
  echo "run_signer_for_gl_node: Starts the signer for a Greenlight (GL) node."
  echo "run_local_cln_node: Starts the local Core Lightning node."
  echo "wait_for_local_funds_confirmation: Waits for funds to be confirmed on the local CLN node."
  echo "wait_for_gl_funds_confirmation: Waits for funds to be confirmed on Greenlight (GL) nodes."
  echo "mine_blocks: Mines a specified number of blocks and waits for funds to be confirmed."
  # shellcheck disable=SC2016
  echo 'lightning-cli --network=regtest --lightning-dir="$LOCAL_LIGHTNING_DIR" listfunds'
  # shellcheck disable=SC2016
  echo 'cd "./examples/javascript" && node node-operations.js $GL_GRPC_PORT $NODE_PUBKEY_1 "ListFunds" "Listfunds" "Listfunds" "{}" && cd "../.."'
fi

# Keep the script running to listen for Ctrl+C
wait
