import os
import grpc
import ssl

class Identity(object):
    """A wrapper encapsulating our certificate conventions."""

    @classmethod
    def from_path(cls, path):
        self = Identity()
        self.path = path
        path = "/ca" if path == "/" else path

        certdir = os.environ.get('GL_CERT_PATH', None)
        assert certdir is not None
        self.caroot_path = os.path.join(certdir, "ca.pem")
        self.caroot = open(self.caroot_path, "rb").read()

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

    @classmethod
    def from_register_result(cls, res):
        self = Identity()
        certdir = os.environ.get('GL_CERT_PATH', None)
        assert certdir is not None
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

    def to_ssl_context(self):
        s = ssl.SSLContext(ssl.PROTOCOL_TLS)
        s.load_cert_chain(self.cert_chain_path, keyfile=self.private_key_path)
        s.load_verify_locations(capath=self.caroot_path)
        s.set_alpn_protocols(['h2'])
        try:
            s.set_npn_protocols(['h2'])
        except NotImplementedError:
            pass
        return s

    def __str__(self):
        return f"Identity[{self.path}]"
