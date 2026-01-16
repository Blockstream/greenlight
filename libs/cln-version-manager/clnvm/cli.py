import importlib
import logging
import logging.config

import sys

import clnvm
from clnvm.cln_version_manager import ClnVersionManager
from importlib import resources


def configure_logging() -> None:
    with resources.path(clnvm, "logging.conf") as fname:
        assert fname is not None, "logging.conf must be bundled as a resource"
        logging.config.fileConfig(fname)


# Handle the optional import and provide a nice error message if it fails
try:
    import click
    from rich.logging import RichHandler
    from rich.console import Console

    logging.basicConfig(
        level="NOTSET", format="%(message)s", datefmt="[%X]", handlers=[RichHandler(console=Console(stderr=True))]
    )
except Exception:
    print("To use clnvm the `cli` feature must be installed")
    print("You can install the feature using")
    print("> pip install cln-version-manager[cli]")
    sys.exit(1)


@click.group()
@click.option("--verbose", is_flag=True)
@click.option("--cache", type=click.Path(), help="Cache directory to store CLN versions")
@click.pass_context
def cli(ctx: click.Context, verbose: bool, cache: str) -> None:
    if verbose:
        configure_logging()
    # Store cache path in context for subcommands
    ctx.ensure_object(dict)
    ctx.obj['cache'] = cache


@cli.command()
@click.option("--force", is_flag=True)
@click.pass_context
def get_all(ctx: click.Context, force: bool) -> None:
    version_manager = ClnVersionManager(cln_path=ctx.obj.get('cache'))
    versions = version_manager.get_versions()
    logging.info(f"Fetching {len(versions)} versions")
    for version in versions:
        try:
            result = version_manager.get(version, force)
            click.echo(result.lightningd)
        except Exception as e:
            click.echo(click.style(str(e), fg="red"), err=True)
            sys.exit(1)


@cli.command()
@click.option("--tag", required=True)
@click.option("--force", is_flag=True)
@click.pass_context
def get(ctx: click.Context, tag: str, force: bool) -> None:
    try:
        version_manager = ClnVersionManager(cln_path=ctx.obj.get('cache'))
        descriptor = version_manager.get_descriptor_from_tag(tag)
        node_version = version_manager.get(descriptor, force)
        click.echo(node_version.lightningd)
    except Exception as e:
        # Print the error and return a non-zero status-code
        click.echo(click.style(str(e), fg="red"), err=True)
        sys.exit(1)


@cli.command()
@click.option("--tag", is_flag=True)
@click.option("--lightningd", is_flag=True)
@click.option("--root-path", is_flag=True)
@click.option("--bin-path", is_flag=True)
@click.pass_context
def latest(ctx: click.Context, tag: bool, lightningd: bool, root_path: bool, bin_path: bool) -> None:
    version_manager = ClnVersionManager(cln_path=ctx.obj.get('cache'))
    latest = version_manager.latest()

    if tag:
        click.echo(latest.name)
    elif lightningd:
        click.echo(latest.lightningd)
    elif bin_path:
        click.echo(latest.bin_path)
    elif root_path:
        click.echo(latest.root_path)
    else:
        click.echo(latest.name)


@cli.command()
@click.pass_context
def info(ctx: click.Context) -> None:
    version_manager = ClnVersionManager(cln_path=ctx.obj.get('cache'))
    click.echo(f"cln Version Manager {clnvm.__version__}")
    click.echo("")
    click.echo(f"cache = {version_manager._cln_path}")


def run() -> None:
    cli()


if __name__ == "__main__":
    run()
