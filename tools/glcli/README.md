# glcli: Greenlight Command Line Interface

A simple tool to issue commands to a Greenlight node and interact with
the scheduler.

## Help

```bash
$glcli scheduler --help
```


```bash
$ glcli --help
```

## Installing

Installing the `glcli` utility can be done with the following command:

```bash
pip install --extra-index-url=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/simple/ glcli
```

In most cases we have prebuilt the binary extension for `gl-client-py`
(which internall depends on `libhsmd`, another binary extension). See
the [`gl-client-py` documentation][glpy-doc] for details on prebuilt
binaries and how to compile on platforms that don't have a prebuilt
version yet.

[glpy-doc]: ../rust/gl-client-py/



