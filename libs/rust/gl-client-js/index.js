"use strict";
const {
    signerNew,
    signerRunInThread,
    signerRunInForeground,
    signerSignMessage,
    signerNodeId,

    tlsConfigNew,
    tlsConfigIdentity,

    schedulerNew,
    schedulerRegister,
    schedulerRecover,
    schedulerSchedule,

    nodeCall,
} = require("./index.node");

const proto = require('./proto.js');

const protobuf = require("protobufjs");
const promisify  = require("util");
const fs = require("fs");
const buffer = require("buffer");

class Signer {
    constructor(secret, network, tls) {
        this.inner = signerNew(secret, network, tls.inner);
	this.tls = tls;
    }

    run_in_thread() {
	signerRunInThread(this.inner);
    }
    run_in_foreground() {
	signerRunInForeground(this.inner);
    }

    node_id() {
	return signerNodeId(this.inner);
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
function parseAmount(amtstr, allow_any=true, allow_all=true) {
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

    list_peers() {
	return this._call(
		"listpeers",
		proto.greenlight.ListPeersRequest,
		proto.greenlight.ListPeersResponse,
		{}
	    )
    }

    connect_peer() {
	throw "unimplemented"
    }

    close() {
	throw "unimplemented"
    }

    disconnect_peer() {
	throw "unimplemented"
    }

    new_address() {
	throw "unimplemented"
    }

    withdraw() {
	throw "unimplemented"
    }

    fund_channel() {
	throw "unimplemented"
    }

    create_invoice() {
	throw "unimplemented"
    }

    pay() {
	throw "unimplemented"
    }

    keysend() {
	throw "unimplemented"
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
