const path = require('path');
const axios = require('axios');
const protobuf = require('protobufjs');

async function encodePayload(clnNode, method, payload) {
    const methodRequest = clnNode.lookupType(`cln.${method}Request`);
    const errMsg = methodRequest.verify(payload);
    if (errMsg) throw new Error(errMsg);
    const requestPayload = methodRequest.create(payload);
    const encodedPayload = methodRequest.encode(requestPayload).finish();
    const flags = Buffer.alloc(1);
    flags.writeUInt8(0, 0);
    const header = Buffer.alloc(4);
    header.writeUInt32BE(encodedPayload.length, 0);
    return Buffer.concat([flags, header, encodedPayload]);
}

async function sendRequest(port, authPubkey, methodUrl, encodedPayload) {
    const buffer = Buffer.alloc(8);
    buffer.writeUInt32BE(Math.floor(Date.now() / 1000), 4);
    const axiosConfig = {
        responseType: 'arraybuffer',
        headers: {
            'content-type': 'application/grpc',
            'accept': 'application/grpc',
            'glauthpubkey': Buffer.from(authPubkey, 'hex').toString('base64'),
            'glauthsig': 'A'.repeat(64),
            'glts': buffer.toString('base64'),
        },
    };
    return await axios.post(`http://localhost:${port}/cln.Node/${methodUrl}`, encodedPayload, axiosConfig);
}

function transformValue(key, value) {
    if ((value.type && value.type === "Buffer") || value instanceof Buffer || value instanceof Uint8Array) {
      return Buffer.from(value).toString('hex');
    }
    if (value.msat && !Number.isNaN(parseInt(value.msat))) {
      return parseInt(value.msat);
    }
    return value;
}
  
function decodeResponse(clnNode, method, response) {
    const methodResponse = clnNode.lookupType(`cln.${method}Response`);
    const dataBuffer = Buffer.from(response.data);
    const resFlag = dataBuffer.subarray(0, 1);
    const resDataLength = dataBuffer.subarray(1, 5);
    const responseData = dataBuffer.subarray(5);
    const grpcStatus = +response.headers['grpc-status'];
    if (grpcStatus !== 0) {
        let errorMessage = 'None';
        try {
            errorMessage = decodeURIComponent(new TextDecoder('utf-8').decode(responseData)).trim();
            if (errorMessage == 'None') {
                errorMessage = grpcStatus;
            }
        } catch (decodeError) {
            errorMessage = decodeError;
        }
        throw new Error(errorMessage);
    }
    const decodedRes = methodResponse.decode(responseData);
    const decodedResObject = methodResponse.toObject(decodedRes, {
        longs: String,
        enums: String,
        bytes: Buffer,
        defaults: true,
        arrays: true,
        objects: true,
    });
    return JSON.parse(JSON.stringify(decodedResObject, transformValue));
  }

async function callMethod(port, auth_pubkey, clnNode, method, methodReq, methodRes, methodPayload) {
    await new Promise(resolve => setTimeout(resolve, 1000));
    const encodedPayload = await encodePayload(clnNode, methodReq, methodPayload);
    try {
        const response = await sendRequest(port, auth_pubkey, method, encodedPayload);
        const responseJSON = decodeResponse(clnNode, methodRes, response);
        console.log(JSON.stringify(responseJSON, null, 2));
    } catch (error) {
        throw error;
    }
}

async function main() {
    try {
        const args = process.argv.slice(2);
        const port = args[0];
        const auth_pubkey = args[1];
        const method = args[2];
        const methodReq = args[3];
        const methodRes = args[4];
        const methodPayload = JSON.parse(args[5]);
        const proto_path = [
            path.join(process.cwd(), '../../libs/gl-client/.resources/proto/node.proto'),
            path.join(process.cwd(), '../../libs/gl-client/.resources/proto/primitives.proto')
        ];
        const clnNode = new protobuf.Root().loadSync(proto_path, { keepCase: true });
        await callMethod(port, auth_pubkey, clnNode, method, methodReq, methodRes, methodPayload);
    } catch (error) {
        console.error(error);
    }
}

main();
