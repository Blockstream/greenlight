from pyln.testing.utils import NodeFactory
import pytest
from pyln.testing.fixtures import (
    bitcoind,  # noqa: F401
    node_cls,  # noqa: F401
    test_name,  # noqa: F401
    executor,  # noqa: F401
    db_provider,  # noqa: F401
    jsonschemas,  # noqa: F401
)


class GlNodeFactory(NodeFactory):
    """A temporary shim until pyln-testing learns to run multiple versions

    This adds the v24.02 `--developer` option, which was not required before.

    TODO Remove this shim onces pyln-testing learns about versions
    PR: https://github.com/ElementsProject/lightning/pull/7173
    """

    def get_node(self, options=None, *args, **kwargs):
        # Until pyln-testing learns to differentiate versions we need
        # to do this whenever we start a new node. pyln-testing v24.05
        # promises to be multi-version compatible.
        if options is None:
            options = {}
        options["allow-deprecated-apis"] = True
        options["developer"] = None
        # Disable cln-grpc plugin to avoid port conflicts with GL nodes
        options["disable-plugin"] = "cln-grpc"
        return NodeFactory.get_node(self, options=options, *args, **kwargs)


@pytest.fixture
def node_factory(
    request,  # noqa: F811
    directory,
    test_name,
    bitcoind,
    executor,
    db_provider,
    node_cls,
    jsonschemas,
):
    nf = GlNodeFactory(
        request,
        test_name,
        bitcoind,
        executor,
        directory=directory,
        db_provider=db_provider,
        node_cls=node_cls,
        jsonschemas=jsonschemas,
    )

    yield nf
    nf.killall([not n.may_fail for n in nf.nodes])
