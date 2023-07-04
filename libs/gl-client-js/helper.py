# Yes, cdecker is so good at nodejs that he had to revert to python to
# get things working on windows. ~~ @cdecker :-)

# This tiny script is used to bundle index.node up into the tarball
# that'll be used when installing. The name contains version, platform
# and architecture.

import json
import subprocess
import sys

basedir = subprocess.check_output([
    "git",
    "rev-parse",
    "--show-toplevel"
]).strip().decode('ASCII')

# The architecture is often overridden because we cross-compile
architecture = sys.argv[1]

platform = subprocess.check_output([
    "node",
    "-e",
    "console.log(process.platform)"
]).strip().decode('ASCII')

with open(f"{basedir}/libs/gl-client-js/package.json") as p:
    meta = json.load(p)
    version = meta['version']

print(f"gl-client-{version}-{platform}-{architecture}.tar.gz")

subprocess.check_output([
    "tar",
    "-cvzf",
    f"gl-client-{version}-{platform}-{architecture}.tar.gz",
    "gl-client"
])
