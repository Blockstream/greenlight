fn main() {
    let mut builder = tonic_build::configure();

    if cfg!(feature = "serde") {
        for object in OBJECTS {
            builder = builder.type_attribute(object, ATTR);
        }
    }

    builder
        .compile(
            &["../proto/greenlight.proto", "../proto/scheduler.proto"],
            &["../proto"],
        )
        .unwrap();
}

const ATTR: &'static str = "#[derive(serde::Serialize, serde::Deserialize)]";
const OBJECTS: [&'static str; 44] = [
    "ConnectRequest",
    "ConnectResponse",
    "DisconnectRequest",
    "DisconnectResponse",
    "Outpoint",
    "Confirmation",
    "Amount.unit",
    "Amount",
    "FundChannelRequest",
    "FundChannelResponse",
    "Address",
    "Feerate",
    "Feerate.value",
    "GetInfoRequest",
    "GetInfoResponse",
    "Invoice",
    "InvoiceRequest",
    "KeysendRequest",
    "TlvField",
    "Routehint",
    "RoutehintHop",
    "ListFundsRequest",
    "ListFundsResponse",
    "ListFundsChannel",
    "ListFundsOutput",
    "ListInvoicesRequest",
    "InvoiceIdentifier",
    "InvoiceIdentifier.id",
    "ListInvoicesResponse",
    "ListPaymentsRequest",
    "PaymentIdentifier",
    "PaymentIdentifier.id",
    "ListPaymentsResponse",
    "Payment",
    "ListPeersRequest",
    "ListPeersResponse",
    "Peer",
    "Channel",
    "Htlc",
    "NewAddrRequest",
    "NewAddrResponse",
    "PayRequest",
    "WithdrawRequest",
    "WithdrawResponse",
];
