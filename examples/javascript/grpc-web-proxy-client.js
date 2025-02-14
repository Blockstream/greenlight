const fs = require('fs');
const path = require('path');
const axios = require('axios');
const protobuf = require('protobufjs');

const PORT = process.argv[2] || getPortFromMetadata() || '1111';
const NODE_PUBKEY = 'yournodepubkeyhexvalue00000000000000000000000000000000000000000000';
const AUTH_PUBKEY = Buffer.from(NODE_PUBKEY, 'hex').toString('base64');
const AUTH_SIGNATURE = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
const PROTO_PATHS = [
  path.join(process.cwd(), '../../libs/gl-client/.resources/proto/node.proto'),
  path.join(process.cwd(), '../../libs/gl-client/.resources/proto/primitives.proto')
];

function getPortFromMetadata() {
  try {
    const grpcWebProxyUri = JSON.parse(fs.readFileSync('../../metadata.json')).grpc_web_proxy_uri;
    if (!grpcWebProxyUri) {
      console.error('grpc_web_proxy_uri not found in metadata.json');
      return null;
    }
    const grpc_port = new URL(grpcWebProxyUri).port;
    if (!grpc_port) {
      console.error('Port not found in grpc_web_proxy_uri');
      return null;
    }
    return grpc_port;
  } catch (error) {
    console.error('Error reading metadata.json: ', error.message);
    return null;
  }
}

function getGrpcErrorMessage(grpcStatusCode) {
  const grpcStatusMessages = {
    0: 'OK: The operation completed successfully.',
    1: 'CANCELLED: The operation was cancelled (typically by the caller).',
    2: 'UNKNOWN: Unknown error. Usually means an internal error occurred.',
    3: 'INVALID_ARGUMENT: The client specified an invalid argument.',
    4: 'DEADLINE_EXCEEDED: The operation took too long and exceeded the time limit.',
    5: 'NOT_FOUND: A specified resource was not found.',
    6: 'ALREADY_EXISTS: The resource already exists.',
    7: 'PERMISSION_DENIED: The caller does not have permission to execute the operation.',
    8: 'RESOURCE_EXHAUSTED: A resource (such as quota) was exhausted.',
    9: 'FAILED_PRECONDITION: The operation was rejected due to a failed precondition.',
    10: 'ABORTED: The operation was aborted, typically due to a concurrency issue.',
    11: 'OUT_OF_RANGE: The operation attempted to access an out-of-range value.',
    12: 'UNIMPLEMENTED: The operation is not implemented or supported by the server.',
    13: 'INTERNAL: Internal server error.',
    14: 'UNAVAILABLE: The service is unavailable (e.g., network issues, server down).',
    15: 'DATA_LOSS: Unrecoverable data loss or corruption.',
    16: 'UNAUTHENTICATED: The request is missing or has invalid authentication credentials.'
  }
  return grpcStatusMessages[grpcStatusCode] || "UNKNOWN_STATUS_CODE: The status code returned by gRPC server is not in the list.";
}

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

async function sendRequest(methodUrl, encodedPayload) {
  const buffer = Buffer.alloc(8);
  buffer.writeUInt32BE(Math.floor(Date.now() / 1000), 4);
  const axiosConfig = {
    responseType: 'arraybuffer',
    headers: {
      'content-type': 'application/grpc',
      'accept': 'application/grpc',
      'glauthpubkey': AUTH_PUBKEY,
      'glauthsig': AUTH_SIGNATURE,
      'glts': buffer.toString('base64'),
    },
  };
  return await axios.post(`http://localhost:${PORT}/cln.Node/${methodUrl}`, encodedPayload, axiosConfig);
}

function transformValue(key, value) {
  if ((value.type && value.type === "Buffer") || value instanceof Buffer || value instanceof Uint8Array) {
    return Buffer.from(value).toString('hex');
  }
  if (value.msat && !Number.isNaN(parseInt(value.msat))) {
    // FIXME: Amount.varify check will work with 0 NOT '0'. Amount default is '0'.
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
              errorMessage = getGrpcErrorMessage(grpcStatus);
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

async function fetchNodeData() {
  try {
    const clnNode = new protobuf.Root().loadSync(PROTO_PATHS, { keepCase: true });
    const FeeratesStyle = clnNode.lookupEnum('cln.FeeratesStyle');
    const NewaddrAddresstype = clnNode.lookupEnum('cln.NewaddrAddresstype');
    const methods = ['Getinfo', 'Feerates', 'NewAddr', 'Invoice', 'ListInvoices'];
    const method_payloads = [{}, {style: FeeratesStyle.values.PERKW}, {addresstype: NewaddrAddresstype.values.ALL}, {amount_msat: {amount: {msat: 500000}}, description: 'My coffee', label: 'coffeeinvat' + Date.now()}, {}];
    for (let i = 0; i < methods.length; i++) {
      console.log('--------------------------------------------\n', (i + 1), '-', methods[i], '\n--------------------------------------------');
      console.log('Payload Raw:\n', method_payloads[i]);
      const CapitalizedMethodName = methods[i].charAt(0).toUpperCase() + methods[i].slice(1).toLowerCase();
      const encodedPayload = await encodePayload(clnNode, CapitalizedMethodName, method_payloads[i]);
      console.log('\nPayload Encoded:\n', encodedPayload);
      try {
        const response = await sendRequest(methods[i], encodedPayload);
        console.log('\nResponse Headers:\n', response.headers);
        console.log('\nResponse Data:\n', response.data);
        const responseJSON = decodeResponse(clnNode, CapitalizedMethodName, response);
        console.log('\nResponse Decoded:');
        console.dir(responseJSON, { depth: null, color: true });
      } catch (error) {
        console.error('\nResponse Error:\n', error.response?.status || error.code, ' - ', error.response?.statusText || error.response?.data || error.message || '');
      }
    }
  } catch (error) {
    console.error('Error:', error.message);
    if (error.response) {
      console.error('Error status:', error.response?.status || error.code);
      console.error('Error data:', error.response?.statusText || error.response?.data || error.message || '');
    }
  }
}

fetchNodeData();
