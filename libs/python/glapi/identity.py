import grpc
import os

class Identity(object):
    """A wrapper encapsulating our certificate conventions."""

    @classmethod
    def from_path(cls, path):
        self = Identity()
        certdir = os.path.abspath(
            os.path.join(os.path.dirname(__file__), "..", "certs")
        )
        self.path = path
        # TODO This should be the root certificate in ca.pem, not services.pem
        self.caroot = open(os.path.join(certdir, "ca.pem"), "rb").read()

        splits = path[1:].split("/")
        relpath, fstub = splits[:-1], splits[-1]
        directory = os.path.join(certdir, *relpath)

        self.private_key_path = os.path.join(directory, f"{fstub}-key.pem")
        self.public_key_path = os.path.join(directory, f"{fstub}.pem")
        self.cert_chain_path = os.path.join(directory, f"{fstub}.crt")

        self.private_key = open(self.private_key_path, "rb").read()
        self.public_key = open(self.public_key_path, "rb").read()
        self.cert_chain = open(self.cert_chain_path, "rb").read()
        return self

    def __init__(self, pem, crt, key, caroot):
        self.private_key = key
        self.public_key = pem
        self.cert_chain = crt
        self.caroot = caroot

    @classmethod
    def from_register_result(cls, res):
        self = Identity()
        certdir = os.path.abspath(
            os.path.join(os.path.dirname(__file__), "..", "certs")
        )
        self.private_key = res.device_key.encode("ASCII")
        self.public_key = res.device_cert.encode("ASCII")
        self.cert_chain = res.device_cert.encode("ASCII")
        self.caroot = open(os.path.join(certdir, "ca.pem"), "rb").read()
        return self

    def to_channel_credentials(self):
        return grpc.ssl_channel_credentials(
            root_certificates=self.caroot,
            private_key=self.private_key,
            certificate_chain=self.cert_chain,
        )

    def to_server_credentials(self):
        return grpc.ssl_server_credentials(
            [(self.private_key, self.cert_chain)],
            root_certificates=self.caroot,
            require_client_auth=True,
        )

    def __str__(self):
        return f"Identity[{self.path}]"
