# How to run javascript examples with gltestserver

## Step 1 (Terminal 1): Start the Server
```bash
make gltestserver
```

## Step 2 (Terminal 2): Register the Node
```bash
GL_CA_CRT=$HOME/greenlight/.gltestserver/gl-testserver/certs/ca.crt \
GL_NOBODY_CRT=$HOME/greenlight/.gltestserver/gl-testserver/certs/users/nobody.crt \
GL_NOBODY_KEY=$HOME/greenlight/.gltestserver/gl-testserver/certs/users/nobody-key.pem \
GL_SCHEDULER_GRPC_URI=https://localhost:38067 \
cargo run --bin glcli scheduler register --network=regtest --data-dir=$HOME/greenlight/.gltestserver/gl-testserver
```

## Step 3 (Terminal 2): Schedule the Node
```bash
GL_CA_CRT=$HOME/greenlight/.gltestserver/gl-testserver/certs/ca.crt \
GL_NOBODY_CRT=$HOME/greenlight/.gltestserver/gl-testserver/certs/users/nobody.crt \
GL_NOBODY_KEY=$HOME/greenlight/.gltestserver/gl-testserver/certs/users/nobody-key.pem \
GL_SCHEDULER_GRPC_URI=https://localhost:38067 \
cargo run --bin glcli scheduler schedule --verbose --network=regtest --data-dir=$HOME/greenlight/.gltestserver/gl-testserver
```

## Step 4 (Terminal 2): Start the Signer
```bash
GL_CA_CRT=$HOME/greenlight/.gltestserver/gl-testserver/certs/ca.crt \
GL_NOBODY_CRT=$HOME/greenlight/.gltestserver/gl-testserver/certs/users/nobody.crt \
GL_NOBODY_KEY=$HOME/greenlight/.gltestserver/gl-testserver/certs/users/nobody-key.pem \
GL_SCHEDULER_GRPC_URI=https://localhost:38067 \
cargo run --bin glcli signer run --verbose --network=regtest --data-dir=$HOME/greenlight/.gltestserver/gl-testserver
```

## Step 5 (Terminal 3): Run the Example
### 5.1: Navigate and Install Dependencies for the Example
```bash
cd ./examples/javascript
npm install
```

### 5.2: Get Node ID
```bash
lightning-hsmtool getnodeid $HOME/greenlight/.gltestserver/gl-testserver/hsm_secret
```
Sample Output:	034c46b632a9ff3975fb7cd4e764a36ec476b522be2555e83a3183ab1ee3e36e93

### 5.3: Encode Node ID to Base64
```python
import binascii
import base64
print(base64.b64encode(binascii.unhexlify("<node id from step 5.2>")).decode('utf-8'))
```
Sample Output: A0xGtjKp/zl1+3zU52SjbsR2tSK+JVXoOjGDqx7j426T

### 5.4: Modify Default Values
- Open the file `./examples/javascript/grpc-web-proxy-client.js`.

- Locate the line defining `AUTH_PUBKEY` and replace its value with the Base64-encoded public key output from Step 5.3:

    ```javascript
    const AUTH_PUBKEY = 'replace+this+with+your+base64+encoded+pubkey';
    ```

- Replace the default PORT value `1111` with the port number from `grpc_web_proxy_uri` obtained in Step 1:
    ```javascript
    const PORT = process.argv[2] || '1111';
    ```
    Alternatively, the port number can be passed as a command-line argument when running the nodejs script in the next step.

- Save the changes to the file.

### 5.5: Run the Example
```bash
node grpc-web-proxy-client.js
```
