// node scheduler-operations.js "41725" "ListLsps" "{}"
// node scheduler-operations.js "41725" "GetLspInfo" '{"public_key": "02aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}'
const path = require('path');
const http2 = require('http2');
const fs = require('fs');
const protobuf = require('protobufjs');

const PROTO_PATH = [
  path.join(process.cwd(), '../../libs/proto/glclient/scheduler.proto'),
  path.join(process.cwd(), '../../libs/proto/glclient/greenlight.proto')
];
const glScheduler = new protobuf.Root().loadSync(PROTO_PATH, { keepCase: true });

function encodePayload(method, payload) {
  const methodRequest = glScheduler.lookupType(`scheduler.${method}Request`);
  const requestMessage = methodRequest.encode(payload).finish();
  const messageLength = requestMessage.length;
  const encodedPayload = Buffer.alloc(5 + messageLength);
  encodedPayload.writeUInt8(0, 0);
  encodedPayload.writeUInt32BE(messageLength, 1);
  encodedPayload.set(requestMessage, 5);
  return encodedPayload;
}

function decodeResponse(method, responseData) {
  const ListLspsResponse = glScheduler.lookupType(`scheduler.${method}Response`);
  const responseMessage = responseData.slice(5);
  const decodedResponse = ListLspsResponse.decode(responseMessage);
  const decodedResObject = ListLspsResponse.toObject(decodedResponse, {
    longs: String,
    enums: String,
    bytes: Buffer,
    defaults: true,
    arrays: true,
    objects: true,
  });
  return decodedResObject;
}

function callMethod(glsClient, method, payload) {
  const encodedPayload = encodePayload(method, payload);

  const req = glsClient.request({
    ':path': `/scheduler.Lsps/${method}`,
    ':method': 'POST',
    'content-type': 'application/grpc',
    'te': 'trailers',
  });
  req.write(encodedPayload);

  let responseData = Buffer.alloc(0);
  req.on('data', (chunk) => {
    responseData = Buffer.concat([responseData, chunk]);
  });

  req.on('end', () => {
    const decodedResObject = decodeResponse(method, responseData);
    console.log('Response:', JSON.stringify(decodedResObject, null, 2));
    glsClient.close();
    process.exit(0);
  });

  req.on('error', (error) => {
    console.error('Request error:', error);
    glsClient.close();
    process.exit(1);
  });

  req.end();
}

function main() {
  const args = process.argv.slice(2);
  const port = args[0];
  const method = args[1];
  const payload = JSON.parse(args[2]);

  const glsClient = http2.connect(`https://localhost:${port}`, {
    ca: fs.readFileSync('../../.gltestserver/gl-testserver/certs/ca.crt'),
    rejectUnauthorized: false,
  });

  callMethod(glsClient, method, payload);
}

main();
