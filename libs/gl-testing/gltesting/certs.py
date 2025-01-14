# Utilities to generate a CA and any dependent cetificates
from distutils.util import convert_path
import logging
import tempfile
import json
import os
import subprocess
from cryptography import x509
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat._oid import NameOID
from .identity import Identity


logging.getLogger().setLevel(logging.DEBUG)
logging.getLogger('sh').setLevel(logging.INFO)
csr = json.loads(
    """{
  "CN": "GL root CA",
  "key": {
    "algo": "ecdsa",
    "size": 256
  },
  "names": [
    {
      "C": "US",
      "L": "San Francisco",
      "O": "Test",
      "OU": "CertificateAuthority",
      "ST": "California"
    }
  ],
  "ca": {
    "expiry": "24h"
  },
  "hosts": [
     "localhost"
  ]
}"""
)

config = """
{
  "signing": {
    "profiles": {
      "inner": {
        "usages": [
            "signing",
            "digital signature",
            "key encipherment",
            "cert sign",
            "crl sign",
            "server auth",
            "client auth"
        ],
        "expiry": "87600h",
        "ca_constraint": {
            "is_ca": true,
            "max_path_len": 3,
            "max_path_len_zero": true
        }
      },
      "leaf": {
        "usages": [
            "signing",
            "digital signature",
            "key encipherment",
            "cert sign",
            "crl sign",
            "server auth",
            "client auth"
        ],
        "expiry": "87600h"
      }
    }
  }
}
"""

def path_to_identity(path):
    """Given a path, return a tuple with the files used for the identity.

    This uses our file structure convention to convert from an
    identity path in the hierarchy of TLS certificates to the private
    key and certificate files that represent that ID.

    """
    cwd = os.path.dirname(__file__)
    root = os.environ.get('GL_CERT_PATH', None)
    if root is None:
        root = os.path.abspath(os.path.join(cwd, "..", 'certs'))

    if path == "/":
        return (
            os.path.join(root, "ca.pem"),
            os.path.join(root, "ca-key.pem"),
            os.path.join(root, "ca.crt"),
            os.path.join(root, "ca"),
        )

    parts = path.split("/")[1:]
    path = os.path.join(root, *parts[:-1])
    fname = parts[-1]
    return (
        os.path.join(path, fname + ".pem"),
        os.path.join(path, fname + "-key.pem"),
        os.path.join(path, fname + ".crt"),
        os.path.join(path, fname),
    )

def postprocess_private_key(keyfile):
    result = subprocess.run(["openssl", "pkcs8", "-topk8", "-nocrypt", "-in", keyfile], capture_output=True, text=True)
    if result.returncode == 0:
        converted = result.stdout
        with open(keyfile, "wb") as f:
            f.write(converted.encode())
    else:
        raise RuntimeError(f"OpenSSL command failed with error: {result.stderr}")


def parent_ca(path):
    splits = path.split("/")
    if len(path.split("/")) > 2:
        parent = "/".join(splits[:-1])
    else:
        parent = "/"

    return parent


def gencrt(path, force=False):
    parentca = parent_ca(path)
    parent = path_to_identity(parentca)
    parent_bundle = parent[3] + ".crt"
    path = path_to_identity(path)

    fname = path[3] + ".crt"
    if os.path.exists(fname) and not force:
        logging.info(f"Not overwriting existing file {fname}")
        return fname

    with open(fname, "wb") as f:
        f.write(open(path[0], "rb").read())
        f.write(open(parent_bundle, "rb").read())

    return fname

def genca(idpath):
    """Generate an (intermediate) CA that can sign further certificates"""
    logging.debug(f"Generating a new CA for path {idpath}")
    profile = "inner"
    mycsr = csr.copy()
    mycsr["CN"] = f"GL {idpath}"
    del mycsr["hosts"]
    parent = parent_ca(idpath)
    logging.debug(f"Using CA {parent} as parent")

    _path = idpath
    parent = path_to_identity(parent)
    path = path_to_identity(idpath)

    for f in path:
        if os.path.exists(f):
            logging.info(f"Not overwriting existing file {f}")
            return

    tmpcsr = tempfile.NamedTemporaryFile(mode="w")
    json.dump(mycsr, tmpcsr)
    tmpcsr.flush()

    directory = os.path.dirname(path[0])
    if not os.path.exists(directory):
        os.makedirs(directory)

    certs_json = subprocess.check_output(["cfssl", "gencert", "-initca", tmpcsr.name])
    subprocess.run(["cfssljson", "-bare", path[3]], input=certs_json)
    
    # Write config
    tmpconfig = tempfile.NamedTemporaryFile(mode="w")
    tmpconfig.write(config)
    tmpconfig.flush()
    sign_certs_json = subprocess.check_output(["cfssl", "sign", f"-ca={parent[0]}", f"-ca-key={parent[1]}", f"-config={tmpconfig.name}", f"-profile={profile}", path[3] + ".csr"])
    subprocess.run(["cfssljson", "-bare", path[3]], input=sign_certs_json)
    # Cleanup the temporary certificate signature request
    os.remove(path[3] + ".csr")

    postprocess_private_key(path[1])
    crt = gencrt(_path, force=True)

    return Identity.from_path(idpath)


def gencert(idpath):
    """Generate a leaf certificate to be used for actual communication."""
    logging.debug(f"Generating a new certificate for {idpath}")
    profile = "leaf"
    mycsr = csr.copy()
    mycsr["CN"] = f"GL {idpath}"
    print(mycsr)
    del mycsr["ca"]

    parent = parent_ca(idpath)
    print(f"Using CA {parent} as parent")
    path = path_to_identity(idpath)
    parent = path_to_identity(parent)
    for f in path:
        if os.path.exists(f):
            logging.info(f"Not overwriting existing file {f}")
            return Identity.from_path(idpath)

    tmpcsr = tempfile.NamedTemporaryFile(mode="w")
    json.dump(mycsr, tmpcsr)
    tmpcsr.flush()

    # Write config
    tmpconfig = tempfile.NamedTemporaryFile(mode="w")
    tmpconfig.write(config)
    tmpconfig.flush()
    directory = os.path.dirname(path[0])

    if not os.path.exists(directory):
        os.makedirs(directory)

    certs_json = subprocess.check_output(["cfssl", "gencert", f"-ca={parent[0]}", f"-ca-key={parent[1]}", f"-config={tmpconfig.name}", f"-profile={profile}", tmpcsr.name])
    subprocess.run(["cfssljson", "-bare", path[3]], input=certs_json)
    # Cleanup the temporary certificate signature request
    os.remove(path[3] + ".csr")

    postprocess_private_key(path[1])
    gencrt(idpath, force=True)
    return Identity.from_path(idpath)

def gencert_from_csr(csr: bytes, recover=False, pairing=False):
    """Generate a leaf certificate to be used for actual communication from
    certificate signing request."""
    # Get idpath from CN value in certificate signing request
    mycsr = x509.load_pem_x509_csr(csr, default_backend)
    idpath = mycsr.subject.get_attributes_for_oid(NameOID.COMMON_NAME)[0].value

    if pairing:
        # Create override subject.
        parts = idpath.split("/")
        device_name = parts[3]
        dummy_node_id = "0a0a"
        idpath = f"/users/{dummy_node_id}/{device_name}"
        subject = json.dumps({
            "CN": idpath,
            "names": [
                {
                    "C": mycsr.subject.get_attributes_for_oid(NameOID.COUNTRY_NAME)[0].value,
                    "L": mycsr.subject.get_attributes_for_oid(NameOID.LOCALITY_NAME)[0].value,
                    "O": mycsr.subject.get_attributes_for_oid(NameOID.ORGANIZATION_NAME)[0].value,
                    "ST": mycsr.subject.get_attributes_for_oid(NameOID.STATE_OR_PROVINCE_NAME)[0].value,
                    "OU": mycsr.subject.get_attributes_for_oid(NameOID.ORGANIZATIONAL_UNIT_NAME)[0].value,
                }
            ]
        })
        tmpsubject = tempfile.NamedTemporaryFile(mode="w")
        tmpsubject.write(subject)
        tmpsubject.flush()
        print(f"Write pairing override subject {tmpsubject.name}")

        # Gen CA
        genca(f"/users/{dummy_node_id}")

    parent = parent_ca(idpath)
    print(f"Using CA {parent} as parent")
    path = path_to_identity(idpath)
    parent = path_to_identity(parent)
    for f in path:
        if os.path.exists(f) and not (recover or pairing):
            logging.info(f"Not overwriting existing file {f}")
            return

    tmpcsr = tempfile.NamedTemporaryFile(mode="w")
    tmpcsr.write(csr.decode('ASCII'))
    tmpcsr.flush()

    # Write config
    tmpconfig = tempfile.NamedTemporaryFile(mode="w")
    tmpconfig.write(config)
    tmpconfig.flush()
    directory = os.path.dirname(path[0])

    if not os.path.exists(directory):
        os.makedirs(directory)

    if pairing:
        sign_certs_json = subprocess.check_output(["cfssl", "sign", f"-ca={parent[0]}", f"-ca-key={parent[1]}", tmpcsr.name, tmpsubject.name])
    else:
        sign_certs_json = subprocess.check_output(["cfssl", "sign", f"-ca={parent[0]}", f"-ca-key={parent[1]}", tmpcsr.name])

    subprocess.run(["cfssljson", "-bare", path[3]], input=sign_certs_json)

    # Cleanup the temporary certificate signature request
    os.remove(path[3] + ".csr")
    
    # Generate, read and return cert chain
    gencrt(idpath, force=True)
    assert(os.path.exists(f"{path[3]}.crt"))
    certf = open(f"{path[3]}.crt")
    cert = certf.read()
    certf.close()
    return cert
