import purerpc
from glclient import scheduler_pb2 as scheduler__pb2
from glclient import greenlight_pb2 as greenlight__pb2


class SchedulerServicer(purerpc.Servicer):
    async def Register(self, input_message):
        raise NotImplementedError()

    async def Recover(self, input_message):
        raise NotImplementedError()

    async def GetChallenge(self, input_message):
        raise NotImplementedError()

    async def Schedule(self, input_message):
        raise NotImplementedError()

    async def GetNodeInfo(self, input_message):
        raise NotImplementedError()

    async def MaybeUpgrade(self, input_message):
        raise NotImplementedError()

    async def ListInviteCodes(self, input_message):
        raise NotImplementedError()

    async def ExportNode(self, input_message):
        raise NotImplementedError()

    @property
    def service(self) -> purerpc.Service:
        service_obj = purerpc.Service(
            "scheduler.Scheduler"
        )
        service_obj.add_method(
            "Register",
            self.Register,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.RegistrationRequest,
                scheduler__pb2.RegistrationResponse,
            )
        )
        service_obj.add_method(
            "Recover",
            self.Recover,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.RecoveryRequest,
                scheduler__pb2.RecoveryResponse,
            )
        )
        service_obj.add_method(
            "GetChallenge",
            self.GetChallenge,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ChallengeRequest,
                scheduler__pb2.ChallengeResponse,
            )
        )
        service_obj.add_method(
            "Schedule",
            self.Schedule,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ScheduleRequest,
                scheduler__pb2.NodeInfoResponse,
            )
        )
        service_obj.add_method(
            "GetNodeInfo",
            self.GetNodeInfo,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.NodeInfoRequest,
                scheduler__pb2.NodeInfoResponse,
            )
        )
        service_obj.add_method(
            "MaybeUpgrade",
            self.MaybeUpgrade,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.UpgradeRequest,
                scheduler__pb2.UpgradeResponse,
            )
        )
        service_obj.add_method(
            "ListInviteCodes",
            self.ListInviteCodes,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ListInviteCodesRequest,
                scheduler__pb2.ListInviteCodesResponse,
            )
        )
        service_obj.add_method(
            "ExportNode",
            self.ExportNode,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ExportNodeRequest,
                scheduler__pb2.ExportNodeResponse,
            )
        )
        return service_obj


class SchedulerStub:
    def __init__(self, channel):
        self._client = purerpc.Client(
            "scheduler.Scheduler",
            channel
        )
        self.Register = self._client.get_method_stub(
            "Register",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.RegistrationRequest,
                scheduler__pb2.RegistrationResponse,
            )
        )
        self.Recover = self._client.get_method_stub(
            "Recover",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.RecoveryRequest,
                scheduler__pb2.RecoveryResponse,
            )
        )
        self.GetChallenge = self._client.get_method_stub(
            "GetChallenge",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ChallengeRequest,
                scheduler__pb2.ChallengeResponse,
            )
        )
        self.Schedule = self._client.get_method_stub(
            "Schedule",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ScheduleRequest,
                scheduler__pb2.NodeInfoResponse,
            )
        )
        self.GetNodeInfo = self._client.get_method_stub(
            "GetNodeInfo",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.NodeInfoRequest,
                scheduler__pb2.NodeInfoResponse,
            )
        )
        self.MaybeUpgrade = self._client.get_method_stub(
            "MaybeUpgrade",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.UpgradeRequest,
                scheduler__pb2.UpgradeResponse,
            )
        )
        self.ListInviteCodes = self._client.get_method_stub(
            "ListInviteCodes",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ListInviteCodesRequest,
                scheduler__pb2.ListInviteCodesResponse,
            )
        )
        self.ExportNode = self._client.get_method_stub(
            "ExportNode",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ExportNodeRequest,
                scheduler__pb2.ExportNodeResponse,
            )
        )


class DebugServicer(purerpc.Servicer):
    async def ReportSignerRejection(self, input_message):
        raise NotImplementedError()

    @property
    def service(self) -> purerpc.Service:
        service_obj = purerpc.Service(
            "scheduler.Debug"
        )
        service_obj.add_method(
            "ReportSignerRejection",
            self.ReportSignerRejection,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.SignerRejection,
                greenlight__pb2.Empty,
            )
        )
        return service_obj


class DebugStub:
    def __init__(self, channel):
        self._client = purerpc.Client(
            "scheduler.Debug",
            channel
        )
        self.ReportSignerRejection = self._client.get_method_stub(
            "ReportSignerRejection",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.SignerRejection,
                greenlight__pb2.Empty,
            )
        )


class PairingServicer(purerpc.Servicer):
    async def PairDevice(self, input_messages):
        raise NotImplementedError()

    async def GetPairingData(self, input_message):
        raise NotImplementedError()

    async def ApproveSession(self, input_message):
        raise NotImplementedError()

    @property
    def service(self) -> purerpc.Service:
        service_obj = purerpc.Service(
            "scheduler.Pairing"
        )
        service_obj.add_method(
            "PairDevice",
            self.PairDevice,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.PairDeviceRequest,
                scheduler__pb2.PairDeviceResponse,
            )
        )
        service_obj.add_method(
            "GetPairingData",
            self.GetPairingData,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.GetPairingDataRequest,
                scheduler__pb2.GetPairingDataResponse,
            )
        )
        service_obj.add_method(
            "ApprovePairing",
            self.ApprovePairing,
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ApprovePairingRequest,
                scheduler__pb2.Empty,
            )
        )
        return service_obj


class PairingStub:
    def __init__(self, channel):
        self._client = purerpc.Client(
            "scheduler.Pairing",
            channel
        )
        self.PairDevice = self._client.get_method_stub(
            "PairDevice",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.PairDeviceRequest,
                scheduler__pb2.PairDeviceResponse,
            )
        )
        self.GetPairingData = self._client.get_method_stub(
            "GetPairingData",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.GetPairingDataRequest,
                scheduler__pb2.GetPairingDataResponse,
            )
        )
        self.ApprovePairing = self._client.get_method_stub(
            "ApprovePairing",
            purerpc.RPCSignature(
                purerpc.Cardinality.UNARY_UNARY,
                scheduler__pb2.ApprovePairingRequest,
                scheduler__pb2.Empty,
            )
        )