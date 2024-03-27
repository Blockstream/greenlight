import importlib
import os
import sys
from typing import Optional

import clnvm
from clnvm.cln_version_manager import ClnVersionManager, VersionDescriptor

# Handle the optional import and provide a nice error message if it fails
_click = importlib.util.find_spec("click")
if _click is None:
    print("To use clnvm the `cli` feature must be installed")
    print("You can install the feature using")
    print("> pip install gltesting[cli]")
    exit()

import click


@click.group()
def cli() -> None:
    pass


@cli.command()
@click.option("--force", is_flag=True)
def get_all(force: bool) -> None:
    version_manager = ClnVersionManager()
    versions = version_manager.get_versions()
    for version in versions:
        try:
            result = version_manager.get(version, force)
            click.echo(result.path)
        except Exception as e:
            click.echo(click.style(str(e), fg="red"), err=True)


@cli.command()
@click.option("--tag", required=True)
@click.option("--force", is_flag=True)
def get(tag: str, force: bool) -> None:
    try:
        version_manager = ClnVersionManager()
        descriptor = version_manager.get_descriptor_from_tag(tag)
        node_version = version_manager.get(descriptor, force)
        click.echo(node_version.path)
    except Exception as e:
        # Print the error and return a non-zero status-code
        click.echo(click.style(str(e), fg="red"), err=True)
        sys.exit(1)


@cli.command()
@click.option("--tag", is_flag=True)
@click.option("--lightningd", is_flag=True)
@click.option("--path", is_flag=True)
@click.option("--bin-path", is_flag=True)
def latest(tag: bool, lightningd: bool, path: bool, bin_path: bool) -> None:
    version_manager = ClnVersionManager()
    latest = version_manager.latest()

    if tag:
        click.echo(latest.name)
    elif lightningd:
        click.echo(latest.path)
    elif bin_path:
        head, tail = os.path.split(latest.path)
        click.echo(head)
    elif path:
        # Drop the /usr/local/bin/lightningd part
        parts = latest.path.parts[:-4]
        my_path = os.path.join(*parts)
        click.echo(my_path)
    else:
        click.echo(latest.name)


@cli.command()
def info() -> None:
    version_manager = ClnVersionManager()
    click.echo(f"cln Version Manager {clnvm.__version__}")
    click.echo("")
    click.echo(f"path = {version_manager._cln_path}")


def run() -> None:
    cli()


if __name__ == "__main__":
    run()
