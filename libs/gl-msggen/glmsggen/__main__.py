from typing import TextIO, Tuple, Dict, Any
from textwrap import dedent, indent
import logging
import sys
import re
import click

from msggen.model import (
    ArrayField,
    CompositeField,
    EnumField,
    PrimitiveField,
    Service,
    Method,
)
from msggen.utils import load_jsonrpc_service, combine_schemas
from msggen.patch import VersionAnnotationPatch, OptionalPatch, OverridePatch
from pathlib import Path
from msggen.gen.generator import IGenerator
from glmsggen.udlgen import UdlGenerator, RustGenerator
from msggen.gen.generator import GeneratorChain

logger = logging.getLogger(__name__)


@click.command()
def cli():
    # meta = load_msggen_meta()
    meta = {}
    service = load_jsonrpc_service()

    # TODO: The following requires access to the `msggen.json` file
    # which we currently do not bundle with msggen. So for now we skip
    # the backward and forwards compatibility patch.

    p = VersionAnnotationPatch(meta=meta)
    p.apply(service)
    OptionalPatch().apply(service)

    # Mark some fields we can't map as omitted, and for complex types
    # we manually mapped, use those types instead.
    OverridePatch().apply(service)

    directory = (
        Path(__file__).parent
        / ".."
        / ".."
        / "gl-bindings"
        / "src"
    )

    fname = directory / "glclient.udl"

    with open(fname, "w") as f:
        chain = GeneratorChain()
        chain.add_generator(UdlGenerator(f, meta))
        chain.generate(service)

    fname = directory / "gen.rs"

    with open(fname, "w") as f:
        chain = GeneratorChain()
        chain.add_generator(RustGenerator(f, meta))
        chain.generate(service)


if __name__ == "__main__":
    cli()
