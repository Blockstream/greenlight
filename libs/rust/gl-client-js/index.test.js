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
