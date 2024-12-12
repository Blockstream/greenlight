const path = require('path');
const axios = require('axios');
const protobuf = require('protobufjs');

const PORT = process.argv[2] || '1111';
const AUTH_PUBKEY = 'replace+this+with+your+base64+encoded+pubkey';
const AUTH_SIGNATURE = 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA';
const PROTO_PATHS = [
  path.join(process.cwd(), '../../libs/gl-client/.resources/proto/node.proto'),
  path.join(process.cwd(), '../../libs/gl-client/.resources/proto/primitives.proto')
];

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
  const header = Buffer.alloc(4);
  header.writeUInt8(0, 0);
  const requestPayload = methodRequest.create(payload);
  const encodedPayload = methodRequest.encodeDelimited(requestPayload).finish();
  return Buffer.concat([header, encodedPayload]);
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
  const methodResponse = clnNode.lookupType(`cln.${method}Response`)  
  const offset = 5;
  const responseData = new Uint8Array(response.data).slice(offset);
  const grpcStatus = +response.headers['grpc-status'];
  if (grpcStatus !== 0) {
    let errorDecoded = new TextDecoder("utf-8").decode(responseData);
    if (errorDecoded !== 'None') {
      errorDecoded = JSON.parse(errorDecoded.replace(/([a-zA-Z0-9_]+):/g, '"$1":'));
    } else {
      errorDecoded = {code: grpcStatus, message: getGrpcErrorMessage(grpcStatus)};
    }
    return { grpc_code: grpcStatus, grpc_error: getGrpcErrorMessage(grpcStatus), error: errorDecoded};
  } else {
    // FIXME: Use decodeDelimited
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
        console.error('\nResponse Error:\n', error.response.status, ' - ', error.response.statusText);
      }
    }
  } catch (error) {
    console.error('Error:', error.message);
    if (error.response) {
      console.error('Error status:', error.response.status);
      console.error('Error data:', error.response.data);
    }
  }
}

fetchNodeData();
