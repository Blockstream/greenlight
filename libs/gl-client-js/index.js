"use strict";
var binary = require('@mapbox/node-pre-gyp');
var path = require('path')
var binding_path = binary.find(path.resolve(path.join(__dirname,'./package.json')));

const {
    signerNew,
    signerRunInThread,
    signerRunInForeground,
    signerSignMessage,
    signerNodeId,
    signerVersion,

    tlsConfigNew,
    tlsConfigIdentity,

    schedulerNew,
    schedulerRegister,
    schedulerRecover,
    schedulerSchedule,

    nodeCall,
    nodeCallStreamLog,
    logStreamNext,
    nodeCallStreamIncoming,
    incomingStreamNext,

    signerHandleShutdown,

} = require(binding_path);

const proto = require('./proto.js');

const protobuf = require("protobufjs");
const promisify  = require("util");
const fs = require("fs");
const buffer = require("buffer");

class Signer {
    constructor(secret, network, tls) {
        this.inner = signerNew(secret, network, tls.inner);
	this.tls = tls;
	this.handle = undefined;
    }

    run_in_thread() {
	this.handle = signerRunInThread(this.inner);
	return this.handle;
    }

    shutdown() {
	signerHandleShutdown(this.handle);
	this.handle = undefined;
    }

    run_in_foreground() {
	signerRunInForeground(this.inner);
    }

    node_id() {
	return signerNodeId(this.inner);
    }

    version() {
	return signerVersion(this.inner);
    }
}

class Scheduler {
    constructor(node_id, network, tls) {
	this.inner = schedulerNew(node_id, network);
	this.nodeId = node_id;
	this.network = network;
	this.tls = tls;
    }

    recover(signer) {
	return proto.scheduler.RecoveryResponse.decode(
	    schedulerRecover(this.inner, signer.inner)
	)
    }

    register(signer) {
	return proto.scheduler.RegistrationResponse.decode(
	    schedulerRegister(this.inner, signer.inner)
	)
    }

    schedule() {
	let n = new Node()
	n.inner = schedulerSchedule(this.inner, this.tls.inner)
	return n
    }
}

function ensureByteNodeId(node_id) {
    if (!Buffer.isBuffer(node_id)) {
	return Buffer.from(node_id, "hex");
    } else {
	return node_id;
    }
}

function ensureStrNodeId(node_id) {
    if (Buffer.isBuffer(node_id)) {
	return node_id.toString("hex");
    } else {
	return node_id;
    }
}

/* Parse the amount from the given string. It supports sat, btc and
 * msat as suffixes, as well as `any` and `all` as substitutes. */
function parseAmount(amtstr, {allow_any=true, allow_all=true}={}) {
    let ival = parseInt(amtstr);
    let suffix = amtstr.slice(ival.toString().length);
    let amount = proto.greenlight.Amount.create()

    if (suffix == "msat") {
	amount['millisatoshi'] = ival;
    } else if (suffix == "sat") {
	amount['satoshi'] = ival;
    } else if (suffix == "btc") {
	amount['bitcoin'] = ival;
    } else if (amtstr == "all") {
	if (!allow_all)
	    throw "`all` is not allowed as value for this amount";
	amount['all'] = true;
    } else if (amtstr == "any") {
	if (!allow_any)
	    throw "`any` is not allowed as value for this amount";
	amount['any'] = true;
    } else {
	throw "Unknown amount suffix `" + suffix + "`";
    }
    return amount;
}

function parseConfirmation(blocks) {
    return proto.greenlight.Confirmation.create({blocks: blocks});
}

function parseFeerate(feestr) {
    if (feestr === null)
	return null;

    let ival = parseInt(feestr);
    let suffix = feestr.slice(ival.toString().length);
    let feerate = proto.greenlight.Feerate.create()

    if (feestr.toUpperCase() == "NORMAL")
	feerate['preset'] = proto.greenlight.FeeratePreset.NORMAL;
    else if (feestr.toUpperCase() == "SLOW")
	feerate['preset'] = proto.greenlight.FeeratePreset.SLOW;
    else if (feestr.toUpperCase() == "URGENT")
	feerate['preset'] = proto.greenlight.FeeratePreset.URGENT;
    else if (suffix == "perkw")
	feerate['perkw'] = ival;
    else if (suffix == "perkb")
	feerate['perkb'] = ival;
    else
	throw "Unknown amount suffix `" + suffix + "`";
    return feerate;
}

function parseBtcAddressType(t) {
    let BtcAddressType = proto.greenlight.BtcAddressType;
    if (t == null)
	return null
    else if (t.toUpperCase() == "BECH32")
	return BtcAddressType.BECH32;
    else if (t.toUpperCase() == "P2SH_SEGWIT")
	return BtcAddressType.P2SH_SEGWIT;
    else if (t != null)
	throw "Unknow bitcoin address type " + t + ", allowed values are `bech32` and `p2wsh_segwit`"
}

function parseBtcAddress(a) {
    if (a == null)
	return a;

    return proto.greenlight.BitcoinAddress({address: a});
}

class Node {
    _call(method, reqType, resType, properties) {
	let req = reqType.create(properties)
	var raw = reqType.encode(req).finish();
	raw = nodeCall(
	    this.inner,
	    method,
	    raw
	)
	let resp = resType.decode(raw);
	return resp;
    }

    get_info() {
	return this._call(
	    "get_info",
	    proto.greenlight.GetInfoRequest,
	    proto.greenlight.GetInfoResponse,
	    {}
	)
    }

    stop() {
	try {
	    this._call(
		"stop",
		proto.greenlight.StopRequest,
		proto.greenlight.StopResponse,
		{}
	    )
	    return false;
	} catch {
	    return true;
	}
    }

    list_funds() {
	return this._call(
		"listfunds",
		proto.greenlight.ListFundsRequest,
		proto.greenlight.ListFundsResponse,
		{}
	    )
    }

    list_invoices() {
	return this._call(
		"listinvoices",
		proto.greenlight.ListInvoicesRequest,
		proto.greenlight.ListInvoicesResponse,
		{}
	    )
    }

    list_payments() {
	return this._call(
		"listpayments",
		proto.greenlight.ListPaymentsRequest,
		proto.greenlight.ListPaymentsResponse,
		{}
	    )
    }

    list_peers(node_id=null) {
	return this._call(
	    "listpeers",
	    proto.greenlight.ListPeersRequest,
	    proto.greenlight.ListPeersResponse,
	    {
		nodeId: ensureStrNodeId(node_id)
	    }
	)
    }

    connect_peer(node_id, addr=null) {
	return this._call(
	    "connect_peer",
	    proto.greenlight.ConnectRequest,
	    proto.greenlight.ConnectResponse,
	    {
		nodeId: ensureStrNodeId(node_id),
		addr: addr
	    }
	)
    }

    close(node_id, {unilateraltimeout=null, destination=null}={}) {
	return this._call(
	    "close_channel",
	    proto.greenlight.CloseChannelRequest,
	    proto.greenlight.CloseChannelResponse,
	    {
		nodeId: ensureByteNodeId(node_id),
		unilateraltimeout: unilateraltimeout,
		destination: parseBtcAddress(destination)
	    }
	)
    }

    disconnect_peer(node_id, force=false) {
	return this._call(
	    "disconnect",
	    proto.greenlight.DisconnectRequest,
	    proto.greenlight.DisconnectResponse,
	    {
		nodeId: ensureStrNodeId(node_id),
		force: force
	    }
	)
    }

    new_address(addressType=null) {
	return this._call(
	    "newaddr",
	    proto.greenlight.NewAddrRequest,
	    proto.greenlight.NewAddrResponse,
	    {
		addressType: parseBtcAddressType(addressType)
	    }
	)
    }

    withdraw(destination, amount, {feerate=null, minconf=null}={}) {
	return this._call(
	    "withdraw",
	    proto.greenlight.WithdrawRequest,
	    proto.greenlight.WithdrawResponse,
	    {
		destination: destination,
		amount: parseAmount(amount, {allow_any: false}),
		feerate: parseFeerate(feerate),
		minconf: parseConfirmation(minconf),
		utxos: null,
	    }
	)
    }

    fund_channel(node_id, amount, {feerate=null, announce=false, minconf=null, close_to=null}={}) {
	return this._call(
	    "fund_channel",
	    proto.greenlight.FundChannelRequest,
	    proto.greenlight.FundChannelResponse,
	    {
		nodeId: ensureByteNodeId(node_id),
		amount: parseAmount(amount),
		feerate: parseFeerate(feerate),
		announce: announce,
		minconf: parseConfirmation(minconf),
		close_to: close_to,
	    }
	)
    }

    create_invoice(amount, label, description=null) {
	return this._call(
	    "create_invoice",
	    proto.greenlight.InvoiceRequest,
	    proto.greenlight.Invoice,
	    {
		amount: parseAmount(amount),
		label: label,
		description: description
	    }
	)
    }

    pay(bolt11, {amount=null, timeout=null}={}) {
	return this._call(
	    "pay",
	    proto.greenlight.PaymentRequest,
	    proto.greenlight.Payment,
	    {
		bolt11: bolt11,
		amount: parseAmount(amount),
		timeout: timeout
	    }
	)
    }

    keysend(node_id, amount, label=null, {routehints=null, extratlvs=null}={}) {
	return this._call(
	    "keysend",
	    proto.greenlight.KeysendRequest,
	    proto.greenlight.Payment,
	    {
		nodeId: ensureByteNodeId(node_id),
		amount: parseAmount(amount),
		label: label,
		routehints: routehints, // TODO: Provide parseRoutehints helper
		extratlvs: extratlvs // TODO: Provide parseTlvs helper
	    }
	)
    }

    stream_log() {
	return new Streaming(nodeCallStreamLog(this.inner), proto.greenlight.LogEntry, logStreamNext)
    }

    stream_incoming() {
	return new Streaming(nodeCallStreamIncoming(this.inner), proto.greenlight.IncomingPayment, incomingStreamNext)
    }
}

class Streaming {
    constructor(stream, typ, func) {
	this.stream = stream;
	this.typ = typ
	this.func = func
    }

    next() {
	return this.typ.decode(this.func(this.stream));
    }
}

class TlsConfig {
    constructor() {
	this.inner = tlsConfigNew();
    }

    load_file(cert_path, key_path) {
	let cert = fs.readFileSync(cert_path);
	let key = fs.readFileSync(key_path);
	this.inner = this.identity(cert, key);
	return this;
    }

    identity(cert, key) {
	return tlsConfigIdentity(this.inner, cert, key)
    }
}

module.exports = {
    Signer: Signer,
    Scheduler: Scheduler,
    Node: Node,
    TlsConfig: TlsConfig
}
