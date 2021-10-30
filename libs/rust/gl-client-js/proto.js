/*eslint-disable block-scoped-var, id-length, no-control-regex, no-magic-numbers, no-prototype-builtins, no-redeclare, no-shadow, no-var, sort-vars*/
(function(global, factory) { /* global define, require, module */

    /* AMD */ if (typeof define === 'function' && define.amd)
        define(["protobufjs/minimal"], factory);

    /* CommonJS */ else if (typeof require === 'function' && typeof module === 'object' && module && module.exports)
        module.exports = factory(require("protobufjs/minimal"));

})(this, function($protobuf) {
    "use strict";

    // Common aliases
    var $Reader = $protobuf.Reader, $Writer = $protobuf.Writer, $util = $protobuf.util;
    
    // Exported root namespace
    var $root = $protobuf.roots["default"] || ($protobuf.roots["default"] = {});
    
    $root.scheduler = (function() {
    
        /**
         * Namespace scheduler.
         * @exports scheduler
         * @namespace
         */
        var scheduler = {};
    
        scheduler.Scheduler = (function() {
    
            /**
             * Constructs a new Scheduler service.
             * @memberof scheduler
             * @classdesc Represents a Scheduler
             * @extends $protobuf.rpc.Service
             * @constructor
             * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
             * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
             * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
             */
            function Scheduler(rpcImpl, requestDelimited, responseDelimited) {
                $protobuf.rpc.Service.call(this, rpcImpl, requestDelimited, responseDelimited);
            }
    
            (Scheduler.prototype = Object.create($protobuf.rpc.Service.prototype)).constructor = Scheduler;
    
            /**
             * Creates new Scheduler service using the specified rpc implementation.
             * @function create
             * @memberof scheduler.Scheduler
             * @static
             * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
             * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
             * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
             * @returns {Scheduler} RPC service. Useful where requests and/or responses are streamed.
             */
            Scheduler.create = function create(rpcImpl, requestDelimited, responseDelimited) {
                return new this(rpcImpl, requestDelimited, responseDelimited);
            };
    
            /**
             * Callback as used by {@link scheduler.Scheduler#register}.
             * @memberof scheduler.Scheduler
             * @typedef RegisterCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {scheduler.RegistrationResponse} [response] RegistrationResponse
             */
    
            /**
             * Calls Register.
             * @function register
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IRegistrationRequest} request RegistrationRequest message or plain object
             * @param {scheduler.Scheduler.RegisterCallback} callback Node-style callback called with the error, if any, and RegistrationResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Scheduler.prototype.register = function register(request, callback) {
                return this.rpcCall(register, $root.scheduler.RegistrationRequest, $root.scheduler.RegistrationResponse, request, callback);
            }, "name", { value: "Register" });
    
            /**
             * Calls Register.
             * @function register
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IRegistrationRequest} request RegistrationRequest message or plain object
             * @returns {Promise<scheduler.RegistrationResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link scheduler.Scheduler#recover}.
             * @memberof scheduler.Scheduler
             * @typedef RecoverCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {scheduler.RecoveryResponse} [response] RecoveryResponse
             */
    
            /**
             * Calls Recover.
             * @function recover
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IRecoveryRequest} request RecoveryRequest message or plain object
             * @param {scheduler.Scheduler.RecoverCallback} callback Node-style callback called with the error, if any, and RecoveryResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Scheduler.prototype.recover = function recover(request, callback) {
                return this.rpcCall(recover, $root.scheduler.RecoveryRequest, $root.scheduler.RecoveryResponse, request, callback);
            }, "name", { value: "Recover" });
    
            /**
             * Calls Recover.
             * @function recover
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IRecoveryRequest} request RecoveryRequest message or plain object
             * @returns {Promise<scheduler.RecoveryResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link scheduler.Scheduler#getChallenge}.
             * @memberof scheduler.Scheduler
             * @typedef GetChallengeCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {scheduler.ChallengeResponse} [response] ChallengeResponse
             */
    
            /**
             * Calls GetChallenge.
             * @function getChallenge
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IChallengeRequest} request ChallengeRequest message or plain object
             * @param {scheduler.Scheduler.GetChallengeCallback} callback Node-style callback called with the error, if any, and ChallengeResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Scheduler.prototype.getChallenge = function getChallenge(request, callback) {
                return this.rpcCall(getChallenge, $root.scheduler.ChallengeRequest, $root.scheduler.ChallengeResponse, request, callback);
            }, "name", { value: "GetChallenge" });
    
            /**
             * Calls GetChallenge.
             * @function getChallenge
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IChallengeRequest} request ChallengeRequest message or plain object
             * @returns {Promise<scheduler.ChallengeResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link scheduler.Scheduler#schedule}.
             * @memberof scheduler.Scheduler
             * @typedef ScheduleCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {scheduler.NodeInfoResponse} [response] NodeInfoResponse
             */
    
            /**
             * Calls Schedule.
             * @function schedule
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IScheduleRequest} request ScheduleRequest message or plain object
             * @param {scheduler.Scheduler.ScheduleCallback} callback Node-style callback called with the error, if any, and NodeInfoResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Scheduler.prototype.schedule = function schedule(request, callback) {
                return this.rpcCall(schedule, $root.scheduler.ScheduleRequest, $root.scheduler.NodeInfoResponse, request, callback);
            }, "name", { value: "Schedule" });
    
            /**
             * Calls Schedule.
             * @function schedule
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.IScheduleRequest} request ScheduleRequest message or plain object
             * @returns {Promise<scheduler.NodeInfoResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link scheduler.Scheduler#getNodeInfo}.
             * @memberof scheduler.Scheduler
             * @typedef GetNodeInfoCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {scheduler.NodeInfoResponse} [response] NodeInfoResponse
             */
    
            /**
             * Calls GetNodeInfo.
             * @function getNodeInfo
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.INodeInfoRequest} request NodeInfoRequest message or plain object
             * @param {scheduler.Scheduler.GetNodeInfoCallback} callback Node-style callback called with the error, if any, and NodeInfoResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Scheduler.prototype.getNodeInfo = function getNodeInfo(request, callback) {
                return this.rpcCall(getNodeInfo, $root.scheduler.NodeInfoRequest, $root.scheduler.NodeInfoResponse, request, callback);
            }, "name", { value: "GetNodeInfo" });
    
            /**
             * Calls GetNodeInfo.
             * @function getNodeInfo
             * @memberof scheduler.Scheduler
             * @instance
             * @param {scheduler.INodeInfoRequest} request NodeInfoRequest message or plain object
             * @returns {Promise<scheduler.NodeInfoResponse>} Promise
             * @variation 2
             */
    
            return Scheduler;
        })();
    
        scheduler.ChallengeRequest = (function() {
    
            /**
             * Properties of a ChallengeRequest.
             * @memberof scheduler
             * @interface IChallengeRequest
             * @property {scheduler.ChallengeScope|null} [scope] ChallengeRequest scope
             * @property {Uint8Array|null} [nodeId] ChallengeRequest nodeId
             */
    
            /**
             * Constructs a new ChallengeRequest.
             * @memberof scheduler
             * @classdesc Represents a ChallengeRequest.
             * @implements IChallengeRequest
             * @constructor
             * @param {scheduler.IChallengeRequest=} [properties] Properties to set
             */
            function ChallengeRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ChallengeRequest scope.
             * @member {scheduler.ChallengeScope} scope
             * @memberof scheduler.ChallengeRequest
             * @instance
             */
            ChallengeRequest.prototype.scope = 0;
    
            /**
             * ChallengeRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof scheduler.ChallengeRequest
             * @instance
             */
            ChallengeRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * Creates a new ChallengeRequest instance using the specified properties.
             * @function create
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {scheduler.IChallengeRequest=} [properties] Properties to set
             * @returns {scheduler.ChallengeRequest} ChallengeRequest instance
             */
            ChallengeRequest.create = function create(properties) {
                return new ChallengeRequest(properties);
            };
    
            /**
             * Encodes the specified ChallengeRequest message. Does not implicitly {@link scheduler.ChallengeRequest.verify|verify} messages.
             * @function encode
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {scheduler.IChallengeRequest} message ChallengeRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ChallengeRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.scope != null && Object.hasOwnProperty.call(message, "scope"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.scope);
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.nodeId);
                return writer;
            };
    
            /**
             * Encodes the specified ChallengeRequest message, length delimited. Does not implicitly {@link scheduler.ChallengeRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {scheduler.IChallengeRequest} message ChallengeRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ChallengeRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ChallengeRequest message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.ChallengeRequest} ChallengeRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ChallengeRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.ChallengeRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.scope = reader.int32();
                        break;
                    case 2:
                        message.nodeId = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ChallengeRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.ChallengeRequest} ChallengeRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ChallengeRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ChallengeRequest message.
             * @function verify
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ChallengeRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.scope != null && message.hasOwnProperty("scope"))
                    switch (message.scope) {
                    default:
                        return "scope: enum value expected";
                    case 0:
                    case 1:
                        break;
                    }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                return null;
            };
    
            /**
             * Creates a ChallengeRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.ChallengeRequest} ChallengeRequest
             */
            ChallengeRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.ChallengeRequest)
                    return object;
                var message = new $root.scheduler.ChallengeRequest();
                switch (object.scope) {
                case "REGISTER":
                case 0:
                    message.scope = 0;
                    break;
                case "RECOVER":
                case 1:
                    message.scope = 1;
                    break;
                }
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                return message;
            };
    
            /**
             * Creates a plain object from a ChallengeRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.ChallengeRequest
             * @static
             * @param {scheduler.ChallengeRequest} message ChallengeRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ChallengeRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.scope = options.enums === String ? "REGISTER" : 0;
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                }
                if (message.scope != null && message.hasOwnProperty("scope"))
                    object.scope = options.enums === String ? $root.scheduler.ChallengeScope[message.scope] : message.scope;
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                return object;
            };
    
            /**
             * Converts this ChallengeRequest to JSON.
             * @function toJSON
             * @memberof scheduler.ChallengeRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ChallengeRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ChallengeRequest;
        })();
    
        scheduler.ChallengeResponse = (function() {
    
            /**
             * Properties of a ChallengeResponse.
             * @memberof scheduler
             * @interface IChallengeResponse
             * @property {Uint8Array|null} [challenge] ChallengeResponse challenge
             */
    
            /**
             * Constructs a new ChallengeResponse.
             * @memberof scheduler
             * @classdesc Represents a ChallengeResponse.
             * @implements IChallengeResponse
             * @constructor
             * @param {scheduler.IChallengeResponse=} [properties] Properties to set
             */
            function ChallengeResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ChallengeResponse challenge.
             * @member {Uint8Array} challenge
             * @memberof scheduler.ChallengeResponse
             * @instance
             */
            ChallengeResponse.prototype.challenge = $util.newBuffer([]);
    
            /**
             * Creates a new ChallengeResponse instance using the specified properties.
             * @function create
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {scheduler.IChallengeResponse=} [properties] Properties to set
             * @returns {scheduler.ChallengeResponse} ChallengeResponse instance
             */
            ChallengeResponse.create = function create(properties) {
                return new ChallengeResponse(properties);
            };
    
            /**
             * Encodes the specified ChallengeResponse message. Does not implicitly {@link scheduler.ChallengeResponse.verify|verify} messages.
             * @function encode
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {scheduler.IChallengeResponse} message ChallengeResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ChallengeResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.challenge != null && Object.hasOwnProperty.call(message, "challenge"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.challenge);
                return writer;
            };
    
            /**
             * Encodes the specified ChallengeResponse message, length delimited. Does not implicitly {@link scheduler.ChallengeResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {scheduler.IChallengeResponse} message ChallengeResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ChallengeResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ChallengeResponse message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.ChallengeResponse} ChallengeResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ChallengeResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.ChallengeResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.challenge = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ChallengeResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.ChallengeResponse} ChallengeResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ChallengeResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ChallengeResponse message.
             * @function verify
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ChallengeResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.challenge != null && message.hasOwnProperty("challenge"))
                    if (!(message.challenge && typeof message.challenge.length === "number" || $util.isString(message.challenge)))
                        return "challenge: buffer expected";
                return null;
            };
    
            /**
             * Creates a ChallengeResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.ChallengeResponse} ChallengeResponse
             */
            ChallengeResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.ChallengeResponse)
                    return object;
                var message = new $root.scheduler.ChallengeResponse();
                if (object.challenge != null)
                    if (typeof object.challenge === "string")
                        $util.base64.decode(object.challenge, message.challenge = $util.newBuffer($util.base64.length(object.challenge)), 0);
                    else if (object.challenge.length)
                        message.challenge = object.challenge;
                return message;
            };
    
            /**
             * Creates a plain object from a ChallengeResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.ChallengeResponse
             * @static
             * @param {scheduler.ChallengeResponse} message ChallengeResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ChallengeResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    if (options.bytes === String)
                        object.challenge = "";
                    else {
                        object.challenge = [];
                        if (options.bytes !== Array)
                            object.challenge = $util.newBuffer(object.challenge);
                    }
                if (message.challenge != null && message.hasOwnProperty("challenge"))
                    object.challenge = options.bytes === String ? $util.base64.encode(message.challenge, 0, message.challenge.length) : options.bytes === Array ? Array.prototype.slice.call(message.challenge) : message.challenge;
                return object;
            };
    
            /**
             * Converts this ChallengeResponse to JSON.
             * @function toJSON
             * @memberof scheduler.ChallengeResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ChallengeResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ChallengeResponse;
        })();
    
        /**
         * ChallengeScope enum.
         * @name scheduler.ChallengeScope
         * @enum {number}
         * @property {number} REGISTER=0 REGISTER value
         * @property {number} RECOVER=1 RECOVER value
         */
        scheduler.ChallengeScope = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "REGISTER"] = 0;
            values[valuesById[1] = "RECOVER"] = 1;
            return values;
        })();
    
        scheduler.RegistrationRequest = (function() {
    
            /**
             * Properties of a RegistrationRequest.
             * @memberof scheduler
             * @interface IRegistrationRequest
             * @property {Uint8Array|null} [nodeId] RegistrationRequest nodeId
             * @property {Uint8Array|null} [bip32Key] RegistrationRequest bip32Key
             * @property {string|null} [email] RegistrationRequest email
             * @property {string|null} [network] RegistrationRequest network
             * @property {Uint8Array|null} [challenge] RegistrationRequest challenge
             * @property {Uint8Array|null} [signature] RegistrationRequest signature
             */
    
            /**
             * Constructs a new RegistrationRequest.
             * @memberof scheduler
             * @classdesc Represents a RegistrationRequest.
             * @implements IRegistrationRequest
             * @constructor
             * @param {scheduler.IRegistrationRequest=} [properties] Properties to set
             */
            function RegistrationRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * RegistrationRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof scheduler.RegistrationRequest
             * @instance
             */
            RegistrationRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * RegistrationRequest bip32Key.
             * @member {Uint8Array} bip32Key
             * @memberof scheduler.RegistrationRequest
             * @instance
             */
            RegistrationRequest.prototype.bip32Key = $util.newBuffer([]);
    
            /**
             * RegistrationRequest email.
             * @member {string} email
             * @memberof scheduler.RegistrationRequest
             * @instance
             */
            RegistrationRequest.prototype.email = "";
    
            /**
             * RegistrationRequest network.
             * @member {string} network
             * @memberof scheduler.RegistrationRequest
             * @instance
             */
            RegistrationRequest.prototype.network = "";
    
            /**
             * RegistrationRequest challenge.
             * @member {Uint8Array} challenge
             * @memberof scheduler.RegistrationRequest
             * @instance
             */
            RegistrationRequest.prototype.challenge = $util.newBuffer([]);
    
            /**
             * RegistrationRequest signature.
             * @member {Uint8Array} signature
             * @memberof scheduler.RegistrationRequest
             * @instance
             */
            RegistrationRequest.prototype.signature = $util.newBuffer([]);
    
            /**
             * Creates a new RegistrationRequest instance using the specified properties.
             * @function create
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {scheduler.IRegistrationRequest=} [properties] Properties to set
             * @returns {scheduler.RegistrationRequest} RegistrationRequest instance
             */
            RegistrationRequest.create = function create(properties) {
                return new RegistrationRequest(properties);
            };
    
            /**
             * Encodes the specified RegistrationRequest message. Does not implicitly {@link scheduler.RegistrationRequest.verify|verify} messages.
             * @function encode
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {scheduler.IRegistrationRequest} message RegistrationRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RegistrationRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.bip32Key != null && Object.hasOwnProperty.call(message, "bip32Key"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.bip32Key);
                if (message.email != null && Object.hasOwnProperty.call(message, "email"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.email);
                if (message.network != null && Object.hasOwnProperty.call(message, "network"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.network);
                if (message.challenge != null && Object.hasOwnProperty.call(message, "challenge"))
                    writer.uint32(/* id 5, wireType 2 =*/42).bytes(message.challenge);
                if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                    writer.uint32(/* id 6, wireType 2 =*/50).bytes(message.signature);
                return writer;
            };
    
            /**
             * Encodes the specified RegistrationRequest message, length delimited. Does not implicitly {@link scheduler.RegistrationRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {scheduler.IRegistrationRequest} message RegistrationRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RegistrationRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a RegistrationRequest message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.RegistrationRequest} RegistrationRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RegistrationRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.RegistrationRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.bip32Key = reader.bytes();
                        break;
                    case 3:
                        message.email = reader.string();
                        break;
                    case 4:
                        message.network = reader.string();
                        break;
                    case 5:
                        message.challenge = reader.bytes();
                        break;
                    case 6:
                        message.signature = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a RegistrationRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.RegistrationRequest} RegistrationRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RegistrationRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a RegistrationRequest message.
             * @function verify
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RegistrationRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.bip32Key != null && message.hasOwnProperty("bip32Key"))
                    if (!(message.bip32Key && typeof message.bip32Key.length === "number" || $util.isString(message.bip32Key)))
                        return "bip32Key: buffer expected";
                if (message.email != null && message.hasOwnProperty("email"))
                    if (!$util.isString(message.email))
                        return "email: string expected";
                if (message.network != null && message.hasOwnProperty("network"))
                    if (!$util.isString(message.network))
                        return "network: string expected";
                if (message.challenge != null && message.hasOwnProperty("challenge"))
                    if (!(message.challenge && typeof message.challenge.length === "number" || $util.isString(message.challenge)))
                        return "challenge: buffer expected";
                if (message.signature != null && message.hasOwnProperty("signature"))
                    if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
                        return "signature: buffer expected";
                return null;
            };
    
            /**
             * Creates a RegistrationRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.RegistrationRequest} RegistrationRequest
             */
            RegistrationRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.RegistrationRequest)
                    return object;
                var message = new $root.scheduler.RegistrationRequest();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.bip32Key != null)
                    if (typeof object.bip32Key === "string")
                        $util.base64.decode(object.bip32Key, message.bip32Key = $util.newBuffer($util.base64.length(object.bip32Key)), 0);
                    else if (object.bip32Key.length)
                        message.bip32Key = object.bip32Key;
                if (object.email != null)
                    message.email = String(object.email);
                if (object.network != null)
                    message.network = String(object.network);
                if (object.challenge != null)
                    if (typeof object.challenge === "string")
                        $util.base64.decode(object.challenge, message.challenge = $util.newBuffer($util.base64.length(object.challenge)), 0);
                    else if (object.challenge.length)
                        message.challenge = object.challenge;
                if (object.signature != null)
                    if (typeof object.signature === "string")
                        $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                    else if (object.signature.length)
                        message.signature = object.signature;
                return message;
            };
    
            /**
             * Creates a plain object from a RegistrationRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.RegistrationRequest
             * @static
             * @param {scheduler.RegistrationRequest} message RegistrationRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RegistrationRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    if (options.bytes === String)
                        object.bip32Key = "";
                    else {
                        object.bip32Key = [];
                        if (options.bytes !== Array)
                            object.bip32Key = $util.newBuffer(object.bip32Key);
                    }
                    object.email = "";
                    object.network = "";
                    if (options.bytes === String)
                        object.challenge = "";
                    else {
                        object.challenge = [];
                        if (options.bytes !== Array)
                            object.challenge = $util.newBuffer(object.challenge);
                    }
                    if (options.bytes === String)
                        object.signature = "";
                    else {
                        object.signature = [];
                        if (options.bytes !== Array)
                            object.signature = $util.newBuffer(object.signature);
                    }
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.bip32Key != null && message.hasOwnProperty("bip32Key"))
                    object.bip32Key = options.bytes === String ? $util.base64.encode(message.bip32Key, 0, message.bip32Key.length) : options.bytes === Array ? Array.prototype.slice.call(message.bip32Key) : message.bip32Key;
                if (message.email != null && message.hasOwnProperty("email"))
                    object.email = message.email;
                if (message.network != null && message.hasOwnProperty("network"))
                    object.network = message.network;
                if (message.challenge != null && message.hasOwnProperty("challenge"))
                    object.challenge = options.bytes === String ? $util.base64.encode(message.challenge, 0, message.challenge.length) : options.bytes === Array ? Array.prototype.slice.call(message.challenge) : message.challenge;
                if (message.signature != null && message.hasOwnProperty("signature"))
                    object.signature = options.bytes === String ? $util.base64.encode(message.signature, 0, message.signature.length) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
                return object;
            };
    
            /**
             * Converts this RegistrationRequest to JSON.
             * @function toJSON
             * @memberof scheduler.RegistrationRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RegistrationRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return RegistrationRequest;
        })();
    
        scheduler.RegistrationResponse = (function() {
    
            /**
             * Properties of a RegistrationResponse.
             * @memberof scheduler
             * @interface IRegistrationResponse
             * @property {string|null} [deviceCert] RegistrationResponse deviceCert
             * @property {string|null} [deviceKey] RegistrationResponse deviceKey
             */
    
            /**
             * Constructs a new RegistrationResponse.
             * @memberof scheduler
             * @classdesc Represents a RegistrationResponse.
             * @implements IRegistrationResponse
             * @constructor
             * @param {scheduler.IRegistrationResponse=} [properties] Properties to set
             */
            function RegistrationResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * RegistrationResponse deviceCert.
             * @member {string} deviceCert
             * @memberof scheduler.RegistrationResponse
             * @instance
             */
            RegistrationResponse.prototype.deviceCert = "";
    
            /**
             * RegistrationResponse deviceKey.
             * @member {string} deviceKey
             * @memberof scheduler.RegistrationResponse
             * @instance
             */
            RegistrationResponse.prototype.deviceKey = "";
    
            /**
             * Creates a new RegistrationResponse instance using the specified properties.
             * @function create
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {scheduler.IRegistrationResponse=} [properties] Properties to set
             * @returns {scheduler.RegistrationResponse} RegistrationResponse instance
             */
            RegistrationResponse.create = function create(properties) {
                return new RegistrationResponse(properties);
            };
    
            /**
             * Encodes the specified RegistrationResponse message. Does not implicitly {@link scheduler.RegistrationResponse.verify|verify} messages.
             * @function encode
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {scheduler.IRegistrationResponse} message RegistrationResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RegistrationResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.deviceCert != null && Object.hasOwnProperty.call(message, "deviceCert"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.deviceCert);
                if (message.deviceKey != null && Object.hasOwnProperty.call(message, "deviceKey"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.deviceKey);
                return writer;
            };
    
            /**
             * Encodes the specified RegistrationResponse message, length delimited. Does not implicitly {@link scheduler.RegistrationResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {scheduler.IRegistrationResponse} message RegistrationResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RegistrationResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a RegistrationResponse message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.RegistrationResponse} RegistrationResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RegistrationResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.RegistrationResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.deviceCert = reader.string();
                        break;
                    case 2:
                        message.deviceKey = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a RegistrationResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.RegistrationResponse} RegistrationResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RegistrationResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a RegistrationResponse message.
             * @function verify
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RegistrationResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.deviceCert != null && message.hasOwnProperty("deviceCert"))
                    if (!$util.isString(message.deviceCert))
                        return "deviceCert: string expected";
                if (message.deviceKey != null && message.hasOwnProperty("deviceKey"))
                    if (!$util.isString(message.deviceKey))
                        return "deviceKey: string expected";
                return null;
            };
    
            /**
             * Creates a RegistrationResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.RegistrationResponse} RegistrationResponse
             */
            RegistrationResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.RegistrationResponse)
                    return object;
                var message = new $root.scheduler.RegistrationResponse();
                if (object.deviceCert != null)
                    message.deviceCert = String(object.deviceCert);
                if (object.deviceKey != null)
                    message.deviceKey = String(object.deviceKey);
                return message;
            };
    
            /**
             * Creates a plain object from a RegistrationResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.RegistrationResponse
             * @static
             * @param {scheduler.RegistrationResponse} message RegistrationResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RegistrationResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.deviceCert = "";
                    object.deviceKey = "";
                }
                if (message.deviceCert != null && message.hasOwnProperty("deviceCert"))
                    object.deviceCert = message.deviceCert;
                if (message.deviceKey != null && message.hasOwnProperty("deviceKey"))
                    object.deviceKey = message.deviceKey;
                return object;
            };
    
            /**
             * Converts this RegistrationResponse to JSON.
             * @function toJSON
             * @memberof scheduler.RegistrationResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RegistrationResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return RegistrationResponse;
        })();
    
        scheduler.ScheduleRequest = (function() {
    
            /**
             * Properties of a ScheduleRequest.
             * @memberof scheduler
             * @interface IScheduleRequest
             * @property {Uint8Array|null} [nodeId] ScheduleRequest nodeId
             */
    
            /**
             * Constructs a new ScheduleRequest.
             * @memberof scheduler
             * @classdesc Represents a ScheduleRequest.
             * @implements IScheduleRequest
             * @constructor
             * @param {scheduler.IScheduleRequest=} [properties] Properties to set
             */
            function ScheduleRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ScheduleRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof scheduler.ScheduleRequest
             * @instance
             */
            ScheduleRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * Creates a new ScheduleRequest instance using the specified properties.
             * @function create
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {scheduler.IScheduleRequest=} [properties] Properties to set
             * @returns {scheduler.ScheduleRequest} ScheduleRequest instance
             */
            ScheduleRequest.create = function create(properties) {
                return new ScheduleRequest(properties);
            };
    
            /**
             * Encodes the specified ScheduleRequest message. Does not implicitly {@link scheduler.ScheduleRequest.verify|verify} messages.
             * @function encode
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {scheduler.IScheduleRequest} message ScheduleRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ScheduleRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                return writer;
            };
    
            /**
             * Encodes the specified ScheduleRequest message, length delimited. Does not implicitly {@link scheduler.ScheduleRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {scheduler.IScheduleRequest} message ScheduleRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ScheduleRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ScheduleRequest message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.ScheduleRequest} ScheduleRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ScheduleRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.ScheduleRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ScheduleRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.ScheduleRequest} ScheduleRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ScheduleRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ScheduleRequest message.
             * @function verify
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ScheduleRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                return null;
            };
    
            /**
             * Creates a ScheduleRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.ScheduleRequest} ScheduleRequest
             */
            ScheduleRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.ScheduleRequest)
                    return object;
                var message = new $root.scheduler.ScheduleRequest();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                return message;
            };
    
            /**
             * Creates a plain object from a ScheduleRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.ScheduleRequest
             * @static
             * @param {scheduler.ScheduleRequest} message ScheduleRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ScheduleRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                return object;
            };
    
            /**
             * Converts this ScheduleRequest to JSON.
             * @function toJSON
             * @memberof scheduler.ScheduleRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ScheduleRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ScheduleRequest;
        })();
    
        scheduler.NodeInfoRequest = (function() {
    
            /**
             * Properties of a NodeInfoRequest.
             * @memberof scheduler
             * @interface INodeInfoRequest
             * @property {Uint8Array|null} [nodeId] NodeInfoRequest nodeId
             * @property {boolean|null} [wait] NodeInfoRequest wait
             */
    
            /**
             * Constructs a new NodeInfoRequest.
             * @memberof scheduler
             * @classdesc Represents a NodeInfoRequest.
             * @implements INodeInfoRequest
             * @constructor
             * @param {scheduler.INodeInfoRequest=} [properties] Properties to set
             */
            function NodeInfoRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * NodeInfoRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof scheduler.NodeInfoRequest
             * @instance
             */
            NodeInfoRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * NodeInfoRequest wait.
             * @member {boolean} wait
             * @memberof scheduler.NodeInfoRequest
             * @instance
             */
            NodeInfoRequest.prototype.wait = false;
    
            /**
             * Creates a new NodeInfoRequest instance using the specified properties.
             * @function create
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {scheduler.INodeInfoRequest=} [properties] Properties to set
             * @returns {scheduler.NodeInfoRequest} NodeInfoRequest instance
             */
            NodeInfoRequest.create = function create(properties) {
                return new NodeInfoRequest(properties);
            };
    
            /**
             * Encodes the specified NodeInfoRequest message. Does not implicitly {@link scheduler.NodeInfoRequest.verify|verify} messages.
             * @function encode
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {scheduler.INodeInfoRequest} message NodeInfoRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NodeInfoRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.wait != null && Object.hasOwnProperty.call(message, "wait"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.wait);
                return writer;
            };
    
            /**
             * Encodes the specified NodeInfoRequest message, length delimited. Does not implicitly {@link scheduler.NodeInfoRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {scheduler.INodeInfoRequest} message NodeInfoRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NodeInfoRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a NodeInfoRequest message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.NodeInfoRequest} NodeInfoRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NodeInfoRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.NodeInfoRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.wait = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a NodeInfoRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.NodeInfoRequest} NodeInfoRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NodeInfoRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a NodeInfoRequest message.
             * @function verify
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            NodeInfoRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.wait != null && message.hasOwnProperty("wait"))
                    if (typeof message.wait !== "boolean")
                        return "wait: boolean expected";
                return null;
            };
    
            /**
             * Creates a NodeInfoRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.NodeInfoRequest} NodeInfoRequest
             */
            NodeInfoRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.NodeInfoRequest)
                    return object;
                var message = new $root.scheduler.NodeInfoRequest();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.wait != null)
                    message.wait = Boolean(object.wait);
                return message;
            };
    
            /**
             * Creates a plain object from a NodeInfoRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.NodeInfoRequest
             * @static
             * @param {scheduler.NodeInfoRequest} message NodeInfoRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            NodeInfoRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.wait = false;
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.wait != null && message.hasOwnProperty("wait"))
                    object.wait = message.wait;
                return object;
            };
    
            /**
             * Converts this NodeInfoRequest to JSON.
             * @function toJSON
             * @memberof scheduler.NodeInfoRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            NodeInfoRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return NodeInfoRequest;
        })();
    
        scheduler.NodeInfoResponse = (function() {
    
            /**
             * Properties of a NodeInfoResponse.
             * @memberof scheduler
             * @interface INodeInfoResponse
             * @property {Uint8Array|null} [nodeId] NodeInfoResponse nodeId
             * @property {string|null} [grpcUri] NodeInfoResponse grpcUri
             */
    
            /**
             * Constructs a new NodeInfoResponse.
             * @memberof scheduler
             * @classdesc Represents a NodeInfoResponse.
             * @implements INodeInfoResponse
             * @constructor
             * @param {scheduler.INodeInfoResponse=} [properties] Properties to set
             */
            function NodeInfoResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * NodeInfoResponse nodeId.
             * @member {Uint8Array} nodeId
             * @memberof scheduler.NodeInfoResponse
             * @instance
             */
            NodeInfoResponse.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * NodeInfoResponse grpcUri.
             * @member {string} grpcUri
             * @memberof scheduler.NodeInfoResponse
             * @instance
             */
            NodeInfoResponse.prototype.grpcUri = "";
    
            /**
             * Creates a new NodeInfoResponse instance using the specified properties.
             * @function create
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {scheduler.INodeInfoResponse=} [properties] Properties to set
             * @returns {scheduler.NodeInfoResponse} NodeInfoResponse instance
             */
            NodeInfoResponse.create = function create(properties) {
                return new NodeInfoResponse(properties);
            };
    
            /**
             * Encodes the specified NodeInfoResponse message. Does not implicitly {@link scheduler.NodeInfoResponse.verify|verify} messages.
             * @function encode
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {scheduler.INodeInfoResponse} message NodeInfoResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NodeInfoResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.grpcUri != null && Object.hasOwnProperty.call(message, "grpcUri"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.grpcUri);
                return writer;
            };
    
            /**
             * Encodes the specified NodeInfoResponse message, length delimited. Does not implicitly {@link scheduler.NodeInfoResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {scheduler.INodeInfoResponse} message NodeInfoResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NodeInfoResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a NodeInfoResponse message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.NodeInfoResponse} NodeInfoResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NodeInfoResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.NodeInfoResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.grpcUri = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a NodeInfoResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.NodeInfoResponse} NodeInfoResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NodeInfoResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a NodeInfoResponse message.
             * @function verify
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            NodeInfoResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.grpcUri != null && message.hasOwnProperty("grpcUri"))
                    if (!$util.isString(message.grpcUri))
                        return "grpcUri: string expected";
                return null;
            };
    
            /**
             * Creates a NodeInfoResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.NodeInfoResponse} NodeInfoResponse
             */
            NodeInfoResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.NodeInfoResponse)
                    return object;
                var message = new $root.scheduler.NodeInfoResponse();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.grpcUri != null)
                    message.grpcUri = String(object.grpcUri);
                return message;
            };
    
            /**
             * Creates a plain object from a NodeInfoResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.NodeInfoResponse
             * @static
             * @param {scheduler.NodeInfoResponse} message NodeInfoResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            NodeInfoResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.grpcUri = "";
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.grpcUri != null && message.hasOwnProperty("grpcUri"))
                    object.grpcUri = message.grpcUri;
                return object;
            };
    
            /**
             * Converts this NodeInfoResponse to JSON.
             * @function toJSON
             * @memberof scheduler.NodeInfoResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            NodeInfoResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return NodeInfoResponse;
        })();
    
        scheduler.RecoveryRequest = (function() {
    
            /**
             * Properties of a RecoveryRequest.
             * @memberof scheduler
             * @interface IRecoveryRequest
             * @property {Uint8Array|null} [challenge] RecoveryRequest challenge
             * @property {Uint8Array|null} [signature] RecoveryRequest signature
             * @property {Uint8Array|null} [nodeId] RecoveryRequest nodeId
             */
    
            /**
             * Constructs a new RecoveryRequest.
             * @memberof scheduler
             * @classdesc Represents a RecoveryRequest.
             * @implements IRecoveryRequest
             * @constructor
             * @param {scheduler.IRecoveryRequest=} [properties] Properties to set
             */
            function RecoveryRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * RecoveryRequest challenge.
             * @member {Uint8Array} challenge
             * @memberof scheduler.RecoveryRequest
             * @instance
             */
            RecoveryRequest.prototype.challenge = $util.newBuffer([]);
    
            /**
             * RecoveryRequest signature.
             * @member {Uint8Array} signature
             * @memberof scheduler.RecoveryRequest
             * @instance
             */
            RecoveryRequest.prototype.signature = $util.newBuffer([]);
    
            /**
             * RecoveryRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof scheduler.RecoveryRequest
             * @instance
             */
            RecoveryRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * Creates a new RecoveryRequest instance using the specified properties.
             * @function create
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {scheduler.IRecoveryRequest=} [properties] Properties to set
             * @returns {scheduler.RecoveryRequest} RecoveryRequest instance
             */
            RecoveryRequest.create = function create(properties) {
                return new RecoveryRequest(properties);
            };
    
            /**
             * Encodes the specified RecoveryRequest message. Does not implicitly {@link scheduler.RecoveryRequest.verify|verify} messages.
             * @function encode
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {scheduler.IRecoveryRequest} message RecoveryRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RecoveryRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.challenge != null && Object.hasOwnProperty.call(message, "challenge"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.challenge);
                if (message.signature != null && Object.hasOwnProperty.call(message, "signature"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.signature);
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.nodeId);
                return writer;
            };
    
            /**
             * Encodes the specified RecoveryRequest message, length delimited. Does not implicitly {@link scheduler.RecoveryRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {scheduler.IRecoveryRequest} message RecoveryRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RecoveryRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a RecoveryRequest message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.RecoveryRequest} RecoveryRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RecoveryRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.RecoveryRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.challenge = reader.bytes();
                        break;
                    case 2:
                        message.signature = reader.bytes();
                        break;
                    case 3:
                        message.nodeId = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a RecoveryRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.RecoveryRequest} RecoveryRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RecoveryRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a RecoveryRequest message.
             * @function verify
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RecoveryRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.challenge != null && message.hasOwnProperty("challenge"))
                    if (!(message.challenge && typeof message.challenge.length === "number" || $util.isString(message.challenge)))
                        return "challenge: buffer expected";
                if (message.signature != null && message.hasOwnProperty("signature"))
                    if (!(message.signature && typeof message.signature.length === "number" || $util.isString(message.signature)))
                        return "signature: buffer expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                return null;
            };
    
            /**
             * Creates a RecoveryRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.RecoveryRequest} RecoveryRequest
             */
            RecoveryRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.RecoveryRequest)
                    return object;
                var message = new $root.scheduler.RecoveryRequest();
                if (object.challenge != null)
                    if (typeof object.challenge === "string")
                        $util.base64.decode(object.challenge, message.challenge = $util.newBuffer($util.base64.length(object.challenge)), 0);
                    else if (object.challenge.length)
                        message.challenge = object.challenge;
                if (object.signature != null)
                    if (typeof object.signature === "string")
                        $util.base64.decode(object.signature, message.signature = $util.newBuffer($util.base64.length(object.signature)), 0);
                    else if (object.signature.length)
                        message.signature = object.signature;
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                return message;
            };
    
            /**
             * Creates a plain object from a RecoveryRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.RecoveryRequest
             * @static
             * @param {scheduler.RecoveryRequest} message RecoveryRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RecoveryRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.challenge = "";
                    else {
                        object.challenge = [];
                        if (options.bytes !== Array)
                            object.challenge = $util.newBuffer(object.challenge);
                    }
                    if (options.bytes === String)
                        object.signature = "";
                    else {
                        object.signature = [];
                        if (options.bytes !== Array)
                            object.signature = $util.newBuffer(object.signature);
                    }
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                }
                if (message.challenge != null && message.hasOwnProperty("challenge"))
                    object.challenge = options.bytes === String ? $util.base64.encode(message.challenge, 0, message.challenge.length) : options.bytes === Array ? Array.prototype.slice.call(message.challenge) : message.challenge;
                if (message.signature != null && message.hasOwnProperty("signature"))
                    object.signature = options.bytes === String ? $util.base64.encode(message.signature, 0, message.signature.length) : options.bytes === Array ? Array.prototype.slice.call(message.signature) : message.signature;
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                return object;
            };
    
            /**
             * Converts this RecoveryRequest to JSON.
             * @function toJSON
             * @memberof scheduler.RecoveryRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RecoveryRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return RecoveryRequest;
        })();
    
        scheduler.RecoveryResponse = (function() {
    
            /**
             * Properties of a RecoveryResponse.
             * @memberof scheduler
             * @interface IRecoveryResponse
             * @property {string|null} [deviceCert] RecoveryResponse deviceCert
             * @property {string|null} [deviceKey] RecoveryResponse deviceKey
             */
    
            /**
             * Constructs a new RecoveryResponse.
             * @memberof scheduler
             * @classdesc Represents a RecoveryResponse.
             * @implements IRecoveryResponse
             * @constructor
             * @param {scheduler.IRecoveryResponse=} [properties] Properties to set
             */
            function RecoveryResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * RecoveryResponse deviceCert.
             * @member {string} deviceCert
             * @memberof scheduler.RecoveryResponse
             * @instance
             */
            RecoveryResponse.prototype.deviceCert = "";
    
            /**
             * RecoveryResponse deviceKey.
             * @member {string} deviceKey
             * @memberof scheduler.RecoveryResponse
             * @instance
             */
            RecoveryResponse.prototype.deviceKey = "";
    
            /**
             * Creates a new RecoveryResponse instance using the specified properties.
             * @function create
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {scheduler.IRecoveryResponse=} [properties] Properties to set
             * @returns {scheduler.RecoveryResponse} RecoveryResponse instance
             */
            RecoveryResponse.create = function create(properties) {
                return new RecoveryResponse(properties);
            };
    
            /**
             * Encodes the specified RecoveryResponse message. Does not implicitly {@link scheduler.RecoveryResponse.verify|verify} messages.
             * @function encode
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {scheduler.IRecoveryResponse} message RecoveryResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RecoveryResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.deviceCert != null && Object.hasOwnProperty.call(message, "deviceCert"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.deviceCert);
                if (message.deviceKey != null && Object.hasOwnProperty.call(message, "deviceKey"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.deviceKey);
                return writer;
            };
    
            /**
             * Encodes the specified RecoveryResponse message, length delimited. Does not implicitly {@link scheduler.RecoveryResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {scheduler.IRecoveryResponse} message RecoveryResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RecoveryResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a RecoveryResponse message from the specified reader or buffer.
             * @function decode
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {scheduler.RecoveryResponse} RecoveryResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RecoveryResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.scheduler.RecoveryResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.deviceCert = reader.string();
                        break;
                    case 2:
                        message.deviceKey = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a RecoveryResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {scheduler.RecoveryResponse} RecoveryResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RecoveryResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a RecoveryResponse message.
             * @function verify
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RecoveryResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.deviceCert != null && message.hasOwnProperty("deviceCert"))
                    if (!$util.isString(message.deviceCert))
                        return "deviceCert: string expected";
                if (message.deviceKey != null && message.hasOwnProperty("deviceKey"))
                    if (!$util.isString(message.deviceKey))
                        return "deviceKey: string expected";
                return null;
            };
    
            /**
             * Creates a RecoveryResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {scheduler.RecoveryResponse} RecoveryResponse
             */
            RecoveryResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.scheduler.RecoveryResponse)
                    return object;
                var message = new $root.scheduler.RecoveryResponse();
                if (object.deviceCert != null)
                    message.deviceCert = String(object.deviceCert);
                if (object.deviceKey != null)
                    message.deviceKey = String(object.deviceKey);
                return message;
            };
    
            /**
             * Creates a plain object from a RecoveryResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof scheduler.RecoveryResponse
             * @static
             * @param {scheduler.RecoveryResponse} message RecoveryResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RecoveryResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.deviceCert = "";
                    object.deviceKey = "";
                }
                if (message.deviceCert != null && message.hasOwnProperty("deviceCert"))
                    object.deviceCert = message.deviceCert;
                if (message.deviceKey != null && message.hasOwnProperty("deviceKey"))
                    object.deviceKey = message.deviceKey;
                return object;
            };
    
            /**
             * Converts this RecoveryResponse to JSON.
             * @function toJSON
             * @memberof scheduler.RecoveryResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RecoveryResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return RecoveryResponse;
        })();
    
        return scheduler;
    })();
    
    $root.greenlight = (function() {
    
        /**
         * Namespace greenlight.
         * @exports greenlight
         * @namespace
         */
        var greenlight = {};
    
        greenlight.Node = (function() {
    
            /**
             * Constructs a new Node service.
             * @memberof greenlight
             * @classdesc Represents a Node
             * @extends $protobuf.rpc.Service
             * @constructor
             * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
             * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
             * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
             */
            function Node(rpcImpl, requestDelimited, responseDelimited) {
                $protobuf.rpc.Service.call(this, rpcImpl, requestDelimited, responseDelimited);
            }
    
            (Node.prototype = Object.create($protobuf.rpc.Service.prototype)).constructor = Node;
    
            /**
             * Creates new Node service using the specified rpc implementation.
             * @function create
             * @memberof greenlight.Node
             * @static
             * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
             * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
             * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
             * @returns {Node} RPC service. Useful where requests and/or responses are streamed.
             */
            Node.create = function create(rpcImpl, requestDelimited, responseDelimited) {
                return new this(rpcImpl, requestDelimited, responseDelimited);
            };
    
            /**
             * Callback as used by {@link greenlight.Node#getInfo}.
             * @memberof greenlight.Node
             * @typedef GetInfoCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.GetInfoResponse} [response] GetInfoResponse
             */
    
            /**
             * Calls GetInfo.
             * @function getInfo
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IGetInfoRequest} request GetInfoRequest message or plain object
             * @param {greenlight.Node.GetInfoCallback} callback Node-style callback called with the error, if any, and GetInfoResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.getInfo = function getInfo(request, callback) {
                return this.rpcCall(getInfo, $root.greenlight.GetInfoRequest, $root.greenlight.GetInfoResponse, request, callback);
            }, "name", { value: "GetInfo" });
    
            /**
             * Calls GetInfo.
             * @function getInfo
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IGetInfoRequest} request GetInfoRequest message or plain object
             * @returns {Promise<greenlight.GetInfoResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#stop}.
             * @memberof greenlight.Node
             * @typedef StopCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.StopResponse} [response] StopResponse
             */
    
            /**
             * Calls Stop.
             * @function stop
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IStopRequest} request StopRequest message or plain object
             * @param {greenlight.Node.StopCallback} callback Node-style callback called with the error, if any, and StopResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.stop = function stop(request, callback) {
                return this.rpcCall(stop, $root.greenlight.StopRequest, $root.greenlight.StopResponse, request, callback);
            }, "name", { value: "Stop" });
    
            /**
             * Calls Stop.
             * @function stop
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IStopRequest} request StopRequest message or plain object
             * @returns {Promise<greenlight.StopResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#connectPeer}.
             * @memberof greenlight.Node
             * @typedef ConnectPeerCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.ConnectResponse} [response] ConnectResponse
             */
    
            /**
             * Calls ConnectPeer.
             * @function connectPeer
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IConnectRequest} request ConnectRequest message or plain object
             * @param {greenlight.Node.ConnectPeerCallback} callback Node-style callback called with the error, if any, and ConnectResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.connectPeer = function connectPeer(request, callback) {
                return this.rpcCall(connectPeer, $root.greenlight.ConnectRequest, $root.greenlight.ConnectResponse, request, callback);
            }, "name", { value: "ConnectPeer" });
    
            /**
             * Calls ConnectPeer.
             * @function connectPeer
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IConnectRequest} request ConnectRequest message or plain object
             * @returns {Promise<greenlight.ConnectResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#listPeers}.
             * @memberof greenlight.Node
             * @typedef ListPeersCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.ListPeersResponse} [response] ListPeersResponse
             */
    
            /**
             * Calls ListPeers.
             * @function listPeers
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListPeersRequest} request ListPeersRequest message or plain object
             * @param {greenlight.Node.ListPeersCallback} callback Node-style callback called with the error, if any, and ListPeersResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.listPeers = function listPeers(request, callback) {
                return this.rpcCall(listPeers, $root.greenlight.ListPeersRequest, $root.greenlight.ListPeersResponse, request, callback);
            }, "name", { value: "ListPeers" });
    
            /**
             * Calls ListPeers.
             * @function listPeers
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListPeersRequest} request ListPeersRequest message or plain object
             * @returns {Promise<greenlight.ListPeersResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#disconnect}.
             * @memberof greenlight.Node
             * @typedef DisconnectCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.DisconnectResponse} [response] DisconnectResponse
             */
    
            /**
             * Calls Disconnect.
             * @function disconnect
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IDisconnectRequest} request DisconnectRequest message or plain object
             * @param {greenlight.Node.DisconnectCallback} callback Node-style callback called with the error, if any, and DisconnectResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.disconnect = function disconnect(request, callback) {
                return this.rpcCall(disconnect, $root.greenlight.DisconnectRequest, $root.greenlight.DisconnectResponse, request, callback);
            }, "name", { value: "Disconnect" });
    
            /**
             * Calls Disconnect.
             * @function disconnect
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IDisconnectRequest} request DisconnectRequest message or plain object
             * @returns {Promise<greenlight.DisconnectResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#newAddr}.
             * @memberof greenlight.Node
             * @typedef NewAddrCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.NewAddrResponse} [response] NewAddrResponse
             */
    
            /**
             * Calls NewAddr.
             * @function newAddr
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.INewAddrRequest} request NewAddrRequest message or plain object
             * @param {greenlight.Node.NewAddrCallback} callback Node-style callback called with the error, if any, and NewAddrResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.newAddr = function newAddr(request, callback) {
                return this.rpcCall(newAddr, $root.greenlight.NewAddrRequest, $root.greenlight.NewAddrResponse, request, callback);
            }, "name", { value: "NewAddr" });
    
            /**
             * Calls NewAddr.
             * @function newAddr
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.INewAddrRequest} request NewAddrRequest message or plain object
             * @returns {Promise<greenlight.NewAddrResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#listFunds}.
             * @memberof greenlight.Node
             * @typedef ListFundsCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.ListFundsResponse} [response] ListFundsResponse
             */
    
            /**
             * Calls ListFunds.
             * @function listFunds
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListFundsRequest} request ListFundsRequest message or plain object
             * @param {greenlight.Node.ListFundsCallback} callback Node-style callback called with the error, if any, and ListFundsResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.listFunds = function listFunds(request, callback) {
                return this.rpcCall(listFunds, $root.greenlight.ListFundsRequest, $root.greenlight.ListFundsResponse, request, callback);
            }, "name", { value: "ListFunds" });
    
            /**
             * Calls ListFunds.
             * @function listFunds
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListFundsRequest} request ListFundsRequest message or plain object
             * @returns {Promise<greenlight.ListFundsResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#withdraw}.
             * @memberof greenlight.Node
             * @typedef WithdrawCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.WithdrawResponse} [response] WithdrawResponse
             */
    
            /**
             * Calls Withdraw.
             * @function withdraw
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IWithdrawRequest} request WithdrawRequest message or plain object
             * @param {greenlight.Node.WithdrawCallback} callback Node-style callback called with the error, if any, and WithdrawResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.withdraw = function withdraw(request, callback) {
                return this.rpcCall(withdraw, $root.greenlight.WithdrawRequest, $root.greenlight.WithdrawResponse, request, callback);
            }, "name", { value: "Withdraw" });
    
            /**
             * Calls Withdraw.
             * @function withdraw
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IWithdrawRequest} request WithdrawRequest message or plain object
             * @returns {Promise<greenlight.WithdrawResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#fundChannel}.
             * @memberof greenlight.Node
             * @typedef FundChannelCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.FundChannelResponse} [response] FundChannelResponse
             */
    
            /**
             * Calls FundChannel.
             * @function fundChannel
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IFundChannelRequest} request FundChannelRequest message or plain object
             * @param {greenlight.Node.FundChannelCallback} callback Node-style callback called with the error, if any, and FundChannelResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.fundChannel = function fundChannel(request, callback) {
                return this.rpcCall(fundChannel, $root.greenlight.FundChannelRequest, $root.greenlight.FundChannelResponse, request, callback);
            }, "name", { value: "FundChannel" });
    
            /**
             * Calls FundChannel.
             * @function fundChannel
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IFundChannelRequest} request FundChannelRequest message or plain object
             * @returns {Promise<greenlight.FundChannelResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#closeChannel}.
             * @memberof greenlight.Node
             * @typedef CloseChannelCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.CloseChannelResponse} [response] CloseChannelResponse
             */
    
            /**
             * Calls CloseChannel.
             * @function closeChannel
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.ICloseChannelRequest} request CloseChannelRequest message or plain object
             * @param {greenlight.Node.CloseChannelCallback} callback Node-style callback called with the error, if any, and CloseChannelResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.closeChannel = function closeChannel(request, callback) {
                return this.rpcCall(closeChannel, $root.greenlight.CloseChannelRequest, $root.greenlight.CloseChannelResponse, request, callback);
            }, "name", { value: "CloseChannel" });
    
            /**
             * Calls CloseChannel.
             * @function closeChannel
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.ICloseChannelRequest} request CloseChannelRequest message or plain object
             * @returns {Promise<greenlight.CloseChannelResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#createInvoice}.
             * @memberof greenlight.Node
             * @typedef CreateInvoiceCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.Invoice} [response] Invoice
             */
    
            /**
             * Calls CreateInvoice.
             * @function createInvoice
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IInvoiceRequest} request InvoiceRequest message or plain object
             * @param {greenlight.Node.CreateInvoiceCallback} callback Node-style callback called with the error, if any, and Invoice
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.createInvoice = function createInvoice(request, callback) {
                return this.rpcCall(createInvoice, $root.greenlight.InvoiceRequest, $root.greenlight.Invoice, request, callback);
            }, "name", { value: "CreateInvoice" });
    
            /**
             * Calls CreateInvoice.
             * @function createInvoice
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IInvoiceRequest} request InvoiceRequest message or plain object
             * @returns {Promise<greenlight.Invoice>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#pay}.
             * @memberof greenlight.Node
             * @typedef PayCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.Payment} [response] Payment
             */
    
            /**
             * Calls Pay.
             * @function pay
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IPayRequest} request PayRequest message or plain object
             * @param {greenlight.Node.PayCallback} callback Node-style callback called with the error, if any, and Payment
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.pay = function pay(request, callback) {
                return this.rpcCall(pay, $root.greenlight.PayRequest, $root.greenlight.Payment, request, callback);
            }, "name", { value: "Pay" });
    
            /**
             * Calls Pay.
             * @function pay
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IPayRequest} request PayRequest message or plain object
             * @returns {Promise<greenlight.Payment>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#keysend}.
             * @memberof greenlight.Node
             * @typedef KeysendCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.Payment} [response] Payment
             */
    
            /**
             * Calls Keysend.
             * @function keysend
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IKeysendRequest} request KeysendRequest message or plain object
             * @param {greenlight.Node.KeysendCallback} callback Node-style callback called with the error, if any, and Payment
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.keysend = function keysend(request, callback) {
                return this.rpcCall(keysend, $root.greenlight.KeysendRequest, $root.greenlight.Payment, request, callback);
            }, "name", { value: "Keysend" });
    
            /**
             * Calls Keysend.
             * @function keysend
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IKeysendRequest} request KeysendRequest message or plain object
             * @returns {Promise<greenlight.Payment>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#listPayments}.
             * @memberof greenlight.Node
             * @typedef ListPaymentsCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.ListPaymentsResponse} [response] ListPaymentsResponse
             */
    
            /**
             * Calls ListPayments.
             * @function listPayments
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListPaymentsRequest} request ListPaymentsRequest message or plain object
             * @param {greenlight.Node.ListPaymentsCallback} callback Node-style callback called with the error, if any, and ListPaymentsResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.listPayments = function listPayments(request, callback) {
                return this.rpcCall(listPayments, $root.greenlight.ListPaymentsRequest, $root.greenlight.ListPaymentsResponse, request, callback);
            }, "name", { value: "ListPayments" });
    
            /**
             * Calls ListPayments.
             * @function listPayments
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListPaymentsRequest} request ListPaymentsRequest message or plain object
             * @returns {Promise<greenlight.ListPaymentsResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#listInvoices}.
             * @memberof greenlight.Node
             * @typedef ListInvoicesCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.ListInvoicesResponse} [response] ListInvoicesResponse
             */
    
            /**
             * Calls ListInvoices.
             * @function listInvoices
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListInvoicesRequest} request ListInvoicesRequest message or plain object
             * @param {greenlight.Node.ListInvoicesCallback} callback Node-style callback called with the error, if any, and ListInvoicesResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.listInvoices = function listInvoices(request, callback) {
                return this.rpcCall(listInvoices, $root.greenlight.ListInvoicesRequest, $root.greenlight.ListInvoicesResponse, request, callback);
            }, "name", { value: "ListInvoices" });
    
            /**
             * Calls ListInvoices.
             * @function listInvoices
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IListInvoicesRequest} request ListInvoicesRequest message or plain object
             * @returns {Promise<greenlight.ListInvoicesResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#streamIncoming}.
             * @memberof greenlight.Node
             * @typedef StreamIncomingCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.IncomingPayment} [response] IncomingPayment
             */
    
            /**
             * Calls StreamIncoming.
             * @function streamIncoming
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IStreamIncomingFilter} request StreamIncomingFilter message or plain object
             * @param {greenlight.Node.StreamIncomingCallback} callback Node-style callback called with the error, if any, and IncomingPayment
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.streamIncoming = function streamIncoming(request, callback) {
                return this.rpcCall(streamIncoming, $root.greenlight.StreamIncomingFilter, $root.greenlight.IncomingPayment, request, callback);
            }, "name", { value: "StreamIncoming" });
    
            /**
             * Calls StreamIncoming.
             * @function streamIncoming
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IStreamIncomingFilter} request StreamIncomingFilter message or plain object
             * @returns {Promise<greenlight.IncomingPayment>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#streamLog}.
             * @memberof greenlight.Node
             * @typedef StreamLogCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.LogEntry} [response] LogEntry
             */
    
            /**
             * Calls StreamLog.
             * @function streamLog
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IStreamLogRequest} request StreamLogRequest message or plain object
             * @param {greenlight.Node.StreamLogCallback} callback Node-style callback called with the error, if any, and LogEntry
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.streamLog = function streamLog(request, callback) {
                return this.rpcCall(streamLog, $root.greenlight.StreamLogRequest, $root.greenlight.LogEntry, request, callback);
            }, "name", { value: "StreamLog" });
    
            /**
             * Calls StreamLog.
             * @function streamLog
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IStreamLogRequest} request StreamLogRequest message or plain object
             * @returns {Promise<greenlight.LogEntry>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#streamHsmRequests}.
             * @memberof greenlight.Node
             * @typedef StreamHsmRequestsCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.HsmRequest} [response] HsmRequest
             */
    
            /**
             * Calls StreamHsmRequests.
             * @function streamHsmRequests
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IEmpty} request Empty message or plain object
             * @param {greenlight.Node.StreamHsmRequestsCallback} callback Node-style callback called with the error, if any, and HsmRequest
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.streamHsmRequests = function streamHsmRequests(request, callback) {
                return this.rpcCall(streamHsmRequests, $root.greenlight.Empty, $root.greenlight.HsmRequest, request, callback);
            }, "name", { value: "StreamHsmRequests" });
    
            /**
             * Calls StreamHsmRequests.
             * @function streamHsmRequests
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IEmpty} request Empty message or plain object
             * @returns {Promise<greenlight.HsmRequest>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Node#respondHsmRequest}.
             * @memberof greenlight.Node
             * @typedef RespondHsmRequestCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.Empty} [response] Empty
             */
    
            /**
             * Calls RespondHsmRequest.
             * @function respondHsmRequest
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IHsmResponse} request HsmResponse message or plain object
             * @param {greenlight.Node.RespondHsmRequestCallback} callback Node-style callback called with the error, if any, and Empty
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Node.prototype.respondHsmRequest = function respondHsmRequest(request, callback) {
                return this.rpcCall(respondHsmRequest, $root.greenlight.HsmResponse, $root.greenlight.Empty, request, callback);
            }, "name", { value: "RespondHsmRequest" });
    
            /**
             * Calls RespondHsmRequest.
             * @function respondHsmRequest
             * @memberof greenlight.Node
             * @instance
             * @param {greenlight.IHsmResponse} request HsmResponse message or plain object
             * @returns {Promise<greenlight.Empty>} Promise
             * @variation 2
             */
    
            return Node;
        })();
    
        greenlight.HsmRequestContext = (function() {
    
            /**
             * Properties of a HsmRequestContext.
             * @memberof greenlight
             * @interface IHsmRequestContext
             * @property {Uint8Array|null} [nodeId] HsmRequestContext nodeId
             * @property {number|Long|null} [dbid] HsmRequestContext dbid
             * @property {number|Long|null} [capabilities] HsmRequestContext capabilities
             */
    
            /**
             * Constructs a new HsmRequestContext.
             * @memberof greenlight
             * @classdesc Represents a HsmRequestContext.
             * @implements IHsmRequestContext
             * @constructor
             * @param {greenlight.IHsmRequestContext=} [properties] Properties to set
             */
            function HsmRequestContext(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * HsmRequestContext nodeId.
             * @member {Uint8Array} nodeId
             * @memberof greenlight.HsmRequestContext
             * @instance
             */
            HsmRequestContext.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * HsmRequestContext dbid.
             * @member {number|Long} dbid
             * @memberof greenlight.HsmRequestContext
             * @instance
             */
            HsmRequestContext.prototype.dbid = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * HsmRequestContext capabilities.
             * @member {number|Long} capabilities
             * @memberof greenlight.HsmRequestContext
             * @instance
             */
            HsmRequestContext.prototype.capabilities = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * Creates a new HsmRequestContext instance using the specified properties.
             * @function create
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {greenlight.IHsmRequestContext=} [properties] Properties to set
             * @returns {greenlight.HsmRequestContext} HsmRequestContext instance
             */
            HsmRequestContext.create = function create(properties) {
                return new HsmRequestContext(properties);
            };
    
            /**
             * Encodes the specified HsmRequestContext message. Does not implicitly {@link greenlight.HsmRequestContext.verify|verify} messages.
             * @function encode
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {greenlight.IHsmRequestContext} message HsmRequestContext message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            HsmRequestContext.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.dbid != null && Object.hasOwnProperty.call(message, "dbid"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.dbid);
                if (message.capabilities != null && Object.hasOwnProperty.call(message, "capabilities"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.capabilities);
                return writer;
            };
    
            /**
             * Encodes the specified HsmRequestContext message, length delimited. Does not implicitly {@link greenlight.HsmRequestContext.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {greenlight.IHsmRequestContext} message HsmRequestContext message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            HsmRequestContext.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a HsmRequestContext message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.HsmRequestContext} HsmRequestContext
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            HsmRequestContext.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.HsmRequestContext();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.dbid = reader.uint64();
                        break;
                    case 3:
                        message.capabilities = reader.uint64();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a HsmRequestContext message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.HsmRequestContext} HsmRequestContext
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            HsmRequestContext.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a HsmRequestContext message.
             * @function verify
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            HsmRequestContext.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.dbid != null && message.hasOwnProperty("dbid"))
                    if (!$util.isInteger(message.dbid) && !(message.dbid && $util.isInteger(message.dbid.low) && $util.isInteger(message.dbid.high)))
                        return "dbid: integer|Long expected";
                if (message.capabilities != null && message.hasOwnProperty("capabilities"))
                    if (!$util.isInteger(message.capabilities) && !(message.capabilities && $util.isInteger(message.capabilities.low) && $util.isInteger(message.capabilities.high)))
                        return "capabilities: integer|Long expected";
                return null;
            };
    
            /**
             * Creates a HsmRequestContext message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.HsmRequestContext} HsmRequestContext
             */
            HsmRequestContext.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.HsmRequestContext)
                    return object;
                var message = new $root.greenlight.HsmRequestContext();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.dbid != null)
                    if ($util.Long)
                        (message.dbid = $util.Long.fromValue(object.dbid)).unsigned = true;
                    else if (typeof object.dbid === "string")
                        message.dbid = parseInt(object.dbid, 10);
                    else if (typeof object.dbid === "number")
                        message.dbid = object.dbid;
                    else if (typeof object.dbid === "object")
                        message.dbid = new $util.LongBits(object.dbid.low >>> 0, object.dbid.high >>> 0).toNumber(true);
                if (object.capabilities != null)
                    if ($util.Long)
                        (message.capabilities = $util.Long.fromValue(object.capabilities)).unsigned = true;
                    else if (typeof object.capabilities === "string")
                        message.capabilities = parseInt(object.capabilities, 10);
                    else if (typeof object.capabilities === "number")
                        message.capabilities = object.capabilities;
                    else if (typeof object.capabilities === "object")
                        message.capabilities = new $util.LongBits(object.capabilities.low >>> 0, object.capabilities.high >>> 0).toNumber(true);
                return message;
            };
    
            /**
             * Creates a plain object from a HsmRequestContext message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.HsmRequestContext
             * @static
             * @param {greenlight.HsmRequestContext} message HsmRequestContext
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            HsmRequestContext.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.dbid = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.dbid = options.longs === String ? "0" : 0;
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.capabilities = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.capabilities = options.longs === String ? "0" : 0;
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.dbid != null && message.hasOwnProperty("dbid"))
                    if (typeof message.dbid === "number")
                        object.dbid = options.longs === String ? String(message.dbid) : message.dbid;
                    else
                        object.dbid = options.longs === String ? $util.Long.prototype.toString.call(message.dbid) : options.longs === Number ? new $util.LongBits(message.dbid.low >>> 0, message.dbid.high >>> 0).toNumber(true) : message.dbid;
                if (message.capabilities != null && message.hasOwnProperty("capabilities"))
                    if (typeof message.capabilities === "number")
                        object.capabilities = options.longs === String ? String(message.capabilities) : message.capabilities;
                    else
                        object.capabilities = options.longs === String ? $util.Long.prototype.toString.call(message.capabilities) : options.longs === Number ? new $util.LongBits(message.capabilities.low >>> 0, message.capabilities.high >>> 0).toNumber(true) : message.capabilities;
                return object;
            };
    
            /**
             * Converts this HsmRequestContext to JSON.
             * @function toJSON
             * @memberof greenlight.HsmRequestContext
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            HsmRequestContext.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return HsmRequestContext;
        })();
    
        greenlight.HsmResponse = (function() {
    
            /**
             * Properties of a HsmResponse.
             * @memberof greenlight
             * @interface IHsmResponse
             * @property {number|null} [requestId] HsmResponse requestId
             * @property {Uint8Array|null} [raw] HsmResponse raw
             */
    
            /**
             * Constructs a new HsmResponse.
             * @memberof greenlight
             * @classdesc Represents a HsmResponse.
             * @implements IHsmResponse
             * @constructor
             * @param {greenlight.IHsmResponse=} [properties] Properties to set
             */
            function HsmResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * HsmResponse requestId.
             * @member {number} requestId
             * @memberof greenlight.HsmResponse
             * @instance
             */
            HsmResponse.prototype.requestId = 0;
    
            /**
             * HsmResponse raw.
             * @member {Uint8Array} raw
             * @memberof greenlight.HsmResponse
             * @instance
             */
            HsmResponse.prototype.raw = $util.newBuffer([]);
    
            /**
             * Creates a new HsmResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.HsmResponse
             * @static
             * @param {greenlight.IHsmResponse=} [properties] Properties to set
             * @returns {greenlight.HsmResponse} HsmResponse instance
             */
            HsmResponse.create = function create(properties) {
                return new HsmResponse(properties);
            };
    
            /**
             * Encodes the specified HsmResponse message. Does not implicitly {@link greenlight.HsmResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.HsmResponse
             * @static
             * @param {greenlight.IHsmResponse} message HsmResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            HsmResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.requestId != null && Object.hasOwnProperty.call(message, "requestId"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.requestId);
                if (message.raw != null && Object.hasOwnProperty.call(message, "raw"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.raw);
                return writer;
            };
    
            /**
             * Encodes the specified HsmResponse message, length delimited. Does not implicitly {@link greenlight.HsmResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.HsmResponse
             * @static
             * @param {greenlight.IHsmResponse} message HsmResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            HsmResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a HsmResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.HsmResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.HsmResponse} HsmResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            HsmResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.HsmResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.requestId = reader.uint32();
                        break;
                    case 2:
                        message.raw = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a HsmResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.HsmResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.HsmResponse} HsmResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            HsmResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a HsmResponse message.
             * @function verify
             * @memberof greenlight.HsmResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            HsmResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.requestId != null && message.hasOwnProperty("requestId"))
                    if (!$util.isInteger(message.requestId))
                        return "requestId: integer expected";
                if (message.raw != null && message.hasOwnProperty("raw"))
                    if (!(message.raw && typeof message.raw.length === "number" || $util.isString(message.raw)))
                        return "raw: buffer expected";
                return null;
            };
    
            /**
             * Creates a HsmResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.HsmResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.HsmResponse} HsmResponse
             */
            HsmResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.HsmResponse)
                    return object;
                var message = new $root.greenlight.HsmResponse();
                if (object.requestId != null)
                    message.requestId = object.requestId >>> 0;
                if (object.raw != null)
                    if (typeof object.raw === "string")
                        $util.base64.decode(object.raw, message.raw = $util.newBuffer($util.base64.length(object.raw)), 0);
                    else if (object.raw.length)
                        message.raw = object.raw;
                return message;
            };
    
            /**
             * Creates a plain object from a HsmResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.HsmResponse
             * @static
             * @param {greenlight.HsmResponse} message HsmResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            HsmResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.requestId = 0;
                    if (options.bytes === String)
                        object.raw = "";
                    else {
                        object.raw = [];
                        if (options.bytes !== Array)
                            object.raw = $util.newBuffer(object.raw);
                    }
                }
                if (message.requestId != null && message.hasOwnProperty("requestId"))
                    object.requestId = message.requestId;
                if (message.raw != null && message.hasOwnProperty("raw"))
                    object.raw = options.bytes === String ? $util.base64.encode(message.raw, 0, message.raw.length) : options.bytes === Array ? Array.prototype.slice.call(message.raw) : message.raw;
                return object;
            };
    
            /**
             * Converts this HsmResponse to JSON.
             * @function toJSON
             * @memberof greenlight.HsmResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            HsmResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return HsmResponse;
        })();
    
        greenlight.HsmRequest = (function() {
    
            /**
             * Properties of a HsmRequest.
             * @memberof greenlight
             * @interface IHsmRequest
             * @property {number|null} [requestId] HsmRequest requestId
             * @property {greenlight.IHsmRequestContext|null} [context] HsmRequest context
             * @property {Uint8Array|null} [raw] HsmRequest raw
             */
    
            /**
             * Constructs a new HsmRequest.
             * @memberof greenlight
             * @classdesc Represents a HsmRequest.
             * @implements IHsmRequest
             * @constructor
             * @param {greenlight.IHsmRequest=} [properties] Properties to set
             */
            function HsmRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * HsmRequest requestId.
             * @member {number} requestId
             * @memberof greenlight.HsmRequest
             * @instance
             */
            HsmRequest.prototype.requestId = 0;
    
            /**
             * HsmRequest context.
             * @member {greenlight.IHsmRequestContext|null|undefined} context
             * @memberof greenlight.HsmRequest
             * @instance
             */
            HsmRequest.prototype.context = null;
    
            /**
             * HsmRequest raw.
             * @member {Uint8Array} raw
             * @memberof greenlight.HsmRequest
             * @instance
             */
            HsmRequest.prototype.raw = $util.newBuffer([]);
    
            /**
             * Creates a new HsmRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.HsmRequest
             * @static
             * @param {greenlight.IHsmRequest=} [properties] Properties to set
             * @returns {greenlight.HsmRequest} HsmRequest instance
             */
            HsmRequest.create = function create(properties) {
                return new HsmRequest(properties);
            };
    
            /**
             * Encodes the specified HsmRequest message. Does not implicitly {@link greenlight.HsmRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.HsmRequest
             * @static
             * @param {greenlight.IHsmRequest} message HsmRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            HsmRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.requestId != null && Object.hasOwnProperty.call(message, "requestId"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.requestId);
                if (message.context != null && Object.hasOwnProperty.call(message, "context"))
                    $root.greenlight.HsmRequestContext.encode(message.context, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.raw != null && Object.hasOwnProperty.call(message, "raw"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.raw);
                return writer;
            };
    
            /**
             * Encodes the specified HsmRequest message, length delimited. Does not implicitly {@link greenlight.HsmRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.HsmRequest
             * @static
             * @param {greenlight.IHsmRequest} message HsmRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            HsmRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a HsmRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.HsmRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.HsmRequest} HsmRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            HsmRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.HsmRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.requestId = reader.uint32();
                        break;
                    case 2:
                        message.context = $root.greenlight.HsmRequestContext.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.raw = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a HsmRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.HsmRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.HsmRequest} HsmRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            HsmRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a HsmRequest message.
             * @function verify
             * @memberof greenlight.HsmRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            HsmRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.requestId != null && message.hasOwnProperty("requestId"))
                    if (!$util.isInteger(message.requestId))
                        return "requestId: integer expected";
                if (message.context != null && message.hasOwnProperty("context")) {
                    var error = $root.greenlight.HsmRequestContext.verify(message.context);
                    if (error)
                        return "context." + error;
                }
                if (message.raw != null && message.hasOwnProperty("raw"))
                    if (!(message.raw && typeof message.raw.length === "number" || $util.isString(message.raw)))
                        return "raw: buffer expected";
                return null;
            };
    
            /**
             * Creates a HsmRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.HsmRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.HsmRequest} HsmRequest
             */
            HsmRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.HsmRequest)
                    return object;
                var message = new $root.greenlight.HsmRequest();
                if (object.requestId != null)
                    message.requestId = object.requestId >>> 0;
                if (object.context != null) {
                    if (typeof object.context !== "object")
                        throw TypeError(".greenlight.HsmRequest.context: object expected");
                    message.context = $root.greenlight.HsmRequestContext.fromObject(object.context);
                }
                if (object.raw != null)
                    if (typeof object.raw === "string")
                        $util.base64.decode(object.raw, message.raw = $util.newBuffer($util.base64.length(object.raw)), 0);
                    else if (object.raw.length)
                        message.raw = object.raw;
                return message;
            };
    
            /**
             * Creates a plain object from a HsmRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.HsmRequest
             * @static
             * @param {greenlight.HsmRequest} message HsmRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            HsmRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.requestId = 0;
                    object.context = null;
                    if (options.bytes === String)
                        object.raw = "";
                    else {
                        object.raw = [];
                        if (options.bytes !== Array)
                            object.raw = $util.newBuffer(object.raw);
                    }
                }
                if (message.requestId != null && message.hasOwnProperty("requestId"))
                    object.requestId = message.requestId;
                if (message.context != null && message.hasOwnProperty("context"))
                    object.context = $root.greenlight.HsmRequestContext.toObject(message.context, options);
                if (message.raw != null && message.hasOwnProperty("raw"))
                    object.raw = options.bytes === String ? $util.base64.encode(message.raw, 0, message.raw.length) : options.bytes === Array ? Array.prototype.slice.call(message.raw) : message.raw;
                return object;
            };
    
            /**
             * Converts this HsmRequest to JSON.
             * @function toJSON
             * @memberof greenlight.HsmRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            HsmRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return HsmRequest;
        })();
    
        greenlight.Empty = (function() {
    
            /**
             * Properties of an Empty.
             * @memberof greenlight
             * @interface IEmpty
             */
    
            /**
             * Constructs a new Empty.
             * @memberof greenlight
             * @classdesc Represents an Empty.
             * @implements IEmpty
             * @constructor
             * @param {greenlight.IEmpty=} [properties] Properties to set
             */
            function Empty(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new Empty instance using the specified properties.
             * @function create
             * @memberof greenlight.Empty
             * @static
             * @param {greenlight.IEmpty=} [properties] Properties to set
             * @returns {greenlight.Empty} Empty instance
             */
            Empty.create = function create(properties) {
                return new Empty(properties);
            };
    
            /**
             * Encodes the specified Empty message. Does not implicitly {@link greenlight.Empty.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Empty
             * @static
             * @param {greenlight.IEmpty} message Empty message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Empty.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified Empty message, length delimited. Does not implicitly {@link greenlight.Empty.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Empty
             * @static
             * @param {greenlight.IEmpty} message Empty message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Empty.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an Empty message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Empty
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Empty} Empty
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Empty.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Empty();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an Empty message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Empty
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Empty} Empty
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Empty.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an Empty message.
             * @function verify
             * @memberof greenlight.Empty
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Empty.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates an Empty message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Empty
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Empty} Empty
             */
            Empty.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Empty)
                    return object;
                return new $root.greenlight.Empty();
            };
    
            /**
             * Creates a plain object from an Empty message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Empty
             * @static
             * @param {greenlight.Empty} message Empty
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Empty.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this Empty to JSON.
             * @function toJSON
             * @memberof greenlight.Empty
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Empty.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Empty;
        })();
    
        greenlight.Hsm = (function() {
    
            /**
             * Constructs a new Hsm service.
             * @memberof greenlight
             * @classdesc Represents a Hsm
             * @extends $protobuf.rpc.Service
             * @constructor
             * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
             * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
             * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
             */
            function Hsm(rpcImpl, requestDelimited, responseDelimited) {
                $protobuf.rpc.Service.call(this, rpcImpl, requestDelimited, responseDelimited);
            }
    
            (Hsm.prototype = Object.create($protobuf.rpc.Service.prototype)).constructor = Hsm;
    
            /**
             * Creates new Hsm service using the specified rpc implementation.
             * @function create
             * @memberof greenlight.Hsm
             * @static
             * @param {$protobuf.RPCImpl} rpcImpl RPC implementation
             * @param {boolean} [requestDelimited=false] Whether requests are length-delimited
             * @param {boolean} [responseDelimited=false] Whether responses are length-delimited
             * @returns {Hsm} RPC service. Useful where requests and/or responses are streamed.
             */
            Hsm.create = function create(rpcImpl, requestDelimited, responseDelimited) {
                return new this(rpcImpl, requestDelimited, responseDelimited);
            };
    
            /**
             * Callback as used by {@link greenlight.Hsm#request}.
             * @memberof greenlight.Hsm
             * @typedef RequestCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.HsmResponse} [response] HsmResponse
             */
    
            /**
             * Calls Request.
             * @function request
             * @memberof greenlight.Hsm
             * @instance
             * @param {greenlight.IHsmRequest} request HsmRequest message or plain object
             * @param {greenlight.Hsm.RequestCallback} callback Node-style callback called with the error, if any, and HsmResponse
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Hsm.prototype.request = function request(request, callback) {
                return this.rpcCall(request, $root.greenlight.HsmRequest, $root.greenlight.HsmResponse, request, callback);
            }, "name", { value: "Request" });
    
            /**
             * Calls Request.
             * @function request
             * @memberof greenlight.Hsm
             * @instance
             * @param {greenlight.IHsmRequest} request HsmRequest message or plain object
             * @returns {Promise<greenlight.HsmResponse>} Promise
             * @variation 2
             */
    
            /**
             * Callback as used by {@link greenlight.Hsm#ping}.
             * @memberof greenlight.Hsm
             * @typedef PingCallback
             * @type {function}
             * @param {Error|null} error Error, if any
             * @param {greenlight.Empty} [response] Empty
             */
    
            /**
             * Calls Ping.
             * @function ping
             * @memberof greenlight.Hsm
             * @instance
             * @param {greenlight.IEmpty} request Empty message or plain object
             * @param {greenlight.Hsm.PingCallback} callback Node-style callback called with the error, if any, and Empty
             * @returns {undefined}
             * @variation 1
             */
            Object.defineProperty(Hsm.prototype.ping = function ping(request, callback) {
                return this.rpcCall(ping, $root.greenlight.Empty, $root.greenlight.Empty, request, callback);
            }, "name", { value: "Ping" });
    
            /**
             * Calls Ping.
             * @function ping
             * @memberof greenlight.Hsm
             * @instance
             * @param {greenlight.IEmpty} request Empty message or plain object
             * @returns {Promise<greenlight.Empty>} Promise
             * @variation 2
             */
    
            return Hsm;
        })();
    
        /**
         * NetAddressType enum.
         * @name greenlight.NetAddressType
         * @enum {number}
         * @property {number} Ipv4=0 Ipv4 value
         * @property {number} Ipv6=1 Ipv6 value
         * @property {number} TorV2=2 TorV2 value
         * @property {number} TorV3=3 TorV3 value
         */
        greenlight.NetAddressType = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "Ipv4"] = 0;
            values[valuesById[1] = "Ipv6"] = 1;
            values[valuesById[2] = "TorV2"] = 2;
            values[valuesById[3] = "TorV3"] = 3;
            return values;
        })();
    
        greenlight.Address = (function() {
    
            /**
             * Properties of an Address.
             * @memberof greenlight
             * @interface IAddress
             * @property {greenlight.NetAddressType|null} [type] Address type
             * @property {string|null} [addr] Address addr
             * @property {number|null} [port] Address port
             */
    
            /**
             * Constructs a new Address.
             * @memberof greenlight
             * @classdesc Represents an Address.
             * @implements IAddress
             * @constructor
             * @param {greenlight.IAddress=} [properties] Properties to set
             */
            function Address(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Address type.
             * @member {greenlight.NetAddressType} type
             * @memberof greenlight.Address
             * @instance
             */
            Address.prototype.type = 0;
    
            /**
             * Address addr.
             * @member {string} addr
             * @memberof greenlight.Address
             * @instance
             */
            Address.prototype.addr = "";
    
            /**
             * Address port.
             * @member {number} port
             * @memberof greenlight.Address
             * @instance
             */
            Address.prototype.port = 0;
    
            /**
             * Creates a new Address instance using the specified properties.
             * @function create
             * @memberof greenlight.Address
             * @static
             * @param {greenlight.IAddress=} [properties] Properties to set
             * @returns {greenlight.Address} Address instance
             */
            Address.create = function create(properties) {
                return new Address(properties);
            };
    
            /**
             * Encodes the specified Address message. Does not implicitly {@link greenlight.Address.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Address
             * @static
             * @param {greenlight.IAddress} message Address message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Address.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.type != null && Object.hasOwnProperty.call(message, "type"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.type);
                if (message.addr != null && Object.hasOwnProperty.call(message, "addr"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.addr);
                if (message.port != null && Object.hasOwnProperty.call(message, "port"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.port);
                return writer;
            };
    
            /**
             * Encodes the specified Address message, length delimited. Does not implicitly {@link greenlight.Address.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Address
             * @static
             * @param {greenlight.IAddress} message Address message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Address.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an Address message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Address
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Address} Address
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Address.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Address();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.type = reader.int32();
                        break;
                    case 2:
                        message.addr = reader.string();
                        break;
                    case 3:
                        message.port = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an Address message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Address
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Address} Address
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Address.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an Address message.
             * @function verify
             * @memberof greenlight.Address
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Address.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.type != null && message.hasOwnProperty("type"))
                    switch (message.type) {
                    default:
                        return "type: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                    case 3:
                        break;
                    }
                if (message.addr != null && message.hasOwnProperty("addr"))
                    if (!$util.isString(message.addr))
                        return "addr: string expected";
                if (message.port != null && message.hasOwnProperty("port"))
                    if (!$util.isInteger(message.port))
                        return "port: integer expected";
                return null;
            };
    
            /**
             * Creates an Address message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Address
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Address} Address
             */
            Address.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Address)
                    return object;
                var message = new $root.greenlight.Address();
                switch (object.type) {
                case "Ipv4":
                case 0:
                    message.type = 0;
                    break;
                case "Ipv6":
                case 1:
                    message.type = 1;
                    break;
                case "TorV2":
                case 2:
                    message.type = 2;
                    break;
                case "TorV3":
                case 3:
                    message.type = 3;
                    break;
                }
                if (object.addr != null)
                    message.addr = String(object.addr);
                if (object.port != null)
                    message.port = object.port >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from an Address message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Address
             * @static
             * @param {greenlight.Address} message Address
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Address.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.type = options.enums === String ? "Ipv4" : 0;
                    object.addr = "";
                    object.port = 0;
                }
                if (message.type != null && message.hasOwnProperty("type"))
                    object.type = options.enums === String ? $root.greenlight.NetAddressType[message.type] : message.type;
                if (message.addr != null && message.hasOwnProperty("addr"))
                    object.addr = message.addr;
                if (message.port != null && message.hasOwnProperty("port"))
                    object.port = message.port;
                return object;
            };
    
            /**
             * Converts this Address to JSON.
             * @function toJSON
             * @memberof greenlight.Address
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Address.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Address;
        })();
    
        greenlight.GetInfoRequest = (function() {
    
            /**
             * Properties of a GetInfoRequest.
             * @memberof greenlight
             * @interface IGetInfoRequest
             */
    
            /**
             * Constructs a new GetInfoRequest.
             * @memberof greenlight
             * @classdesc Represents a GetInfoRequest.
             * @implements IGetInfoRequest
             * @constructor
             * @param {greenlight.IGetInfoRequest=} [properties] Properties to set
             */
            function GetInfoRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new GetInfoRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {greenlight.IGetInfoRequest=} [properties] Properties to set
             * @returns {greenlight.GetInfoRequest} GetInfoRequest instance
             */
            GetInfoRequest.create = function create(properties) {
                return new GetInfoRequest(properties);
            };
    
            /**
             * Encodes the specified GetInfoRequest message. Does not implicitly {@link greenlight.GetInfoRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {greenlight.IGetInfoRequest} message GetInfoRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            GetInfoRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified GetInfoRequest message, length delimited. Does not implicitly {@link greenlight.GetInfoRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {greenlight.IGetInfoRequest} message GetInfoRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            GetInfoRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a GetInfoRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.GetInfoRequest} GetInfoRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            GetInfoRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.GetInfoRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a GetInfoRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.GetInfoRequest} GetInfoRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            GetInfoRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a GetInfoRequest message.
             * @function verify
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            GetInfoRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates a GetInfoRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.GetInfoRequest} GetInfoRequest
             */
            GetInfoRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.GetInfoRequest)
                    return object;
                return new $root.greenlight.GetInfoRequest();
            };
    
            /**
             * Creates a plain object from a GetInfoRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.GetInfoRequest
             * @static
             * @param {greenlight.GetInfoRequest} message GetInfoRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            GetInfoRequest.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this GetInfoRequest to JSON.
             * @function toJSON
             * @memberof greenlight.GetInfoRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            GetInfoRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return GetInfoRequest;
        })();
    
        greenlight.GetInfoResponse = (function() {
    
            /**
             * Properties of a GetInfoResponse.
             * @memberof greenlight
             * @interface IGetInfoResponse
             * @property {Uint8Array|null} [nodeId] GetInfoResponse nodeId
             * @property {string|null} [alias] GetInfoResponse alias
             * @property {Uint8Array|null} [color] GetInfoResponse color
             * @property {number|null} [numPeers] GetInfoResponse numPeers
             * @property {Array.<greenlight.IAddress>|null} [addresses] GetInfoResponse addresses
             * @property {string|null} [version] GetInfoResponse version
             * @property {number|null} [blockheight] GetInfoResponse blockheight
             * @property {string|null} [network] GetInfoResponse network
             */
    
            /**
             * Constructs a new GetInfoResponse.
             * @memberof greenlight
             * @classdesc Represents a GetInfoResponse.
             * @implements IGetInfoResponse
             * @constructor
             * @param {greenlight.IGetInfoResponse=} [properties] Properties to set
             */
            function GetInfoResponse(properties) {
                this.addresses = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * GetInfoResponse nodeId.
             * @member {Uint8Array} nodeId
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * GetInfoResponse alias.
             * @member {string} alias
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.alias = "";
    
            /**
             * GetInfoResponse color.
             * @member {Uint8Array} color
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.color = $util.newBuffer([]);
    
            /**
             * GetInfoResponse numPeers.
             * @member {number} numPeers
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.numPeers = 0;
    
            /**
             * GetInfoResponse addresses.
             * @member {Array.<greenlight.IAddress>} addresses
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.addresses = $util.emptyArray;
    
            /**
             * GetInfoResponse version.
             * @member {string} version
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.version = "";
    
            /**
             * GetInfoResponse blockheight.
             * @member {number} blockheight
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.blockheight = 0;
    
            /**
             * GetInfoResponse network.
             * @member {string} network
             * @memberof greenlight.GetInfoResponse
             * @instance
             */
            GetInfoResponse.prototype.network = "";
    
            /**
             * Creates a new GetInfoResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {greenlight.IGetInfoResponse=} [properties] Properties to set
             * @returns {greenlight.GetInfoResponse} GetInfoResponse instance
             */
            GetInfoResponse.create = function create(properties) {
                return new GetInfoResponse(properties);
            };
    
            /**
             * Encodes the specified GetInfoResponse message. Does not implicitly {@link greenlight.GetInfoResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {greenlight.IGetInfoResponse} message GetInfoResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            GetInfoResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.alias != null && Object.hasOwnProperty.call(message, "alias"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.alias);
                if (message.color != null && Object.hasOwnProperty.call(message, "color"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.color);
                if (message.numPeers != null && Object.hasOwnProperty.call(message, "numPeers"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.numPeers);
                if (message.addresses != null && message.addresses.length)
                    for (var i = 0; i < message.addresses.length; ++i)
                        $root.greenlight.Address.encode(message.addresses[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.version != null && Object.hasOwnProperty.call(message, "version"))
                    writer.uint32(/* id 6, wireType 2 =*/50).string(message.version);
                if (message.blockheight != null && Object.hasOwnProperty.call(message, "blockheight"))
                    writer.uint32(/* id 7, wireType 0 =*/56).uint32(message.blockheight);
                if (message.network != null && Object.hasOwnProperty.call(message, "network"))
                    writer.uint32(/* id 8, wireType 2 =*/66).string(message.network);
                return writer;
            };
    
            /**
             * Encodes the specified GetInfoResponse message, length delimited. Does not implicitly {@link greenlight.GetInfoResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {greenlight.IGetInfoResponse} message GetInfoResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            GetInfoResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a GetInfoResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.GetInfoResponse} GetInfoResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            GetInfoResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.GetInfoResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.alias = reader.string();
                        break;
                    case 3:
                        message.color = reader.bytes();
                        break;
                    case 4:
                        message.numPeers = reader.uint32();
                        break;
                    case 5:
                        if (!(message.addresses && message.addresses.length))
                            message.addresses = [];
                        message.addresses.push($root.greenlight.Address.decode(reader, reader.uint32()));
                        break;
                    case 6:
                        message.version = reader.string();
                        break;
                    case 7:
                        message.blockheight = reader.uint32();
                        break;
                    case 8:
                        message.network = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a GetInfoResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.GetInfoResponse} GetInfoResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            GetInfoResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a GetInfoResponse message.
             * @function verify
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            GetInfoResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.alias != null && message.hasOwnProperty("alias"))
                    if (!$util.isString(message.alias))
                        return "alias: string expected";
                if (message.color != null && message.hasOwnProperty("color"))
                    if (!(message.color && typeof message.color.length === "number" || $util.isString(message.color)))
                        return "color: buffer expected";
                if (message.numPeers != null && message.hasOwnProperty("numPeers"))
                    if (!$util.isInteger(message.numPeers))
                        return "numPeers: integer expected";
                if (message.addresses != null && message.hasOwnProperty("addresses")) {
                    if (!Array.isArray(message.addresses))
                        return "addresses: array expected";
                    for (var i = 0; i < message.addresses.length; ++i) {
                        var error = $root.greenlight.Address.verify(message.addresses[i]);
                        if (error)
                            return "addresses." + error;
                    }
                }
                if (message.version != null && message.hasOwnProperty("version"))
                    if (!$util.isString(message.version))
                        return "version: string expected";
                if (message.blockheight != null && message.hasOwnProperty("blockheight"))
                    if (!$util.isInteger(message.blockheight))
                        return "blockheight: integer expected";
                if (message.network != null && message.hasOwnProperty("network"))
                    if (!$util.isString(message.network))
                        return "network: string expected";
                return null;
            };
    
            /**
             * Creates a GetInfoResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.GetInfoResponse} GetInfoResponse
             */
            GetInfoResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.GetInfoResponse)
                    return object;
                var message = new $root.greenlight.GetInfoResponse();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.alias != null)
                    message.alias = String(object.alias);
                if (object.color != null)
                    if (typeof object.color === "string")
                        $util.base64.decode(object.color, message.color = $util.newBuffer($util.base64.length(object.color)), 0);
                    else if (object.color.length)
                        message.color = object.color;
                if (object.numPeers != null)
                    message.numPeers = object.numPeers >>> 0;
                if (object.addresses) {
                    if (!Array.isArray(object.addresses))
                        throw TypeError(".greenlight.GetInfoResponse.addresses: array expected");
                    message.addresses = [];
                    for (var i = 0; i < object.addresses.length; ++i) {
                        if (typeof object.addresses[i] !== "object")
                            throw TypeError(".greenlight.GetInfoResponse.addresses: object expected");
                        message.addresses[i] = $root.greenlight.Address.fromObject(object.addresses[i]);
                    }
                }
                if (object.version != null)
                    message.version = String(object.version);
                if (object.blockheight != null)
                    message.blockheight = object.blockheight >>> 0;
                if (object.network != null)
                    message.network = String(object.network);
                return message;
            };
    
            /**
             * Creates a plain object from a GetInfoResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.GetInfoResponse
             * @static
             * @param {greenlight.GetInfoResponse} message GetInfoResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            GetInfoResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.addresses = [];
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.alias = "";
                    if (options.bytes === String)
                        object.color = "";
                    else {
                        object.color = [];
                        if (options.bytes !== Array)
                            object.color = $util.newBuffer(object.color);
                    }
                    object.numPeers = 0;
                    object.version = "";
                    object.blockheight = 0;
                    object.network = "";
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.alias != null && message.hasOwnProperty("alias"))
                    object.alias = message.alias;
                if (message.color != null && message.hasOwnProperty("color"))
                    object.color = options.bytes === String ? $util.base64.encode(message.color, 0, message.color.length) : options.bytes === Array ? Array.prototype.slice.call(message.color) : message.color;
                if (message.numPeers != null && message.hasOwnProperty("numPeers"))
                    object.numPeers = message.numPeers;
                if (message.addresses && message.addresses.length) {
                    object.addresses = [];
                    for (var j = 0; j < message.addresses.length; ++j)
                        object.addresses[j] = $root.greenlight.Address.toObject(message.addresses[j], options);
                }
                if (message.version != null && message.hasOwnProperty("version"))
                    object.version = message.version;
                if (message.blockheight != null && message.hasOwnProperty("blockheight"))
                    object.blockheight = message.blockheight;
                if (message.network != null && message.hasOwnProperty("network"))
                    object.network = message.network;
                return object;
            };
    
            /**
             * Converts this GetInfoResponse to JSON.
             * @function toJSON
             * @memberof greenlight.GetInfoResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            GetInfoResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return GetInfoResponse;
        })();
    
        greenlight.StopRequest = (function() {
    
            /**
             * Properties of a StopRequest.
             * @memberof greenlight
             * @interface IStopRequest
             */
    
            /**
             * Constructs a new StopRequest.
             * @memberof greenlight
             * @classdesc Represents a StopRequest.
             * @implements IStopRequest
             * @constructor
             * @param {greenlight.IStopRequest=} [properties] Properties to set
             */
            function StopRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new StopRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.StopRequest
             * @static
             * @param {greenlight.IStopRequest=} [properties] Properties to set
             * @returns {greenlight.StopRequest} StopRequest instance
             */
            StopRequest.create = function create(properties) {
                return new StopRequest(properties);
            };
    
            /**
             * Encodes the specified StopRequest message. Does not implicitly {@link greenlight.StopRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.StopRequest
             * @static
             * @param {greenlight.IStopRequest} message StopRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StopRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified StopRequest message, length delimited. Does not implicitly {@link greenlight.StopRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.StopRequest
             * @static
             * @param {greenlight.IStopRequest} message StopRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StopRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a StopRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.StopRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.StopRequest} StopRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StopRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.StopRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a StopRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.StopRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.StopRequest} StopRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StopRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a StopRequest message.
             * @function verify
             * @memberof greenlight.StopRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StopRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates a StopRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.StopRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.StopRequest} StopRequest
             */
            StopRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.StopRequest)
                    return object;
                return new $root.greenlight.StopRequest();
            };
    
            /**
             * Creates a plain object from a StopRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.StopRequest
             * @static
             * @param {greenlight.StopRequest} message StopRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StopRequest.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this StopRequest to JSON.
             * @function toJSON
             * @memberof greenlight.StopRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StopRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return StopRequest;
        })();
    
        greenlight.StopResponse = (function() {
    
            /**
             * Properties of a StopResponse.
             * @memberof greenlight
             * @interface IStopResponse
             */
    
            /**
             * Constructs a new StopResponse.
             * @memberof greenlight
             * @classdesc Represents a StopResponse.
             * @implements IStopResponse
             * @constructor
             * @param {greenlight.IStopResponse=} [properties] Properties to set
             */
            function StopResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new StopResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.StopResponse
             * @static
             * @param {greenlight.IStopResponse=} [properties] Properties to set
             * @returns {greenlight.StopResponse} StopResponse instance
             */
            StopResponse.create = function create(properties) {
                return new StopResponse(properties);
            };
    
            /**
             * Encodes the specified StopResponse message. Does not implicitly {@link greenlight.StopResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.StopResponse
             * @static
             * @param {greenlight.IStopResponse} message StopResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StopResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified StopResponse message, length delimited. Does not implicitly {@link greenlight.StopResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.StopResponse
             * @static
             * @param {greenlight.IStopResponse} message StopResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StopResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a StopResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.StopResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.StopResponse} StopResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StopResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.StopResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a StopResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.StopResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.StopResponse} StopResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StopResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a StopResponse message.
             * @function verify
             * @memberof greenlight.StopResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StopResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates a StopResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.StopResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.StopResponse} StopResponse
             */
            StopResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.StopResponse)
                    return object;
                return new $root.greenlight.StopResponse();
            };
    
            /**
             * Creates a plain object from a StopResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.StopResponse
             * @static
             * @param {greenlight.StopResponse} message StopResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StopResponse.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this StopResponse to JSON.
             * @function toJSON
             * @memberof greenlight.StopResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StopResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return StopResponse;
        })();
    
        greenlight.ConnectRequest = (function() {
    
            /**
             * Properties of a ConnectRequest.
             * @memberof greenlight
             * @interface IConnectRequest
             * @property {string|null} [nodeId] ConnectRequest nodeId
             * @property {string|null} [addr] ConnectRequest addr
             */
    
            /**
             * Constructs a new ConnectRequest.
             * @memberof greenlight
             * @classdesc Represents a ConnectRequest.
             * @implements IConnectRequest
             * @constructor
             * @param {greenlight.IConnectRequest=} [properties] Properties to set
             */
            function ConnectRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ConnectRequest nodeId.
             * @member {string} nodeId
             * @memberof greenlight.ConnectRequest
             * @instance
             */
            ConnectRequest.prototype.nodeId = "";
    
            /**
             * ConnectRequest addr.
             * @member {string} addr
             * @memberof greenlight.ConnectRequest
             * @instance
             */
            ConnectRequest.prototype.addr = "";
    
            /**
             * Creates a new ConnectRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {greenlight.IConnectRequest=} [properties] Properties to set
             * @returns {greenlight.ConnectRequest} ConnectRequest instance
             */
            ConnectRequest.create = function create(properties) {
                return new ConnectRequest(properties);
            };
    
            /**
             * Encodes the specified ConnectRequest message. Does not implicitly {@link greenlight.ConnectRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {greenlight.IConnectRequest} message ConnectRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ConnectRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.nodeId);
                if (message.addr != null && Object.hasOwnProperty.call(message, "addr"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.addr);
                return writer;
            };
    
            /**
             * Encodes the specified ConnectRequest message, length delimited. Does not implicitly {@link greenlight.ConnectRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {greenlight.IConnectRequest} message ConnectRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ConnectRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ConnectRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ConnectRequest} ConnectRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ConnectRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ConnectRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.string();
                        break;
                    case 2:
                        message.addr = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ConnectRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ConnectRequest} ConnectRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ConnectRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ConnectRequest message.
             * @function verify
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ConnectRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!$util.isString(message.nodeId))
                        return "nodeId: string expected";
                if (message.addr != null && message.hasOwnProperty("addr"))
                    if (!$util.isString(message.addr))
                        return "addr: string expected";
                return null;
            };
    
            /**
             * Creates a ConnectRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ConnectRequest} ConnectRequest
             */
            ConnectRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ConnectRequest)
                    return object;
                var message = new $root.greenlight.ConnectRequest();
                if (object.nodeId != null)
                    message.nodeId = String(object.nodeId);
                if (object.addr != null)
                    message.addr = String(object.addr);
                return message;
            };
    
            /**
             * Creates a plain object from a ConnectRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ConnectRequest
             * @static
             * @param {greenlight.ConnectRequest} message ConnectRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ConnectRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.nodeId = "";
                    object.addr = "";
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = message.nodeId;
                if (message.addr != null && message.hasOwnProperty("addr"))
                    object.addr = message.addr;
                return object;
            };
    
            /**
             * Converts this ConnectRequest to JSON.
             * @function toJSON
             * @memberof greenlight.ConnectRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ConnectRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ConnectRequest;
        })();
    
        greenlight.ConnectResponse = (function() {
    
            /**
             * Properties of a ConnectResponse.
             * @memberof greenlight
             * @interface IConnectResponse
             * @property {string|null} [nodeId] ConnectResponse nodeId
             * @property {string|null} [features] ConnectResponse features
             */
    
            /**
             * Constructs a new ConnectResponse.
             * @memberof greenlight
             * @classdesc Represents a ConnectResponse.
             * @implements IConnectResponse
             * @constructor
             * @param {greenlight.IConnectResponse=} [properties] Properties to set
             */
            function ConnectResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ConnectResponse nodeId.
             * @member {string} nodeId
             * @memberof greenlight.ConnectResponse
             * @instance
             */
            ConnectResponse.prototype.nodeId = "";
    
            /**
             * ConnectResponse features.
             * @member {string} features
             * @memberof greenlight.ConnectResponse
             * @instance
             */
            ConnectResponse.prototype.features = "";
    
            /**
             * Creates a new ConnectResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {greenlight.IConnectResponse=} [properties] Properties to set
             * @returns {greenlight.ConnectResponse} ConnectResponse instance
             */
            ConnectResponse.create = function create(properties) {
                return new ConnectResponse(properties);
            };
    
            /**
             * Encodes the specified ConnectResponse message. Does not implicitly {@link greenlight.ConnectResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {greenlight.IConnectResponse} message ConnectResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ConnectResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.nodeId);
                if (message.features != null && Object.hasOwnProperty.call(message, "features"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.features);
                return writer;
            };
    
            /**
             * Encodes the specified ConnectResponse message, length delimited. Does not implicitly {@link greenlight.ConnectResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {greenlight.IConnectResponse} message ConnectResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ConnectResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ConnectResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ConnectResponse} ConnectResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ConnectResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ConnectResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.string();
                        break;
                    case 2:
                        message.features = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ConnectResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ConnectResponse} ConnectResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ConnectResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ConnectResponse message.
             * @function verify
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ConnectResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!$util.isString(message.nodeId))
                        return "nodeId: string expected";
                if (message.features != null && message.hasOwnProperty("features"))
                    if (!$util.isString(message.features))
                        return "features: string expected";
                return null;
            };
    
            /**
             * Creates a ConnectResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ConnectResponse} ConnectResponse
             */
            ConnectResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ConnectResponse)
                    return object;
                var message = new $root.greenlight.ConnectResponse();
                if (object.nodeId != null)
                    message.nodeId = String(object.nodeId);
                if (object.features != null)
                    message.features = String(object.features);
                return message;
            };
    
            /**
             * Creates a plain object from a ConnectResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ConnectResponse
             * @static
             * @param {greenlight.ConnectResponse} message ConnectResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ConnectResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.nodeId = "";
                    object.features = "";
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = message.nodeId;
                if (message.features != null && message.hasOwnProperty("features"))
                    object.features = message.features;
                return object;
            };
    
            /**
             * Converts this ConnectResponse to JSON.
             * @function toJSON
             * @memberof greenlight.ConnectResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ConnectResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ConnectResponse;
        })();
    
        greenlight.ListPeersRequest = (function() {
    
            /**
             * Properties of a ListPeersRequest.
             * @memberof greenlight
             * @interface IListPeersRequest
             * @property {string|null} [nodeId] ListPeersRequest nodeId
             */
    
            /**
             * Constructs a new ListPeersRequest.
             * @memberof greenlight
             * @classdesc Represents a ListPeersRequest.
             * @implements IListPeersRequest
             * @constructor
             * @param {greenlight.IListPeersRequest=} [properties] Properties to set
             */
            function ListPeersRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListPeersRequest nodeId.
             * @member {string} nodeId
             * @memberof greenlight.ListPeersRequest
             * @instance
             */
            ListPeersRequest.prototype.nodeId = "";
    
            /**
             * Creates a new ListPeersRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {greenlight.IListPeersRequest=} [properties] Properties to set
             * @returns {greenlight.ListPeersRequest} ListPeersRequest instance
             */
            ListPeersRequest.create = function create(properties) {
                return new ListPeersRequest(properties);
            };
    
            /**
             * Encodes the specified ListPeersRequest message. Does not implicitly {@link greenlight.ListPeersRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {greenlight.IListPeersRequest} message ListPeersRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPeersRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.nodeId);
                return writer;
            };
    
            /**
             * Encodes the specified ListPeersRequest message, length delimited. Does not implicitly {@link greenlight.ListPeersRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {greenlight.IListPeersRequest} message ListPeersRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPeersRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListPeersRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListPeersRequest} ListPeersRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPeersRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListPeersRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListPeersRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListPeersRequest} ListPeersRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPeersRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListPeersRequest message.
             * @function verify
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListPeersRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!$util.isString(message.nodeId))
                        return "nodeId: string expected";
                return null;
            };
    
            /**
             * Creates a ListPeersRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListPeersRequest} ListPeersRequest
             */
            ListPeersRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListPeersRequest)
                    return object;
                var message = new $root.greenlight.ListPeersRequest();
                if (object.nodeId != null)
                    message.nodeId = String(object.nodeId);
                return message;
            };
    
            /**
             * Creates a plain object from a ListPeersRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListPeersRequest
             * @static
             * @param {greenlight.ListPeersRequest} message ListPeersRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListPeersRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.nodeId = "";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = message.nodeId;
                return object;
            };
    
            /**
             * Converts this ListPeersRequest to JSON.
             * @function toJSON
             * @memberof greenlight.ListPeersRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListPeersRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListPeersRequest;
        })();
    
        greenlight.Htlc = (function() {
    
            /**
             * Properties of a Htlc.
             * @memberof greenlight
             * @interface IHtlc
             * @property {string|null} [direction] Htlc direction
             * @property {number|Long|null} [id] Htlc id
             * @property {string|null} [amount] Htlc amount
             * @property {number|Long|null} [expiry] Htlc expiry
             * @property {string|null} [paymentHash] Htlc paymentHash
             * @property {string|null} [state] Htlc state
             * @property {boolean|null} [localTrimmed] Htlc localTrimmed
             */
    
            /**
             * Constructs a new Htlc.
             * @memberof greenlight
             * @classdesc Represents a Htlc.
             * @implements IHtlc
             * @constructor
             * @param {greenlight.IHtlc=} [properties] Properties to set
             */
            function Htlc(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Htlc direction.
             * @member {string} direction
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.direction = "";
    
            /**
             * Htlc id.
             * @member {number|Long} id
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.id = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * Htlc amount.
             * @member {string} amount
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.amount = "";
    
            /**
             * Htlc expiry.
             * @member {number|Long} expiry
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.expiry = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * Htlc paymentHash.
             * @member {string} paymentHash
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.paymentHash = "";
    
            /**
             * Htlc state.
             * @member {string} state
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.state = "";
    
            /**
             * Htlc localTrimmed.
             * @member {boolean} localTrimmed
             * @memberof greenlight.Htlc
             * @instance
             */
            Htlc.prototype.localTrimmed = false;
    
            /**
             * Creates a new Htlc instance using the specified properties.
             * @function create
             * @memberof greenlight.Htlc
             * @static
             * @param {greenlight.IHtlc=} [properties] Properties to set
             * @returns {greenlight.Htlc} Htlc instance
             */
            Htlc.create = function create(properties) {
                return new Htlc(properties);
            };
    
            /**
             * Encodes the specified Htlc message. Does not implicitly {@link greenlight.Htlc.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Htlc
             * @static
             * @param {greenlight.IHtlc} message Htlc message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Htlc.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.direction != null && Object.hasOwnProperty.call(message, "direction"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.direction);
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.id);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.amount);
                if (message.expiry != null && Object.hasOwnProperty.call(message, "expiry"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.expiry);
                if (message.paymentHash != null && Object.hasOwnProperty.call(message, "paymentHash"))
                    writer.uint32(/* id 5, wireType 2 =*/42).string(message.paymentHash);
                if (message.state != null && Object.hasOwnProperty.call(message, "state"))
                    writer.uint32(/* id 6, wireType 2 =*/50).string(message.state);
                if (message.localTrimmed != null && Object.hasOwnProperty.call(message, "localTrimmed"))
                    writer.uint32(/* id 7, wireType 0 =*/56).bool(message.localTrimmed);
                return writer;
            };
    
            /**
             * Encodes the specified Htlc message, length delimited. Does not implicitly {@link greenlight.Htlc.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Htlc
             * @static
             * @param {greenlight.IHtlc} message Htlc message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Htlc.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Htlc message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Htlc
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Htlc} Htlc
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Htlc.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Htlc();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.direction = reader.string();
                        break;
                    case 2:
                        message.id = reader.uint64();
                        break;
                    case 3:
                        message.amount = reader.string();
                        break;
                    case 4:
                        message.expiry = reader.uint64();
                        break;
                    case 5:
                        message.paymentHash = reader.string();
                        break;
                    case 6:
                        message.state = reader.string();
                        break;
                    case 7:
                        message.localTrimmed = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Htlc message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Htlc
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Htlc} Htlc
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Htlc.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Htlc message.
             * @function verify
             * @memberof greenlight.Htlc
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Htlc.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.direction != null && message.hasOwnProperty("direction"))
                    if (!$util.isString(message.direction))
                        return "direction: string expected";
                if (message.id != null && message.hasOwnProperty("id"))
                    if (!$util.isInteger(message.id) && !(message.id && $util.isInteger(message.id.low) && $util.isInteger(message.id.high)))
                        return "id: integer|Long expected";
                if (message.amount != null && message.hasOwnProperty("amount"))
                    if (!$util.isString(message.amount))
                        return "amount: string expected";
                if (message.expiry != null && message.hasOwnProperty("expiry"))
                    if (!$util.isInteger(message.expiry) && !(message.expiry && $util.isInteger(message.expiry.low) && $util.isInteger(message.expiry.high)))
                        return "expiry: integer|Long expected";
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    if (!$util.isString(message.paymentHash))
                        return "paymentHash: string expected";
                if (message.state != null && message.hasOwnProperty("state"))
                    if (!$util.isString(message.state))
                        return "state: string expected";
                if (message.localTrimmed != null && message.hasOwnProperty("localTrimmed"))
                    if (typeof message.localTrimmed !== "boolean")
                        return "localTrimmed: boolean expected";
                return null;
            };
    
            /**
             * Creates a Htlc message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Htlc
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Htlc} Htlc
             */
            Htlc.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Htlc)
                    return object;
                var message = new $root.greenlight.Htlc();
                if (object.direction != null)
                    message.direction = String(object.direction);
                if (object.id != null)
                    if ($util.Long)
                        (message.id = $util.Long.fromValue(object.id)).unsigned = true;
                    else if (typeof object.id === "string")
                        message.id = parseInt(object.id, 10);
                    else if (typeof object.id === "number")
                        message.id = object.id;
                    else if (typeof object.id === "object")
                        message.id = new $util.LongBits(object.id.low >>> 0, object.id.high >>> 0).toNumber(true);
                if (object.amount != null)
                    message.amount = String(object.amount);
                if (object.expiry != null)
                    if ($util.Long)
                        (message.expiry = $util.Long.fromValue(object.expiry)).unsigned = true;
                    else if (typeof object.expiry === "string")
                        message.expiry = parseInt(object.expiry, 10);
                    else if (typeof object.expiry === "number")
                        message.expiry = object.expiry;
                    else if (typeof object.expiry === "object")
                        message.expiry = new $util.LongBits(object.expiry.low >>> 0, object.expiry.high >>> 0).toNumber(true);
                if (object.paymentHash != null)
                    message.paymentHash = String(object.paymentHash);
                if (object.state != null)
                    message.state = String(object.state);
                if (object.localTrimmed != null)
                    message.localTrimmed = Boolean(object.localTrimmed);
                return message;
            };
    
            /**
             * Creates a plain object from a Htlc message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Htlc
             * @static
             * @param {greenlight.Htlc} message Htlc
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Htlc.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.direction = "";
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.id = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.id = options.longs === String ? "0" : 0;
                    object.amount = "";
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.expiry = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.expiry = options.longs === String ? "0" : 0;
                    object.paymentHash = "";
                    object.state = "";
                    object.localTrimmed = false;
                }
                if (message.direction != null && message.hasOwnProperty("direction"))
                    object.direction = message.direction;
                if (message.id != null && message.hasOwnProperty("id"))
                    if (typeof message.id === "number")
                        object.id = options.longs === String ? String(message.id) : message.id;
                    else
                        object.id = options.longs === String ? $util.Long.prototype.toString.call(message.id) : options.longs === Number ? new $util.LongBits(message.id.low >>> 0, message.id.high >>> 0).toNumber(true) : message.id;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = message.amount;
                if (message.expiry != null && message.hasOwnProperty("expiry"))
                    if (typeof message.expiry === "number")
                        object.expiry = options.longs === String ? String(message.expiry) : message.expiry;
                    else
                        object.expiry = options.longs === String ? $util.Long.prototype.toString.call(message.expiry) : options.longs === Number ? new $util.LongBits(message.expiry.low >>> 0, message.expiry.high >>> 0).toNumber(true) : message.expiry;
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    object.paymentHash = message.paymentHash;
                if (message.state != null && message.hasOwnProperty("state"))
                    object.state = message.state;
                if (message.localTrimmed != null && message.hasOwnProperty("localTrimmed"))
                    object.localTrimmed = message.localTrimmed;
                return object;
            };
    
            /**
             * Converts this Htlc to JSON.
             * @function toJSON
             * @memberof greenlight.Htlc
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Htlc.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Htlc;
        })();
    
        greenlight.Channel = (function() {
    
            /**
             * Properties of a Channel.
             * @memberof greenlight
             * @interface IChannel
             * @property {string|null} [state] Channel state
             * @property {string|null} [owner] Channel owner
             * @property {string|null} [shortChannelId] Channel shortChannelId
             * @property {number|null} [direction] Channel direction
             * @property {string|null} [channelId] Channel channelId
             * @property {string|null} [fundingTxid] Channel fundingTxid
             * @property {string|null} [closeToAddr] Channel closeToAddr
             * @property {string|null} [closeTo] Channel closeTo
             * @property {boolean|null} ["private"] Channel private
             * @property {string|null} [total] Channel total
             * @property {string|null} [dustLimit] Channel dustLimit
             * @property {string|null} [spendable] Channel spendable
             * @property {string|null} [receivable] Channel receivable
             * @property {number|null} [theirToSelfDelay] Channel theirToSelfDelay
             * @property {number|null} [ourToSelfDelay] Channel ourToSelfDelay
             * @property {Array.<string>|null} [status] Channel status
             * @property {Array.<greenlight.IHtlc>|null} [htlcs] Channel htlcs
             */
    
            /**
             * Constructs a new Channel.
             * @memberof greenlight
             * @classdesc Represents a Channel.
             * @implements IChannel
             * @constructor
             * @param {greenlight.IChannel=} [properties] Properties to set
             */
            function Channel(properties) {
                this.status = [];
                this.htlcs = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Channel state.
             * @member {string} state
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.state = "";
    
            /**
             * Channel owner.
             * @member {string} owner
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.owner = "";
    
            /**
             * Channel shortChannelId.
             * @member {string} shortChannelId
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.shortChannelId = "";
    
            /**
             * Channel direction.
             * @member {number} direction
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.direction = 0;
    
            /**
             * Channel channelId.
             * @member {string} channelId
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.channelId = "";
    
            /**
             * Channel fundingTxid.
             * @member {string} fundingTxid
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.fundingTxid = "";
    
            /**
             * Channel closeToAddr.
             * @member {string} closeToAddr
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.closeToAddr = "";
    
            /**
             * Channel closeTo.
             * @member {string} closeTo
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.closeTo = "";
    
            /**
             * Channel private.
             * @member {boolean} private
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype["private"] = false;
    
            /**
             * Channel total.
             * @member {string} total
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.total = "";
    
            /**
             * Channel dustLimit.
             * @member {string} dustLimit
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.dustLimit = "";
    
            /**
             * Channel spendable.
             * @member {string} spendable
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.spendable = "";
    
            /**
             * Channel receivable.
             * @member {string} receivable
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.receivable = "";
    
            /**
             * Channel theirToSelfDelay.
             * @member {number} theirToSelfDelay
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.theirToSelfDelay = 0;
    
            /**
             * Channel ourToSelfDelay.
             * @member {number} ourToSelfDelay
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.ourToSelfDelay = 0;
    
            /**
             * Channel status.
             * @member {Array.<string>} status
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.status = $util.emptyArray;
    
            /**
             * Channel htlcs.
             * @member {Array.<greenlight.IHtlc>} htlcs
             * @memberof greenlight.Channel
             * @instance
             */
            Channel.prototype.htlcs = $util.emptyArray;
    
            /**
             * Creates a new Channel instance using the specified properties.
             * @function create
             * @memberof greenlight.Channel
             * @static
             * @param {greenlight.IChannel=} [properties] Properties to set
             * @returns {greenlight.Channel} Channel instance
             */
            Channel.create = function create(properties) {
                return new Channel(properties);
            };
    
            /**
             * Encodes the specified Channel message. Does not implicitly {@link greenlight.Channel.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Channel
             * @static
             * @param {greenlight.IChannel} message Channel message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Channel.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.state != null && Object.hasOwnProperty.call(message, "state"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.state);
                if (message.owner != null && Object.hasOwnProperty.call(message, "owner"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.owner);
                if (message.shortChannelId != null && Object.hasOwnProperty.call(message, "shortChannelId"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.shortChannelId);
                if (message.direction != null && Object.hasOwnProperty.call(message, "direction"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.direction);
                if (message.channelId != null && Object.hasOwnProperty.call(message, "channelId"))
                    writer.uint32(/* id 5, wireType 2 =*/42).string(message.channelId);
                if (message.fundingTxid != null && Object.hasOwnProperty.call(message, "fundingTxid"))
                    writer.uint32(/* id 6, wireType 2 =*/50).string(message.fundingTxid);
                if (message.closeToAddr != null && Object.hasOwnProperty.call(message, "closeToAddr"))
                    writer.uint32(/* id 7, wireType 2 =*/58).string(message.closeToAddr);
                if (message.closeTo != null && Object.hasOwnProperty.call(message, "closeTo"))
                    writer.uint32(/* id 8, wireType 2 =*/66).string(message.closeTo);
                if (message["private"] != null && Object.hasOwnProperty.call(message, "private"))
                    writer.uint32(/* id 9, wireType 0 =*/72).bool(message["private"]);
                if (message.total != null && Object.hasOwnProperty.call(message, "total"))
                    writer.uint32(/* id 10, wireType 2 =*/82).string(message.total);
                if (message.dustLimit != null && Object.hasOwnProperty.call(message, "dustLimit"))
                    writer.uint32(/* id 11, wireType 2 =*/90).string(message.dustLimit);
                if (message.spendable != null && Object.hasOwnProperty.call(message, "spendable"))
                    writer.uint32(/* id 12, wireType 2 =*/98).string(message.spendable);
                if (message.receivable != null && Object.hasOwnProperty.call(message, "receivable"))
                    writer.uint32(/* id 13, wireType 2 =*/106).string(message.receivable);
                if (message.theirToSelfDelay != null && Object.hasOwnProperty.call(message, "theirToSelfDelay"))
                    writer.uint32(/* id 14, wireType 0 =*/112).uint32(message.theirToSelfDelay);
                if (message.ourToSelfDelay != null && Object.hasOwnProperty.call(message, "ourToSelfDelay"))
                    writer.uint32(/* id 15, wireType 0 =*/120).uint32(message.ourToSelfDelay);
                if (message.status != null && message.status.length)
                    for (var i = 0; i < message.status.length; ++i)
                        writer.uint32(/* id 16, wireType 2 =*/130).string(message.status[i]);
                if (message.htlcs != null && message.htlcs.length)
                    for (var i = 0; i < message.htlcs.length; ++i)
                        $root.greenlight.Htlc.encode(message.htlcs[i], writer.uint32(/* id 17, wireType 2 =*/138).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified Channel message, length delimited. Does not implicitly {@link greenlight.Channel.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Channel
             * @static
             * @param {greenlight.IChannel} message Channel message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Channel.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Channel message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Channel
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Channel} Channel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Channel.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Channel();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.state = reader.string();
                        break;
                    case 2:
                        message.owner = reader.string();
                        break;
                    case 3:
                        message.shortChannelId = reader.string();
                        break;
                    case 4:
                        message.direction = reader.uint32();
                        break;
                    case 5:
                        message.channelId = reader.string();
                        break;
                    case 6:
                        message.fundingTxid = reader.string();
                        break;
                    case 7:
                        message.closeToAddr = reader.string();
                        break;
                    case 8:
                        message.closeTo = reader.string();
                        break;
                    case 9:
                        message["private"] = reader.bool();
                        break;
                    case 10:
                        message.total = reader.string();
                        break;
                    case 11:
                        message.dustLimit = reader.string();
                        break;
                    case 12:
                        message.spendable = reader.string();
                        break;
                    case 13:
                        message.receivable = reader.string();
                        break;
                    case 14:
                        message.theirToSelfDelay = reader.uint32();
                        break;
                    case 15:
                        message.ourToSelfDelay = reader.uint32();
                        break;
                    case 16:
                        if (!(message.status && message.status.length))
                            message.status = [];
                        message.status.push(reader.string());
                        break;
                    case 17:
                        if (!(message.htlcs && message.htlcs.length))
                            message.htlcs = [];
                        message.htlcs.push($root.greenlight.Htlc.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Channel message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Channel
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Channel} Channel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Channel.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Channel message.
             * @function verify
             * @memberof greenlight.Channel
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Channel.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.state != null && message.hasOwnProperty("state"))
                    if (!$util.isString(message.state))
                        return "state: string expected";
                if (message.owner != null && message.hasOwnProperty("owner"))
                    if (!$util.isString(message.owner))
                        return "owner: string expected";
                if (message.shortChannelId != null && message.hasOwnProperty("shortChannelId"))
                    if (!$util.isString(message.shortChannelId))
                        return "shortChannelId: string expected";
                if (message.direction != null && message.hasOwnProperty("direction"))
                    if (!$util.isInteger(message.direction))
                        return "direction: integer expected";
                if (message.channelId != null && message.hasOwnProperty("channelId"))
                    if (!$util.isString(message.channelId))
                        return "channelId: string expected";
                if (message.fundingTxid != null && message.hasOwnProperty("fundingTxid"))
                    if (!$util.isString(message.fundingTxid))
                        return "fundingTxid: string expected";
                if (message.closeToAddr != null && message.hasOwnProperty("closeToAddr"))
                    if (!$util.isString(message.closeToAddr))
                        return "closeToAddr: string expected";
                if (message.closeTo != null && message.hasOwnProperty("closeTo"))
                    if (!$util.isString(message.closeTo))
                        return "closeTo: string expected";
                if (message["private"] != null && message.hasOwnProperty("private"))
                    if (typeof message["private"] !== "boolean")
                        return "private: boolean expected";
                if (message.total != null && message.hasOwnProperty("total"))
                    if (!$util.isString(message.total))
                        return "total: string expected";
                if (message.dustLimit != null && message.hasOwnProperty("dustLimit"))
                    if (!$util.isString(message.dustLimit))
                        return "dustLimit: string expected";
                if (message.spendable != null && message.hasOwnProperty("spendable"))
                    if (!$util.isString(message.spendable))
                        return "spendable: string expected";
                if (message.receivable != null && message.hasOwnProperty("receivable"))
                    if (!$util.isString(message.receivable))
                        return "receivable: string expected";
                if (message.theirToSelfDelay != null && message.hasOwnProperty("theirToSelfDelay"))
                    if (!$util.isInteger(message.theirToSelfDelay))
                        return "theirToSelfDelay: integer expected";
                if (message.ourToSelfDelay != null && message.hasOwnProperty("ourToSelfDelay"))
                    if (!$util.isInteger(message.ourToSelfDelay))
                        return "ourToSelfDelay: integer expected";
                if (message.status != null && message.hasOwnProperty("status")) {
                    if (!Array.isArray(message.status))
                        return "status: array expected";
                    for (var i = 0; i < message.status.length; ++i)
                        if (!$util.isString(message.status[i]))
                            return "status: string[] expected";
                }
                if (message.htlcs != null && message.hasOwnProperty("htlcs")) {
                    if (!Array.isArray(message.htlcs))
                        return "htlcs: array expected";
                    for (var i = 0; i < message.htlcs.length; ++i) {
                        var error = $root.greenlight.Htlc.verify(message.htlcs[i]);
                        if (error)
                            return "htlcs." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a Channel message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Channel
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Channel} Channel
             */
            Channel.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Channel)
                    return object;
                var message = new $root.greenlight.Channel();
                if (object.state != null)
                    message.state = String(object.state);
                if (object.owner != null)
                    message.owner = String(object.owner);
                if (object.shortChannelId != null)
                    message.shortChannelId = String(object.shortChannelId);
                if (object.direction != null)
                    message.direction = object.direction >>> 0;
                if (object.channelId != null)
                    message.channelId = String(object.channelId);
                if (object.fundingTxid != null)
                    message.fundingTxid = String(object.fundingTxid);
                if (object.closeToAddr != null)
                    message.closeToAddr = String(object.closeToAddr);
                if (object.closeTo != null)
                    message.closeTo = String(object.closeTo);
                if (object["private"] != null)
                    message["private"] = Boolean(object["private"]);
                if (object.total != null)
                    message.total = String(object.total);
                if (object.dustLimit != null)
                    message.dustLimit = String(object.dustLimit);
                if (object.spendable != null)
                    message.spendable = String(object.spendable);
                if (object.receivable != null)
                    message.receivable = String(object.receivable);
                if (object.theirToSelfDelay != null)
                    message.theirToSelfDelay = object.theirToSelfDelay >>> 0;
                if (object.ourToSelfDelay != null)
                    message.ourToSelfDelay = object.ourToSelfDelay >>> 0;
                if (object.status) {
                    if (!Array.isArray(object.status))
                        throw TypeError(".greenlight.Channel.status: array expected");
                    message.status = [];
                    for (var i = 0; i < object.status.length; ++i)
                        message.status[i] = String(object.status[i]);
                }
                if (object.htlcs) {
                    if (!Array.isArray(object.htlcs))
                        throw TypeError(".greenlight.Channel.htlcs: array expected");
                    message.htlcs = [];
                    for (var i = 0; i < object.htlcs.length; ++i) {
                        if (typeof object.htlcs[i] !== "object")
                            throw TypeError(".greenlight.Channel.htlcs: object expected");
                        message.htlcs[i] = $root.greenlight.Htlc.fromObject(object.htlcs[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a Channel message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Channel
             * @static
             * @param {greenlight.Channel} message Channel
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Channel.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults) {
                    object.status = [];
                    object.htlcs = [];
                }
                if (options.defaults) {
                    object.state = "";
                    object.owner = "";
                    object.shortChannelId = "";
                    object.direction = 0;
                    object.channelId = "";
                    object.fundingTxid = "";
                    object.closeToAddr = "";
                    object.closeTo = "";
                    object["private"] = false;
                    object.total = "";
                    object.dustLimit = "";
                    object.spendable = "";
                    object.receivable = "";
                    object.theirToSelfDelay = 0;
                    object.ourToSelfDelay = 0;
                }
                if (message.state != null && message.hasOwnProperty("state"))
                    object.state = message.state;
                if (message.owner != null && message.hasOwnProperty("owner"))
                    object.owner = message.owner;
                if (message.shortChannelId != null && message.hasOwnProperty("shortChannelId"))
                    object.shortChannelId = message.shortChannelId;
                if (message.direction != null && message.hasOwnProperty("direction"))
                    object.direction = message.direction;
                if (message.channelId != null && message.hasOwnProperty("channelId"))
                    object.channelId = message.channelId;
                if (message.fundingTxid != null && message.hasOwnProperty("fundingTxid"))
                    object.fundingTxid = message.fundingTxid;
                if (message.closeToAddr != null && message.hasOwnProperty("closeToAddr"))
                    object.closeToAddr = message.closeToAddr;
                if (message.closeTo != null && message.hasOwnProperty("closeTo"))
                    object.closeTo = message.closeTo;
                if (message["private"] != null && message.hasOwnProperty("private"))
                    object["private"] = message["private"];
                if (message.total != null && message.hasOwnProperty("total"))
                    object.total = message.total;
                if (message.dustLimit != null && message.hasOwnProperty("dustLimit"))
                    object.dustLimit = message.dustLimit;
                if (message.spendable != null && message.hasOwnProperty("spendable"))
                    object.spendable = message.spendable;
                if (message.receivable != null && message.hasOwnProperty("receivable"))
                    object.receivable = message.receivable;
                if (message.theirToSelfDelay != null && message.hasOwnProperty("theirToSelfDelay"))
                    object.theirToSelfDelay = message.theirToSelfDelay;
                if (message.ourToSelfDelay != null && message.hasOwnProperty("ourToSelfDelay"))
                    object.ourToSelfDelay = message.ourToSelfDelay;
                if (message.status && message.status.length) {
                    object.status = [];
                    for (var j = 0; j < message.status.length; ++j)
                        object.status[j] = message.status[j];
                }
                if (message.htlcs && message.htlcs.length) {
                    object.htlcs = [];
                    for (var j = 0; j < message.htlcs.length; ++j)
                        object.htlcs[j] = $root.greenlight.Htlc.toObject(message.htlcs[j], options);
                }
                return object;
            };
    
            /**
             * Converts this Channel to JSON.
             * @function toJSON
             * @memberof greenlight.Channel
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Channel.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Channel;
        })();
    
        greenlight.Peer = (function() {
    
            /**
             * Properties of a Peer.
             * @memberof greenlight
             * @interface IPeer
             * @property {Uint8Array|null} [id] Peer id
             * @property {boolean|null} [connected] Peer connected
             * @property {Array.<greenlight.IAddress>|null} [addresses] Peer addresses
             * @property {string|null} [features] Peer features
             * @property {Array.<greenlight.IChannel>|null} [channels] Peer channels
             */
    
            /**
             * Constructs a new Peer.
             * @memberof greenlight
             * @classdesc Represents a Peer.
             * @implements IPeer
             * @constructor
             * @param {greenlight.IPeer=} [properties] Properties to set
             */
            function Peer(properties) {
                this.addresses = [];
                this.channels = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Peer id.
             * @member {Uint8Array} id
             * @memberof greenlight.Peer
             * @instance
             */
            Peer.prototype.id = $util.newBuffer([]);
    
            /**
             * Peer connected.
             * @member {boolean} connected
             * @memberof greenlight.Peer
             * @instance
             */
            Peer.prototype.connected = false;
    
            /**
             * Peer addresses.
             * @member {Array.<greenlight.IAddress>} addresses
             * @memberof greenlight.Peer
             * @instance
             */
            Peer.prototype.addresses = $util.emptyArray;
    
            /**
             * Peer features.
             * @member {string} features
             * @memberof greenlight.Peer
             * @instance
             */
            Peer.prototype.features = "";
    
            /**
             * Peer channels.
             * @member {Array.<greenlight.IChannel>} channels
             * @memberof greenlight.Peer
             * @instance
             */
            Peer.prototype.channels = $util.emptyArray;
    
            /**
             * Creates a new Peer instance using the specified properties.
             * @function create
             * @memberof greenlight.Peer
             * @static
             * @param {greenlight.IPeer=} [properties] Properties to set
             * @returns {greenlight.Peer} Peer instance
             */
            Peer.create = function create(properties) {
                return new Peer(properties);
            };
    
            /**
             * Encodes the specified Peer message. Does not implicitly {@link greenlight.Peer.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Peer
             * @static
             * @param {greenlight.IPeer} message Peer message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Peer.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.id != null && Object.hasOwnProperty.call(message, "id"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.id);
                if (message.connected != null && Object.hasOwnProperty.call(message, "connected"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.connected);
                if (message.addresses != null && message.addresses.length)
                    for (var i = 0; i < message.addresses.length; ++i)
                        $root.greenlight.Address.encode(message.addresses[i], writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.features != null && Object.hasOwnProperty.call(message, "features"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.features);
                if (message.channels != null && message.channels.length)
                    for (var i = 0; i < message.channels.length; ++i)
                        $root.greenlight.Channel.encode(message.channels[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified Peer message, length delimited. Does not implicitly {@link greenlight.Peer.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Peer
             * @static
             * @param {greenlight.IPeer} message Peer message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Peer.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Peer message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Peer
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Peer} Peer
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Peer.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Peer();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.id = reader.bytes();
                        break;
                    case 2:
                        message.connected = reader.bool();
                        break;
                    case 3:
                        if (!(message.addresses && message.addresses.length))
                            message.addresses = [];
                        message.addresses.push($root.greenlight.Address.decode(reader, reader.uint32()));
                        break;
                    case 4:
                        message.features = reader.string();
                        break;
                    case 5:
                        if (!(message.channels && message.channels.length))
                            message.channels = [];
                        message.channels.push($root.greenlight.Channel.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Peer message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Peer
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Peer} Peer
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Peer.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Peer message.
             * @function verify
             * @memberof greenlight.Peer
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Peer.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.id != null && message.hasOwnProperty("id"))
                    if (!(message.id && typeof message.id.length === "number" || $util.isString(message.id)))
                        return "id: buffer expected";
                if (message.connected != null && message.hasOwnProperty("connected"))
                    if (typeof message.connected !== "boolean")
                        return "connected: boolean expected";
                if (message.addresses != null && message.hasOwnProperty("addresses")) {
                    if (!Array.isArray(message.addresses))
                        return "addresses: array expected";
                    for (var i = 0; i < message.addresses.length; ++i) {
                        var error = $root.greenlight.Address.verify(message.addresses[i]);
                        if (error)
                            return "addresses." + error;
                    }
                }
                if (message.features != null && message.hasOwnProperty("features"))
                    if (!$util.isString(message.features))
                        return "features: string expected";
                if (message.channels != null && message.hasOwnProperty("channels")) {
                    if (!Array.isArray(message.channels))
                        return "channels: array expected";
                    for (var i = 0; i < message.channels.length; ++i) {
                        var error = $root.greenlight.Channel.verify(message.channels[i]);
                        if (error)
                            return "channels." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a Peer message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Peer
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Peer} Peer
             */
            Peer.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Peer)
                    return object;
                var message = new $root.greenlight.Peer();
                if (object.id != null)
                    if (typeof object.id === "string")
                        $util.base64.decode(object.id, message.id = $util.newBuffer($util.base64.length(object.id)), 0);
                    else if (object.id.length)
                        message.id = object.id;
                if (object.connected != null)
                    message.connected = Boolean(object.connected);
                if (object.addresses) {
                    if (!Array.isArray(object.addresses))
                        throw TypeError(".greenlight.Peer.addresses: array expected");
                    message.addresses = [];
                    for (var i = 0; i < object.addresses.length; ++i) {
                        if (typeof object.addresses[i] !== "object")
                            throw TypeError(".greenlight.Peer.addresses: object expected");
                        message.addresses[i] = $root.greenlight.Address.fromObject(object.addresses[i]);
                    }
                }
                if (object.features != null)
                    message.features = String(object.features);
                if (object.channels) {
                    if (!Array.isArray(object.channels))
                        throw TypeError(".greenlight.Peer.channels: array expected");
                    message.channels = [];
                    for (var i = 0; i < object.channels.length; ++i) {
                        if (typeof object.channels[i] !== "object")
                            throw TypeError(".greenlight.Peer.channels: object expected");
                        message.channels[i] = $root.greenlight.Channel.fromObject(object.channels[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a Peer message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Peer
             * @static
             * @param {greenlight.Peer} message Peer
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Peer.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults) {
                    object.addresses = [];
                    object.channels = [];
                }
                if (options.defaults) {
                    if (options.bytes === String)
                        object.id = "";
                    else {
                        object.id = [];
                        if (options.bytes !== Array)
                            object.id = $util.newBuffer(object.id);
                    }
                    object.connected = false;
                    object.features = "";
                }
                if (message.id != null && message.hasOwnProperty("id"))
                    object.id = options.bytes === String ? $util.base64.encode(message.id, 0, message.id.length) : options.bytes === Array ? Array.prototype.slice.call(message.id) : message.id;
                if (message.connected != null && message.hasOwnProperty("connected"))
                    object.connected = message.connected;
                if (message.addresses && message.addresses.length) {
                    object.addresses = [];
                    for (var j = 0; j < message.addresses.length; ++j)
                        object.addresses[j] = $root.greenlight.Address.toObject(message.addresses[j], options);
                }
                if (message.features != null && message.hasOwnProperty("features"))
                    object.features = message.features;
                if (message.channels && message.channels.length) {
                    object.channels = [];
                    for (var j = 0; j < message.channels.length; ++j)
                        object.channels[j] = $root.greenlight.Channel.toObject(message.channels[j], options);
                }
                return object;
            };
    
            /**
             * Converts this Peer to JSON.
             * @function toJSON
             * @memberof greenlight.Peer
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Peer.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Peer;
        })();
    
        greenlight.ListPeersResponse = (function() {
    
            /**
             * Properties of a ListPeersResponse.
             * @memberof greenlight
             * @interface IListPeersResponse
             * @property {Array.<greenlight.IPeer>|null} [peers] ListPeersResponse peers
             */
    
            /**
             * Constructs a new ListPeersResponse.
             * @memberof greenlight
             * @classdesc Represents a ListPeersResponse.
             * @implements IListPeersResponse
             * @constructor
             * @param {greenlight.IListPeersResponse=} [properties] Properties to set
             */
            function ListPeersResponse(properties) {
                this.peers = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListPeersResponse peers.
             * @member {Array.<greenlight.IPeer>} peers
             * @memberof greenlight.ListPeersResponse
             * @instance
             */
            ListPeersResponse.prototype.peers = $util.emptyArray;
    
            /**
             * Creates a new ListPeersResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {greenlight.IListPeersResponse=} [properties] Properties to set
             * @returns {greenlight.ListPeersResponse} ListPeersResponse instance
             */
            ListPeersResponse.create = function create(properties) {
                return new ListPeersResponse(properties);
            };
    
            /**
             * Encodes the specified ListPeersResponse message. Does not implicitly {@link greenlight.ListPeersResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {greenlight.IListPeersResponse} message ListPeersResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPeersResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.peers != null && message.peers.length)
                    for (var i = 0; i < message.peers.length; ++i)
                        $root.greenlight.Peer.encode(message.peers[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListPeersResponse message, length delimited. Does not implicitly {@link greenlight.ListPeersResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {greenlight.IListPeersResponse} message ListPeersResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPeersResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListPeersResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListPeersResponse} ListPeersResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPeersResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListPeersResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.peers && message.peers.length))
                            message.peers = [];
                        message.peers.push($root.greenlight.Peer.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListPeersResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListPeersResponse} ListPeersResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPeersResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListPeersResponse message.
             * @function verify
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListPeersResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.peers != null && message.hasOwnProperty("peers")) {
                    if (!Array.isArray(message.peers))
                        return "peers: array expected";
                    for (var i = 0; i < message.peers.length; ++i) {
                        var error = $root.greenlight.Peer.verify(message.peers[i]);
                        if (error)
                            return "peers." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a ListPeersResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListPeersResponse} ListPeersResponse
             */
            ListPeersResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListPeersResponse)
                    return object;
                var message = new $root.greenlight.ListPeersResponse();
                if (object.peers) {
                    if (!Array.isArray(object.peers))
                        throw TypeError(".greenlight.ListPeersResponse.peers: array expected");
                    message.peers = [];
                    for (var i = 0; i < object.peers.length; ++i) {
                        if (typeof object.peers[i] !== "object")
                            throw TypeError(".greenlight.ListPeersResponse.peers: object expected");
                        message.peers[i] = $root.greenlight.Peer.fromObject(object.peers[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListPeersResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListPeersResponse
             * @static
             * @param {greenlight.ListPeersResponse} message ListPeersResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListPeersResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.peers = [];
                if (message.peers && message.peers.length) {
                    object.peers = [];
                    for (var j = 0; j < message.peers.length; ++j)
                        object.peers[j] = $root.greenlight.Peer.toObject(message.peers[j], options);
                }
                return object;
            };
    
            /**
             * Converts this ListPeersResponse to JSON.
             * @function toJSON
             * @memberof greenlight.ListPeersResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListPeersResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListPeersResponse;
        })();
    
        greenlight.DisconnectRequest = (function() {
    
            /**
             * Properties of a DisconnectRequest.
             * @memberof greenlight
             * @interface IDisconnectRequest
             * @property {string|null} [nodeId] DisconnectRequest nodeId
             * @property {boolean|null} [force] DisconnectRequest force
             */
    
            /**
             * Constructs a new DisconnectRequest.
             * @memberof greenlight
             * @classdesc Represents a DisconnectRequest.
             * @implements IDisconnectRequest
             * @constructor
             * @param {greenlight.IDisconnectRequest=} [properties] Properties to set
             */
            function DisconnectRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * DisconnectRequest nodeId.
             * @member {string} nodeId
             * @memberof greenlight.DisconnectRequest
             * @instance
             */
            DisconnectRequest.prototype.nodeId = "";
    
            /**
             * DisconnectRequest force.
             * @member {boolean} force
             * @memberof greenlight.DisconnectRequest
             * @instance
             */
            DisconnectRequest.prototype.force = false;
    
            /**
             * Creates a new DisconnectRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {greenlight.IDisconnectRequest=} [properties] Properties to set
             * @returns {greenlight.DisconnectRequest} DisconnectRequest instance
             */
            DisconnectRequest.create = function create(properties) {
                return new DisconnectRequest(properties);
            };
    
            /**
             * Encodes the specified DisconnectRequest message. Does not implicitly {@link greenlight.DisconnectRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {greenlight.IDisconnectRequest} message DisconnectRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DisconnectRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.nodeId);
                if (message.force != null && Object.hasOwnProperty.call(message, "force"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.force);
                return writer;
            };
    
            /**
             * Encodes the specified DisconnectRequest message, length delimited. Does not implicitly {@link greenlight.DisconnectRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {greenlight.IDisconnectRequest} message DisconnectRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DisconnectRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a DisconnectRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.DisconnectRequest} DisconnectRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DisconnectRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.DisconnectRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.string();
                        break;
                    case 2:
                        message.force = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a DisconnectRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.DisconnectRequest} DisconnectRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DisconnectRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a DisconnectRequest message.
             * @function verify
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            DisconnectRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!$util.isString(message.nodeId))
                        return "nodeId: string expected";
                if (message.force != null && message.hasOwnProperty("force"))
                    if (typeof message.force !== "boolean")
                        return "force: boolean expected";
                return null;
            };
    
            /**
             * Creates a DisconnectRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.DisconnectRequest} DisconnectRequest
             */
            DisconnectRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.DisconnectRequest)
                    return object;
                var message = new $root.greenlight.DisconnectRequest();
                if (object.nodeId != null)
                    message.nodeId = String(object.nodeId);
                if (object.force != null)
                    message.force = Boolean(object.force);
                return message;
            };
    
            /**
             * Creates a plain object from a DisconnectRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.DisconnectRequest
             * @static
             * @param {greenlight.DisconnectRequest} message DisconnectRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            DisconnectRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.nodeId = "";
                    object.force = false;
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = message.nodeId;
                if (message.force != null && message.hasOwnProperty("force"))
                    object.force = message.force;
                return object;
            };
    
            /**
             * Converts this DisconnectRequest to JSON.
             * @function toJSON
             * @memberof greenlight.DisconnectRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            DisconnectRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return DisconnectRequest;
        })();
    
        greenlight.DisconnectResponse = (function() {
    
            /**
             * Properties of a DisconnectResponse.
             * @memberof greenlight
             * @interface IDisconnectResponse
             */
    
            /**
             * Constructs a new DisconnectResponse.
             * @memberof greenlight
             * @classdesc Represents a DisconnectResponse.
             * @implements IDisconnectResponse
             * @constructor
             * @param {greenlight.IDisconnectResponse=} [properties] Properties to set
             */
            function DisconnectResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new DisconnectResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {greenlight.IDisconnectResponse=} [properties] Properties to set
             * @returns {greenlight.DisconnectResponse} DisconnectResponse instance
             */
            DisconnectResponse.create = function create(properties) {
                return new DisconnectResponse(properties);
            };
    
            /**
             * Encodes the specified DisconnectResponse message. Does not implicitly {@link greenlight.DisconnectResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {greenlight.IDisconnectResponse} message DisconnectResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DisconnectResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified DisconnectResponse message, length delimited. Does not implicitly {@link greenlight.DisconnectResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {greenlight.IDisconnectResponse} message DisconnectResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            DisconnectResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a DisconnectResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.DisconnectResponse} DisconnectResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DisconnectResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.DisconnectResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a DisconnectResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.DisconnectResponse} DisconnectResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            DisconnectResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a DisconnectResponse message.
             * @function verify
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            DisconnectResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates a DisconnectResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.DisconnectResponse} DisconnectResponse
             */
            DisconnectResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.DisconnectResponse)
                    return object;
                return new $root.greenlight.DisconnectResponse();
            };
    
            /**
             * Creates a plain object from a DisconnectResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.DisconnectResponse
             * @static
             * @param {greenlight.DisconnectResponse} message DisconnectResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            DisconnectResponse.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this DisconnectResponse to JSON.
             * @function toJSON
             * @memberof greenlight.DisconnectResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            DisconnectResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return DisconnectResponse;
        })();
    
        /**
         * BtcAddressType enum.
         * @name greenlight.BtcAddressType
         * @enum {number}
         * @property {number} BECH32=0 BECH32 value
         * @property {number} P2SH_SEGWIT=1 P2SH_SEGWIT value
         */
        greenlight.BtcAddressType = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "BECH32"] = 0;
            values[valuesById[1] = "P2SH_SEGWIT"] = 1;
            return values;
        })();
    
        greenlight.NewAddrRequest = (function() {
    
            /**
             * Properties of a NewAddrRequest.
             * @memberof greenlight
             * @interface INewAddrRequest
             * @property {greenlight.BtcAddressType|null} [addressType] NewAddrRequest addressType
             */
    
            /**
             * Constructs a new NewAddrRequest.
             * @memberof greenlight
             * @classdesc Represents a NewAddrRequest.
             * @implements INewAddrRequest
             * @constructor
             * @param {greenlight.INewAddrRequest=} [properties] Properties to set
             */
            function NewAddrRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * NewAddrRequest addressType.
             * @member {greenlight.BtcAddressType} addressType
             * @memberof greenlight.NewAddrRequest
             * @instance
             */
            NewAddrRequest.prototype.addressType = 0;
    
            /**
             * Creates a new NewAddrRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {greenlight.INewAddrRequest=} [properties] Properties to set
             * @returns {greenlight.NewAddrRequest} NewAddrRequest instance
             */
            NewAddrRequest.create = function create(properties) {
                return new NewAddrRequest(properties);
            };
    
            /**
             * Encodes the specified NewAddrRequest message. Does not implicitly {@link greenlight.NewAddrRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {greenlight.INewAddrRequest} message NewAddrRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NewAddrRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.addressType != null && Object.hasOwnProperty.call(message, "addressType"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.addressType);
                return writer;
            };
    
            /**
             * Encodes the specified NewAddrRequest message, length delimited. Does not implicitly {@link greenlight.NewAddrRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {greenlight.INewAddrRequest} message NewAddrRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NewAddrRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a NewAddrRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.NewAddrRequest} NewAddrRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NewAddrRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.NewAddrRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.addressType = reader.int32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a NewAddrRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.NewAddrRequest} NewAddrRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NewAddrRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a NewAddrRequest message.
             * @function verify
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            NewAddrRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.addressType != null && message.hasOwnProperty("addressType"))
                    switch (message.addressType) {
                    default:
                        return "addressType: enum value expected";
                    case 0:
                    case 1:
                        break;
                    }
                return null;
            };
    
            /**
             * Creates a NewAddrRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.NewAddrRequest} NewAddrRequest
             */
            NewAddrRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.NewAddrRequest)
                    return object;
                var message = new $root.greenlight.NewAddrRequest();
                switch (object.addressType) {
                case "BECH32":
                case 0:
                    message.addressType = 0;
                    break;
                case "P2SH_SEGWIT":
                case 1:
                    message.addressType = 1;
                    break;
                }
                return message;
            };
    
            /**
             * Creates a plain object from a NewAddrRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.NewAddrRequest
             * @static
             * @param {greenlight.NewAddrRequest} message NewAddrRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            NewAddrRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.addressType = options.enums === String ? "BECH32" : 0;
                if (message.addressType != null && message.hasOwnProperty("addressType"))
                    object.addressType = options.enums === String ? $root.greenlight.BtcAddressType[message.addressType] : message.addressType;
                return object;
            };
    
            /**
             * Converts this NewAddrRequest to JSON.
             * @function toJSON
             * @memberof greenlight.NewAddrRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            NewAddrRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return NewAddrRequest;
        })();
    
        greenlight.NewAddrResponse = (function() {
    
            /**
             * Properties of a NewAddrResponse.
             * @memberof greenlight
             * @interface INewAddrResponse
             * @property {greenlight.BtcAddressType|null} [addressType] NewAddrResponse addressType
             * @property {string|null} [address] NewAddrResponse address
             */
    
            /**
             * Constructs a new NewAddrResponse.
             * @memberof greenlight
             * @classdesc Represents a NewAddrResponse.
             * @implements INewAddrResponse
             * @constructor
             * @param {greenlight.INewAddrResponse=} [properties] Properties to set
             */
            function NewAddrResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * NewAddrResponse addressType.
             * @member {greenlight.BtcAddressType} addressType
             * @memberof greenlight.NewAddrResponse
             * @instance
             */
            NewAddrResponse.prototype.addressType = 0;
    
            /**
             * NewAddrResponse address.
             * @member {string} address
             * @memberof greenlight.NewAddrResponse
             * @instance
             */
            NewAddrResponse.prototype.address = "";
    
            /**
             * Creates a new NewAddrResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {greenlight.INewAddrResponse=} [properties] Properties to set
             * @returns {greenlight.NewAddrResponse} NewAddrResponse instance
             */
            NewAddrResponse.create = function create(properties) {
                return new NewAddrResponse(properties);
            };
    
            /**
             * Encodes the specified NewAddrResponse message. Does not implicitly {@link greenlight.NewAddrResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {greenlight.INewAddrResponse} message NewAddrResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NewAddrResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.addressType != null && Object.hasOwnProperty.call(message, "addressType"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.addressType);
                if (message.address != null && Object.hasOwnProperty.call(message, "address"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.address);
                return writer;
            };
    
            /**
             * Encodes the specified NewAddrResponse message, length delimited. Does not implicitly {@link greenlight.NewAddrResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {greenlight.INewAddrResponse} message NewAddrResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            NewAddrResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a NewAddrResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.NewAddrResponse} NewAddrResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NewAddrResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.NewAddrResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.addressType = reader.int32();
                        break;
                    case 2:
                        message.address = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a NewAddrResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.NewAddrResponse} NewAddrResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            NewAddrResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a NewAddrResponse message.
             * @function verify
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            NewAddrResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.addressType != null && message.hasOwnProperty("addressType"))
                    switch (message.addressType) {
                    default:
                        return "addressType: enum value expected";
                    case 0:
                    case 1:
                        break;
                    }
                if (message.address != null && message.hasOwnProperty("address"))
                    if (!$util.isString(message.address))
                        return "address: string expected";
                return null;
            };
    
            /**
             * Creates a NewAddrResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.NewAddrResponse} NewAddrResponse
             */
            NewAddrResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.NewAddrResponse)
                    return object;
                var message = new $root.greenlight.NewAddrResponse();
                switch (object.addressType) {
                case "BECH32":
                case 0:
                    message.addressType = 0;
                    break;
                case "P2SH_SEGWIT":
                case 1:
                    message.addressType = 1;
                    break;
                }
                if (object.address != null)
                    message.address = String(object.address);
                return message;
            };
    
            /**
             * Creates a plain object from a NewAddrResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.NewAddrResponse
             * @static
             * @param {greenlight.NewAddrResponse} message NewAddrResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            NewAddrResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.addressType = options.enums === String ? "BECH32" : 0;
                    object.address = "";
                }
                if (message.addressType != null && message.hasOwnProperty("addressType"))
                    object.addressType = options.enums === String ? $root.greenlight.BtcAddressType[message.addressType] : message.addressType;
                if (message.address != null && message.hasOwnProperty("address"))
                    object.address = message.address;
                return object;
            };
    
            /**
             * Converts this NewAddrResponse to JSON.
             * @function toJSON
             * @memberof greenlight.NewAddrResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            NewAddrResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return NewAddrResponse;
        })();
    
        greenlight.ListFundsRequest = (function() {
    
            /**
             * Properties of a ListFundsRequest.
             * @memberof greenlight
             * @interface IListFundsRequest
             * @property {greenlight.IConfirmation|null} [minconf] ListFundsRequest minconf
             */
    
            /**
             * Constructs a new ListFundsRequest.
             * @memberof greenlight
             * @classdesc Represents a ListFundsRequest.
             * @implements IListFundsRequest
             * @constructor
             * @param {greenlight.IListFundsRequest=} [properties] Properties to set
             */
            function ListFundsRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListFundsRequest minconf.
             * @member {greenlight.IConfirmation|null|undefined} minconf
             * @memberof greenlight.ListFundsRequest
             * @instance
             */
            ListFundsRequest.prototype.minconf = null;
    
            /**
             * Creates a new ListFundsRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {greenlight.IListFundsRequest=} [properties] Properties to set
             * @returns {greenlight.ListFundsRequest} ListFundsRequest instance
             */
            ListFundsRequest.create = function create(properties) {
                return new ListFundsRequest(properties);
            };
    
            /**
             * Encodes the specified ListFundsRequest message. Does not implicitly {@link greenlight.ListFundsRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {greenlight.IListFundsRequest} message ListFundsRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.minconf != null && Object.hasOwnProperty.call(message, "minconf"))
                    $root.greenlight.Confirmation.encode(message.minconf, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListFundsRequest message, length delimited. Does not implicitly {@link greenlight.ListFundsRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {greenlight.IListFundsRequest} message ListFundsRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListFundsRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListFundsRequest} ListFundsRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListFundsRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.minconf = $root.greenlight.Confirmation.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListFundsRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListFundsRequest} ListFundsRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListFundsRequest message.
             * @function verify
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListFundsRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.minconf != null && message.hasOwnProperty("minconf")) {
                    var error = $root.greenlight.Confirmation.verify(message.minconf);
                    if (error)
                        return "minconf." + error;
                }
                return null;
            };
    
            /**
             * Creates a ListFundsRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListFundsRequest} ListFundsRequest
             */
            ListFundsRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListFundsRequest)
                    return object;
                var message = new $root.greenlight.ListFundsRequest();
                if (object.minconf != null) {
                    if (typeof object.minconf !== "object")
                        throw TypeError(".greenlight.ListFundsRequest.minconf: object expected");
                    message.minconf = $root.greenlight.Confirmation.fromObject(object.minconf);
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListFundsRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListFundsRequest
             * @static
             * @param {greenlight.ListFundsRequest} message ListFundsRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListFundsRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.minconf = null;
                if (message.minconf != null && message.hasOwnProperty("minconf"))
                    object.minconf = $root.greenlight.Confirmation.toObject(message.minconf, options);
                return object;
            };
    
            /**
             * Converts this ListFundsRequest to JSON.
             * @function toJSON
             * @memberof greenlight.ListFundsRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListFundsRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListFundsRequest;
        })();
    
        /**
         * OutputStatus enum.
         * @name greenlight.OutputStatus
         * @enum {number}
         * @property {number} CONFIRMED=0 CONFIRMED value
         * @property {number} UNCONFIRMED=1 UNCONFIRMED value
         */
        greenlight.OutputStatus = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "CONFIRMED"] = 0;
            values[valuesById[1] = "UNCONFIRMED"] = 1;
            return values;
        })();
    
        greenlight.ListFundsOutput = (function() {
    
            /**
             * Properties of a ListFundsOutput.
             * @memberof greenlight
             * @interface IListFundsOutput
             * @property {greenlight.IOutpoint|null} [output] ListFundsOutput output
             * @property {greenlight.IAmount|null} [amount] ListFundsOutput amount
             * @property {string|null} [address] ListFundsOutput address
             * @property {greenlight.OutputStatus|null} [status] ListFundsOutput status
             */
    
            /**
             * Constructs a new ListFundsOutput.
             * @memberof greenlight
             * @classdesc Represents a ListFundsOutput.
             * @implements IListFundsOutput
             * @constructor
             * @param {greenlight.IListFundsOutput=} [properties] Properties to set
             */
            function ListFundsOutput(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListFundsOutput output.
             * @member {greenlight.IOutpoint|null|undefined} output
             * @memberof greenlight.ListFundsOutput
             * @instance
             */
            ListFundsOutput.prototype.output = null;
    
            /**
             * ListFundsOutput amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.ListFundsOutput
             * @instance
             */
            ListFundsOutput.prototype.amount = null;
    
            /**
             * ListFundsOutput address.
             * @member {string} address
             * @memberof greenlight.ListFundsOutput
             * @instance
             */
            ListFundsOutput.prototype.address = "";
    
            /**
             * ListFundsOutput status.
             * @member {greenlight.OutputStatus} status
             * @memberof greenlight.ListFundsOutput
             * @instance
             */
            ListFundsOutput.prototype.status = 0;
    
            /**
             * Creates a new ListFundsOutput instance using the specified properties.
             * @function create
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {greenlight.IListFundsOutput=} [properties] Properties to set
             * @returns {greenlight.ListFundsOutput} ListFundsOutput instance
             */
            ListFundsOutput.create = function create(properties) {
                return new ListFundsOutput(properties);
            };
    
            /**
             * Encodes the specified ListFundsOutput message. Does not implicitly {@link greenlight.ListFundsOutput.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {greenlight.IListFundsOutput} message ListFundsOutput message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsOutput.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.output != null && Object.hasOwnProperty.call(message, "output"))
                    $root.greenlight.Outpoint.encode(message.output, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.address != null && Object.hasOwnProperty.call(message, "address"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.address);
                if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                    writer.uint32(/* id 5, wireType 0 =*/40).int32(message.status);
                return writer;
            };
    
            /**
             * Encodes the specified ListFundsOutput message, length delimited. Does not implicitly {@link greenlight.ListFundsOutput.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {greenlight.IListFundsOutput} message ListFundsOutput message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsOutput.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListFundsOutput message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListFundsOutput} ListFundsOutput
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsOutput.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListFundsOutput();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.output = $root.greenlight.Outpoint.decode(reader, reader.uint32());
                        break;
                    case 2:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.address = reader.string();
                        break;
                    case 5:
                        message.status = reader.int32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListFundsOutput message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListFundsOutput} ListFundsOutput
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsOutput.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListFundsOutput message.
             * @function verify
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListFundsOutput.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.output != null && message.hasOwnProperty("output")) {
                    var error = $root.greenlight.Outpoint.verify(message.output);
                    if (error)
                        return "output." + error;
                }
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.address != null && message.hasOwnProperty("address"))
                    if (!$util.isString(message.address))
                        return "address: string expected";
                if (message.status != null && message.hasOwnProperty("status"))
                    switch (message.status) {
                    default:
                        return "status: enum value expected";
                    case 0:
                    case 1:
                        break;
                    }
                return null;
            };
    
            /**
             * Creates a ListFundsOutput message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListFundsOutput} ListFundsOutput
             */
            ListFundsOutput.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListFundsOutput)
                    return object;
                var message = new $root.greenlight.ListFundsOutput();
                if (object.output != null) {
                    if (typeof object.output !== "object")
                        throw TypeError(".greenlight.ListFundsOutput.output: object expected");
                    message.output = $root.greenlight.Outpoint.fromObject(object.output);
                }
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.ListFundsOutput.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.address != null)
                    message.address = String(object.address);
                switch (object.status) {
                case "CONFIRMED":
                case 0:
                    message.status = 0;
                    break;
                case "UNCONFIRMED":
                case 1:
                    message.status = 1;
                    break;
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListFundsOutput message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListFundsOutput
             * @static
             * @param {greenlight.ListFundsOutput} message ListFundsOutput
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListFundsOutput.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.output = null;
                    object.amount = null;
                    object.address = "";
                    object.status = options.enums === String ? "CONFIRMED" : 0;
                }
                if (message.output != null && message.hasOwnProperty("output"))
                    object.output = $root.greenlight.Outpoint.toObject(message.output, options);
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.address != null && message.hasOwnProperty("address"))
                    object.address = message.address;
                if (message.status != null && message.hasOwnProperty("status"))
                    object.status = options.enums === String ? $root.greenlight.OutputStatus[message.status] : message.status;
                return object;
            };
    
            /**
             * Converts this ListFundsOutput to JSON.
             * @function toJSON
             * @memberof greenlight.ListFundsOutput
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListFundsOutput.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListFundsOutput;
        })();
    
        greenlight.ListFundsChannel = (function() {
    
            /**
             * Properties of a ListFundsChannel.
             * @memberof greenlight
             * @interface IListFundsChannel
             * @property {Uint8Array|null} [peerId] ListFundsChannel peerId
             * @property {boolean|null} [connected] ListFundsChannel connected
             * @property {number|Long|null} [shortChannelId] ListFundsChannel shortChannelId
             * @property {number|Long|null} [ourAmountMsat] ListFundsChannel ourAmountMsat
             * @property {number|Long|null} [amountMsat] ListFundsChannel amountMsat
             * @property {Uint8Array|null} [fundingTxid] ListFundsChannel fundingTxid
             * @property {number|null} [fundingOutput] ListFundsChannel fundingOutput
             */
    
            /**
             * Constructs a new ListFundsChannel.
             * @memberof greenlight
             * @classdesc Represents a ListFundsChannel.
             * @implements IListFundsChannel
             * @constructor
             * @param {greenlight.IListFundsChannel=} [properties] Properties to set
             */
            function ListFundsChannel(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListFundsChannel peerId.
             * @member {Uint8Array} peerId
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.peerId = $util.newBuffer([]);
    
            /**
             * ListFundsChannel connected.
             * @member {boolean} connected
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.connected = false;
    
            /**
             * ListFundsChannel shortChannelId.
             * @member {number|Long} shortChannelId
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.shortChannelId = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * ListFundsChannel ourAmountMsat.
             * @member {number|Long} ourAmountMsat
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.ourAmountMsat = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * ListFundsChannel amountMsat.
             * @member {number|Long} amountMsat
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.amountMsat = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * ListFundsChannel fundingTxid.
             * @member {Uint8Array} fundingTxid
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.fundingTxid = $util.newBuffer([]);
    
            /**
             * ListFundsChannel fundingOutput.
             * @member {number} fundingOutput
             * @memberof greenlight.ListFundsChannel
             * @instance
             */
            ListFundsChannel.prototype.fundingOutput = 0;
    
            /**
             * Creates a new ListFundsChannel instance using the specified properties.
             * @function create
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {greenlight.IListFundsChannel=} [properties] Properties to set
             * @returns {greenlight.ListFundsChannel} ListFundsChannel instance
             */
            ListFundsChannel.create = function create(properties) {
                return new ListFundsChannel(properties);
            };
    
            /**
             * Encodes the specified ListFundsChannel message. Does not implicitly {@link greenlight.ListFundsChannel.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {greenlight.IListFundsChannel} message ListFundsChannel message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsChannel.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.peerId != null && Object.hasOwnProperty.call(message, "peerId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.peerId);
                if (message.connected != null && Object.hasOwnProperty.call(message, "connected"))
                    writer.uint32(/* id 2, wireType 0 =*/16).bool(message.connected);
                if (message.shortChannelId != null && Object.hasOwnProperty.call(message, "shortChannelId"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.shortChannelId);
                if (message.ourAmountMsat != null && Object.hasOwnProperty.call(message, "ourAmountMsat"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint64(message.ourAmountMsat);
                if (message.amountMsat != null && Object.hasOwnProperty.call(message, "amountMsat"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.amountMsat);
                if (message.fundingTxid != null && Object.hasOwnProperty.call(message, "fundingTxid"))
                    writer.uint32(/* id 6, wireType 2 =*/50).bytes(message.fundingTxid);
                if (message.fundingOutput != null && Object.hasOwnProperty.call(message, "fundingOutput"))
                    writer.uint32(/* id 7, wireType 0 =*/56).uint32(message.fundingOutput);
                return writer;
            };
    
            /**
             * Encodes the specified ListFundsChannel message, length delimited. Does not implicitly {@link greenlight.ListFundsChannel.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {greenlight.IListFundsChannel} message ListFundsChannel message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsChannel.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListFundsChannel message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListFundsChannel} ListFundsChannel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsChannel.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListFundsChannel();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.peerId = reader.bytes();
                        break;
                    case 2:
                        message.connected = reader.bool();
                        break;
                    case 3:
                        message.shortChannelId = reader.uint64();
                        break;
                    case 4:
                        message.ourAmountMsat = reader.uint64();
                        break;
                    case 5:
                        message.amountMsat = reader.uint64();
                        break;
                    case 6:
                        message.fundingTxid = reader.bytes();
                        break;
                    case 7:
                        message.fundingOutput = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListFundsChannel message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListFundsChannel} ListFundsChannel
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsChannel.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListFundsChannel message.
             * @function verify
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListFundsChannel.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.peerId != null && message.hasOwnProperty("peerId"))
                    if (!(message.peerId && typeof message.peerId.length === "number" || $util.isString(message.peerId)))
                        return "peerId: buffer expected";
                if (message.connected != null && message.hasOwnProperty("connected"))
                    if (typeof message.connected !== "boolean")
                        return "connected: boolean expected";
                if (message.shortChannelId != null && message.hasOwnProperty("shortChannelId"))
                    if (!$util.isInteger(message.shortChannelId) && !(message.shortChannelId && $util.isInteger(message.shortChannelId.low) && $util.isInteger(message.shortChannelId.high)))
                        return "shortChannelId: integer|Long expected";
                if (message.ourAmountMsat != null && message.hasOwnProperty("ourAmountMsat"))
                    if (!$util.isInteger(message.ourAmountMsat) && !(message.ourAmountMsat && $util.isInteger(message.ourAmountMsat.low) && $util.isInteger(message.ourAmountMsat.high)))
                        return "ourAmountMsat: integer|Long expected";
                if (message.amountMsat != null && message.hasOwnProperty("amountMsat"))
                    if (!$util.isInteger(message.amountMsat) && !(message.amountMsat && $util.isInteger(message.amountMsat.low) && $util.isInteger(message.amountMsat.high)))
                        return "amountMsat: integer|Long expected";
                if (message.fundingTxid != null && message.hasOwnProperty("fundingTxid"))
                    if (!(message.fundingTxid && typeof message.fundingTxid.length === "number" || $util.isString(message.fundingTxid)))
                        return "fundingTxid: buffer expected";
                if (message.fundingOutput != null && message.hasOwnProperty("fundingOutput"))
                    if (!$util.isInteger(message.fundingOutput))
                        return "fundingOutput: integer expected";
                return null;
            };
    
            /**
             * Creates a ListFundsChannel message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListFundsChannel} ListFundsChannel
             */
            ListFundsChannel.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListFundsChannel)
                    return object;
                var message = new $root.greenlight.ListFundsChannel();
                if (object.peerId != null)
                    if (typeof object.peerId === "string")
                        $util.base64.decode(object.peerId, message.peerId = $util.newBuffer($util.base64.length(object.peerId)), 0);
                    else if (object.peerId.length)
                        message.peerId = object.peerId;
                if (object.connected != null)
                    message.connected = Boolean(object.connected);
                if (object.shortChannelId != null)
                    if ($util.Long)
                        (message.shortChannelId = $util.Long.fromValue(object.shortChannelId)).unsigned = true;
                    else if (typeof object.shortChannelId === "string")
                        message.shortChannelId = parseInt(object.shortChannelId, 10);
                    else if (typeof object.shortChannelId === "number")
                        message.shortChannelId = object.shortChannelId;
                    else if (typeof object.shortChannelId === "object")
                        message.shortChannelId = new $util.LongBits(object.shortChannelId.low >>> 0, object.shortChannelId.high >>> 0).toNumber(true);
                if (object.ourAmountMsat != null)
                    if ($util.Long)
                        (message.ourAmountMsat = $util.Long.fromValue(object.ourAmountMsat)).unsigned = true;
                    else if (typeof object.ourAmountMsat === "string")
                        message.ourAmountMsat = parseInt(object.ourAmountMsat, 10);
                    else if (typeof object.ourAmountMsat === "number")
                        message.ourAmountMsat = object.ourAmountMsat;
                    else if (typeof object.ourAmountMsat === "object")
                        message.ourAmountMsat = new $util.LongBits(object.ourAmountMsat.low >>> 0, object.ourAmountMsat.high >>> 0).toNumber(true);
                if (object.amountMsat != null)
                    if ($util.Long)
                        (message.amountMsat = $util.Long.fromValue(object.amountMsat)).unsigned = true;
                    else if (typeof object.amountMsat === "string")
                        message.amountMsat = parseInt(object.amountMsat, 10);
                    else if (typeof object.amountMsat === "number")
                        message.amountMsat = object.amountMsat;
                    else if (typeof object.amountMsat === "object")
                        message.amountMsat = new $util.LongBits(object.amountMsat.low >>> 0, object.amountMsat.high >>> 0).toNumber(true);
                if (object.fundingTxid != null)
                    if (typeof object.fundingTxid === "string")
                        $util.base64.decode(object.fundingTxid, message.fundingTxid = $util.newBuffer($util.base64.length(object.fundingTxid)), 0);
                    else if (object.fundingTxid.length)
                        message.fundingTxid = object.fundingTxid;
                if (object.fundingOutput != null)
                    message.fundingOutput = object.fundingOutput >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from a ListFundsChannel message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListFundsChannel
             * @static
             * @param {greenlight.ListFundsChannel} message ListFundsChannel
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListFundsChannel.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.peerId = "";
                    else {
                        object.peerId = [];
                        if (options.bytes !== Array)
                            object.peerId = $util.newBuffer(object.peerId);
                    }
                    object.connected = false;
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.shortChannelId = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.shortChannelId = options.longs === String ? "0" : 0;
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.ourAmountMsat = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.ourAmountMsat = options.longs === String ? "0" : 0;
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.amountMsat = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.amountMsat = options.longs === String ? "0" : 0;
                    if (options.bytes === String)
                        object.fundingTxid = "";
                    else {
                        object.fundingTxid = [];
                        if (options.bytes !== Array)
                            object.fundingTxid = $util.newBuffer(object.fundingTxid);
                    }
                    object.fundingOutput = 0;
                }
                if (message.peerId != null && message.hasOwnProperty("peerId"))
                    object.peerId = options.bytes === String ? $util.base64.encode(message.peerId, 0, message.peerId.length) : options.bytes === Array ? Array.prototype.slice.call(message.peerId) : message.peerId;
                if (message.connected != null && message.hasOwnProperty("connected"))
                    object.connected = message.connected;
                if (message.shortChannelId != null && message.hasOwnProperty("shortChannelId"))
                    if (typeof message.shortChannelId === "number")
                        object.shortChannelId = options.longs === String ? String(message.shortChannelId) : message.shortChannelId;
                    else
                        object.shortChannelId = options.longs === String ? $util.Long.prototype.toString.call(message.shortChannelId) : options.longs === Number ? new $util.LongBits(message.shortChannelId.low >>> 0, message.shortChannelId.high >>> 0).toNumber(true) : message.shortChannelId;
                if (message.ourAmountMsat != null && message.hasOwnProperty("ourAmountMsat"))
                    if (typeof message.ourAmountMsat === "number")
                        object.ourAmountMsat = options.longs === String ? String(message.ourAmountMsat) : message.ourAmountMsat;
                    else
                        object.ourAmountMsat = options.longs === String ? $util.Long.prototype.toString.call(message.ourAmountMsat) : options.longs === Number ? new $util.LongBits(message.ourAmountMsat.low >>> 0, message.ourAmountMsat.high >>> 0).toNumber(true) : message.ourAmountMsat;
                if (message.amountMsat != null && message.hasOwnProperty("amountMsat"))
                    if (typeof message.amountMsat === "number")
                        object.amountMsat = options.longs === String ? String(message.amountMsat) : message.amountMsat;
                    else
                        object.amountMsat = options.longs === String ? $util.Long.prototype.toString.call(message.amountMsat) : options.longs === Number ? new $util.LongBits(message.amountMsat.low >>> 0, message.amountMsat.high >>> 0).toNumber(true) : message.amountMsat;
                if (message.fundingTxid != null && message.hasOwnProperty("fundingTxid"))
                    object.fundingTxid = options.bytes === String ? $util.base64.encode(message.fundingTxid, 0, message.fundingTxid.length) : options.bytes === Array ? Array.prototype.slice.call(message.fundingTxid) : message.fundingTxid;
                if (message.fundingOutput != null && message.hasOwnProperty("fundingOutput"))
                    object.fundingOutput = message.fundingOutput;
                return object;
            };
    
            /**
             * Converts this ListFundsChannel to JSON.
             * @function toJSON
             * @memberof greenlight.ListFundsChannel
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListFundsChannel.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListFundsChannel;
        })();
    
        greenlight.ListFundsResponse = (function() {
    
            /**
             * Properties of a ListFundsResponse.
             * @memberof greenlight
             * @interface IListFundsResponse
             * @property {Array.<greenlight.IListFundsOutput>|null} [outputs] ListFundsResponse outputs
             * @property {Array.<greenlight.IListFundsChannel>|null} [channels] ListFundsResponse channels
             */
    
            /**
             * Constructs a new ListFundsResponse.
             * @memberof greenlight
             * @classdesc Represents a ListFundsResponse.
             * @implements IListFundsResponse
             * @constructor
             * @param {greenlight.IListFundsResponse=} [properties] Properties to set
             */
            function ListFundsResponse(properties) {
                this.outputs = [];
                this.channels = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListFundsResponse outputs.
             * @member {Array.<greenlight.IListFundsOutput>} outputs
             * @memberof greenlight.ListFundsResponse
             * @instance
             */
            ListFundsResponse.prototype.outputs = $util.emptyArray;
    
            /**
             * ListFundsResponse channels.
             * @member {Array.<greenlight.IListFundsChannel>} channels
             * @memberof greenlight.ListFundsResponse
             * @instance
             */
            ListFundsResponse.prototype.channels = $util.emptyArray;
    
            /**
             * Creates a new ListFundsResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {greenlight.IListFundsResponse=} [properties] Properties to set
             * @returns {greenlight.ListFundsResponse} ListFundsResponse instance
             */
            ListFundsResponse.create = function create(properties) {
                return new ListFundsResponse(properties);
            };
    
            /**
             * Encodes the specified ListFundsResponse message. Does not implicitly {@link greenlight.ListFundsResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {greenlight.IListFundsResponse} message ListFundsResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.outputs != null && message.outputs.length)
                    for (var i = 0; i < message.outputs.length; ++i)
                        $root.greenlight.ListFundsOutput.encode(message.outputs[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.channels != null && message.channels.length)
                    for (var i = 0; i < message.channels.length; ++i)
                        $root.greenlight.ListFundsChannel.encode(message.channels[i], writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListFundsResponse message, length delimited. Does not implicitly {@link greenlight.ListFundsResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {greenlight.IListFundsResponse} message ListFundsResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListFundsResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListFundsResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListFundsResponse} ListFundsResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListFundsResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.outputs && message.outputs.length))
                            message.outputs = [];
                        message.outputs.push($root.greenlight.ListFundsOutput.decode(reader, reader.uint32()));
                        break;
                    case 2:
                        if (!(message.channels && message.channels.length))
                            message.channels = [];
                        message.channels.push($root.greenlight.ListFundsChannel.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListFundsResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListFundsResponse} ListFundsResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListFundsResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListFundsResponse message.
             * @function verify
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListFundsResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.outputs != null && message.hasOwnProperty("outputs")) {
                    if (!Array.isArray(message.outputs))
                        return "outputs: array expected";
                    for (var i = 0; i < message.outputs.length; ++i) {
                        var error = $root.greenlight.ListFundsOutput.verify(message.outputs[i]);
                        if (error)
                            return "outputs." + error;
                    }
                }
                if (message.channels != null && message.hasOwnProperty("channels")) {
                    if (!Array.isArray(message.channels))
                        return "channels: array expected";
                    for (var i = 0; i < message.channels.length; ++i) {
                        var error = $root.greenlight.ListFundsChannel.verify(message.channels[i]);
                        if (error)
                            return "channels." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a ListFundsResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListFundsResponse} ListFundsResponse
             */
            ListFundsResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListFundsResponse)
                    return object;
                var message = new $root.greenlight.ListFundsResponse();
                if (object.outputs) {
                    if (!Array.isArray(object.outputs))
                        throw TypeError(".greenlight.ListFundsResponse.outputs: array expected");
                    message.outputs = [];
                    for (var i = 0; i < object.outputs.length; ++i) {
                        if (typeof object.outputs[i] !== "object")
                            throw TypeError(".greenlight.ListFundsResponse.outputs: object expected");
                        message.outputs[i] = $root.greenlight.ListFundsOutput.fromObject(object.outputs[i]);
                    }
                }
                if (object.channels) {
                    if (!Array.isArray(object.channels))
                        throw TypeError(".greenlight.ListFundsResponse.channels: array expected");
                    message.channels = [];
                    for (var i = 0; i < object.channels.length; ++i) {
                        if (typeof object.channels[i] !== "object")
                            throw TypeError(".greenlight.ListFundsResponse.channels: object expected");
                        message.channels[i] = $root.greenlight.ListFundsChannel.fromObject(object.channels[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListFundsResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListFundsResponse
             * @static
             * @param {greenlight.ListFundsResponse} message ListFundsResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListFundsResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults) {
                    object.outputs = [];
                    object.channels = [];
                }
                if (message.outputs && message.outputs.length) {
                    object.outputs = [];
                    for (var j = 0; j < message.outputs.length; ++j)
                        object.outputs[j] = $root.greenlight.ListFundsOutput.toObject(message.outputs[j], options);
                }
                if (message.channels && message.channels.length) {
                    object.channels = [];
                    for (var j = 0; j < message.channels.length; ++j)
                        object.channels[j] = $root.greenlight.ListFundsChannel.toObject(message.channels[j], options);
                }
                return object;
            };
    
            /**
             * Converts this ListFundsResponse to JSON.
             * @function toJSON
             * @memberof greenlight.ListFundsResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListFundsResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListFundsResponse;
        })();
    
        /**
         * FeeratePreset enum.
         * @name greenlight.FeeratePreset
         * @enum {number}
         * @property {number} NORMAL=0 NORMAL value
         * @property {number} SLOW=1 SLOW value
         * @property {number} URGENT=2 URGENT value
         */
        greenlight.FeeratePreset = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "NORMAL"] = 0;
            values[valuesById[1] = "SLOW"] = 1;
            values[valuesById[2] = "URGENT"] = 2;
            return values;
        })();
    
        greenlight.Feerate = (function() {
    
            /**
             * Properties of a Feerate.
             * @memberof greenlight
             * @interface IFeerate
             * @property {greenlight.FeeratePreset|null} [preset] Feerate preset
             * @property {number|Long|null} [perkw] Feerate perkw
             * @property {number|Long|null} [perkb] Feerate perkb
             */
    
            /**
             * Constructs a new Feerate.
             * @memberof greenlight
             * @classdesc Represents a Feerate.
             * @implements IFeerate
             * @constructor
             * @param {greenlight.IFeerate=} [properties] Properties to set
             */
            function Feerate(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Feerate preset.
             * @member {greenlight.FeeratePreset|null|undefined} preset
             * @memberof greenlight.Feerate
             * @instance
             */
            Feerate.prototype.preset = null;
    
            /**
             * Feerate perkw.
             * @member {number|Long|null|undefined} perkw
             * @memberof greenlight.Feerate
             * @instance
             */
            Feerate.prototype.perkw = null;
    
            /**
             * Feerate perkb.
             * @member {number|Long|null|undefined} perkb
             * @memberof greenlight.Feerate
             * @instance
             */
            Feerate.prototype.perkb = null;
    
            // OneOf field names bound to virtual getters and setters
            var $oneOfFields;
    
            /**
             * Feerate value.
             * @member {"preset"|"perkw"|"perkb"|undefined} value
             * @memberof greenlight.Feerate
             * @instance
             */
            Object.defineProperty(Feerate.prototype, "value", {
                get: $util.oneOfGetter($oneOfFields = ["preset", "perkw", "perkb"]),
                set: $util.oneOfSetter($oneOfFields)
            });
    
            /**
             * Creates a new Feerate instance using the specified properties.
             * @function create
             * @memberof greenlight.Feerate
             * @static
             * @param {greenlight.IFeerate=} [properties] Properties to set
             * @returns {greenlight.Feerate} Feerate instance
             */
            Feerate.create = function create(properties) {
                return new Feerate(properties);
            };
    
            /**
             * Encodes the specified Feerate message. Does not implicitly {@link greenlight.Feerate.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Feerate
             * @static
             * @param {greenlight.IFeerate} message Feerate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Feerate.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.preset != null && Object.hasOwnProperty.call(message, "preset"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.preset);
                if (message.perkw != null && Object.hasOwnProperty.call(message, "perkw"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint64(message.perkw);
                if (message.perkb != null && Object.hasOwnProperty.call(message, "perkb"))
                    writer.uint32(/* id 6, wireType 0 =*/48).uint64(message.perkb);
                return writer;
            };
    
            /**
             * Encodes the specified Feerate message, length delimited. Does not implicitly {@link greenlight.Feerate.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Feerate
             * @static
             * @param {greenlight.IFeerate} message Feerate message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Feerate.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Feerate message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Feerate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Feerate} Feerate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Feerate.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Feerate();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.preset = reader.int32();
                        break;
                    case 5:
                        message.perkw = reader.uint64();
                        break;
                    case 6:
                        message.perkb = reader.uint64();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Feerate message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Feerate
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Feerate} Feerate
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Feerate.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Feerate message.
             * @function verify
             * @memberof greenlight.Feerate
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Feerate.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                var properties = {};
                if (message.preset != null && message.hasOwnProperty("preset")) {
                    properties.value = 1;
                    switch (message.preset) {
                    default:
                        return "preset: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                        break;
                    }
                }
                if (message.perkw != null && message.hasOwnProperty("perkw")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (!$util.isInteger(message.perkw) && !(message.perkw && $util.isInteger(message.perkw.low) && $util.isInteger(message.perkw.high)))
                        return "perkw: integer|Long expected";
                }
                if (message.perkb != null && message.hasOwnProperty("perkb")) {
                    if (properties.value === 1)
                        return "value: multiple values";
                    properties.value = 1;
                    if (!$util.isInteger(message.perkb) && !(message.perkb && $util.isInteger(message.perkb.low) && $util.isInteger(message.perkb.high)))
                        return "perkb: integer|Long expected";
                }
                return null;
            };
    
            /**
             * Creates a Feerate message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Feerate
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Feerate} Feerate
             */
            Feerate.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Feerate)
                    return object;
                var message = new $root.greenlight.Feerate();
                switch (object.preset) {
                case "NORMAL":
                case 0:
                    message.preset = 0;
                    break;
                case "SLOW":
                case 1:
                    message.preset = 1;
                    break;
                case "URGENT":
                case 2:
                    message.preset = 2;
                    break;
                }
                if (object.perkw != null)
                    if ($util.Long)
                        (message.perkw = $util.Long.fromValue(object.perkw)).unsigned = true;
                    else if (typeof object.perkw === "string")
                        message.perkw = parseInt(object.perkw, 10);
                    else if (typeof object.perkw === "number")
                        message.perkw = object.perkw;
                    else if (typeof object.perkw === "object")
                        message.perkw = new $util.LongBits(object.perkw.low >>> 0, object.perkw.high >>> 0).toNumber(true);
                if (object.perkb != null)
                    if ($util.Long)
                        (message.perkb = $util.Long.fromValue(object.perkb)).unsigned = true;
                    else if (typeof object.perkb === "string")
                        message.perkb = parseInt(object.perkb, 10);
                    else if (typeof object.perkb === "number")
                        message.perkb = object.perkb;
                    else if (typeof object.perkb === "object")
                        message.perkb = new $util.LongBits(object.perkb.low >>> 0, object.perkb.high >>> 0).toNumber(true);
                return message;
            };
    
            /**
             * Creates a plain object from a Feerate message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Feerate
             * @static
             * @param {greenlight.Feerate} message Feerate
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Feerate.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (message.preset != null && message.hasOwnProperty("preset")) {
                    object.preset = options.enums === String ? $root.greenlight.FeeratePreset[message.preset] : message.preset;
                    if (options.oneofs)
                        object.value = "preset";
                }
                if (message.perkw != null && message.hasOwnProperty("perkw")) {
                    if (typeof message.perkw === "number")
                        object.perkw = options.longs === String ? String(message.perkw) : message.perkw;
                    else
                        object.perkw = options.longs === String ? $util.Long.prototype.toString.call(message.perkw) : options.longs === Number ? new $util.LongBits(message.perkw.low >>> 0, message.perkw.high >>> 0).toNumber(true) : message.perkw;
                    if (options.oneofs)
                        object.value = "perkw";
                }
                if (message.perkb != null && message.hasOwnProperty("perkb")) {
                    if (typeof message.perkb === "number")
                        object.perkb = options.longs === String ? String(message.perkb) : message.perkb;
                    else
                        object.perkb = options.longs === String ? $util.Long.prototype.toString.call(message.perkb) : options.longs === Number ? new $util.LongBits(message.perkb.low >>> 0, message.perkb.high >>> 0).toNumber(true) : message.perkb;
                    if (options.oneofs)
                        object.value = "perkb";
                }
                return object;
            };
    
            /**
             * Converts this Feerate to JSON.
             * @function toJSON
             * @memberof greenlight.Feerate
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Feerate.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Feerate;
        })();
    
        greenlight.Confirmation = (function() {
    
            /**
             * Properties of a Confirmation.
             * @memberof greenlight
             * @interface IConfirmation
             * @property {number|null} [blocks] Confirmation blocks
             */
    
            /**
             * Constructs a new Confirmation.
             * @memberof greenlight
             * @classdesc Represents a Confirmation.
             * @implements IConfirmation
             * @constructor
             * @param {greenlight.IConfirmation=} [properties] Properties to set
             */
            function Confirmation(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Confirmation blocks.
             * @member {number} blocks
             * @memberof greenlight.Confirmation
             * @instance
             */
            Confirmation.prototype.blocks = 0;
    
            /**
             * Creates a new Confirmation instance using the specified properties.
             * @function create
             * @memberof greenlight.Confirmation
             * @static
             * @param {greenlight.IConfirmation=} [properties] Properties to set
             * @returns {greenlight.Confirmation} Confirmation instance
             */
            Confirmation.create = function create(properties) {
                return new Confirmation(properties);
            };
    
            /**
             * Encodes the specified Confirmation message. Does not implicitly {@link greenlight.Confirmation.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Confirmation
             * @static
             * @param {greenlight.IConfirmation} message Confirmation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Confirmation.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.blocks != null && Object.hasOwnProperty.call(message, "blocks"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.blocks);
                return writer;
            };
    
            /**
             * Encodes the specified Confirmation message, length delimited. Does not implicitly {@link greenlight.Confirmation.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Confirmation
             * @static
             * @param {greenlight.IConfirmation} message Confirmation message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Confirmation.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Confirmation message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Confirmation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Confirmation} Confirmation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Confirmation.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Confirmation();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.blocks = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Confirmation message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Confirmation
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Confirmation} Confirmation
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Confirmation.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Confirmation message.
             * @function verify
             * @memberof greenlight.Confirmation
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Confirmation.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.blocks != null && message.hasOwnProperty("blocks"))
                    if (!$util.isInteger(message.blocks))
                        return "blocks: integer expected";
                return null;
            };
    
            /**
             * Creates a Confirmation message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Confirmation
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Confirmation} Confirmation
             */
            Confirmation.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Confirmation)
                    return object;
                var message = new $root.greenlight.Confirmation();
                if (object.blocks != null)
                    message.blocks = object.blocks >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from a Confirmation message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Confirmation
             * @static
             * @param {greenlight.Confirmation} message Confirmation
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Confirmation.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.blocks = 0;
                if (message.blocks != null && message.hasOwnProperty("blocks"))
                    object.blocks = message.blocks;
                return object;
            };
    
            /**
             * Converts this Confirmation to JSON.
             * @function toJSON
             * @memberof greenlight.Confirmation
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Confirmation.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Confirmation;
        })();
    
        greenlight.WithdrawRequest = (function() {
    
            /**
             * Properties of a WithdrawRequest.
             * @memberof greenlight
             * @interface IWithdrawRequest
             * @property {string|null} [destination] WithdrawRequest destination
             * @property {greenlight.IAmount|null} [amount] WithdrawRequest amount
             * @property {greenlight.IFeerate|null} [feerate] WithdrawRequest feerate
             * @property {greenlight.IConfirmation|null} [minconf] WithdrawRequest minconf
             * @property {Array.<greenlight.IOutpoint>|null} [utxos] WithdrawRequest utxos
             */
    
            /**
             * Constructs a new WithdrawRequest.
             * @memberof greenlight
             * @classdesc Represents a WithdrawRequest.
             * @implements IWithdrawRequest
             * @constructor
             * @param {greenlight.IWithdrawRequest=} [properties] Properties to set
             */
            function WithdrawRequest(properties) {
                this.utxos = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * WithdrawRequest destination.
             * @member {string} destination
             * @memberof greenlight.WithdrawRequest
             * @instance
             */
            WithdrawRequest.prototype.destination = "";
    
            /**
             * WithdrawRequest amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.WithdrawRequest
             * @instance
             */
            WithdrawRequest.prototype.amount = null;
    
            /**
             * WithdrawRequest feerate.
             * @member {greenlight.IFeerate|null|undefined} feerate
             * @memberof greenlight.WithdrawRequest
             * @instance
             */
            WithdrawRequest.prototype.feerate = null;
    
            /**
             * WithdrawRequest minconf.
             * @member {greenlight.IConfirmation|null|undefined} minconf
             * @memberof greenlight.WithdrawRequest
             * @instance
             */
            WithdrawRequest.prototype.minconf = null;
    
            /**
             * WithdrawRequest utxos.
             * @member {Array.<greenlight.IOutpoint>} utxos
             * @memberof greenlight.WithdrawRequest
             * @instance
             */
            WithdrawRequest.prototype.utxos = $util.emptyArray;
    
            /**
             * Creates a new WithdrawRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {greenlight.IWithdrawRequest=} [properties] Properties to set
             * @returns {greenlight.WithdrawRequest} WithdrawRequest instance
             */
            WithdrawRequest.create = function create(properties) {
                return new WithdrawRequest(properties);
            };
    
            /**
             * Encodes the specified WithdrawRequest message. Does not implicitly {@link greenlight.WithdrawRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {greenlight.IWithdrawRequest} message WithdrawRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            WithdrawRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.destination != null && Object.hasOwnProperty.call(message, "destination"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.destination);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.feerate != null && Object.hasOwnProperty.call(message, "feerate"))
                    $root.greenlight.Feerate.encode(message.feerate, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.minconf != null && Object.hasOwnProperty.call(message, "minconf"))
                    $root.greenlight.Confirmation.encode(message.minconf, writer.uint32(/* id 7, wireType 2 =*/58).fork()).ldelim();
                if (message.utxos != null && message.utxos.length)
                    for (var i = 0; i < message.utxos.length; ++i)
                        $root.greenlight.Outpoint.encode(message.utxos[i], writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified WithdrawRequest message, length delimited. Does not implicitly {@link greenlight.WithdrawRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {greenlight.IWithdrawRequest} message WithdrawRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            WithdrawRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a WithdrawRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.WithdrawRequest} WithdrawRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            WithdrawRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.WithdrawRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.destination = reader.string();
                        break;
                    case 2:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.feerate = $root.greenlight.Feerate.decode(reader, reader.uint32());
                        break;
                    case 7:
                        message.minconf = $root.greenlight.Confirmation.decode(reader, reader.uint32());
                        break;
                    case 8:
                        if (!(message.utxos && message.utxos.length))
                            message.utxos = [];
                        message.utxos.push($root.greenlight.Outpoint.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a WithdrawRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.WithdrawRequest} WithdrawRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            WithdrawRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a WithdrawRequest message.
             * @function verify
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            WithdrawRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.destination != null && message.hasOwnProperty("destination"))
                    if (!$util.isString(message.destination))
                        return "destination: string expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.feerate != null && message.hasOwnProperty("feerate")) {
                    var error = $root.greenlight.Feerate.verify(message.feerate);
                    if (error)
                        return "feerate." + error;
                }
                if (message.minconf != null && message.hasOwnProperty("minconf")) {
                    var error = $root.greenlight.Confirmation.verify(message.minconf);
                    if (error)
                        return "minconf." + error;
                }
                if (message.utxos != null && message.hasOwnProperty("utxos")) {
                    if (!Array.isArray(message.utxos))
                        return "utxos: array expected";
                    for (var i = 0; i < message.utxos.length; ++i) {
                        var error = $root.greenlight.Outpoint.verify(message.utxos[i]);
                        if (error)
                            return "utxos." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a WithdrawRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.WithdrawRequest} WithdrawRequest
             */
            WithdrawRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.WithdrawRequest)
                    return object;
                var message = new $root.greenlight.WithdrawRequest();
                if (object.destination != null)
                    message.destination = String(object.destination);
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.WithdrawRequest.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.feerate != null) {
                    if (typeof object.feerate !== "object")
                        throw TypeError(".greenlight.WithdrawRequest.feerate: object expected");
                    message.feerate = $root.greenlight.Feerate.fromObject(object.feerate);
                }
                if (object.minconf != null) {
                    if (typeof object.minconf !== "object")
                        throw TypeError(".greenlight.WithdrawRequest.minconf: object expected");
                    message.minconf = $root.greenlight.Confirmation.fromObject(object.minconf);
                }
                if (object.utxos) {
                    if (!Array.isArray(object.utxos))
                        throw TypeError(".greenlight.WithdrawRequest.utxos: array expected");
                    message.utxos = [];
                    for (var i = 0; i < object.utxos.length; ++i) {
                        if (typeof object.utxos[i] !== "object")
                            throw TypeError(".greenlight.WithdrawRequest.utxos: object expected");
                        message.utxos[i] = $root.greenlight.Outpoint.fromObject(object.utxos[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a WithdrawRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.WithdrawRequest
             * @static
             * @param {greenlight.WithdrawRequest} message WithdrawRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            WithdrawRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.utxos = [];
                if (options.defaults) {
                    object.destination = "";
                    object.amount = null;
                    object.feerate = null;
                    object.minconf = null;
                }
                if (message.destination != null && message.hasOwnProperty("destination"))
                    object.destination = message.destination;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.feerate != null && message.hasOwnProperty("feerate"))
                    object.feerate = $root.greenlight.Feerate.toObject(message.feerate, options);
                if (message.minconf != null && message.hasOwnProperty("minconf"))
                    object.minconf = $root.greenlight.Confirmation.toObject(message.minconf, options);
                if (message.utxos && message.utxos.length) {
                    object.utxos = [];
                    for (var j = 0; j < message.utxos.length; ++j)
                        object.utxos[j] = $root.greenlight.Outpoint.toObject(message.utxos[j], options);
                }
                return object;
            };
    
            /**
             * Converts this WithdrawRequest to JSON.
             * @function toJSON
             * @memberof greenlight.WithdrawRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            WithdrawRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return WithdrawRequest;
        })();
    
        greenlight.WithdrawResponse = (function() {
    
            /**
             * Properties of a WithdrawResponse.
             * @memberof greenlight
             * @interface IWithdrawResponse
             * @property {Uint8Array|null} [tx] WithdrawResponse tx
             * @property {Uint8Array|null} [txid] WithdrawResponse txid
             */
    
            /**
             * Constructs a new WithdrawResponse.
             * @memberof greenlight
             * @classdesc Represents a WithdrawResponse.
             * @implements IWithdrawResponse
             * @constructor
             * @param {greenlight.IWithdrawResponse=} [properties] Properties to set
             */
            function WithdrawResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * WithdrawResponse tx.
             * @member {Uint8Array} tx
             * @memberof greenlight.WithdrawResponse
             * @instance
             */
            WithdrawResponse.prototype.tx = $util.newBuffer([]);
    
            /**
             * WithdrawResponse txid.
             * @member {Uint8Array} txid
             * @memberof greenlight.WithdrawResponse
             * @instance
             */
            WithdrawResponse.prototype.txid = $util.newBuffer([]);
    
            /**
             * Creates a new WithdrawResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {greenlight.IWithdrawResponse=} [properties] Properties to set
             * @returns {greenlight.WithdrawResponse} WithdrawResponse instance
             */
            WithdrawResponse.create = function create(properties) {
                return new WithdrawResponse(properties);
            };
    
            /**
             * Encodes the specified WithdrawResponse message. Does not implicitly {@link greenlight.WithdrawResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {greenlight.IWithdrawResponse} message WithdrawResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            WithdrawResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.tx != null && Object.hasOwnProperty.call(message, "tx"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.tx);
                if (message.txid != null && Object.hasOwnProperty.call(message, "txid"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.txid);
                return writer;
            };
    
            /**
             * Encodes the specified WithdrawResponse message, length delimited. Does not implicitly {@link greenlight.WithdrawResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {greenlight.IWithdrawResponse} message WithdrawResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            WithdrawResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a WithdrawResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.WithdrawResponse} WithdrawResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            WithdrawResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.WithdrawResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.tx = reader.bytes();
                        break;
                    case 2:
                        message.txid = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a WithdrawResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.WithdrawResponse} WithdrawResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            WithdrawResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a WithdrawResponse message.
             * @function verify
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            WithdrawResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.tx != null && message.hasOwnProperty("tx"))
                    if (!(message.tx && typeof message.tx.length === "number" || $util.isString(message.tx)))
                        return "tx: buffer expected";
                if (message.txid != null && message.hasOwnProperty("txid"))
                    if (!(message.txid && typeof message.txid.length === "number" || $util.isString(message.txid)))
                        return "txid: buffer expected";
                return null;
            };
    
            /**
             * Creates a WithdrawResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.WithdrawResponse} WithdrawResponse
             */
            WithdrawResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.WithdrawResponse)
                    return object;
                var message = new $root.greenlight.WithdrawResponse();
                if (object.tx != null)
                    if (typeof object.tx === "string")
                        $util.base64.decode(object.tx, message.tx = $util.newBuffer($util.base64.length(object.tx)), 0);
                    else if (object.tx.length)
                        message.tx = object.tx;
                if (object.txid != null)
                    if (typeof object.txid === "string")
                        $util.base64.decode(object.txid, message.txid = $util.newBuffer($util.base64.length(object.txid)), 0);
                    else if (object.txid.length)
                        message.txid = object.txid;
                return message;
            };
    
            /**
             * Creates a plain object from a WithdrawResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.WithdrawResponse
             * @static
             * @param {greenlight.WithdrawResponse} message WithdrawResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            WithdrawResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.tx = "";
                    else {
                        object.tx = [];
                        if (options.bytes !== Array)
                            object.tx = $util.newBuffer(object.tx);
                    }
                    if (options.bytes === String)
                        object.txid = "";
                    else {
                        object.txid = [];
                        if (options.bytes !== Array)
                            object.txid = $util.newBuffer(object.txid);
                    }
                }
                if (message.tx != null && message.hasOwnProperty("tx"))
                    object.tx = options.bytes === String ? $util.base64.encode(message.tx, 0, message.tx.length) : options.bytes === Array ? Array.prototype.slice.call(message.tx) : message.tx;
                if (message.txid != null && message.hasOwnProperty("txid"))
                    object.txid = options.bytes === String ? $util.base64.encode(message.txid, 0, message.txid.length) : options.bytes === Array ? Array.prototype.slice.call(message.txid) : message.txid;
                return object;
            };
    
            /**
             * Converts this WithdrawResponse to JSON.
             * @function toJSON
             * @memberof greenlight.WithdrawResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            WithdrawResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return WithdrawResponse;
        })();
    
        greenlight.FundChannelRequest = (function() {
    
            /**
             * Properties of a FundChannelRequest.
             * @memberof greenlight
             * @interface IFundChannelRequest
             * @property {Uint8Array|null} [nodeId] FundChannelRequest nodeId
             * @property {greenlight.IAmount|null} [amount] FundChannelRequest amount
             * @property {greenlight.IFeerate|null} [feerate] FundChannelRequest feerate
             * @property {boolean|null} [announce] FundChannelRequest announce
             * @property {greenlight.IConfirmation|null} [minconf] FundChannelRequest minconf
             * @property {string|null} [closeTo] FundChannelRequest closeTo
             */
    
            /**
             * Constructs a new FundChannelRequest.
             * @memberof greenlight
             * @classdesc Represents a FundChannelRequest.
             * @implements IFundChannelRequest
             * @constructor
             * @param {greenlight.IFundChannelRequest=} [properties] Properties to set
             */
            function FundChannelRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * FundChannelRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof greenlight.FundChannelRequest
             * @instance
             */
            FundChannelRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * FundChannelRequest amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.FundChannelRequest
             * @instance
             */
            FundChannelRequest.prototype.amount = null;
    
            /**
             * FundChannelRequest feerate.
             * @member {greenlight.IFeerate|null|undefined} feerate
             * @memberof greenlight.FundChannelRequest
             * @instance
             */
            FundChannelRequest.prototype.feerate = null;
    
            /**
             * FundChannelRequest announce.
             * @member {boolean} announce
             * @memberof greenlight.FundChannelRequest
             * @instance
             */
            FundChannelRequest.prototype.announce = false;
    
            /**
             * FundChannelRequest minconf.
             * @member {greenlight.IConfirmation|null|undefined} minconf
             * @memberof greenlight.FundChannelRequest
             * @instance
             */
            FundChannelRequest.prototype.minconf = null;
    
            /**
             * FundChannelRequest closeTo.
             * @member {string} closeTo
             * @memberof greenlight.FundChannelRequest
             * @instance
             */
            FundChannelRequest.prototype.closeTo = "";
    
            /**
             * Creates a new FundChannelRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {greenlight.IFundChannelRequest=} [properties] Properties to set
             * @returns {greenlight.FundChannelRequest} FundChannelRequest instance
             */
            FundChannelRequest.create = function create(properties) {
                return new FundChannelRequest(properties);
            };
    
            /**
             * Encodes the specified FundChannelRequest message. Does not implicitly {@link greenlight.FundChannelRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {greenlight.IFundChannelRequest} message FundChannelRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FundChannelRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.feerate != null && Object.hasOwnProperty.call(message, "feerate"))
                    $root.greenlight.Feerate.encode(message.feerate, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.announce != null && Object.hasOwnProperty.call(message, "announce"))
                    writer.uint32(/* id 7, wireType 0 =*/56).bool(message.announce);
                if (message.minconf != null && Object.hasOwnProperty.call(message, "minconf"))
                    $root.greenlight.Confirmation.encode(message.minconf, writer.uint32(/* id 8, wireType 2 =*/66).fork()).ldelim();
                if (message.closeTo != null && Object.hasOwnProperty.call(message, "closeTo"))
                    writer.uint32(/* id 10, wireType 2 =*/82).string(message.closeTo);
                return writer;
            };
    
            /**
             * Encodes the specified FundChannelRequest message, length delimited. Does not implicitly {@link greenlight.FundChannelRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {greenlight.IFundChannelRequest} message FundChannelRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FundChannelRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a FundChannelRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.FundChannelRequest} FundChannelRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FundChannelRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.FundChannelRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.feerate = $root.greenlight.Feerate.decode(reader, reader.uint32());
                        break;
                    case 7:
                        message.announce = reader.bool();
                        break;
                    case 8:
                        message.minconf = $root.greenlight.Confirmation.decode(reader, reader.uint32());
                        break;
                    case 10:
                        message.closeTo = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a FundChannelRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.FundChannelRequest} FundChannelRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FundChannelRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a FundChannelRequest message.
             * @function verify
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FundChannelRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.feerate != null && message.hasOwnProperty("feerate")) {
                    var error = $root.greenlight.Feerate.verify(message.feerate);
                    if (error)
                        return "feerate." + error;
                }
                if (message.announce != null && message.hasOwnProperty("announce"))
                    if (typeof message.announce !== "boolean")
                        return "announce: boolean expected";
                if (message.minconf != null && message.hasOwnProperty("minconf")) {
                    var error = $root.greenlight.Confirmation.verify(message.minconf);
                    if (error)
                        return "minconf." + error;
                }
                if (message.closeTo != null && message.hasOwnProperty("closeTo"))
                    if (!$util.isString(message.closeTo))
                        return "closeTo: string expected";
                return null;
            };
    
            /**
             * Creates a FundChannelRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.FundChannelRequest} FundChannelRequest
             */
            FundChannelRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.FundChannelRequest)
                    return object;
                var message = new $root.greenlight.FundChannelRequest();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.FundChannelRequest.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.feerate != null) {
                    if (typeof object.feerate !== "object")
                        throw TypeError(".greenlight.FundChannelRequest.feerate: object expected");
                    message.feerate = $root.greenlight.Feerate.fromObject(object.feerate);
                }
                if (object.announce != null)
                    message.announce = Boolean(object.announce);
                if (object.minconf != null) {
                    if (typeof object.minconf !== "object")
                        throw TypeError(".greenlight.FundChannelRequest.minconf: object expected");
                    message.minconf = $root.greenlight.Confirmation.fromObject(object.minconf);
                }
                if (object.closeTo != null)
                    message.closeTo = String(object.closeTo);
                return message;
            };
    
            /**
             * Creates a plain object from a FundChannelRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.FundChannelRequest
             * @static
             * @param {greenlight.FundChannelRequest} message FundChannelRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FundChannelRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.amount = null;
                    object.feerate = null;
                    object.announce = false;
                    object.minconf = null;
                    object.closeTo = "";
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.feerate != null && message.hasOwnProperty("feerate"))
                    object.feerate = $root.greenlight.Feerate.toObject(message.feerate, options);
                if (message.announce != null && message.hasOwnProperty("announce"))
                    object.announce = message.announce;
                if (message.minconf != null && message.hasOwnProperty("minconf"))
                    object.minconf = $root.greenlight.Confirmation.toObject(message.minconf, options);
                if (message.closeTo != null && message.hasOwnProperty("closeTo"))
                    object.closeTo = message.closeTo;
                return object;
            };
    
            /**
             * Converts this FundChannelRequest to JSON.
             * @function toJSON
             * @memberof greenlight.FundChannelRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FundChannelRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return FundChannelRequest;
        })();
    
        greenlight.Outpoint = (function() {
    
            /**
             * Properties of an Outpoint.
             * @memberof greenlight
             * @interface IOutpoint
             * @property {Uint8Array|null} [txid] Outpoint txid
             * @property {number|null} [outnum] Outpoint outnum
             */
    
            /**
             * Constructs a new Outpoint.
             * @memberof greenlight
             * @classdesc Represents an Outpoint.
             * @implements IOutpoint
             * @constructor
             * @param {greenlight.IOutpoint=} [properties] Properties to set
             */
            function Outpoint(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Outpoint txid.
             * @member {Uint8Array} txid
             * @memberof greenlight.Outpoint
             * @instance
             */
            Outpoint.prototype.txid = $util.newBuffer([]);
    
            /**
             * Outpoint outnum.
             * @member {number} outnum
             * @memberof greenlight.Outpoint
             * @instance
             */
            Outpoint.prototype.outnum = 0;
    
            /**
             * Creates a new Outpoint instance using the specified properties.
             * @function create
             * @memberof greenlight.Outpoint
             * @static
             * @param {greenlight.IOutpoint=} [properties] Properties to set
             * @returns {greenlight.Outpoint} Outpoint instance
             */
            Outpoint.create = function create(properties) {
                return new Outpoint(properties);
            };
    
            /**
             * Encodes the specified Outpoint message. Does not implicitly {@link greenlight.Outpoint.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Outpoint
             * @static
             * @param {greenlight.IOutpoint} message Outpoint message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Outpoint.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.txid != null && Object.hasOwnProperty.call(message, "txid"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.txid);
                if (message.outnum != null && Object.hasOwnProperty.call(message, "outnum"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint32(message.outnum);
                return writer;
            };
    
            /**
             * Encodes the specified Outpoint message, length delimited. Does not implicitly {@link greenlight.Outpoint.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Outpoint
             * @static
             * @param {greenlight.IOutpoint} message Outpoint message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Outpoint.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an Outpoint message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Outpoint
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Outpoint} Outpoint
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Outpoint.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Outpoint();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.txid = reader.bytes();
                        break;
                    case 2:
                        message.outnum = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an Outpoint message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Outpoint
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Outpoint} Outpoint
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Outpoint.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an Outpoint message.
             * @function verify
             * @memberof greenlight.Outpoint
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Outpoint.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.txid != null && message.hasOwnProperty("txid"))
                    if (!(message.txid && typeof message.txid.length === "number" || $util.isString(message.txid)))
                        return "txid: buffer expected";
                if (message.outnum != null && message.hasOwnProperty("outnum"))
                    if (!$util.isInteger(message.outnum))
                        return "outnum: integer expected";
                return null;
            };
    
            /**
             * Creates an Outpoint message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Outpoint
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Outpoint} Outpoint
             */
            Outpoint.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Outpoint)
                    return object;
                var message = new $root.greenlight.Outpoint();
                if (object.txid != null)
                    if (typeof object.txid === "string")
                        $util.base64.decode(object.txid, message.txid = $util.newBuffer($util.base64.length(object.txid)), 0);
                    else if (object.txid.length)
                        message.txid = object.txid;
                if (object.outnum != null)
                    message.outnum = object.outnum >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from an Outpoint message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Outpoint
             * @static
             * @param {greenlight.Outpoint} message Outpoint
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Outpoint.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.txid = "";
                    else {
                        object.txid = [];
                        if (options.bytes !== Array)
                            object.txid = $util.newBuffer(object.txid);
                    }
                    object.outnum = 0;
                }
                if (message.txid != null && message.hasOwnProperty("txid"))
                    object.txid = options.bytes === String ? $util.base64.encode(message.txid, 0, message.txid.length) : options.bytes === Array ? Array.prototype.slice.call(message.txid) : message.txid;
                if (message.outnum != null && message.hasOwnProperty("outnum"))
                    object.outnum = message.outnum;
                return object;
            };
    
            /**
             * Converts this Outpoint to JSON.
             * @function toJSON
             * @memberof greenlight.Outpoint
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Outpoint.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Outpoint;
        })();
    
        greenlight.FundChannelResponse = (function() {
    
            /**
             * Properties of a FundChannelResponse.
             * @memberof greenlight
             * @interface IFundChannelResponse
             * @property {Uint8Array|null} [tx] FundChannelResponse tx
             * @property {greenlight.IOutpoint|null} [outpoint] FundChannelResponse outpoint
             * @property {Uint8Array|null} [channelId] FundChannelResponse channelId
             * @property {string|null} [closeTo] FundChannelResponse closeTo
             */
    
            /**
             * Constructs a new FundChannelResponse.
             * @memberof greenlight
             * @classdesc Represents a FundChannelResponse.
             * @implements IFundChannelResponse
             * @constructor
             * @param {greenlight.IFundChannelResponse=} [properties] Properties to set
             */
            function FundChannelResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * FundChannelResponse tx.
             * @member {Uint8Array} tx
             * @memberof greenlight.FundChannelResponse
             * @instance
             */
            FundChannelResponse.prototype.tx = $util.newBuffer([]);
    
            /**
             * FundChannelResponse outpoint.
             * @member {greenlight.IOutpoint|null|undefined} outpoint
             * @memberof greenlight.FundChannelResponse
             * @instance
             */
            FundChannelResponse.prototype.outpoint = null;
    
            /**
             * FundChannelResponse channelId.
             * @member {Uint8Array} channelId
             * @memberof greenlight.FundChannelResponse
             * @instance
             */
            FundChannelResponse.prototype.channelId = $util.newBuffer([]);
    
            /**
             * FundChannelResponse closeTo.
             * @member {string} closeTo
             * @memberof greenlight.FundChannelResponse
             * @instance
             */
            FundChannelResponse.prototype.closeTo = "";
    
            /**
             * Creates a new FundChannelResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {greenlight.IFundChannelResponse=} [properties] Properties to set
             * @returns {greenlight.FundChannelResponse} FundChannelResponse instance
             */
            FundChannelResponse.create = function create(properties) {
                return new FundChannelResponse(properties);
            };
    
            /**
             * Encodes the specified FundChannelResponse message. Does not implicitly {@link greenlight.FundChannelResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {greenlight.IFundChannelResponse} message FundChannelResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FundChannelResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.tx != null && Object.hasOwnProperty.call(message, "tx"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.tx);
                if (message.outpoint != null && Object.hasOwnProperty.call(message, "outpoint"))
                    $root.greenlight.Outpoint.encode(message.outpoint, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.channelId != null && Object.hasOwnProperty.call(message, "channelId"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.channelId);
                if (message.closeTo != null && Object.hasOwnProperty.call(message, "closeTo"))
                    writer.uint32(/* id 4, wireType 2 =*/34).string(message.closeTo);
                return writer;
            };
    
            /**
             * Encodes the specified FundChannelResponse message, length delimited. Does not implicitly {@link greenlight.FundChannelResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {greenlight.IFundChannelResponse} message FundChannelResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            FundChannelResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a FundChannelResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.FundChannelResponse} FundChannelResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FundChannelResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.FundChannelResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.tx = reader.bytes();
                        break;
                    case 2:
                        message.outpoint = $root.greenlight.Outpoint.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.channelId = reader.bytes();
                        break;
                    case 4:
                        message.closeTo = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a FundChannelResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.FundChannelResponse} FundChannelResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            FundChannelResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a FundChannelResponse message.
             * @function verify
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            FundChannelResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.tx != null && message.hasOwnProperty("tx"))
                    if (!(message.tx && typeof message.tx.length === "number" || $util.isString(message.tx)))
                        return "tx: buffer expected";
                if (message.outpoint != null && message.hasOwnProperty("outpoint")) {
                    var error = $root.greenlight.Outpoint.verify(message.outpoint);
                    if (error)
                        return "outpoint." + error;
                }
                if (message.channelId != null && message.hasOwnProperty("channelId"))
                    if (!(message.channelId && typeof message.channelId.length === "number" || $util.isString(message.channelId)))
                        return "channelId: buffer expected";
                if (message.closeTo != null && message.hasOwnProperty("closeTo"))
                    if (!$util.isString(message.closeTo))
                        return "closeTo: string expected";
                return null;
            };
    
            /**
             * Creates a FundChannelResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.FundChannelResponse} FundChannelResponse
             */
            FundChannelResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.FundChannelResponse)
                    return object;
                var message = new $root.greenlight.FundChannelResponse();
                if (object.tx != null)
                    if (typeof object.tx === "string")
                        $util.base64.decode(object.tx, message.tx = $util.newBuffer($util.base64.length(object.tx)), 0);
                    else if (object.tx.length)
                        message.tx = object.tx;
                if (object.outpoint != null) {
                    if (typeof object.outpoint !== "object")
                        throw TypeError(".greenlight.FundChannelResponse.outpoint: object expected");
                    message.outpoint = $root.greenlight.Outpoint.fromObject(object.outpoint);
                }
                if (object.channelId != null)
                    if (typeof object.channelId === "string")
                        $util.base64.decode(object.channelId, message.channelId = $util.newBuffer($util.base64.length(object.channelId)), 0);
                    else if (object.channelId.length)
                        message.channelId = object.channelId;
                if (object.closeTo != null)
                    message.closeTo = String(object.closeTo);
                return message;
            };
    
            /**
             * Creates a plain object from a FundChannelResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.FundChannelResponse
             * @static
             * @param {greenlight.FundChannelResponse} message FundChannelResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            FundChannelResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.tx = "";
                    else {
                        object.tx = [];
                        if (options.bytes !== Array)
                            object.tx = $util.newBuffer(object.tx);
                    }
                    object.outpoint = null;
                    if (options.bytes === String)
                        object.channelId = "";
                    else {
                        object.channelId = [];
                        if (options.bytes !== Array)
                            object.channelId = $util.newBuffer(object.channelId);
                    }
                    object.closeTo = "";
                }
                if (message.tx != null && message.hasOwnProperty("tx"))
                    object.tx = options.bytes === String ? $util.base64.encode(message.tx, 0, message.tx.length) : options.bytes === Array ? Array.prototype.slice.call(message.tx) : message.tx;
                if (message.outpoint != null && message.hasOwnProperty("outpoint"))
                    object.outpoint = $root.greenlight.Outpoint.toObject(message.outpoint, options);
                if (message.channelId != null && message.hasOwnProperty("channelId"))
                    object.channelId = options.bytes === String ? $util.base64.encode(message.channelId, 0, message.channelId.length) : options.bytes === Array ? Array.prototype.slice.call(message.channelId) : message.channelId;
                if (message.closeTo != null && message.hasOwnProperty("closeTo"))
                    object.closeTo = message.closeTo;
                return object;
            };
    
            /**
             * Converts this FundChannelResponse to JSON.
             * @function toJSON
             * @memberof greenlight.FundChannelResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            FundChannelResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return FundChannelResponse;
        })();
    
        greenlight.Timeout = (function() {
    
            /**
             * Properties of a Timeout.
             * @memberof greenlight
             * @interface ITimeout
             * @property {number|null} [seconds] Timeout seconds
             */
    
            /**
             * Constructs a new Timeout.
             * @memberof greenlight
             * @classdesc Represents a Timeout.
             * @implements ITimeout
             * @constructor
             * @param {greenlight.ITimeout=} [properties] Properties to set
             */
            function Timeout(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Timeout seconds.
             * @member {number} seconds
             * @memberof greenlight.Timeout
             * @instance
             */
            Timeout.prototype.seconds = 0;
    
            /**
             * Creates a new Timeout instance using the specified properties.
             * @function create
             * @memberof greenlight.Timeout
             * @static
             * @param {greenlight.ITimeout=} [properties] Properties to set
             * @returns {greenlight.Timeout} Timeout instance
             */
            Timeout.create = function create(properties) {
                return new Timeout(properties);
            };
    
            /**
             * Encodes the specified Timeout message. Does not implicitly {@link greenlight.Timeout.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Timeout
             * @static
             * @param {greenlight.ITimeout} message Timeout message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Timeout.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.seconds != null && Object.hasOwnProperty.call(message, "seconds"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint32(message.seconds);
                return writer;
            };
    
            /**
             * Encodes the specified Timeout message, length delimited. Does not implicitly {@link greenlight.Timeout.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Timeout
             * @static
             * @param {greenlight.ITimeout} message Timeout message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Timeout.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Timeout message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Timeout
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Timeout} Timeout
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Timeout.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Timeout();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.seconds = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Timeout message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Timeout
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Timeout} Timeout
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Timeout.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Timeout message.
             * @function verify
             * @memberof greenlight.Timeout
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Timeout.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.seconds != null && message.hasOwnProperty("seconds"))
                    if (!$util.isInteger(message.seconds))
                        return "seconds: integer expected";
                return null;
            };
    
            /**
             * Creates a Timeout message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Timeout
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Timeout} Timeout
             */
            Timeout.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Timeout)
                    return object;
                var message = new $root.greenlight.Timeout();
                if (object.seconds != null)
                    message.seconds = object.seconds >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from a Timeout message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Timeout
             * @static
             * @param {greenlight.Timeout} message Timeout
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Timeout.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.seconds = 0;
                if (message.seconds != null && message.hasOwnProperty("seconds"))
                    object.seconds = message.seconds;
                return object;
            };
    
            /**
             * Converts this Timeout to JSON.
             * @function toJSON
             * @memberof greenlight.Timeout
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Timeout.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Timeout;
        })();
    
        greenlight.BitcoinAddress = (function() {
    
            /**
             * Properties of a BitcoinAddress.
             * @memberof greenlight
             * @interface IBitcoinAddress
             * @property {string|null} [address] BitcoinAddress address
             */
    
            /**
             * Constructs a new BitcoinAddress.
             * @memberof greenlight
             * @classdesc Represents a BitcoinAddress.
             * @implements IBitcoinAddress
             * @constructor
             * @param {greenlight.IBitcoinAddress=} [properties] Properties to set
             */
            function BitcoinAddress(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * BitcoinAddress address.
             * @member {string} address
             * @memberof greenlight.BitcoinAddress
             * @instance
             */
            BitcoinAddress.prototype.address = "";
    
            /**
             * Creates a new BitcoinAddress instance using the specified properties.
             * @function create
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {greenlight.IBitcoinAddress=} [properties] Properties to set
             * @returns {greenlight.BitcoinAddress} BitcoinAddress instance
             */
            BitcoinAddress.create = function create(properties) {
                return new BitcoinAddress(properties);
            };
    
            /**
             * Encodes the specified BitcoinAddress message. Does not implicitly {@link greenlight.BitcoinAddress.verify|verify} messages.
             * @function encode
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {greenlight.IBitcoinAddress} message BitcoinAddress message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            BitcoinAddress.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.address != null && Object.hasOwnProperty.call(message, "address"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.address);
                return writer;
            };
    
            /**
             * Encodes the specified BitcoinAddress message, length delimited. Does not implicitly {@link greenlight.BitcoinAddress.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {greenlight.IBitcoinAddress} message BitcoinAddress message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            BitcoinAddress.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a BitcoinAddress message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.BitcoinAddress} BitcoinAddress
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            BitcoinAddress.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.BitcoinAddress();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.address = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a BitcoinAddress message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.BitcoinAddress} BitcoinAddress
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            BitcoinAddress.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a BitcoinAddress message.
             * @function verify
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            BitcoinAddress.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.address != null && message.hasOwnProperty("address"))
                    if (!$util.isString(message.address))
                        return "address: string expected";
                return null;
            };
    
            /**
             * Creates a BitcoinAddress message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.BitcoinAddress} BitcoinAddress
             */
            BitcoinAddress.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.BitcoinAddress)
                    return object;
                var message = new $root.greenlight.BitcoinAddress();
                if (object.address != null)
                    message.address = String(object.address);
                return message;
            };
    
            /**
             * Creates a plain object from a BitcoinAddress message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.BitcoinAddress
             * @static
             * @param {greenlight.BitcoinAddress} message BitcoinAddress
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            BitcoinAddress.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.address = "";
                if (message.address != null && message.hasOwnProperty("address"))
                    object.address = message.address;
                return object;
            };
    
            /**
             * Converts this BitcoinAddress to JSON.
             * @function toJSON
             * @memberof greenlight.BitcoinAddress
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            BitcoinAddress.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return BitcoinAddress;
        })();
    
        greenlight.CloseChannelRequest = (function() {
    
            /**
             * Properties of a CloseChannelRequest.
             * @memberof greenlight
             * @interface ICloseChannelRequest
             * @property {Uint8Array|null} [nodeId] CloseChannelRequest nodeId
             * @property {greenlight.ITimeout|null} [unilateraltimeout] CloseChannelRequest unilateraltimeout
             * @property {greenlight.IBitcoinAddress|null} [destination] CloseChannelRequest destination
             */
    
            /**
             * Constructs a new CloseChannelRequest.
             * @memberof greenlight
             * @classdesc Represents a CloseChannelRequest.
             * @implements ICloseChannelRequest
             * @constructor
             * @param {greenlight.ICloseChannelRequest=} [properties] Properties to set
             */
            function CloseChannelRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * CloseChannelRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof greenlight.CloseChannelRequest
             * @instance
             */
            CloseChannelRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * CloseChannelRequest unilateraltimeout.
             * @member {greenlight.ITimeout|null|undefined} unilateraltimeout
             * @memberof greenlight.CloseChannelRequest
             * @instance
             */
            CloseChannelRequest.prototype.unilateraltimeout = null;
    
            /**
             * CloseChannelRequest destination.
             * @member {greenlight.IBitcoinAddress|null|undefined} destination
             * @memberof greenlight.CloseChannelRequest
             * @instance
             */
            CloseChannelRequest.prototype.destination = null;
    
            /**
             * Creates a new CloseChannelRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {greenlight.ICloseChannelRequest=} [properties] Properties to set
             * @returns {greenlight.CloseChannelRequest} CloseChannelRequest instance
             */
            CloseChannelRequest.create = function create(properties) {
                return new CloseChannelRequest(properties);
            };
    
            /**
             * Encodes the specified CloseChannelRequest message. Does not implicitly {@link greenlight.CloseChannelRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {greenlight.ICloseChannelRequest} message CloseChannelRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CloseChannelRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.unilateraltimeout != null && Object.hasOwnProperty.call(message, "unilateraltimeout"))
                    $root.greenlight.Timeout.encode(message.unilateraltimeout, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.destination != null && Object.hasOwnProperty.call(message, "destination"))
                    $root.greenlight.BitcoinAddress.encode(message.destination, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified CloseChannelRequest message, length delimited. Does not implicitly {@link greenlight.CloseChannelRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {greenlight.ICloseChannelRequest} message CloseChannelRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CloseChannelRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a CloseChannelRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.CloseChannelRequest} CloseChannelRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CloseChannelRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.CloseChannelRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.unilateraltimeout = $root.greenlight.Timeout.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.destination = $root.greenlight.BitcoinAddress.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a CloseChannelRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.CloseChannelRequest} CloseChannelRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CloseChannelRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a CloseChannelRequest message.
             * @function verify
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            CloseChannelRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.unilateraltimeout != null && message.hasOwnProperty("unilateraltimeout")) {
                    var error = $root.greenlight.Timeout.verify(message.unilateraltimeout);
                    if (error)
                        return "unilateraltimeout." + error;
                }
                if (message.destination != null && message.hasOwnProperty("destination")) {
                    var error = $root.greenlight.BitcoinAddress.verify(message.destination);
                    if (error)
                        return "destination." + error;
                }
                return null;
            };
    
            /**
             * Creates a CloseChannelRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.CloseChannelRequest} CloseChannelRequest
             */
            CloseChannelRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.CloseChannelRequest)
                    return object;
                var message = new $root.greenlight.CloseChannelRequest();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.unilateraltimeout != null) {
                    if (typeof object.unilateraltimeout !== "object")
                        throw TypeError(".greenlight.CloseChannelRequest.unilateraltimeout: object expected");
                    message.unilateraltimeout = $root.greenlight.Timeout.fromObject(object.unilateraltimeout);
                }
                if (object.destination != null) {
                    if (typeof object.destination !== "object")
                        throw TypeError(".greenlight.CloseChannelRequest.destination: object expected");
                    message.destination = $root.greenlight.BitcoinAddress.fromObject(object.destination);
                }
                return message;
            };
    
            /**
             * Creates a plain object from a CloseChannelRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.CloseChannelRequest
             * @static
             * @param {greenlight.CloseChannelRequest} message CloseChannelRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            CloseChannelRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.unilateraltimeout = null;
                    object.destination = null;
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.unilateraltimeout != null && message.hasOwnProperty("unilateraltimeout"))
                    object.unilateraltimeout = $root.greenlight.Timeout.toObject(message.unilateraltimeout, options);
                if (message.destination != null && message.hasOwnProperty("destination"))
                    object.destination = $root.greenlight.BitcoinAddress.toObject(message.destination, options);
                return object;
            };
    
            /**
             * Converts this CloseChannelRequest to JSON.
             * @function toJSON
             * @memberof greenlight.CloseChannelRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            CloseChannelRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return CloseChannelRequest;
        })();
    
        /**
         * CloseChannelType enum.
         * @name greenlight.CloseChannelType
         * @enum {number}
         * @property {number} MUTUAL=0 MUTUAL value
         * @property {number} UNILATERAL=1 UNILATERAL value
         */
        greenlight.CloseChannelType = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "MUTUAL"] = 0;
            values[valuesById[1] = "UNILATERAL"] = 1;
            return values;
        })();
    
        greenlight.CloseChannelResponse = (function() {
    
            /**
             * Properties of a CloseChannelResponse.
             * @memberof greenlight
             * @interface ICloseChannelResponse
             * @property {greenlight.CloseChannelType|null} [closeType] CloseChannelResponse closeType
             * @property {Uint8Array|null} [tx] CloseChannelResponse tx
             * @property {Uint8Array|null} [txid] CloseChannelResponse txid
             */
    
            /**
             * Constructs a new CloseChannelResponse.
             * @memberof greenlight
             * @classdesc Represents a CloseChannelResponse.
             * @implements ICloseChannelResponse
             * @constructor
             * @param {greenlight.ICloseChannelResponse=} [properties] Properties to set
             */
            function CloseChannelResponse(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * CloseChannelResponse closeType.
             * @member {greenlight.CloseChannelType} closeType
             * @memberof greenlight.CloseChannelResponse
             * @instance
             */
            CloseChannelResponse.prototype.closeType = 0;
    
            /**
             * CloseChannelResponse tx.
             * @member {Uint8Array} tx
             * @memberof greenlight.CloseChannelResponse
             * @instance
             */
            CloseChannelResponse.prototype.tx = $util.newBuffer([]);
    
            /**
             * CloseChannelResponse txid.
             * @member {Uint8Array} txid
             * @memberof greenlight.CloseChannelResponse
             * @instance
             */
            CloseChannelResponse.prototype.txid = $util.newBuffer([]);
    
            /**
             * Creates a new CloseChannelResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {greenlight.ICloseChannelResponse=} [properties] Properties to set
             * @returns {greenlight.CloseChannelResponse} CloseChannelResponse instance
             */
            CloseChannelResponse.create = function create(properties) {
                return new CloseChannelResponse(properties);
            };
    
            /**
             * Encodes the specified CloseChannelResponse message. Does not implicitly {@link greenlight.CloseChannelResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {greenlight.ICloseChannelResponse} message CloseChannelResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CloseChannelResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.closeType != null && Object.hasOwnProperty.call(message, "closeType"))
                    writer.uint32(/* id 1, wireType 0 =*/8).int32(message.closeType);
                if (message.tx != null && Object.hasOwnProperty.call(message, "tx"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.tx);
                if (message.txid != null && Object.hasOwnProperty.call(message, "txid"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.txid);
                return writer;
            };
    
            /**
             * Encodes the specified CloseChannelResponse message, length delimited. Does not implicitly {@link greenlight.CloseChannelResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {greenlight.ICloseChannelResponse} message CloseChannelResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            CloseChannelResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a CloseChannelResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.CloseChannelResponse} CloseChannelResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CloseChannelResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.CloseChannelResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.closeType = reader.int32();
                        break;
                    case 2:
                        message.tx = reader.bytes();
                        break;
                    case 3:
                        message.txid = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a CloseChannelResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.CloseChannelResponse} CloseChannelResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            CloseChannelResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a CloseChannelResponse message.
             * @function verify
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            CloseChannelResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.closeType != null && message.hasOwnProperty("closeType"))
                    switch (message.closeType) {
                    default:
                        return "closeType: enum value expected";
                    case 0:
                    case 1:
                        break;
                    }
                if (message.tx != null && message.hasOwnProperty("tx"))
                    if (!(message.tx && typeof message.tx.length === "number" || $util.isString(message.tx)))
                        return "tx: buffer expected";
                if (message.txid != null && message.hasOwnProperty("txid"))
                    if (!(message.txid && typeof message.txid.length === "number" || $util.isString(message.txid)))
                        return "txid: buffer expected";
                return null;
            };
    
            /**
             * Creates a CloseChannelResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.CloseChannelResponse} CloseChannelResponse
             */
            CloseChannelResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.CloseChannelResponse)
                    return object;
                var message = new $root.greenlight.CloseChannelResponse();
                switch (object.closeType) {
                case "MUTUAL":
                case 0:
                    message.closeType = 0;
                    break;
                case "UNILATERAL":
                case 1:
                    message.closeType = 1;
                    break;
                }
                if (object.tx != null)
                    if (typeof object.tx === "string")
                        $util.base64.decode(object.tx, message.tx = $util.newBuffer($util.base64.length(object.tx)), 0);
                    else if (object.tx.length)
                        message.tx = object.tx;
                if (object.txid != null)
                    if (typeof object.txid === "string")
                        $util.base64.decode(object.txid, message.txid = $util.newBuffer($util.base64.length(object.txid)), 0);
                    else if (object.txid.length)
                        message.txid = object.txid;
                return message;
            };
    
            /**
             * Creates a plain object from a CloseChannelResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.CloseChannelResponse
             * @static
             * @param {greenlight.CloseChannelResponse} message CloseChannelResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            CloseChannelResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.closeType = options.enums === String ? "MUTUAL" : 0;
                    if (options.bytes === String)
                        object.tx = "";
                    else {
                        object.tx = [];
                        if (options.bytes !== Array)
                            object.tx = $util.newBuffer(object.tx);
                    }
                    if (options.bytes === String)
                        object.txid = "";
                    else {
                        object.txid = [];
                        if (options.bytes !== Array)
                            object.txid = $util.newBuffer(object.txid);
                    }
                }
                if (message.closeType != null && message.hasOwnProperty("closeType"))
                    object.closeType = options.enums === String ? $root.greenlight.CloseChannelType[message.closeType] : message.closeType;
                if (message.tx != null && message.hasOwnProperty("tx"))
                    object.tx = options.bytes === String ? $util.base64.encode(message.tx, 0, message.tx.length) : options.bytes === Array ? Array.prototype.slice.call(message.tx) : message.tx;
                if (message.txid != null && message.hasOwnProperty("txid"))
                    object.txid = options.bytes === String ? $util.base64.encode(message.txid, 0, message.txid.length) : options.bytes === Array ? Array.prototype.slice.call(message.txid) : message.txid;
                return object;
            };
    
            /**
             * Converts this CloseChannelResponse to JSON.
             * @function toJSON
             * @memberof greenlight.CloseChannelResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            CloseChannelResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return CloseChannelResponse;
        })();
    
        greenlight.Amount = (function() {
    
            /**
             * Properties of an Amount.
             * @memberof greenlight
             * @interface IAmount
             * @property {number|Long|null} [millisatoshi] Amount millisatoshi
             * @property {number|Long|null} [satoshi] Amount satoshi
             * @property {number|Long|null} [bitcoin] Amount bitcoin
             * @property {boolean|null} [all] Amount all
             * @property {boolean|null} [any] Amount any
             */
    
            /**
             * Constructs a new Amount.
             * @memberof greenlight
             * @classdesc Represents an Amount.
             * @implements IAmount
             * @constructor
             * @param {greenlight.IAmount=} [properties] Properties to set
             */
            function Amount(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Amount millisatoshi.
             * @member {number|Long|null|undefined} millisatoshi
             * @memberof greenlight.Amount
             * @instance
             */
            Amount.prototype.millisatoshi = null;
    
            /**
             * Amount satoshi.
             * @member {number|Long|null|undefined} satoshi
             * @memberof greenlight.Amount
             * @instance
             */
            Amount.prototype.satoshi = null;
    
            /**
             * Amount bitcoin.
             * @member {number|Long|null|undefined} bitcoin
             * @memberof greenlight.Amount
             * @instance
             */
            Amount.prototype.bitcoin = null;
    
            /**
             * Amount all.
             * @member {boolean|null|undefined} all
             * @memberof greenlight.Amount
             * @instance
             */
            Amount.prototype.all = null;
    
            /**
             * Amount any.
             * @member {boolean|null|undefined} any
             * @memberof greenlight.Amount
             * @instance
             */
            Amount.prototype.any = null;
    
            // OneOf field names bound to virtual getters and setters
            var $oneOfFields;
    
            /**
             * Amount unit.
             * @member {"millisatoshi"|"satoshi"|"bitcoin"|"all"|"any"|undefined} unit
             * @memberof greenlight.Amount
             * @instance
             */
            Object.defineProperty(Amount.prototype, "unit", {
                get: $util.oneOfGetter($oneOfFields = ["millisatoshi", "satoshi", "bitcoin", "all", "any"]),
                set: $util.oneOfSetter($oneOfFields)
            });
    
            /**
             * Creates a new Amount instance using the specified properties.
             * @function create
             * @memberof greenlight.Amount
             * @static
             * @param {greenlight.IAmount=} [properties] Properties to set
             * @returns {greenlight.Amount} Amount instance
             */
            Amount.create = function create(properties) {
                return new Amount(properties);
            };
    
            /**
             * Encodes the specified Amount message. Does not implicitly {@link greenlight.Amount.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Amount
             * @static
             * @param {greenlight.IAmount} message Amount message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Amount.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.millisatoshi != null && Object.hasOwnProperty.call(message, "millisatoshi"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.millisatoshi);
                if (message.satoshi != null && Object.hasOwnProperty.call(message, "satoshi"))
                    writer.uint32(/* id 2, wireType 0 =*/16).uint64(message.satoshi);
                if (message.bitcoin != null && Object.hasOwnProperty.call(message, "bitcoin"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.bitcoin);
                if (message.all != null && Object.hasOwnProperty.call(message, "all"))
                    writer.uint32(/* id 4, wireType 0 =*/32).bool(message.all);
                if (message.any != null && Object.hasOwnProperty.call(message, "any"))
                    writer.uint32(/* id 5, wireType 0 =*/40).bool(message.any);
                return writer;
            };
    
            /**
             * Encodes the specified Amount message, length delimited. Does not implicitly {@link greenlight.Amount.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Amount
             * @static
             * @param {greenlight.IAmount} message Amount message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Amount.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an Amount message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Amount
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Amount} Amount
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Amount.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Amount();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.millisatoshi = reader.uint64();
                        break;
                    case 2:
                        message.satoshi = reader.uint64();
                        break;
                    case 3:
                        message.bitcoin = reader.uint64();
                        break;
                    case 4:
                        message.all = reader.bool();
                        break;
                    case 5:
                        message.any = reader.bool();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an Amount message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Amount
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Amount} Amount
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Amount.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an Amount message.
             * @function verify
             * @memberof greenlight.Amount
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Amount.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                var properties = {};
                if (message.millisatoshi != null && message.hasOwnProperty("millisatoshi")) {
                    properties.unit = 1;
                    if (!$util.isInteger(message.millisatoshi) && !(message.millisatoshi && $util.isInteger(message.millisatoshi.low) && $util.isInteger(message.millisatoshi.high)))
                        return "millisatoshi: integer|Long expected";
                }
                if (message.satoshi != null && message.hasOwnProperty("satoshi")) {
                    if (properties.unit === 1)
                        return "unit: multiple values";
                    properties.unit = 1;
                    if (!$util.isInteger(message.satoshi) && !(message.satoshi && $util.isInteger(message.satoshi.low) && $util.isInteger(message.satoshi.high)))
                        return "satoshi: integer|Long expected";
                }
                if (message.bitcoin != null && message.hasOwnProperty("bitcoin")) {
                    if (properties.unit === 1)
                        return "unit: multiple values";
                    properties.unit = 1;
                    if (!$util.isInteger(message.bitcoin) && !(message.bitcoin && $util.isInteger(message.bitcoin.low) && $util.isInteger(message.bitcoin.high)))
                        return "bitcoin: integer|Long expected";
                }
                if (message.all != null && message.hasOwnProperty("all")) {
                    if (properties.unit === 1)
                        return "unit: multiple values";
                    properties.unit = 1;
                    if (typeof message.all !== "boolean")
                        return "all: boolean expected";
                }
                if (message.any != null && message.hasOwnProperty("any")) {
                    if (properties.unit === 1)
                        return "unit: multiple values";
                    properties.unit = 1;
                    if (typeof message.any !== "boolean")
                        return "any: boolean expected";
                }
                return null;
            };
    
            /**
             * Creates an Amount message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Amount
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Amount} Amount
             */
            Amount.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Amount)
                    return object;
                var message = new $root.greenlight.Amount();
                if (object.millisatoshi != null)
                    if ($util.Long)
                        (message.millisatoshi = $util.Long.fromValue(object.millisatoshi)).unsigned = true;
                    else if (typeof object.millisatoshi === "string")
                        message.millisatoshi = parseInt(object.millisatoshi, 10);
                    else if (typeof object.millisatoshi === "number")
                        message.millisatoshi = object.millisatoshi;
                    else if (typeof object.millisatoshi === "object")
                        message.millisatoshi = new $util.LongBits(object.millisatoshi.low >>> 0, object.millisatoshi.high >>> 0).toNumber(true);
                if (object.satoshi != null)
                    if ($util.Long)
                        (message.satoshi = $util.Long.fromValue(object.satoshi)).unsigned = true;
                    else if (typeof object.satoshi === "string")
                        message.satoshi = parseInt(object.satoshi, 10);
                    else if (typeof object.satoshi === "number")
                        message.satoshi = object.satoshi;
                    else if (typeof object.satoshi === "object")
                        message.satoshi = new $util.LongBits(object.satoshi.low >>> 0, object.satoshi.high >>> 0).toNumber(true);
                if (object.bitcoin != null)
                    if ($util.Long)
                        (message.bitcoin = $util.Long.fromValue(object.bitcoin)).unsigned = true;
                    else if (typeof object.bitcoin === "string")
                        message.bitcoin = parseInt(object.bitcoin, 10);
                    else if (typeof object.bitcoin === "number")
                        message.bitcoin = object.bitcoin;
                    else if (typeof object.bitcoin === "object")
                        message.bitcoin = new $util.LongBits(object.bitcoin.low >>> 0, object.bitcoin.high >>> 0).toNumber(true);
                if (object.all != null)
                    message.all = Boolean(object.all);
                if (object.any != null)
                    message.any = Boolean(object.any);
                return message;
            };
    
            /**
             * Creates a plain object from an Amount message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Amount
             * @static
             * @param {greenlight.Amount} message Amount
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Amount.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (message.millisatoshi != null && message.hasOwnProperty("millisatoshi")) {
                    if (typeof message.millisatoshi === "number")
                        object.millisatoshi = options.longs === String ? String(message.millisatoshi) : message.millisatoshi;
                    else
                        object.millisatoshi = options.longs === String ? $util.Long.prototype.toString.call(message.millisatoshi) : options.longs === Number ? new $util.LongBits(message.millisatoshi.low >>> 0, message.millisatoshi.high >>> 0).toNumber(true) : message.millisatoshi;
                    if (options.oneofs)
                        object.unit = "millisatoshi";
                }
                if (message.satoshi != null && message.hasOwnProperty("satoshi")) {
                    if (typeof message.satoshi === "number")
                        object.satoshi = options.longs === String ? String(message.satoshi) : message.satoshi;
                    else
                        object.satoshi = options.longs === String ? $util.Long.prototype.toString.call(message.satoshi) : options.longs === Number ? new $util.LongBits(message.satoshi.low >>> 0, message.satoshi.high >>> 0).toNumber(true) : message.satoshi;
                    if (options.oneofs)
                        object.unit = "satoshi";
                }
                if (message.bitcoin != null && message.hasOwnProperty("bitcoin")) {
                    if (typeof message.bitcoin === "number")
                        object.bitcoin = options.longs === String ? String(message.bitcoin) : message.bitcoin;
                    else
                        object.bitcoin = options.longs === String ? $util.Long.prototype.toString.call(message.bitcoin) : options.longs === Number ? new $util.LongBits(message.bitcoin.low >>> 0, message.bitcoin.high >>> 0).toNumber(true) : message.bitcoin;
                    if (options.oneofs)
                        object.unit = "bitcoin";
                }
                if (message.all != null && message.hasOwnProperty("all")) {
                    object.all = message.all;
                    if (options.oneofs)
                        object.unit = "all";
                }
                if (message.any != null && message.hasOwnProperty("any")) {
                    object.any = message.any;
                    if (options.oneofs)
                        object.unit = "any";
                }
                return object;
            };
    
            /**
             * Converts this Amount to JSON.
             * @function toJSON
             * @memberof greenlight.Amount
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Amount.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Amount;
        })();
    
        greenlight.InvoiceRequest = (function() {
    
            /**
             * Properties of an InvoiceRequest.
             * @memberof greenlight
             * @interface IInvoiceRequest
             * @property {greenlight.IAmount|null} [amount] InvoiceRequest amount
             * @property {string|null} [label] InvoiceRequest label
             * @property {string|null} [description] InvoiceRequest description
             */
    
            /**
             * Constructs a new InvoiceRequest.
             * @memberof greenlight
             * @classdesc Represents an InvoiceRequest.
             * @implements IInvoiceRequest
             * @constructor
             * @param {greenlight.IInvoiceRequest=} [properties] Properties to set
             */
            function InvoiceRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * InvoiceRequest amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.InvoiceRequest
             * @instance
             */
            InvoiceRequest.prototype.amount = null;
    
            /**
             * InvoiceRequest label.
             * @member {string} label
             * @memberof greenlight.InvoiceRequest
             * @instance
             */
            InvoiceRequest.prototype.label = "";
    
            /**
             * InvoiceRequest description.
             * @member {string} description
             * @memberof greenlight.InvoiceRequest
             * @instance
             */
            InvoiceRequest.prototype.description = "";
    
            /**
             * Creates a new InvoiceRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {greenlight.IInvoiceRequest=} [properties] Properties to set
             * @returns {greenlight.InvoiceRequest} InvoiceRequest instance
             */
            InvoiceRequest.create = function create(properties) {
                return new InvoiceRequest(properties);
            };
    
            /**
             * Encodes the specified InvoiceRequest message. Does not implicitly {@link greenlight.InvoiceRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {greenlight.IInvoiceRequest} message InvoiceRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            InvoiceRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.label);
                if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.description);
                return writer;
            };
    
            /**
             * Encodes the specified InvoiceRequest message, length delimited. Does not implicitly {@link greenlight.InvoiceRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {greenlight.IInvoiceRequest} message InvoiceRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            InvoiceRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an InvoiceRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.InvoiceRequest} InvoiceRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            InvoiceRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.InvoiceRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 2:
                        message.label = reader.string();
                        break;
                    case 3:
                        message.description = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an InvoiceRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.InvoiceRequest} InvoiceRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            InvoiceRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an InvoiceRequest message.
             * @function verify
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            InvoiceRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.label != null && message.hasOwnProperty("label"))
                    if (!$util.isString(message.label))
                        return "label: string expected";
                if (message.description != null && message.hasOwnProperty("description"))
                    if (!$util.isString(message.description))
                        return "description: string expected";
                return null;
            };
    
            /**
             * Creates an InvoiceRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.InvoiceRequest} InvoiceRequest
             */
            InvoiceRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.InvoiceRequest)
                    return object;
                var message = new $root.greenlight.InvoiceRequest();
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.InvoiceRequest.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.label != null)
                    message.label = String(object.label);
                if (object.description != null)
                    message.description = String(object.description);
                return message;
            };
    
            /**
             * Creates a plain object from an InvoiceRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.InvoiceRequest
             * @static
             * @param {greenlight.InvoiceRequest} message InvoiceRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            InvoiceRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.amount = null;
                    object.label = "";
                    object.description = "";
                }
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.label != null && message.hasOwnProperty("label"))
                    object.label = message.label;
                if (message.description != null && message.hasOwnProperty("description"))
                    object.description = message.description;
                return object;
            };
    
            /**
             * Converts this InvoiceRequest to JSON.
             * @function toJSON
             * @memberof greenlight.InvoiceRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            InvoiceRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return InvoiceRequest;
        })();
    
        /**
         * InvoiceStatus enum.
         * @name greenlight.InvoiceStatus
         * @enum {number}
         * @property {number} UNPAID=0 UNPAID value
         * @property {number} PAID=1 PAID value
         * @property {number} EXPIRED=2 EXPIRED value
         */
        greenlight.InvoiceStatus = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "UNPAID"] = 0;
            values[valuesById[1] = "PAID"] = 1;
            values[valuesById[2] = "EXPIRED"] = 2;
            return values;
        })();
    
        greenlight.Invoice = (function() {
    
            /**
             * Properties of an Invoice.
             * @memberof greenlight
             * @interface IInvoice
             * @property {string|null} [label] Invoice label
             * @property {string|null} [description] Invoice description
             * @property {greenlight.IAmount|null} [amount] Invoice amount
             * @property {greenlight.IAmount|null} [received] Invoice received
             * @property {greenlight.InvoiceStatus|null} [status] Invoice status
             * @property {number|null} [paymentTime] Invoice paymentTime
             * @property {number|null} [expiryTime] Invoice expiryTime
             * @property {string|null} [bolt11] Invoice bolt11
             * @property {Uint8Array|null} [paymentHash] Invoice paymentHash
             * @property {Uint8Array|null} [paymentPreimage] Invoice paymentPreimage
             */
    
            /**
             * Constructs a new Invoice.
             * @memberof greenlight
             * @classdesc Represents an Invoice.
             * @implements IInvoice
             * @constructor
             * @param {greenlight.IInvoice=} [properties] Properties to set
             */
            function Invoice(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Invoice label.
             * @member {string} label
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.label = "";
    
            /**
             * Invoice description.
             * @member {string} description
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.description = "";
    
            /**
             * Invoice amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.amount = null;
    
            /**
             * Invoice received.
             * @member {greenlight.IAmount|null|undefined} received
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.received = null;
    
            /**
             * Invoice status.
             * @member {greenlight.InvoiceStatus} status
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.status = 0;
    
            /**
             * Invoice paymentTime.
             * @member {number} paymentTime
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.paymentTime = 0;
    
            /**
             * Invoice expiryTime.
             * @member {number} expiryTime
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.expiryTime = 0;
    
            /**
             * Invoice bolt11.
             * @member {string} bolt11
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.bolt11 = "";
    
            /**
             * Invoice paymentHash.
             * @member {Uint8Array} paymentHash
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.paymentHash = $util.newBuffer([]);
    
            /**
             * Invoice paymentPreimage.
             * @member {Uint8Array} paymentPreimage
             * @memberof greenlight.Invoice
             * @instance
             */
            Invoice.prototype.paymentPreimage = $util.newBuffer([]);
    
            /**
             * Creates a new Invoice instance using the specified properties.
             * @function create
             * @memberof greenlight.Invoice
             * @static
             * @param {greenlight.IInvoice=} [properties] Properties to set
             * @returns {greenlight.Invoice} Invoice instance
             */
            Invoice.create = function create(properties) {
                return new Invoice(properties);
            };
    
            /**
             * Encodes the specified Invoice message. Does not implicitly {@link greenlight.Invoice.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Invoice
             * @static
             * @param {greenlight.IInvoice} message Invoice message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Invoice.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.label);
                if (message.description != null && Object.hasOwnProperty.call(message, "description"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.description);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.received != null && Object.hasOwnProperty.call(message, "received"))
                    $root.greenlight.Amount.encode(message.received, writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                    writer.uint32(/* id 5, wireType 0 =*/40).int32(message.status);
                if (message.paymentTime != null && Object.hasOwnProperty.call(message, "paymentTime"))
                    writer.uint32(/* id 6, wireType 0 =*/48).uint32(message.paymentTime);
                if (message.expiryTime != null && Object.hasOwnProperty.call(message, "expiryTime"))
                    writer.uint32(/* id 7, wireType 0 =*/56).uint32(message.expiryTime);
                if (message.bolt11 != null && Object.hasOwnProperty.call(message, "bolt11"))
                    writer.uint32(/* id 8, wireType 2 =*/66).string(message.bolt11);
                if (message.paymentHash != null && Object.hasOwnProperty.call(message, "paymentHash"))
                    writer.uint32(/* id 9, wireType 2 =*/74).bytes(message.paymentHash);
                if (message.paymentPreimage != null && Object.hasOwnProperty.call(message, "paymentPreimage"))
                    writer.uint32(/* id 10, wireType 2 =*/82).bytes(message.paymentPreimage);
                return writer;
            };
    
            /**
             * Encodes the specified Invoice message, length delimited. Does not implicitly {@link greenlight.Invoice.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Invoice
             * @static
             * @param {greenlight.IInvoice} message Invoice message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Invoice.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an Invoice message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Invoice
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Invoice} Invoice
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Invoice.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Invoice();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.label = reader.string();
                        break;
                    case 2:
                        message.description = reader.string();
                        break;
                    case 3:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 4:
                        message.received = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 5:
                        message.status = reader.int32();
                        break;
                    case 6:
                        message.paymentTime = reader.uint32();
                        break;
                    case 7:
                        message.expiryTime = reader.uint32();
                        break;
                    case 8:
                        message.bolt11 = reader.string();
                        break;
                    case 9:
                        message.paymentHash = reader.bytes();
                        break;
                    case 10:
                        message.paymentPreimage = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an Invoice message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Invoice
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Invoice} Invoice
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Invoice.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an Invoice message.
             * @function verify
             * @memberof greenlight.Invoice
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Invoice.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.label != null && message.hasOwnProperty("label"))
                    if (!$util.isString(message.label))
                        return "label: string expected";
                if (message.description != null && message.hasOwnProperty("description"))
                    if (!$util.isString(message.description))
                        return "description: string expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.received != null && message.hasOwnProperty("received")) {
                    var error = $root.greenlight.Amount.verify(message.received);
                    if (error)
                        return "received." + error;
                }
                if (message.status != null && message.hasOwnProperty("status"))
                    switch (message.status) {
                    default:
                        return "status: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                        break;
                    }
                if (message.paymentTime != null && message.hasOwnProperty("paymentTime"))
                    if (!$util.isInteger(message.paymentTime))
                        return "paymentTime: integer expected";
                if (message.expiryTime != null && message.hasOwnProperty("expiryTime"))
                    if (!$util.isInteger(message.expiryTime))
                        return "expiryTime: integer expected";
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    if (!$util.isString(message.bolt11))
                        return "bolt11: string expected";
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    if (!(message.paymentHash && typeof message.paymentHash.length === "number" || $util.isString(message.paymentHash)))
                        return "paymentHash: buffer expected";
                if (message.paymentPreimage != null && message.hasOwnProperty("paymentPreimage"))
                    if (!(message.paymentPreimage && typeof message.paymentPreimage.length === "number" || $util.isString(message.paymentPreimage)))
                        return "paymentPreimage: buffer expected";
                return null;
            };
    
            /**
             * Creates an Invoice message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Invoice
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Invoice} Invoice
             */
            Invoice.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Invoice)
                    return object;
                var message = new $root.greenlight.Invoice();
                if (object.label != null)
                    message.label = String(object.label);
                if (object.description != null)
                    message.description = String(object.description);
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.Invoice.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.received != null) {
                    if (typeof object.received !== "object")
                        throw TypeError(".greenlight.Invoice.received: object expected");
                    message.received = $root.greenlight.Amount.fromObject(object.received);
                }
                switch (object.status) {
                case "UNPAID":
                case 0:
                    message.status = 0;
                    break;
                case "PAID":
                case 1:
                    message.status = 1;
                    break;
                case "EXPIRED":
                case 2:
                    message.status = 2;
                    break;
                }
                if (object.paymentTime != null)
                    message.paymentTime = object.paymentTime >>> 0;
                if (object.expiryTime != null)
                    message.expiryTime = object.expiryTime >>> 0;
                if (object.bolt11 != null)
                    message.bolt11 = String(object.bolt11);
                if (object.paymentHash != null)
                    if (typeof object.paymentHash === "string")
                        $util.base64.decode(object.paymentHash, message.paymentHash = $util.newBuffer($util.base64.length(object.paymentHash)), 0);
                    else if (object.paymentHash.length)
                        message.paymentHash = object.paymentHash;
                if (object.paymentPreimage != null)
                    if (typeof object.paymentPreimage === "string")
                        $util.base64.decode(object.paymentPreimage, message.paymentPreimage = $util.newBuffer($util.base64.length(object.paymentPreimage)), 0);
                    else if (object.paymentPreimage.length)
                        message.paymentPreimage = object.paymentPreimage;
                return message;
            };
    
            /**
             * Creates a plain object from an Invoice message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Invoice
             * @static
             * @param {greenlight.Invoice} message Invoice
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Invoice.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.label = "";
                    object.description = "";
                    object.amount = null;
                    object.received = null;
                    object.status = options.enums === String ? "UNPAID" : 0;
                    object.paymentTime = 0;
                    object.expiryTime = 0;
                    object.bolt11 = "";
                    if (options.bytes === String)
                        object.paymentHash = "";
                    else {
                        object.paymentHash = [];
                        if (options.bytes !== Array)
                            object.paymentHash = $util.newBuffer(object.paymentHash);
                    }
                    if (options.bytes === String)
                        object.paymentPreimage = "";
                    else {
                        object.paymentPreimage = [];
                        if (options.bytes !== Array)
                            object.paymentPreimage = $util.newBuffer(object.paymentPreimage);
                    }
                }
                if (message.label != null && message.hasOwnProperty("label"))
                    object.label = message.label;
                if (message.description != null && message.hasOwnProperty("description"))
                    object.description = message.description;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.received != null && message.hasOwnProperty("received"))
                    object.received = $root.greenlight.Amount.toObject(message.received, options);
                if (message.status != null && message.hasOwnProperty("status"))
                    object.status = options.enums === String ? $root.greenlight.InvoiceStatus[message.status] : message.status;
                if (message.paymentTime != null && message.hasOwnProperty("paymentTime"))
                    object.paymentTime = message.paymentTime;
                if (message.expiryTime != null && message.hasOwnProperty("expiryTime"))
                    object.expiryTime = message.expiryTime;
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    object.bolt11 = message.bolt11;
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    object.paymentHash = options.bytes === String ? $util.base64.encode(message.paymentHash, 0, message.paymentHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentHash) : message.paymentHash;
                if (message.paymentPreimage != null && message.hasOwnProperty("paymentPreimage"))
                    object.paymentPreimage = options.bytes === String ? $util.base64.encode(message.paymentPreimage, 0, message.paymentPreimage.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentPreimage) : message.paymentPreimage;
                return object;
            };
    
            /**
             * Converts this Invoice to JSON.
             * @function toJSON
             * @memberof greenlight.Invoice
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Invoice.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Invoice;
        })();
    
        greenlight.PayRequest = (function() {
    
            /**
             * Properties of a PayRequest.
             * @memberof greenlight
             * @interface IPayRequest
             * @property {string|null} [bolt11] PayRequest bolt11
             * @property {greenlight.IAmount|null} [amount] PayRequest amount
             * @property {number|null} [timeout] PayRequest timeout
             */
    
            /**
             * Constructs a new PayRequest.
             * @memberof greenlight
             * @classdesc Represents a PayRequest.
             * @implements IPayRequest
             * @constructor
             * @param {greenlight.IPayRequest=} [properties] Properties to set
             */
            function PayRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * PayRequest bolt11.
             * @member {string} bolt11
             * @memberof greenlight.PayRequest
             * @instance
             */
            PayRequest.prototype.bolt11 = "";
    
            /**
             * PayRequest amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.PayRequest
             * @instance
             */
            PayRequest.prototype.amount = null;
    
            /**
             * PayRequest timeout.
             * @member {number} timeout
             * @memberof greenlight.PayRequest
             * @instance
             */
            PayRequest.prototype.timeout = 0;
    
            /**
             * Creates a new PayRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.PayRequest
             * @static
             * @param {greenlight.IPayRequest=} [properties] Properties to set
             * @returns {greenlight.PayRequest} PayRequest instance
             */
            PayRequest.create = function create(properties) {
                return new PayRequest(properties);
            };
    
            /**
             * Encodes the specified PayRequest message. Does not implicitly {@link greenlight.PayRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.PayRequest
             * @static
             * @param {greenlight.IPayRequest} message PayRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PayRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.bolt11 != null && Object.hasOwnProperty.call(message, "bolt11"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.bolt11);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.timeout != null && Object.hasOwnProperty.call(message, "timeout"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint32(message.timeout);
                return writer;
            };
    
            /**
             * Encodes the specified PayRequest message, length delimited. Does not implicitly {@link greenlight.PayRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.PayRequest
             * @static
             * @param {greenlight.IPayRequest} message PayRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PayRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a PayRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.PayRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.PayRequest} PayRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PayRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.PayRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.bolt11 = reader.string();
                        break;
                    case 2:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.timeout = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a PayRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.PayRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.PayRequest} PayRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PayRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a PayRequest message.
             * @function verify
             * @memberof greenlight.PayRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PayRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    if (!$util.isString(message.bolt11))
                        return "bolt11: string expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.timeout != null && message.hasOwnProperty("timeout"))
                    if (!$util.isInteger(message.timeout))
                        return "timeout: integer expected";
                return null;
            };
    
            /**
             * Creates a PayRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.PayRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.PayRequest} PayRequest
             */
            PayRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.PayRequest)
                    return object;
                var message = new $root.greenlight.PayRequest();
                if (object.bolt11 != null)
                    message.bolt11 = String(object.bolt11);
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.PayRequest.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.timeout != null)
                    message.timeout = object.timeout >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from a PayRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.PayRequest
             * @static
             * @param {greenlight.PayRequest} message PayRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PayRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    object.bolt11 = "";
                    object.amount = null;
                    object.timeout = 0;
                }
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    object.bolt11 = message.bolt11;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.timeout != null && message.hasOwnProperty("timeout"))
                    object.timeout = message.timeout;
                return object;
            };
    
            /**
             * Converts this PayRequest to JSON.
             * @function toJSON
             * @memberof greenlight.PayRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PayRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return PayRequest;
        })();
    
        /**
         * PayStatus enum.
         * @name greenlight.PayStatus
         * @enum {number}
         * @property {number} PENDING=0 PENDING value
         * @property {number} COMPLETE=1 COMPLETE value
         * @property {number} FAILED=2 FAILED value
         */
        greenlight.PayStatus = (function() {
            var valuesById = {}, values = Object.create(valuesById);
            values[valuesById[0] = "PENDING"] = 0;
            values[valuesById[1] = "COMPLETE"] = 1;
            values[valuesById[2] = "FAILED"] = 2;
            return values;
        })();
    
        greenlight.Payment = (function() {
    
            /**
             * Properties of a Payment.
             * @memberof greenlight
             * @interface IPayment
             * @property {Uint8Array|null} [destination] Payment destination
             * @property {Uint8Array|null} [paymentHash] Payment paymentHash
             * @property {Uint8Array|null} [paymentPreimage] Payment paymentPreimage
             * @property {greenlight.PayStatus|null} [status] Payment status
             * @property {greenlight.IAmount|null} [amount] Payment amount
             * @property {greenlight.IAmount|null} [amountSent] Payment amountSent
             * @property {string|null} [bolt11] Payment bolt11
             */
    
            /**
             * Constructs a new Payment.
             * @memberof greenlight
             * @classdesc Represents a Payment.
             * @implements IPayment
             * @constructor
             * @param {greenlight.IPayment=} [properties] Properties to set
             */
            function Payment(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Payment destination.
             * @member {Uint8Array} destination
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.destination = $util.newBuffer([]);
    
            /**
             * Payment paymentHash.
             * @member {Uint8Array} paymentHash
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.paymentHash = $util.newBuffer([]);
    
            /**
             * Payment paymentPreimage.
             * @member {Uint8Array} paymentPreimage
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.paymentPreimage = $util.newBuffer([]);
    
            /**
             * Payment status.
             * @member {greenlight.PayStatus} status
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.status = 0;
    
            /**
             * Payment amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.amount = null;
    
            /**
             * Payment amountSent.
             * @member {greenlight.IAmount|null|undefined} amountSent
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.amountSent = null;
    
            /**
             * Payment bolt11.
             * @member {string} bolt11
             * @memberof greenlight.Payment
             * @instance
             */
            Payment.prototype.bolt11 = "";
    
            /**
             * Creates a new Payment instance using the specified properties.
             * @function create
             * @memberof greenlight.Payment
             * @static
             * @param {greenlight.IPayment=} [properties] Properties to set
             * @returns {greenlight.Payment} Payment instance
             */
            Payment.create = function create(properties) {
                return new Payment(properties);
            };
    
            /**
             * Encodes the specified Payment message. Does not implicitly {@link greenlight.Payment.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Payment
             * @static
             * @param {greenlight.IPayment} message Payment message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Payment.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.destination != null && Object.hasOwnProperty.call(message, "destination"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.destination);
                if (message.paymentHash != null && Object.hasOwnProperty.call(message, "paymentHash"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.paymentHash);
                if (message.paymentPreimage != null && Object.hasOwnProperty.call(message, "paymentPreimage"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.paymentPreimage);
                if (message.status != null && Object.hasOwnProperty.call(message, "status"))
                    writer.uint32(/* id 4, wireType 0 =*/32).int32(message.status);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                if (message.amountSent != null && Object.hasOwnProperty.call(message, "amountSent"))
                    $root.greenlight.Amount.encode(message.amountSent, writer.uint32(/* id 6, wireType 2 =*/50).fork()).ldelim();
                if (message.bolt11 != null && Object.hasOwnProperty.call(message, "bolt11"))
                    writer.uint32(/* id 7, wireType 2 =*/58).string(message.bolt11);
                return writer;
            };
    
            /**
             * Encodes the specified Payment message, length delimited. Does not implicitly {@link greenlight.Payment.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Payment
             * @static
             * @param {greenlight.IPayment} message Payment message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Payment.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Payment message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Payment
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Payment} Payment
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Payment.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Payment();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.destination = reader.bytes();
                        break;
                    case 2:
                        message.paymentHash = reader.bytes();
                        break;
                    case 3:
                        message.paymentPreimage = reader.bytes();
                        break;
                    case 4:
                        message.status = reader.int32();
                        break;
                    case 5:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 6:
                        message.amountSent = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 7:
                        message.bolt11 = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Payment message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Payment
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Payment} Payment
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Payment.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Payment message.
             * @function verify
             * @memberof greenlight.Payment
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Payment.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.destination != null && message.hasOwnProperty("destination"))
                    if (!(message.destination && typeof message.destination.length === "number" || $util.isString(message.destination)))
                        return "destination: buffer expected";
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    if (!(message.paymentHash && typeof message.paymentHash.length === "number" || $util.isString(message.paymentHash)))
                        return "paymentHash: buffer expected";
                if (message.paymentPreimage != null && message.hasOwnProperty("paymentPreimage"))
                    if (!(message.paymentPreimage && typeof message.paymentPreimage.length === "number" || $util.isString(message.paymentPreimage)))
                        return "paymentPreimage: buffer expected";
                if (message.status != null && message.hasOwnProperty("status"))
                    switch (message.status) {
                    default:
                        return "status: enum value expected";
                    case 0:
                    case 1:
                    case 2:
                        break;
                    }
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.amountSent != null && message.hasOwnProperty("amountSent")) {
                    var error = $root.greenlight.Amount.verify(message.amountSent);
                    if (error)
                        return "amountSent." + error;
                }
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    if (!$util.isString(message.bolt11))
                        return "bolt11: string expected";
                return null;
            };
    
            /**
             * Creates a Payment message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Payment
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Payment} Payment
             */
            Payment.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Payment)
                    return object;
                var message = new $root.greenlight.Payment();
                if (object.destination != null)
                    if (typeof object.destination === "string")
                        $util.base64.decode(object.destination, message.destination = $util.newBuffer($util.base64.length(object.destination)), 0);
                    else if (object.destination.length)
                        message.destination = object.destination;
                if (object.paymentHash != null)
                    if (typeof object.paymentHash === "string")
                        $util.base64.decode(object.paymentHash, message.paymentHash = $util.newBuffer($util.base64.length(object.paymentHash)), 0);
                    else if (object.paymentHash.length)
                        message.paymentHash = object.paymentHash;
                if (object.paymentPreimage != null)
                    if (typeof object.paymentPreimage === "string")
                        $util.base64.decode(object.paymentPreimage, message.paymentPreimage = $util.newBuffer($util.base64.length(object.paymentPreimage)), 0);
                    else if (object.paymentPreimage.length)
                        message.paymentPreimage = object.paymentPreimage;
                switch (object.status) {
                case "PENDING":
                case 0:
                    message.status = 0;
                    break;
                case "COMPLETE":
                case 1:
                    message.status = 1;
                    break;
                case "FAILED":
                case 2:
                    message.status = 2;
                    break;
                }
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.Payment.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.amountSent != null) {
                    if (typeof object.amountSent !== "object")
                        throw TypeError(".greenlight.Payment.amountSent: object expected");
                    message.amountSent = $root.greenlight.Amount.fromObject(object.amountSent);
                }
                if (object.bolt11 != null)
                    message.bolt11 = String(object.bolt11);
                return message;
            };
    
            /**
             * Creates a plain object from a Payment message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Payment
             * @static
             * @param {greenlight.Payment} message Payment
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Payment.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.destination = "";
                    else {
                        object.destination = [];
                        if (options.bytes !== Array)
                            object.destination = $util.newBuffer(object.destination);
                    }
                    if (options.bytes === String)
                        object.paymentHash = "";
                    else {
                        object.paymentHash = [];
                        if (options.bytes !== Array)
                            object.paymentHash = $util.newBuffer(object.paymentHash);
                    }
                    if (options.bytes === String)
                        object.paymentPreimage = "";
                    else {
                        object.paymentPreimage = [];
                        if (options.bytes !== Array)
                            object.paymentPreimage = $util.newBuffer(object.paymentPreimage);
                    }
                    object.status = options.enums === String ? "PENDING" : 0;
                    object.amount = null;
                    object.amountSent = null;
                    object.bolt11 = "";
                }
                if (message.destination != null && message.hasOwnProperty("destination"))
                    object.destination = options.bytes === String ? $util.base64.encode(message.destination, 0, message.destination.length) : options.bytes === Array ? Array.prototype.slice.call(message.destination) : message.destination;
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    object.paymentHash = options.bytes === String ? $util.base64.encode(message.paymentHash, 0, message.paymentHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentHash) : message.paymentHash;
                if (message.paymentPreimage != null && message.hasOwnProperty("paymentPreimage"))
                    object.paymentPreimage = options.bytes === String ? $util.base64.encode(message.paymentPreimage, 0, message.paymentPreimage.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentPreimage) : message.paymentPreimage;
                if (message.status != null && message.hasOwnProperty("status"))
                    object.status = options.enums === String ? $root.greenlight.PayStatus[message.status] : message.status;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.amountSent != null && message.hasOwnProperty("amountSent"))
                    object.amountSent = $root.greenlight.Amount.toObject(message.amountSent, options);
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    object.bolt11 = message.bolt11;
                return object;
            };
    
            /**
             * Converts this Payment to JSON.
             * @function toJSON
             * @memberof greenlight.Payment
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Payment.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Payment;
        })();
    
        greenlight.PaymentIdentifier = (function() {
    
            /**
             * Properties of a PaymentIdentifier.
             * @memberof greenlight
             * @interface IPaymentIdentifier
             * @property {string|null} [bolt11] PaymentIdentifier bolt11
             * @property {Uint8Array|null} [paymentHash] PaymentIdentifier paymentHash
             */
    
            /**
             * Constructs a new PaymentIdentifier.
             * @memberof greenlight
             * @classdesc Represents a PaymentIdentifier.
             * @implements IPaymentIdentifier
             * @constructor
             * @param {greenlight.IPaymentIdentifier=} [properties] Properties to set
             */
            function PaymentIdentifier(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * PaymentIdentifier bolt11.
             * @member {string|null|undefined} bolt11
             * @memberof greenlight.PaymentIdentifier
             * @instance
             */
            PaymentIdentifier.prototype.bolt11 = null;
    
            /**
             * PaymentIdentifier paymentHash.
             * @member {Uint8Array|null|undefined} paymentHash
             * @memberof greenlight.PaymentIdentifier
             * @instance
             */
            PaymentIdentifier.prototype.paymentHash = null;
    
            // OneOf field names bound to virtual getters and setters
            var $oneOfFields;
    
            /**
             * PaymentIdentifier id.
             * @member {"bolt11"|"paymentHash"|undefined} id
             * @memberof greenlight.PaymentIdentifier
             * @instance
             */
            Object.defineProperty(PaymentIdentifier.prototype, "id", {
                get: $util.oneOfGetter($oneOfFields = ["bolt11", "paymentHash"]),
                set: $util.oneOfSetter($oneOfFields)
            });
    
            /**
             * Creates a new PaymentIdentifier instance using the specified properties.
             * @function create
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {greenlight.IPaymentIdentifier=} [properties] Properties to set
             * @returns {greenlight.PaymentIdentifier} PaymentIdentifier instance
             */
            PaymentIdentifier.create = function create(properties) {
                return new PaymentIdentifier(properties);
            };
    
            /**
             * Encodes the specified PaymentIdentifier message. Does not implicitly {@link greenlight.PaymentIdentifier.verify|verify} messages.
             * @function encode
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {greenlight.IPaymentIdentifier} message PaymentIdentifier message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PaymentIdentifier.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.bolt11 != null && Object.hasOwnProperty.call(message, "bolt11"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.bolt11);
                if (message.paymentHash != null && Object.hasOwnProperty.call(message, "paymentHash"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.paymentHash);
                return writer;
            };
    
            /**
             * Encodes the specified PaymentIdentifier message, length delimited. Does not implicitly {@link greenlight.PaymentIdentifier.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {greenlight.IPaymentIdentifier} message PaymentIdentifier message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            PaymentIdentifier.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a PaymentIdentifier message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.PaymentIdentifier} PaymentIdentifier
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PaymentIdentifier.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.PaymentIdentifier();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.bolt11 = reader.string();
                        break;
                    case 2:
                        message.paymentHash = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a PaymentIdentifier message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.PaymentIdentifier} PaymentIdentifier
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            PaymentIdentifier.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a PaymentIdentifier message.
             * @function verify
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            PaymentIdentifier.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                var properties = {};
                if (message.bolt11 != null && message.hasOwnProperty("bolt11")) {
                    properties.id = 1;
                    if (!$util.isString(message.bolt11))
                        return "bolt11: string expected";
                }
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash")) {
                    if (properties.id === 1)
                        return "id: multiple values";
                    properties.id = 1;
                    if (!(message.paymentHash && typeof message.paymentHash.length === "number" || $util.isString(message.paymentHash)))
                        return "paymentHash: buffer expected";
                }
                return null;
            };
    
            /**
             * Creates a PaymentIdentifier message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.PaymentIdentifier} PaymentIdentifier
             */
            PaymentIdentifier.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.PaymentIdentifier)
                    return object;
                var message = new $root.greenlight.PaymentIdentifier();
                if (object.bolt11 != null)
                    message.bolt11 = String(object.bolt11);
                if (object.paymentHash != null)
                    if (typeof object.paymentHash === "string")
                        $util.base64.decode(object.paymentHash, message.paymentHash = $util.newBuffer($util.base64.length(object.paymentHash)), 0);
                    else if (object.paymentHash.length)
                        message.paymentHash = object.paymentHash;
                return message;
            };
    
            /**
             * Creates a plain object from a PaymentIdentifier message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.PaymentIdentifier
             * @static
             * @param {greenlight.PaymentIdentifier} message PaymentIdentifier
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            PaymentIdentifier.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (message.bolt11 != null && message.hasOwnProperty("bolt11")) {
                    object.bolt11 = message.bolt11;
                    if (options.oneofs)
                        object.id = "bolt11";
                }
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash")) {
                    object.paymentHash = options.bytes === String ? $util.base64.encode(message.paymentHash, 0, message.paymentHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentHash) : message.paymentHash;
                    if (options.oneofs)
                        object.id = "paymentHash";
                }
                return object;
            };
    
            /**
             * Converts this PaymentIdentifier to JSON.
             * @function toJSON
             * @memberof greenlight.PaymentIdentifier
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            PaymentIdentifier.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return PaymentIdentifier;
        })();
    
        greenlight.ListPaymentsRequest = (function() {
    
            /**
             * Properties of a ListPaymentsRequest.
             * @memberof greenlight
             * @interface IListPaymentsRequest
             * @property {greenlight.IPaymentIdentifier|null} [identifier] ListPaymentsRequest identifier
             */
    
            /**
             * Constructs a new ListPaymentsRequest.
             * @memberof greenlight
             * @classdesc Represents a ListPaymentsRequest.
             * @implements IListPaymentsRequest
             * @constructor
             * @param {greenlight.IListPaymentsRequest=} [properties] Properties to set
             */
            function ListPaymentsRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListPaymentsRequest identifier.
             * @member {greenlight.IPaymentIdentifier|null|undefined} identifier
             * @memberof greenlight.ListPaymentsRequest
             * @instance
             */
            ListPaymentsRequest.prototype.identifier = null;
    
            /**
             * Creates a new ListPaymentsRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {greenlight.IListPaymentsRequest=} [properties] Properties to set
             * @returns {greenlight.ListPaymentsRequest} ListPaymentsRequest instance
             */
            ListPaymentsRequest.create = function create(properties) {
                return new ListPaymentsRequest(properties);
            };
    
            /**
             * Encodes the specified ListPaymentsRequest message. Does not implicitly {@link greenlight.ListPaymentsRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {greenlight.IListPaymentsRequest} message ListPaymentsRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPaymentsRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.identifier != null && Object.hasOwnProperty.call(message, "identifier"))
                    $root.greenlight.PaymentIdentifier.encode(message.identifier, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListPaymentsRequest message, length delimited. Does not implicitly {@link greenlight.ListPaymentsRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {greenlight.IListPaymentsRequest} message ListPaymentsRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPaymentsRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListPaymentsRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListPaymentsRequest} ListPaymentsRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPaymentsRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListPaymentsRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.identifier = $root.greenlight.PaymentIdentifier.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListPaymentsRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListPaymentsRequest} ListPaymentsRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPaymentsRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListPaymentsRequest message.
             * @function verify
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListPaymentsRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.identifier != null && message.hasOwnProperty("identifier")) {
                    var error = $root.greenlight.PaymentIdentifier.verify(message.identifier);
                    if (error)
                        return "identifier." + error;
                }
                return null;
            };
    
            /**
             * Creates a ListPaymentsRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListPaymentsRequest} ListPaymentsRequest
             */
            ListPaymentsRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListPaymentsRequest)
                    return object;
                var message = new $root.greenlight.ListPaymentsRequest();
                if (object.identifier != null) {
                    if (typeof object.identifier !== "object")
                        throw TypeError(".greenlight.ListPaymentsRequest.identifier: object expected");
                    message.identifier = $root.greenlight.PaymentIdentifier.fromObject(object.identifier);
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListPaymentsRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListPaymentsRequest
             * @static
             * @param {greenlight.ListPaymentsRequest} message ListPaymentsRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListPaymentsRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.identifier = null;
                if (message.identifier != null && message.hasOwnProperty("identifier"))
                    object.identifier = $root.greenlight.PaymentIdentifier.toObject(message.identifier, options);
                return object;
            };
    
            /**
             * Converts this ListPaymentsRequest to JSON.
             * @function toJSON
             * @memberof greenlight.ListPaymentsRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListPaymentsRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListPaymentsRequest;
        })();
    
        greenlight.ListPaymentsResponse = (function() {
    
            /**
             * Properties of a ListPaymentsResponse.
             * @memberof greenlight
             * @interface IListPaymentsResponse
             * @property {Array.<greenlight.IPayment>|null} [payments] ListPaymentsResponse payments
             */
    
            /**
             * Constructs a new ListPaymentsResponse.
             * @memberof greenlight
             * @classdesc Represents a ListPaymentsResponse.
             * @implements IListPaymentsResponse
             * @constructor
             * @param {greenlight.IListPaymentsResponse=} [properties] Properties to set
             */
            function ListPaymentsResponse(properties) {
                this.payments = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListPaymentsResponse payments.
             * @member {Array.<greenlight.IPayment>} payments
             * @memberof greenlight.ListPaymentsResponse
             * @instance
             */
            ListPaymentsResponse.prototype.payments = $util.emptyArray;
    
            /**
             * Creates a new ListPaymentsResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {greenlight.IListPaymentsResponse=} [properties] Properties to set
             * @returns {greenlight.ListPaymentsResponse} ListPaymentsResponse instance
             */
            ListPaymentsResponse.create = function create(properties) {
                return new ListPaymentsResponse(properties);
            };
    
            /**
             * Encodes the specified ListPaymentsResponse message. Does not implicitly {@link greenlight.ListPaymentsResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {greenlight.IListPaymentsResponse} message ListPaymentsResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPaymentsResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.payments != null && message.payments.length)
                    for (var i = 0; i < message.payments.length; ++i)
                        $root.greenlight.Payment.encode(message.payments[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListPaymentsResponse message, length delimited. Does not implicitly {@link greenlight.ListPaymentsResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {greenlight.IListPaymentsResponse} message ListPaymentsResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListPaymentsResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListPaymentsResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListPaymentsResponse} ListPaymentsResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPaymentsResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListPaymentsResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.payments && message.payments.length))
                            message.payments = [];
                        message.payments.push($root.greenlight.Payment.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListPaymentsResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListPaymentsResponse} ListPaymentsResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListPaymentsResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListPaymentsResponse message.
             * @function verify
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListPaymentsResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.payments != null && message.hasOwnProperty("payments")) {
                    if (!Array.isArray(message.payments))
                        return "payments: array expected";
                    for (var i = 0; i < message.payments.length; ++i) {
                        var error = $root.greenlight.Payment.verify(message.payments[i]);
                        if (error)
                            return "payments." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a ListPaymentsResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListPaymentsResponse} ListPaymentsResponse
             */
            ListPaymentsResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListPaymentsResponse)
                    return object;
                var message = new $root.greenlight.ListPaymentsResponse();
                if (object.payments) {
                    if (!Array.isArray(object.payments))
                        throw TypeError(".greenlight.ListPaymentsResponse.payments: array expected");
                    message.payments = [];
                    for (var i = 0; i < object.payments.length; ++i) {
                        if (typeof object.payments[i] !== "object")
                            throw TypeError(".greenlight.ListPaymentsResponse.payments: object expected");
                        message.payments[i] = $root.greenlight.Payment.fromObject(object.payments[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListPaymentsResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListPaymentsResponse
             * @static
             * @param {greenlight.ListPaymentsResponse} message ListPaymentsResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListPaymentsResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.payments = [];
                if (message.payments && message.payments.length) {
                    object.payments = [];
                    for (var j = 0; j < message.payments.length; ++j)
                        object.payments[j] = $root.greenlight.Payment.toObject(message.payments[j], options);
                }
                return object;
            };
    
            /**
             * Converts this ListPaymentsResponse to JSON.
             * @function toJSON
             * @memberof greenlight.ListPaymentsResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListPaymentsResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListPaymentsResponse;
        })();
    
        greenlight.InvoiceIdentifier = (function() {
    
            /**
             * Properties of an InvoiceIdentifier.
             * @memberof greenlight
             * @interface IInvoiceIdentifier
             * @property {string|null} [label] InvoiceIdentifier label
             * @property {string|null} [invstring] InvoiceIdentifier invstring
             * @property {Uint8Array|null} [paymentHash] InvoiceIdentifier paymentHash
             */
    
            /**
             * Constructs a new InvoiceIdentifier.
             * @memberof greenlight
             * @classdesc Represents an InvoiceIdentifier.
             * @implements IInvoiceIdentifier
             * @constructor
             * @param {greenlight.IInvoiceIdentifier=} [properties] Properties to set
             */
            function InvoiceIdentifier(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * InvoiceIdentifier label.
             * @member {string|null|undefined} label
             * @memberof greenlight.InvoiceIdentifier
             * @instance
             */
            InvoiceIdentifier.prototype.label = null;
    
            /**
             * InvoiceIdentifier invstring.
             * @member {string|null|undefined} invstring
             * @memberof greenlight.InvoiceIdentifier
             * @instance
             */
            InvoiceIdentifier.prototype.invstring = null;
    
            /**
             * InvoiceIdentifier paymentHash.
             * @member {Uint8Array|null|undefined} paymentHash
             * @memberof greenlight.InvoiceIdentifier
             * @instance
             */
            InvoiceIdentifier.prototype.paymentHash = null;
    
            // OneOf field names bound to virtual getters and setters
            var $oneOfFields;
    
            /**
             * InvoiceIdentifier id.
             * @member {"label"|"invstring"|"paymentHash"|undefined} id
             * @memberof greenlight.InvoiceIdentifier
             * @instance
             */
            Object.defineProperty(InvoiceIdentifier.prototype, "id", {
                get: $util.oneOfGetter($oneOfFields = ["label", "invstring", "paymentHash"]),
                set: $util.oneOfSetter($oneOfFields)
            });
    
            /**
             * Creates a new InvoiceIdentifier instance using the specified properties.
             * @function create
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {greenlight.IInvoiceIdentifier=} [properties] Properties to set
             * @returns {greenlight.InvoiceIdentifier} InvoiceIdentifier instance
             */
            InvoiceIdentifier.create = function create(properties) {
                return new InvoiceIdentifier(properties);
            };
    
            /**
             * Encodes the specified InvoiceIdentifier message. Does not implicitly {@link greenlight.InvoiceIdentifier.verify|verify} messages.
             * @function encode
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {greenlight.IInvoiceIdentifier} message InvoiceIdentifier message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            InvoiceIdentifier.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.label);
                if (message.invstring != null && Object.hasOwnProperty.call(message, "invstring"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.invstring);
                if (message.paymentHash != null && Object.hasOwnProperty.call(message, "paymentHash"))
                    writer.uint32(/* id 3, wireType 2 =*/26).bytes(message.paymentHash);
                return writer;
            };
    
            /**
             * Encodes the specified InvoiceIdentifier message, length delimited. Does not implicitly {@link greenlight.InvoiceIdentifier.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {greenlight.IInvoiceIdentifier} message InvoiceIdentifier message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            InvoiceIdentifier.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an InvoiceIdentifier message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.InvoiceIdentifier} InvoiceIdentifier
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            InvoiceIdentifier.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.InvoiceIdentifier();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.label = reader.string();
                        break;
                    case 2:
                        message.invstring = reader.string();
                        break;
                    case 3:
                        message.paymentHash = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an InvoiceIdentifier message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.InvoiceIdentifier} InvoiceIdentifier
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            InvoiceIdentifier.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an InvoiceIdentifier message.
             * @function verify
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            InvoiceIdentifier.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                var properties = {};
                if (message.label != null && message.hasOwnProperty("label")) {
                    properties.id = 1;
                    if (!$util.isString(message.label))
                        return "label: string expected";
                }
                if (message.invstring != null && message.hasOwnProperty("invstring")) {
                    if (properties.id === 1)
                        return "id: multiple values";
                    properties.id = 1;
                    if (!$util.isString(message.invstring))
                        return "invstring: string expected";
                }
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash")) {
                    if (properties.id === 1)
                        return "id: multiple values";
                    properties.id = 1;
                    if (!(message.paymentHash && typeof message.paymentHash.length === "number" || $util.isString(message.paymentHash)))
                        return "paymentHash: buffer expected";
                }
                return null;
            };
    
            /**
             * Creates an InvoiceIdentifier message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.InvoiceIdentifier} InvoiceIdentifier
             */
            InvoiceIdentifier.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.InvoiceIdentifier)
                    return object;
                var message = new $root.greenlight.InvoiceIdentifier();
                if (object.label != null)
                    message.label = String(object.label);
                if (object.invstring != null)
                    message.invstring = String(object.invstring);
                if (object.paymentHash != null)
                    if (typeof object.paymentHash === "string")
                        $util.base64.decode(object.paymentHash, message.paymentHash = $util.newBuffer($util.base64.length(object.paymentHash)), 0);
                    else if (object.paymentHash.length)
                        message.paymentHash = object.paymentHash;
                return message;
            };
    
            /**
             * Creates a plain object from an InvoiceIdentifier message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.InvoiceIdentifier
             * @static
             * @param {greenlight.InvoiceIdentifier} message InvoiceIdentifier
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            InvoiceIdentifier.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (message.label != null && message.hasOwnProperty("label")) {
                    object.label = message.label;
                    if (options.oneofs)
                        object.id = "label";
                }
                if (message.invstring != null && message.hasOwnProperty("invstring")) {
                    object.invstring = message.invstring;
                    if (options.oneofs)
                        object.id = "invstring";
                }
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash")) {
                    object.paymentHash = options.bytes === String ? $util.base64.encode(message.paymentHash, 0, message.paymentHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentHash) : message.paymentHash;
                    if (options.oneofs)
                        object.id = "paymentHash";
                }
                return object;
            };
    
            /**
             * Converts this InvoiceIdentifier to JSON.
             * @function toJSON
             * @memberof greenlight.InvoiceIdentifier
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            InvoiceIdentifier.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return InvoiceIdentifier;
        })();
    
        greenlight.ListInvoicesRequest = (function() {
    
            /**
             * Properties of a ListInvoicesRequest.
             * @memberof greenlight
             * @interface IListInvoicesRequest
             * @property {greenlight.IInvoiceIdentifier|null} [identifier] ListInvoicesRequest identifier
             */
    
            /**
             * Constructs a new ListInvoicesRequest.
             * @memberof greenlight
             * @classdesc Represents a ListInvoicesRequest.
             * @implements IListInvoicesRequest
             * @constructor
             * @param {greenlight.IListInvoicesRequest=} [properties] Properties to set
             */
            function ListInvoicesRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListInvoicesRequest identifier.
             * @member {greenlight.IInvoiceIdentifier|null|undefined} identifier
             * @memberof greenlight.ListInvoicesRequest
             * @instance
             */
            ListInvoicesRequest.prototype.identifier = null;
    
            /**
             * Creates a new ListInvoicesRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {greenlight.IListInvoicesRequest=} [properties] Properties to set
             * @returns {greenlight.ListInvoicesRequest} ListInvoicesRequest instance
             */
            ListInvoicesRequest.create = function create(properties) {
                return new ListInvoicesRequest(properties);
            };
    
            /**
             * Encodes the specified ListInvoicesRequest message. Does not implicitly {@link greenlight.ListInvoicesRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {greenlight.IListInvoicesRequest} message ListInvoicesRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListInvoicesRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.identifier != null && Object.hasOwnProperty.call(message, "identifier"))
                    $root.greenlight.InvoiceIdentifier.encode(message.identifier, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListInvoicesRequest message, length delimited. Does not implicitly {@link greenlight.ListInvoicesRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {greenlight.IListInvoicesRequest} message ListInvoicesRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListInvoicesRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListInvoicesRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListInvoicesRequest} ListInvoicesRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListInvoicesRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListInvoicesRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.identifier = $root.greenlight.InvoiceIdentifier.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListInvoicesRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListInvoicesRequest} ListInvoicesRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListInvoicesRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListInvoicesRequest message.
             * @function verify
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListInvoicesRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.identifier != null && message.hasOwnProperty("identifier")) {
                    var error = $root.greenlight.InvoiceIdentifier.verify(message.identifier);
                    if (error)
                        return "identifier." + error;
                }
                return null;
            };
    
            /**
             * Creates a ListInvoicesRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListInvoicesRequest} ListInvoicesRequest
             */
            ListInvoicesRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListInvoicesRequest)
                    return object;
                var message = new $root.greenlight.ListInvoicesRequest();
                if (object.identifier != null) {
                    if (typeof object.identifier !== "object")
                        throw TypeError(".greenlight.ListInvoicesRequest.identifier: object expected");
                    message.identifier = $root.greenlight.InvoiceIdentifier.fromObject(object.identifier);
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListInvoicesRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListInvoicesRequest
             * @static
             * @param {greenlight.ListInvoicesRequest} message ListInvoicesRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListInvoicesRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.identifier = null;
                if (message.identifier != null && message.hasOwnProperty("identifier"))
                    object.identifier = $root.greenlight.InvoiceIdentifier.toObject(message.identifier, options);
                return object;
            };
    
            /**
             * Converts this ListInvoicesRequest to JSON.
             * @function toJSON
             * @memberof greenlight.ListInvoicesRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListInvoicesRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListInvoicesRequest;
        })();
    
        greenlight.StreamIncomingFilter = (function() {
    
            /**
             * Properties of a StreamIncomingFilter.
             * @memberof greenlight
             * @interface IStreamIncomingFilter
             */
    
            /**
             * Constructs a new StreamIncomingFilter.
             * @memberof greenlight
             * @classdesc Represents a StreamIncomingFilter.
             * @implements IStreamIncomingFilter
             * @constructor
             * @param {greenlight.IStreamIncomingFilter=} [properties] Properties to set
             */
            function StreamIncomingFilter(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new StreamIncomingFilter instance using the specified properties.
             * @function create
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {greenlight.IStreamIncomingFilter=} [properties] Properties to set
             * @returns {greenlight.StreamIncomingFilter} StreamIncomingFilter instance
             */
            StreamIncomingFilter.create = function create(properties) {
                return new StreamIncomingFilter(properties);
            };
    
            /**
             * Encodes the specified StreamIncomingFilter message. Does not implicitly {@link greenlight.StreamIncomingFilter.verify|verify} messages.
             * @function encode
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {greenlight.IStreamIncomingFilter} message StreamIncomingFilter message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamIncomingFilter.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified StreamIncomingFilter message, length delimited. Does not implicitly {@link greenlight.StreamIncomingFilter.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {greenlight.IStreamIncomingFilter} message StreamIncomingFilter message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamIncomingFilter.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a StreamIncomingFilter message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.StreamIncomingFilter} StreamIncomingFilter
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamIncomingFilter.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.StreamIncomingFilter();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a StreamIncomingFilter message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.StreamIncomingFilter} StreamIncomingFilter
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamIncomingFilter.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a StreamIncomingFilter message.
             * @function verify
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StreamIncomingFilter.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates a StreamIncomingFilter message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.StreamIncomingFilter} StreamIncomingFilter
             */
            StreamIncomingFilter.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.StreamIncomingFilter)
                    return object;
                return new $root.greenlight.StreamIncomingFilter();
            };
    
            /**
             * Creates a plain object from a StreamIncomingFilter message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.StreamIncomingFilter
             * @static
             * @param {greenlight.StreamIncomingFilter} message StreamIncomingFilter
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StreamIncomingFilter.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this StreamIncomingFilter to JSON.
             * @function toJSON
             * @memberof greenlight.StreamIncomingFilter
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StreamIncomingFilter.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return StreamIncomingFilter;
        })();
    
        greenlight.ListInvoicesResponse = (function() {
    
            /**
             * Properties of a ListInvoicesResponse.
             * @memberof greenlight
             * @interface IListInvoicesResponse
             * @property {Array.<greenlight.IInvoice>|null} [invoices] ListInvoicesResponse invoices
             */
    
            /**
             * Constructs a new ListInvoicesResponse.
             * @memberof greenlight
             * @classdesc Represents a ListInvoicesResponse.
             * @implements IListInvoicesResponse
             * @constructor
             * @param {greenlight.IListInvoicesResponse=} [properties] Properties to set
             */
            function ListInvoicesResponse(properties) {
                this.invoices = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * ListInvoicesResponse invoices.
             * @member {Array.<greenlight.IInvoice>} invoices
             * @memberof greenlight.ListInvoicesResponse
             * @instance
             */
            ListInvoicesResponse.prototype.invoices = $util.emptyArray;
    
            /**
             * Creates a new ListInvoicesResponse instance using the specified properties.
             * @function create
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {greenlight.IListInvoicesResponse=} [properties] Properties to set
             * @returns {greenlight.ListInvoicesResponse} ListInvoicesResponse instance
             */
            ListInvoicesResponse.create = function create(properties) {
                return new ListInvoicesResponse(properties);
            };
    
            /**
             * Encodes the specified ListInvoicesResponse message. Does not implicitly {@link greenlight.ListInvoicesResponse.verify|verify} messages.
             * @function encode
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {greenlight.IListInvoicesResponse} message ListInvoicesResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListInvoicesResponse.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.invoices != null && message.invoices.length)
                    for (var i = 0; i < message.invoices.length; ++i)
                        $root.greenlight.Invoice.encode(message.invoices[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified ListInvoicesResponse message, length delimited. Does not implicitly {@link greenlight.ListInvoicesResponse.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {greenlight.IListInvoicesResponse} message ListInvoicesResponse message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            ListInvoicesResponse.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a ListInvoicesResponse message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.ListInvoicesResponse} ListInvoicesResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListInvoicesResponse.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.ListInvoicesResponse();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.invoices && message.invoices.length))
                            message.invoices = [];
                        message.invoices.push($root.greenlight.Invoice.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a ListInvoicesResponse message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.ListInvoicesResponse} ListInvoicesResponse
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            ListInvoicesResponse.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a ListInvoicesResponse message.
             * @function verify
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            ListInvoicesResponse.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.invoices != null && message.hasOwnProperty("invoices")) {
                    if (!Array.isArray(message.invoices))
                        return "invoices: array expected";
                    for (var i = 0; i < message.invoices.length; ++i) {
                        var error = $root.greenlight.Invoice.verify(message.invoices[i]);
                        if (error)
                            return "invoices." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a ListInvoicesResponse message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.ListInvoicesResponse} ListInvoicesResponse
             */
            ListInvoicesResponse.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.ListInvoicesResponse)
                    return object;
                var message = new $root.greenlight.ListInvoicesResponse();
                if (object.invoices) {
                    if (!Array.isArray(object.invoices))
                        throw TypeError(".greenlight.ListInvoicesResponse.invoices: array expected");
                    message.invoices = [];
                    for (var i = 0; i < object.invoices.length; ++i) {
                        if (typeof object.invoices[i] !== "object")
                            throw TypeError(".greenlight.ListInvoicesResponse.invoices: object expected");
                        message.invoices[i] = $root.greenlight.Invoice.fromObject(object.invoices[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a ListInvoicesResponse message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.ListInvoicesResponse
             * @static
             * @param {greenlight.ListInvoicesResponse} message ListInvoicesResponse
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            ListInvoicesResponse.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.invoices = [];
                if (message.invoices && message.invoices.length) {
                    object.invoices = [];
                    for (var j = 0; j < message.invoices.length; ++j)
                        object.invoices[j] = $root.greenlight.Invoice.toObject(message.invoices[j], options);
                }
                return object;
            };
    
            /**
             * Converts this ListInvoicesResponse to JSON.
             * @function toJSON
             * @memberof greenlight.ListInvoicesResponse
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            ListInvoicesResponse.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return ListInvoicesResponse;
        })();
    
        greenlight.TlvField = (function() {
    
            /**
             * Properties of a TlvField.
             * @memberof greenlight
             * @interface ITlvField
             * @property {number|Long|null} [type] TlvField type
             * @property {Uint8Array|null} [value] TlvField value
             */
    
            /**
             * Constructs a new TlvField.
             * @memberof greenlight
             * @classdesc Represents a TlvField.
             * @implements ITlvField
             * @constructor
             * @param {greenlight.ITlvField=} [properties] Properties to set
             */
            function TlvField(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * TlvField type.
             * @member {number|Long} type
             * @memberof greenlight.TlvField
             * @instance
             */
            TlvField.prototype.type = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * TlvField value.
             * @member {Uint8Array} value
             * @memberof greenlight.TlvField
             * @instance
             */
            TlvField.prototype.value = $util.newBuffer([]);
    
            /**
             * Creates a new TlvField instance using the specified properties.
             * @function create
             * @memberof greenlight.TlvField
             * @static
             * @param {greenlight.ITlvField=} [properties] Properties to set
             * @returns {greenlight.TlvField} TlvField instance
             */
            TlvField.create = function create(properties) {
                return new TlvField(properties);
            };
    
            /**
             * Encodes the specified TlvField message. Does not implicitly {@link greenlight.TlvField.verify|verify} messages.
             * @function encode
             * @memberof greenlight.TlvField
             * @static
             * @param {greenlight.ITlvField} message TlvField message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TlvField.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.type != null && Object.hasOwnProperty.call(message, "type"))
                    writer.uint32(/* id 1, wireType 0 =*/8).uint64(message.type);
                if (message.value != null && Object.hasOwnProperty.call(message, "value"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.value);
                return writer;
            };
    
            /**
             * Encodes the specified TlvField message, length delimited. Does not implicitly {@link greenlight.TlvField.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.TlvField
             * @static
             * @param {greenlight.ITlvField} message TlvField message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            TlvField.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a TlvField message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.TlvField
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.TlvField} TlvField
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TlvField.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.TlvField();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.type = reader.uint64();
                        break;
                    case 2:
                        message.value = reader.bytes();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a TlvField message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.TlvField
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.TlvField} TlvField
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            TlvField.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a TlvField message.
             * @function verify
             * @memberof greenlight.TlvField
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            TlvField.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.type != null && message.hasOwnProperty("type"))
                    if (!$util.isInteger(message.type) && !(message.type && $util.isInteger(message.type.low) && $util.isInteger(message.type.high)))
                        return "type: integer|Long expected";
                if (message.value != null && message.hasOwnProperty("value"))
                    if (!(message.value && typeof message.value.length === "number" || $util.isString(message.value)))
                        return "value: buffer expected";
                return null;
            };
    
            /**
             * Creates a TlvField message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.TlvField
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.TlvField} TlvField
             */
            TlvField.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.TlvField)
                    return object;
                var message = new $root.greenlight.TlvField();
                if (object.type != null)
                    if ($util.Long)
                        (message.type = $util.Long.fromValue(object.type)).unsigned = true;
                    else if (typeof object.type === "string")
                        message.type = parseInt(object.type, 10);
                    else if (typeof object.type === "number")
                        message.type = object.type;
                    else if (typeof object.type === "object")
                        message.type = new $util.LongBits(object.type.low >>> 0, object.type.high >>> 0).toNumber(true);
                if (object.value != null)
                    if (typeof object.value === "string")
                        $util.base64.decode(object.value, message.value = $util.newBuffer($util.base64.length(object.value)), 0);
                    else if (object.value.length)
                        message.value = object.value;
                return message;
            };
    
            /**
             * Creates a plain object from a TlvField message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.TlvField
             * @static
             * @param {greenlight.TlvField} message TlvField
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            TlvField.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.type = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.type = options.longs === String ? "0" : 0;
                    if (options.bytes === String)
                        object.value = "";
                    else {
                        object.value = [];
                        if (options.bytes !== Array)
                            object.value = $util.newBuffer(object.value);
                    }
                }
                if (message.type != null && message.hasOwnProperty("type"))
                    if (typeof message.type === "number")
                        object.type = options.longs === String ? String(message.type) : message.type;
                    else
                        object.type = options.longs === String ? $util.Long.prototype.toString.call(message.type) : options.longs === Number ? new $util.LongBits(message.type.low >>> 0, message.type.high >>> 0).toNumber(true) : message.type;
                if (message.value != null && message.hasOwnProperty("value"))
                    object.value = options.bytes === String ? $util.base64.encode(message.value, 0, message.value.length) : options.bytes === Array ? Array.prototype.slice.call(message.value) : message.value;
                return object;
            };
    
            /**
             * Converts this TlvField to JSON.
             * @function toJSON
             * @memberof greenlight.TlvField
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            TlvField.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return TlvField;
        })();
    
        greenlight.OffChainPayment = (function() {
    
            /**
             * Properties of an OffChainPayment.
             * @memberof greenlight
             * @interface IOffChainPayment
             * @property {string|null} [label] OffChainPayment label
             * @property {Uint8Array|null} [preimage] OffChainPayment preimage
             * @property {greenlight.IAmount|null} [amount] OffChainPayment amount
             * @property {Array.<greenlight.ITlvField>|null} [extratlvs] OffChainPayment extratlvs
             * @property {Uint8Array|null} [paymentHash] OffChainPayment paymentHash
             * @property {string|null} [bolt11] OffChainPayment bolt11
             */
    
            /**
             * Constructs a new OffChainPayment.
             * @memberof greenlight
             * @classdesc Represents an OffChainPayment.
             * @implements IOffChainPayment
             * @constructor
             * @param {greenlight.IOffChainPayment=} [properties] Properties to set
             */
            function OffChainPayment(properties) {
                this.extratlvs = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * OffChainPayment label.
             * @member {string} label
             * @memberof greenlight.OffChainPayment
             * @instance
             */
            OffChainPayment.prototype.label = "";
    
            /**
             * OffChainPayment preimage.
             * @member {Uint8Array} preimage
             * @memberof greenlight.OffChainPayment
             * @instance
             */
            OffChainPayment.prototype.preimage = $util.newBuffer([]);
    
            /**
             * OffChainPayment amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.OffChainPayment
             * @instance
             */
            OffChainPayment.prototype.amount = null;
    
            /**
             * OffChainPayment extratlvs.
             * @member {Array.<greenlight.ITlvField>} extratlvs
             * @memberof greenlight.OffChainPayment
             * @instance
             */
            OffChainPayment.prototype.extratlvs = $util.emptyArray;
    
            /**
             * OffChainPayment paymentHash.
             * @member {Uint8Array} paymentHash
             * @memberof greenlight.OffChainPayment
             * @instance
             */
            OffChainPayment.prototype.paymentHash = $util.newBuffer([]);
    
            /**
             * OffChainPayment bolt11.
             * @member {string} bolt11
             * @memberof greenlight.OffChainPayment
             * @instance
             */
            OffChainPayment.prototype.bolt11 = "";
    
            /**
             * Creates a new OffChainPayment instance using the specified properties.
             * @function create
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {greenlight.IOffChainPayment=} [properties] Properties to set
             * @returns {greenlight.OffChainPayment} OffChainPayment instance
             */
            OffChainPayment.create = function create(properties) {
                return new OffChainPayment(properties);
            };
    
            /**
             * Encodes the specified OffChainPayment message. Does not implicitly {@link greenlight.OffChainPayment.verify|verify} messages.
             * @function encode
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {greenlight.IOffChainPayment} message OffChainPayment message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OffChainPayment.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.label);
                if (message.preimage != null && Object.hasOwnProperty.call(message, "preimage"))
                    writer.uint32(/* id 2, wireType 2 =*/18).bytes(message.preimage);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 3, wireType 2 =*/26).fork()).ldelim();
                if (message.extratlvs != null && message.extratlvs.length)
                    for (var i = 0; i < message.extratlvs.length; ++i)
                        $root.greenlight.TlvField.encode(message.extratlvs[i], writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.paymentHash != null && Object.hasOwnProperty.call(message, "paymentHash"))
                    writer.uint32(/* id 5, wireType 2 =*/42).bytes(message.paymentHash);
                if (message.bolt11 != null && Object.hasOwnProperty.call(message, "bolt11"))
                    writer.uint32(/* id 6, wireType 2 =*/50).string(message.bolt11);
                return writer;
            };
    
            /**
             * Encodes the specified OffChainPayment message, length delimited. Does not implicitly {@link greenlight.OffChainPayment.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {greenlight.IOffChainPayment} message OffChainPayment message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            OffChainPayment.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an OffChainPayment message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.OffChainPayment} OffChainPayment
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OffChainPayment.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.OffChainPayment();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.label = reader.string();
                        break;
                    case 2:
                        message.preimage = reader.bytes();
                        break;
                    case 3:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 4:
                        if (!(message.extratlvs && message.extratlvs.length))
                            message.extratlvs = [];
                        message.extratlvs.push($root.greenlight.TlvField.decode(reader, reader.uint32()));
                        break;
                    case 5:
                        message.paymentHash = reader.bytes();
                        break;
                    case 6:
                        message.bolt11 = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an OffChainPayment message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.OffChainPayment} OffChainPayment
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            OffChainPayment.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an OffChainPayment message.
             * @function verify
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            OffChainPayment.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.label != null && message.hasOwnProperty("label"))
                    if (!$util.isString(message.label))
                        return "label: string expected";
                if (message.preimage != null && message.hasOwnProperty("preimage"))
                    if (!(message.preimage && typeof message.preimage.length === "number" || $util.isString(message.preimage)))
                        return "preimage: buffer expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.extratlvs != null && message.hasOwnProperty("extratlvs")) {
                    if (!Array.isArray(message.extratlvs))
                        return "extratlvs: array expected";
                    for (var i = 0; i < message.extratlvs.length; ++i) {
                        var error = $root.greenlight.TlvField.verify(message.extratlvs[i]);
                        if (error)
                            return "extratlvs." + error;
                    }
                }
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    if (!(message.paymentHash && typeof message.paymentHash.length === "number" || $util.isString(message.paymentHash)))
                        return "paymentHash: buffer expected";
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    if (!$util.isString(message.bolt11))
                        return "bolt11: string expected";
                return null;
            };
    
            /**
             * Creates an OffChainPayment message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.OffChainPayment} OffChainPayment
             */
            OffChainPayment.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.OffChainPayment)
                    return object;
                var message = new $root.greenlight.OffChainPayment();
                if (object.label != null)
                    message.label = String(object.label);
                if (object.preimage != null)
                    if (typeof object.preimage === "string")
                        $util.base64.decode(object.preimage, message.preimage = $util.newBuffer($util.base64.length(object.preimage)), 0);
                    else if (object.preimage.length)
                        message.preimage = object.preimage;
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.OffChainPayment.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.extratlvs) {
                    if (!Array.isArray(object.extratlvs))
                        throw TypeError(".greenlight.OffChainPayment.extratlvs: array expected");
                    message.extratlvs = [];
                    for (var i = 0; i < object.extratlvs.length; ++i) {
                        if (typeof object.extratlvs[i] !== "object")
                            throw TypeError(".greenlight.OffChainPayment.extratlvs: object expected");
                        message.extratlvs[i] = $root.greenlight.TlvField.fromObject(object.extratlvs[i]);
                    }
                }
                if (object.paymentHash != null)
                    if (typeof object.paymentHash === "string")
                        $util.base64.decode(object.paymentHash, message.paymentHash = $util.newBuffer($util.base64.length(object.paymentHash)), 0);
                    else if (object.paymentHash.length)
                        message.paymentHash = object.paymentHash;
                if (object.bolt11 != null)
                    message.bolt11 = String(object.bolt11);
                return message;
            };
    
            /**
             * Creates a plain object from an OffChainPayment message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.OffChainPayment
             * @static
             * @param {greenlight.OffChainPayment} message OffChainPayment
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            OffChainPayment.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.extratlvs = [];
                if (options.defaults) {
                    object.label = "";
                    if (options.bytes === String)
                        object.preimage = "";
                    else {
                        object.preimage = [];
                        if (options.bytes !== Array)
                            object.preimage = $util.newBuffer(object.preimage);
                    }
                    object.amount = null;
                    if (options.bytes === String)
                        object.paymentHash = "";
                    else {
                        object.paymentHash = [];
                        if (options.bytes !== Array)
                            object.paymentHash = $util.newBuffer(object.paymentHash);
                    }
                    object.bolt11 = "";
                }
                if (message.label != null && message.hasOwnProperty("label"))
                    object.label = message.label;
                if (message.preimage != null && message.hasOwnProperty("preimage"))
                    object.preimage = options.bytes === String ? $util.base64.encode(message.preimage, 0, message.preimage.length) : options.bytes === Array ? Array.prototype.slice.call(message.preimage) : message.preimage;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.extratlvs && message.extratlvs.length) {
                    object.extratlvs = [];
                    for (var j = 0; j < message.extratlvs.length; ++j)
                        object.extratlvs[j] = $root.greenlight.TlvField.toObject(message.extratlvs[j], options);
                }
                if (message.paymentHash != null && message.hasOwnProperty("paymentHash"))
                    object.paymentHash = options.bytes === String ? $util.base64.encode(message.paymentHash, 0, message.paymentHash.length) : options.bytes === Array ? Array.prototype.slice.call(message.paymentHash) : message.paymentHash;
                if (message.bolt11 != null && message.hasOwnProperty("bolt11"))
                    object.bolt11 = message.bolt11;
                return object;
            };
    
            /**
             * Converts this OffChainPayment to JSON.
             * @function toJSON
             * @memberof greenlight.OffChainPayment
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            OffChainPayment.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return OffChainPayment;
        })();
    
        greenlight.IncomingPayment = (function() {
    
            /**
             * Properties of an IncomingPayment.
             * @memberof greenlight
             * @interface IIncomingPayment
             * @property {greenlight.IOffChainPayment|null} [offchain] IncomingPayment offchain
             */
    
            /**
             * Constructs a new IncomingPayment.
             * @memberof greenlight
             * @classdesc Represents an IncomingPayment.
             * @implements IIncomingPayment
             * @constructor
             * @param {greenlight.IIncomingPayment=} [properties] Properties to set
             */
            function IncomingPayment(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * IncomingPayment offchain.
             * @member {greenlight.IOffChainPayment|null|undefined} offchain
             * @memberof greenlight.IncomingPayment
             * @instance
             */
            IncomingPayment.prototype.offchain = null;
    
            // OneOf field names bound to virtual getters and setters
            var $oneOfFields;
    
            /**
             * IncomingPayment details.
             * @member {"offchain"|undefined} details
             * @memberof greenlight.IncomingPayment
             * @instance
             */
            Object.defineProperty(IncomingPayment.prototype, "details", {
                get: $util.oneOfGetter($oneOfFields = ["offchain"]),
                set: $util.oneOfSetter($oneOfFields)
            });
    
            /**
             * Creates a new IncomingPayment instance using the specified properties.
             * @function create
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {greenlight.IIncomingPayment=} [properties] Properties to set
             * @returns {greenlight.IncomingPayment} IncomingPayment instance
             */
            IncomingPayment.create = function create(properties) {
                return new IncomingPayment(properties);
            };
    
            /**
             * Encodes the specified IncomingPayment message. Does not implicitly {@link greenlight.IncomingPayment.verify|verify} messages.
             * @function encode
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {greenlight.IIncomingPayment} message IncomingPayment message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            IncomingPayment.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.offchain != null && Object.hasOwnProperty.call(message, "offchain"))
                    $root.greenlight.OffChainPayment.encode(message.offchain, writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified IncomingPayment message, length delimited. Does not implicitly {@link greenlight.IncomingPayment.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {greenlight.IIncomingPayment} message IncomingPayment message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            IncomingPayment.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes an IncomingPayment message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.IncomingPayment} IncomingPayment
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            IncomingPayment.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.IncomingPayment();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.offchain = $root.greenlight.OffChainPayment.decode(reader, reader.uint32());
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes an IncomingPayment message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.IncomingPayment} IncomingPayment
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            IncomingPayment.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies an IncomingPayment message.
             * @function verify
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            IncomingPayment.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                var properties = {};
                if (message.offchain != null && message.hasOwnProperty("offchain")) {
                    properties.details = 1;
                    {
                        var error = $root.greenlight.OffChainPayment.verify(message.offchain);
                        if (error)
                            return "offchain." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates an IncomingPayment message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.IncomingPayment} IncomingPayment
             */
            IncomingPayment.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.IncomingPayment)
                    return object;
                var message = new $root.greenlight.IncomingPayment();
                if (object.offchain != null) {
                    if (typeof object.offchain !== "object")
                        throw TypeError(".greenlight.IncomingPayment.offchain: object expected");
                    message.offchain = $root.greenlight.OffChainPayment.fromObject(object.offchain);
                }
                return message;
            };
    
            /**
             * Creates a plain object from an IncomingPayment message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.IncomingPayment
             * @static
             * @param {greenlight.IncomingPayment} message IncomingPayment
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            IncomingPayment.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (message.offchain != null && message.hasOwnProperty("offchain")) {
                    object.offchain = $root.greenlight.OffChainPayment.toObject(message.offchain, options);
                    if (options.oneofs)
                        object.details = "offchain";
                }
                return object;
            };
    
            /**
             * Converts this IncomingPayment to JSON.
             * @function toJSON
             * @memberof greenlight.IncomingPayment
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            IncomingPayment.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return IncomingPayment;
        })();
    
        greenlight.RoutehintHop = (function() {
    
            /**
             * Properties of a RoutehintHop.
             * @memberof greenlight
             * @interface IRoutehintHop
             * @property {Uint8Array|null} [nodeId] RoutehintHop nodeId
             * @property {string|null} [shortChannelId] RoutehintHop shortChannelId
             * @property {number|Long|null} [feeBase] RoutehintHop feeBase
             * @property {number|null} [feeProp] RoutehintHop feeProp
             * @property {number|null} [cltvExpiryDelta] RoutehintHop cltvExpiryDelta
             */
    
            /**
             * Constructs a new RoutehintHop.
             * @memberof greenlight
             * @classdesc Represents a RoutehintHop.
             * @implements IRoutehintHop
             * @constructor
             * @param {greenlight.IRoutehintHop=} [properties] Properties to set
             */
            function RoutehintHop(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * RoutehintHop nodeId.
             * @member {Uint8Array} nodeId
             * @memberof greenlight.RoutehintHop
             * @instance
             */
            RoutehintHop.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * RoutehintHop shortChannelId.
             * @member {string} shortChannelId
             * @memberof greenlight.RoutehintHop
             * @instance
             */
            RoutehintHop.prototype.shortChannelId = "";
    
            /**
             * RoutehintHop feeBase.
             * @member {number|Long} feeBase
             * @memberof greenlight.RoutehintHop
             * @instance
             */
            RoutehintHop.prototype.feeBase = $util.Long ? $util.Long.fromBits(0,0,true) : 0;
    
            /**
             * RoutehintHop feeProp.
             * @member {number} feeProp
             * @memberof greenlight.RoutehintHop
             * @instance
             */
            RoutehintHop.prototype.feeProp = 0;
    
            /**
             * RoutehintHop cltvExpiryDelta.
             * @member {number} cltvExpiryDelta
             * @memberof greenlight.RoutehintHop
             * @instance
             */
            RoutehintHop.prototype.cltvExpiryDelta = 0;
    
            /**
             * Creates a new RoutehintHop instance using the specified properties.
             * @function create
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {greenlight.IRoutehintHop=} [properties] Properties to set
             * @returns {greenlight.RoutehintHop} RoutehintHop instance
             */
            RoutehintHop.create = function create(properties) {
                return new RoutehintHop(properties);
            };
    
            /**
             * Encodes the specified RoutehintHop message. Does not implicitly {@link greenlight.RoutehintHop.verify|verify} messages.
             * @function encode
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {greenlight.IRoutehintHop} message RoutehintHop message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RoutehintHop.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.shortChannelId != null && Object.hasOwnProperty.call(message, "shortChannelId"))
                    writer.uint32(/* id 2, wireType 2 =*/18).string(message.shortChannelId);
                if (message.feeBase != null && Object.hasOwnProperty.call(message, "feeBase"))
                    writer.uint32(/* id 3, wireType 0 =*/24).uint64(message.feeBase);
                if (message.feeProp != null && Object.hasOwnProperty.call(message, "feeProp"))
                    writer.uint32(/* id 4, wireType 0 =*/32).uint32(message.feeProp);
                if (message.cltvExpiryDelta != null && Object.hasOwnProperty.call(message, "cltvExpiryDelta"))
                    writer.uint32(/* id 5, wireType 0 =*/40).uint32(message.cltvExpiryDelta);
                return writer;
            };
    
            /**
             * Encodes the specified RoutehintHop message, length delimited. Does not implicitly {@link greenlight.RoutehintHop.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {greenlight.IRoutehintHop} message RoutehintHop message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            RoutehintHop.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a RoutehintHop message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.RoutehintHop} RoutehintHop
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RoutehintHop.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.RoutehintHop();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.shortChannelId = reader.string();
                        break;
                    case 3:
                        message.feeBase = reader.uint64();
                        break;
                    case 4:
                        message.feeProp = reader.uint32();
                        break;
                    case 5:
                        message.cltvExpiryDelta = reader.uint32();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a RoutehintHop message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.RoutehintHop} RoutehintHop
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            RoutehintHop.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a RoutehintHop message.
             * @function verify
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            RoutehintHop.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.shortChannelId != null && message.hasOwnProperty("shortChannelId"))
                    if (!$util.isString(message.shortChannelId))
                        return "shortChannelId: string expected";
                if (message.feeBase != null && message.hasOwnProperty("feeBase"))
                    if (!$util.isInteger(message.feeBase) && !(message.feeBase && $util.isInteger(message.feeBase.low) && $util.isInteger(message.feeBase.high)))
                        return "feeBase: integer|Long expected";
                if (message.feeProp != null && message.hasOwnProperty("feeProp"))
                    if (!$util.isInteger(message.feeProp))
                        return "feeProp: integer expected";
                if (message.cltvExpiryDelta != null && message.hasOwnProperty("cltvExpiryDelta"))
                    if (!$util.isInteger(message.cltvExpiryDelta))
                        return "cltvExpiryDelta: integer expected";
                return null;
            };
    
            /**
             * Creates a RoutehintHop message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.RoutehintHop} RoutehintHop
             */
            RoutehintHop.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.RoutehintHop)
                    return object;
                var message = new $root.greenlight.RoutehintHop();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.shortChannelId != null)
                    message.shortChannelId = String(object.shortChannelId);
                if (object.feeBase != null)
                    if ($util.Long)
                        (message.feeBase = $util.Long.fromValue(object.feeBase)).unsigned = true;
                    else if (typeof object.feeBase === "string")
                        message.feeBase = parseInt(object.feeBase, 10);
                    else if (typeof object.feeBase === "number")
                        message.feeBase = object.feeBase;
                    else if (typeof object.feeBase === "object")
                        message.feeBase = new $util.LongBits(object.feeBase.low >>> 0, object.feeBase.high >>> 0).toNumber(true);
                if (object.feeProp != null)
                    message.feeProp = object.feeProp >>> 0;
                if (object.cltvExpiryDelta != null)
                    message.cltvExpiryDelta = object.cltvExpiryDelta >>> 0;
                return message;
            };
    
            /**
             * Creates a plain object from a RoutehintHop message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.RoutehintHop
             * @static
             * @param {greenlight.RoutehintHop} message RoutehintHop
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            RoutehintHop.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.shortChannelId = "";
                    if ($util.Long) {
                        var long = new $util.Long(0, 0, true);
                        object.feeBase = options.longs === String ? long.toString() : options.longs === Number ? long.toNumber() : long;
                    } else
                        object.feeBase = options.longs === String ? "0" : 0;
                    object.feeProp = 0;
                    object.cltvExpiryDelta = 0;
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.shortChannelId != null && message.hasOwnProperty("shortChannelId"))
                    object.shortChannelId = message.shortChannelId;
                if (message.feeBase != null && message.hasOwnProperty("feeBase"))
                    if (typeof message.feeBase === "number")
                        object.feeBase = options.longs === String ? String(message.feeBase) : message.feeBase;
                    else
                        object.feeBase = options.longs === String ? $util.Long.prototype.toString.call(message.feeBase) : options.longs === Number ? new $util.LongBits(message.feeBase.low >>> 0, message.feeBase.high >>> 0).toNumber(true) : message.feeBase;
                if (message.feeProp != null && message.hasOwnProperty("feeProp"))
                    object.feeProp = message.feeProp;
                if (message.cltvExpiryDelta != null && message.hasOwnProperty("cltvExpiryDelta"))
                    object.cltvExpiryDelta = message.cltvExpiryDelta;
                return object;
            };
    
            /**
             * Converts this RoutehintHop to JSON.
             * @function toJSON
             * @memberof greenlight.RoutehintHop
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            RoutehintHop.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return RoutehintHop;
        })();
    
        greenlight.Routehint = (function() {
    
            /**
             * Properties of a Routehint.
             * @memberof greenlight
             * @interface IRoutehint
             * @property {Array.<greenlight.IRoutehintHop>|null} [hops] Routehint hops
             */
    
            /**
             * Constructs a new Routehint.
             * @memberof greenlight
             * @classdesc Represents a Routehint.
             * @implements IRoutehint
             * @constructor
             * @param {greenlight.IRoutehint=} [properties] Properties to set
             */
            function Routehint(properties) {
                this.hops = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Routehint hops.
             * @member {Array.<greenlight.IRoutehintHop>} hops
             * @memberof greenlight.Routehint
             * @instance
             */
            Routehint.prototype.hops = $util.emptyArray;
    
            /**
             * Creates a new Routehint instance using the specified properties.
             * @function create
             * @memberof greenlight.Routehint
             * @static
             * @param {greenlight.IRoutehint=} [properties] Properties to set
             * @returns {greenlight.Routehint} Routehint instance
             */
            Routehint.create = function create(properties) {
                return new Routehint(properties);
            };
    
            /**
             * Encodes the specified Routehint message. Does not implicitly {@link greenlight.Routehint.verify|verify} messages.
             * @function encode
             * @memberof greenlight.Routehint
             * @static
             * @param {greenlight.IRoutehint} message Routehint message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Routehint.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.hops != null && message.hops.length)
                    for (var i = 0; i < message.hops.length; ++i)
                        $root.greenlight.RoutehintHop.encode(message.hops[i], writer.uint32(/* id 1, wireType 2 =*/10).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified Routehint message, length delimited. Does not implicitly {@link greenlight.Routehint.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.Routehint
             * @static
             * @param {greenlight.IRoutehint} message Routehint message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            Routehint.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a Routehint message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.Routehint
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.Routehint} Routehint
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Routehint.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.Routehint();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        if (!(message.hops && message.hops.length))
                            message.hops = [];
                        message.hops.push($root.greenlight.RoutehintHop.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a Routehint message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.Routehint
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.Routehint} Routehint
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            Routehint.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a Routehint message.
             * @function verify
             * @memberof greenlight.Routehint
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            Routehint.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.hops != null && message.hasOwnProperty("hops")) {
                    if (!Array.isArray(message.hops))
                        return "hops: array expected";
                    for (var i = 0; i < message.hops.length; ++i) {
                        var error = $root.greenlight.RoutehintHop.verify(message.hops[i]);
                        if (error)
                            return "hops." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a Routehint message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.Routehint
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.Routehint} Routehint
             */
            Routehint.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.Routehint)
                    return object;
                var message = new $root.greenlight.Routehint();
                if (object.hops) {
                    if (!Array.isArray(object.hops))
                        throw TypeError(".greenlight.Routehint.hops: array expected");
                    message.hops = [];
                    for (var i = 0; i < object.hops.length; ++i) {
                        if (typeof object.hops[i] !== "object")
                            throw TypeError(".greenlight.Routehint.hops: object expected");
                        message.hops[i] = $root.greenlight.RoutehintHop.fromObject(object.hops[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a Routehint message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.Routehint
             * @static
             * @param {greenlight.Routehint} message Routehint
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            Routehint.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults)
                    object.hops = [];
                if (message.hops && message.hops.length) {
                    object.hops = [];
                    for (var j = 0; j < message.hops.length; ++j)
                        object.hops[j] = $root.greenlight.RoutehintHop.toObject(message.hops[j], options);
                }
                return object;
            };
    
            /**
             * Converts this Routehint to JSON.
             * @function toJSON
             * @memberof greenlight.Routehint
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            Routehint.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return Routehint;
        })();
    
        greenlight.KeysendRequest = (function() {
    
            /**
             * Properties of a KeysendRequest.
             * @memberof greenlight
             * @interface IKeysendRequest
             * @property {Uint8Array|null} [nodeId] KeysendRequest nodeId
             * @property {greenlight.IAmount|null} [amount] KeysendRequest amount
             * @property {string|null} [label] KeysendRequest label
             * @property {Array.<greenlight.IRoutehint>|null} [routehints] KeysendRequest routehints
             * @property {Array.<greenlight.ITlvField>|null} [extratlvs] KeysendRequest extratlvs
             */
    
            /**
             * Constructs a new KeysendRequest.
             * @memberof greenlight
             * @classdesc Represents a KeysendRequest.
             * @implements IKeysendRequest
             * @constructor
             * @param {greenlight.IKeysendRequest=} [properties] Properties to set
             */
            function KeysendRequest(properties) {
                this.routehints = [];
                this.extratlvs = [];
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * KeysendRequest nodeId.
             * @member {Uint8Array} nodeId
             * @memberof greenlight.KeysendRequest
             * @instance
             */
            KeysendRequest.prototype.nodeId = $util.newBuffer([]);
    
            /**
             * KeysendRequest amount.
             * @member {greenlight.IAmount|null|undefined} amount
             * @memberof greenlight.KeysendRequest
             * @instance
             */
            KeysendRequest.prototype.amount = null;
    
            /**
             * KeysendRequest label.
             * @member {string} label
             * @memberof greenlight.KeysendRequest
             * @instance
             */
            KeysendRequest.prototype.label = "";
    
            /**
             * KeysendRequest routehints.
             * @member {Array.<greenlight.IRoutehint>} routehints
             * @memberof greenlight.KeysendRequest
             * @instance
             */
            KeysendRequest.prototype.routehints = $util.emptyArray;
    
            /**
             * KeysendRequest extratlvs.
             * @member {Array.<greenlight.ITlvField>} extratlvs
             * @memberof greenlight.KeysendRequest
             * @instance
             */
            KeysendRequest.prototype.extratlvs = $util.emptyArray;
    
            /**
             * Creates a new KeysendRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {greenlight.IKeysendRequest=} [properties] Properties to set
             * @returns {greenlight.KeysendRequest} KeysendRequest instance
             */
            KeysendRequest.create = function create(properties) {
                return new KeysendRequest(properties);
            };
    
            /**
             * Encodes the specified KeysendRequest message. Does not implicitly {@link greenlight.KeysendRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {greenlight.IKeysendRequest} message KeysendRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KeysendRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.nodeId != null && Object.hasOwnProperty.call(message, "nodeId"))
                    writer.uint32(/* id 1, wireType 2 =*/10).bytes(message.nodeId);
                if (message.amount != null && Object.hasOwnProperty.call(message, "amount"))
                    $root.greenlight.Amount.encode(message.amount, writer.uint32(/* id 2, wireType 2 =*/18).fork()).ldelim();
                if (message.label != null && Object.hasOwnProperty.call(message, "label"))
                    writer.uint32(/* id 3, wireType 2 =*/26).string(message.label);
                if (message.routehints != null && message.routehints.length)
                    for (var i = 0; i < message.routehints.length; ++i)
                        $root.greenlight.Routehint.encode(message.routehints[i], writer.uint32(/* id 4, wireType 2 =*/34).fork()).ldelim();
                if (message.extratlvs != null && message.extratlvs.length)
                    for (var i = 0; i < message.extratlvs.length; ++i)
                        $root.greenlight.TlvField.encode(message.extratlvs[i], writer.uint32(/* id 5, wireType 2 =*/42).fork()).ldelim();
                return writer;
            };
    
            /**
             * Encodes the specified KeysendRequest message, length delimited. Does not implicitly {@link greenlight.KeysendRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {greenlight.IKeysendRequest} message KeysendRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            KeysendRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a KeysendRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.KeysendRequest} KeysendRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KeysendRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.KeysendRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.nodeId = reader.bytes();
                        break;
                    case 2:
                        message.amount = $root.greenlight.Amount.decode(reader, reader.uint32());
                        break;
                    case 3:
                        message.label = reader.string();
                        break;
                    case 4:
                        if (!(message.routehints && message.routehints.length))
                            message.routehints = [];
                        message.routehints.push($root.greenlight.Routehint.decode(reader, reader.uint32()));
                        break;
                    case 5:
                        if (!(message.extratlvs && message.extratlvs.length))
                            message.extratlvs = [];
                        message.extratlvs.push($root.greenlight.TlvField.decode(reader, reader.uint32()));
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a KeysendRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.KeysendRequest} KeysendRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            KeysendRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a KeysendRequest message.
             * @function verify
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            KeysendRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    if (!(message.nodeId && typeof message.nodeId.length === "number" || $util.isString(message.nodeId)))
                        return "nodeId: buffer expected";
                if (message.amount != null && message.hasOwnProperty("amount")) {
                    var error = $root.greenlight.Amount.verify(message.amount);
                    if (error)
                        return "amount." + error;
                }
                if (message.label != null && message.hasOwnProperty("label"))
                    if (!$util.isString(message.label))
                        return "label: string expected";
                if (message.routehints != null && message.hasOwnProperty("routehints")) {
                    if (!Array.isArray(message.routehints))
                        return "routehints: array expected";
                    for (var i = 0; i < message.routehints.length; ++i) {
                        var error = $root.greenlight.Routehint.verify(message.routehints[i]);
                        if (error)
                            return "routehints." + error;
                    }
                }
                if (message.extratlvs != null && message.hasOwnProperty("extratlvs")) {
                    if (!Array.isArray(message.extratlvs))
                        return "extratlvs: array expected";
                    for (var i = 0; i < message.extratlvs.length; ++i) {
                        var error = $root.greenlight.TlvField.verify(message.extratlvs[i]);
                        if (error)
                            return "extratlvs." + error;
                    }
                }
                return null;
            };
    
            /**
             * Creates a KeysendRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.KeysendRequest} KeysendRequest
             */
            KeysendRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.KeysendRequest)
                    return object;
                var message = new $root.greenlight.KeysendRequest();
                if (object.nodeId != null)
                    if (typeof object.nodeId === "string")
                        $util.base64.decode(object.nodeId, message.nodeId = $util.newBuffer($util.base64.length(object.nodeId)), 0);
                    else if (object.nodeId.length)
                        message.nodeId = object.nodeId;
                if (object.amount != null) {
                    if (typeof object.amount !== "object")
                        throw TypeError(".greenlight.KeysendRequest.amount: object expected");
                    message.amount = $root.greenlight.Amount.fromObject(object.amount);
                }
                if (object.label != null)
                    message.label = String(object.label);
                if (object.routehints) {
                    if (!Array.isArray(object.routehints))
                        throw TypeError(".greenlight.KeysendRequest.routehints: array expected");
                    message.routehints = [];
                    for (var i = 0; i < object.routehints.length; ++i) {
                        if (typeof object.routehints[i] !== "object")
                            throw TypeError(".greenlight.KeysendRequest.routehints: object expected");
                        message.routehints[i] = $root.greenlight.Routehint.fromObject(object.routehints[i]);
                    }
                }
                if (object.extratlvs) {
                    if (!Array.isArray(object.extratlvs))
                        throw TypeError(".greenlight.KeysendRequest.extratlvs: array expected");
                    message.extratlvs = [];
                    for (var i = 0; i < object.extratlvs.length; ++i) {
                        if (typeof object.extratlvs[i] !== "object")
                            throw TypeError(".greenlight.KeysendRequest.extratlvs: object expected");
                        message.extratlvs[i] = $root.greenlight.TlvField.fromObject(object.extratlvs[i]);
                    }
                }
                return message;
            };
    
            /**
             * Creates a plain object from a KeysendRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.KeysendRequest
             * @static
             * @param {greenlight.KeysendRequest} message KeysendRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            KeysendRequest.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.arrays || options.defaults) {
                    object.routehints = [];
                    object.extratlvs = [];
                }
                if (options.defaults) {
                    if (options.bytes === String)
                        object.nodeId = "";
                    else {
                        object.nodeId = [];
                        if (options.bytes !== Array)
                            object.nodeId = $util.newBuffer(object.nodeId);
                    }
                    object.amount = null;
                    object.label = "";
                }
                if (message.nodeId != null && message.hasOwnProperty("nodeId"))
                    object.nodeId = options.bytes === String ? $util.base64.encode(message.nodeId, 0, message.nodeId.length) : options.bytes === Array ? Array.prototype.slice.call(message.nodeId) : message.nodeId;
                if (message.amount != null && message.hasOwnProperty("amount"))
                    object.amount = $root.greenlight.Amount.toObject(message.amount, options);
                if (message.label != null && message.hasOwnProperty("label"))
                    object.label = message.label;
                if (message.routehints && message.routehints.length) {
                    object.routehints = [];
                    for (var j = 0; j < message.routehints.length; ++j)
                        object.routehints[j] = $root.greenlight.Routehint.toObject(message.routehints[j], options);
                }
                if (message.extratlvs && message.extratlvs.length) {
                    object.extratlvs = [];
                    for (var j = 0; j < message.extratlvs.length; ++j)
                        object.extratlvs[j] = $root.greenlight.TlvField.toObject(message.extratlvs[j], options);
                }
                return object;
            };
    
            /**
             * Converts this KeysendRequest to JSON.
             * @function toJSON
             * @memberof greenlight.KeysendRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            KeysendRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return KeysendRequest;
        })();
    
        greenlight.StreamLogRequest = (function() {
    
            /**
             * Properties of a StreamLogRequest.
             * @memberof greenlight
             * @interface IStreamLogRequest
             */
    
            /**
             * Constructs a new StreamLogRequest.
             * @memberof greenlight
             * @classdesc Represents a StreamLogRequest.
             * @implements IStreamLogRequest
             * @constructor
             * @param {greenlight.IStreamLogRequest=} [properties] Properties to set
             */
            function StreamLogRequest(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * Creates a new StreamLogRequest instance using the specified properties.
             * @function create
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {greenlight.IStreamLogRequest=} [properties] Properties to set
             * @returns {greenlight.StreamLogRequest} StreamLogRequest instance
             */
            StreamLogRequest.create = function create(properties) {
                return new StreamLogRequest(properties);
            };
    
            /**
             * Encodes the specified StreamLogRequest message. Does not implicitly {@link greenlight.StreamLogRequest.verify|verify} messages.
             * @function encode
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {greenlight.IStreamLogRequest} message StreamLogRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamLogRequest.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                return writer;
            };
    
            /**
             * Encodes the specified StreamLogRequest message, length delimited. Does not implicitly {@link greenlight.StreamLogRequest.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {greenlight.IStreamLogRequest} message StreamLogRequest message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            StreamLogRequest.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a StreamLogRequest message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.StreamLogRequest} StreamLogRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamLogRequest.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.StreamLogRequest();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a StreamLogRequest message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.StreamLogRequest} StreamLogRequest
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            StreamLogRequest.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a StreamLogRequest message.
             * @function verify
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            StreamLogRequest.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                return null;
            };
    
            /**
             * Creates a StreamLogRequest message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.StreamLogRequest} StreamLogRequest
             */
            StreamLogRequest.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.StreamLogRequest)
                    return object;
                return new $root.greenlight.StreamLogRequest();
            };
    
            /**
             * Creates a plain object from a StreamLogRequest message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.StreamLogRequest
             * @static
             * @param {greenlight.StreamLogRequest} message StreamLogRequest
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            StreamLogRequest.toObject = function toObject() {
                return {};
            };
    
            /**
             * Converts this StreamLogRequest to JSON.
             * @function toJSON
             * @memberof greenlight.StreamLogRequest
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            StreamLogRequest.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return StreamLogRequest;
        })();
    
        greenlight.LogEntry = (function() {
    
            /**
             * Properties of a LogEntry.
             * @memberof greenlight
             * @interface ILogEntry
             * @property {string|null} [line] LogEntry line
             */
    
            /**
             * Constructs a new LogEntry.
             * @memberof greenlight
             * @classdesc Represents a LogEntry.
             * @implements ILogEntry
             * @constructor
             * @param {greenlight.ILogEntry=} [properties] Properties to set
             */
            function LogEntry(properties) {
                if (properties)
                    for (var keys = Object.keys(properties), i = 0; i < keys.length; ++i)
                        if (properties[keys[i]] != null)
                            this[keys[i]] = properties[keys[i]];
            }
    
            /**
             * LogEntry line.
             * @member {string} line
             * @memberof greenlight.LogEntry
             * @instance
             */
            LogEntry.prototype.line = "";
    
            /**
             * Creates a new LogEntry instance using the specified properties.
             * @function create
             * @memberof greenlight.LogEntry
             * @static
             * @param {greenlight.ILogEntry=} [properties] Properties to set
             * @returns {greenlight.LogEntry} LogEntry instance
             */
            LogEntry.create = function create(properties) {
                return new LogEntry(properties);
            };
    
            /**
             * Encodes the specified LogEntry message. Does not implicitly {@link greenlight.LogEntry.verify|verify} messages.
             * @function encode
             * @memberof greenlight.LogEntry
             * @static
             * @param {greenlight.ILogEntry} message LogEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            LogEntry.encode = function encode(message, writer) {
                if (!writer)
                    writer = $Writer.create();
                if (message.line != null && Object.hasOwnProperty.call(message, "line"))
                    writer.uint32(/* id 1, wireType 2 =*/10).string(message.line);
                return writer;
            };
    
            /**
             * Encodes the specified LogEntry message, length delimited. Does not implicitly {@link greenlight.LogEntry.verify|verify} messages.
             * @function encodeDelimited
             * @memberof greenlight.LogEntry
             * @static
             * @param {greenlight.ILogEntry} message LogEntry message or plain object to encode
             * @param {$protobuf.Writer} [writer] Writer to encode to
             * @returns {$protobuf.Writer} Writer
             */
            LogEntry.encodeDelimited = function encodeDelimited(message, writer) {
                return this.encode(message, writer).ldelim();
            };
    
            /**
             * Decodes a LogEntry message from the specified reader or buffer.
             * @function decode
             * @memberof greenlight.LogEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @param {number} [length] Message length if known beforehand
             * @returns {greenlight.LogEntry} LogEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            LogEntry.decode = function decode(reader, length) {
                if (!(reader instanceof $Reader))
                    reader = $Reader.create(reader);
                var end = length === undefined ? reader.len : reader.pos + length, message = new $root.greenlight.LogEntry();
                while (reader.pos < end) {
                    var tag = reader.uint32();
                    switch (tag >>> 3) {
                    case 1:
                        message.line = reader.string();
                        break;
                    default:
                        reader.skipType(tag & 7);
                        break;
                    }
                }
                return message;
            };
    
            /**
             * Decodes a LogEntry message from the specified reader or buffer, length delimited.
             * @function decodeDelimited
             * @memberof greenlight.LogEntry
             * @static
             * @param {$protobuf.Reader|Uint8Array} reader Reader or buffer to decode from
             * @returns {greenlight.LogEntry} LogEntry
             * @throws {Error} If the payload is not a reader or valid buffer
             * @throws {$protobuf.util.ProtocolError} If required fields are missing
             */
            LogEntry.decodeDelimited = function decodeDelimited(reader) {
                if (!(reader instanceof $Reader))
                    reader = new $Reader(reader);
                return this.decode(reader, reader.uint32());
            };
    
            /**
             * Verifies a LogEntry message.
             * @function verify
             * @memberof greenlight.LogEntry
             * @static
             * @param {Object.<string,*>} message Plain object to verify
             * @returns {string|null} `null` if valid, otherwise the reason why it is not
             */
            LogEntry.verify = function verify(message) {
                if (typeof message !== "object" || message === null)
                    return "object expected";
                if (message.line != null && message.hasOwnProperty("line"))
                    if (!$util.isString(message.line))
                        return "line: string expected";
                return null;
            };
    
            /**
             * Creates a LogEntry message from a plain object. Also converts values to their respective internal types.
             * @function fromObject
             * @memberof greenlight.LogEntry
             * @static
             * @param {Object.<string,*>} object Plain object
             * @returns {greenlight.LogEntry} LogEntry
             */
            LogEntry.fromObject = function fromObject(object) {
                if (object instanceof $root.greenlight.LogEntry)
                    return object;
                var message = new $root.greenlight.LogEntry();
                if (object.line != null)
                    message.line = String(object.line);
                return message;
            };
    
            /**
             * Creates a plain object from a LogEntry message. Also converts values to other types if specified.
             * @function toObject
             * @memberof greenlight.LogEntry
             * @static
             * @param {greenlight.LogEntry} message LogEntry
             * @param {$protobuf.IConversionOptions} [options] Conversion options
             * @returns {Object.<string,*>} Plain object
             */
            LogEntry.toObject = function toObject(message, options) {
                if (!options)
                    options = {};
                var object = {};
                if (options.defaults)
                    object.line = "";
                if (message.line != null && message.hasOwnProperty("line"))
                    object.line = message.line;
                return object;
            };
    
            /**
             * Converts this LogEntry to JSON.
             * @function toJSON
             * @memberof greenlight.LogEntry
             * @instance
             * @returns {Object.<string,*>} JSON object
             */
            LogEntry.prototype.toJSON = function toJSON() {
                return this.constructor.toObject(this, $protobuf.util.toJSONOptions);
            };
    
            return LogEntry;
        })();
    
        return greenlight;
    })();

    return $root;
});
