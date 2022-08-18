const glclient = require('./index');
const buffer = require('buffer');
const prom = require('timers/promises');
const fs = require("fs");

var rewire = require('rewire');
var gl = rewire('./index.js');
const proto = require('./proto.js');

test('parseFeerate', () => {
    let Feerate = proto.greenlight.Feerate;
    let FeeratePreset = proto.greenlight.FeeratePreset;
    var parseFeerate = gl.__get__('parseFeerate');

    expect(parseFeerate('normal')).toEqual(Feerate.create({preset: FeeratePreset.NORMAL}));
    expect(parseFeerate('SLOW')).toEqual(Feerate.create({preset: FeeratePreset.SLOW}));
    expect(parseFeerate('Urgent')).toEqual(Feerate.create({preset: FeeratePreset.URGENT}));

    expect(parseFeerate('123perkw')).toEqual(Feerate.create({perkw: 123}));
    expect(parseFeerate('42perkb')).toEqual(Feerate.create({perkb: 42}));
})

test('parseConfirmation', () => {
    let Confirmation = proto.greenlight.Confirmation;
    var parseConfirmation = gl.__get__('parseConfirmation');
    expect(parseConfirmation(1)).toEqual(Confirmation.create({blocks: 1}));
})

test('parseAmount', () => {
    let Amount = proto.greenlight.Amount;
    var parseAmount = gl.__get__('parseAmount');

    expect(parseAmount('1msat')).toEqual(Amount.create({millisatoshi: 1}));
    expect(parseAmount('2sat')).toEqual(Amount.create({satoshi: 2}));
    expect(parseAmount('3btc')).toEqual(Amount.create({bitcoin: 3}));
})

test('parseBtcAddressType', () => {
    let BtcAddressType = proto.greenlight.BtcAddressType;
    var parseBtcAddressType = gl.__get__('parseBtcAddressType');

    expect(parseBtcAddressType()).toEqual(null);
    expect(parseBtcAddressType('bEch32')).toEqual(BtcAddressType.BECH32);
    expect(parseBtcAddressType('P2sh_segwit')).toEqual(BtcAddressType.P2SH_SEGWIT);
})

// Just a simple test to check if we can load the binary extension
// alright.
test('instantiate TlsConfig', () => {
    var tls = new glclient.TlsConfig();
    console.log(tls);
})

test('signer.version()', () => {
    var signer = new glclient.Signer(
	buffer.Buffer("00000000000000000000000000000000"),
	"bitcoin",
	new glclient.TlsConfig()
    );
    let v = signer.version();
    expect(v).toEqual("v0.10.2");
})

test('Test signer startup and shutdown', () => {
    var signer = new glclient.Signer(
	buffer.Buffer("00000000000000000000000000000000"),
	"bitcoin",
	new glclient.TlsConfig()
    );
    expect(signer.handle).toBeUndefined();
    signer.run_in_thread();
    expect(signer.handle).toBeDefined();
    signer.shutdown();
    expect(signer.handle).toBeUndefined();
})

test('Test node scheduler and getinfo', async () => {
    var signer = new glclient.Signer(
	buffer.Buffer("00000000000000000000000000000000"),
	"testnet",
	new glclient.TlsConfig()
    );

    // Don't want to add keys here, so let's recover
    var scheduler = new glclient.Scheduler(signer.node_id(), "testnet", new glclient.TlsConfig())
    var rec = scheduler.recover(signer)

    console.log(rec.deviceCert)
    console.log(rec.deviceKey)
    var tls = new glclient.TlsConfig();
    tls.inner = tls.identity(
	buffer.Buffer(rec.deviceCert),
	buffer.Buffer(rec.deviceKey)
    );

    // Now that we have an identity matching the key above, we have to
    // reinit the scheduler stub to use it.
    var scheduler = new glclient.Scheduler(
	signer.node_id(),
	"testnet",
	tls
    )
    var node = scheduler.schedule();
    var info = await node.get_info();
    console.log(info);
})
