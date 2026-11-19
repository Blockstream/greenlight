#!/usr/bin/env python3
"""Work out what a release publishes, for .github/workflows/release.yml.

Reads every artifact's version from the repository and asks its registry
whether that version is published. A real release publishes only what is
missing. A dry run exercises every selected artifact, published or not,
so it can run at any time.

Also fails on mistakes that only the registries would catch otherwise:
wildcard dependency requirements, which crates.io rejects on upload, and
artifacts that must share a version but don't.

Runs locally too, it only needs cargo and network access:

    python3 .github/scripts/release_plan.py --dry-run

On GitHub it writes its decisions to $GITHUB_OUTPUT and a table to
$GITHUB_STEP_SUMMARY.
"""

import argparse
import json
import os
import subprocess
import sys
import tomllib
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SWIFT_REPO = "https://github.com/Blockstream/gl-sdk-swift"
USER_AGENT = "greenlight-release-plan (https://github.com/Blockstream/greenlight)"
GROUPS = ("crates", "python", "npm", "swift", "maven")


def published(url: str) -> bool:
    """True on 200, False on 404, an error for anything else."""
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    try:
        with urllib.request.urlopen(req, timeout=30):
            return True
    except urllib.error.HTTPError as e:
        if e.code == 404:
            return False
        raise SystemExit(f"error: {url} answered {e.code}, cannot tell whether it is published")


def swift_tag_exists(version: str) -> bool:
    out = subprocess.run(
        ["git", "ls-remote", "--tags", SWIFT_REPO, f"refs/tags/{version}"],
        check=True, capture_output=True, text=True,
    ).stdout
    return bool(out.strip())


def pyproject_version(path: str) -> str:
    with open(ROOT / path, "rb") as f:
        return tomllib.load(f)["project"]["version"]


def gradle_property(path: str, key: str) -> str:
    for line in (ROOT / path).read_text().splitlines():
        k, _, v = line.partition("=")
        if k.strip() == key:
            return v.strip()
    raise SystemExit(f"error: no {key} in {path}")


def publish_order(packages: dict) -> list:
    """Workspace crates ordered so each comes after its workspace dependencies."""
    deps = {
        name: {
            d["name"] for d in pkg["dependencies"]
            if d["name"] in packages and d["kind"] != "dev"
        }
        for name, pkg in packages.items()
    }
    order, done = [], set()
    while len(order) < len(packages):
        ready = sorted(n for n in packages if n not in done and deps[n] <= done)
        if not ready:
            raise SystemExit(f"error: dependency cycle among {sorted(set(packages) - done)}")
        order += ready
        done |= set(ready)
    return order


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--select", default=",".join(GROUPS),
                    help=f"comma-separated subset of {','.join(GROUPS)}")
    args = ap.parse_args()
    selected = {g for g in args.select.split(",") if g}
    if selected - set(GROUPS):
        raise SystemExit(f"error: unknown group(s) {sorted(selected - set(GROUPS))}")

    meta = json.loads(subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout)
    workspace = {p["name"]: p for p in meta["packages"]}
    crates = {n: p for n, p in workspace.items() if p["publish"] != []}

    errors = []
    rows = []  # (group, artifact, version, registry, published, action)

    def decide(group, artifact, version, registry, is_published):
        if group not in selected:
            action = "not selected"
        elif is_published:
            action = "dry run (already published)" if args.dry_run else "skip, already published"
        else:
            action = "dry run" if args.dry_run else "publish"
        rows.append((group, artifact, version, registry, is_published, action))
        return group in selected and (args.dry_run or not is_published)

    # crates.io rejects wildcard requirements only on upload.
    for name, pkg in crates.items():
        for d in pkg["dependencies"]:
            if d["req"] == "*" and d["kind"] != "dev":
                errors.append(f'{name}: dependency `{d["name"]} = "*"`, crates.io rejects wildcard requirements')

    crate_list = []
    for name in publish_order(crates):
        v = crates[name]["version"]
        if decide("crates", name, v, "crates.io",
                  published(f"https://crates.io/api/v1/crates/{name}/{v}")):
            crate_list.append(name)

    sdk = workspace["gl-sdk"]["version"]
    gl_client_py = pyproject_version("libs/gl-client-py/pyproject.toml")
    gl_sdk_py = pyproject_version("libs/gl-sdk/pyproject.toml")
    package_json = json.loads((ROOT / "libs/gl-sdk-napi/package.json").read_text())
    maven = gradle_property("libs/gl-sdk-android/gradle.properties", "libraryVersion")

    # The bindings of one SDK release share the gl-sdk version, and each
    # binding's crate carries the version of the package built from it.
    for what, version, want in [
        ("libs/gl-sdk/pyproject.toml", gl_sdk_py, sdk),
        ("libraryVersion in libs/gl-sdk-android/gradle.properties", maven, sdk),
        ("libs/gl-client-py/pyproject.toml", gl_client_py, workspace["gl-client-py"]["version"]),
        ("libs/gl-sdk-napi/package.json", package_json["version"], workspace["gl-sdk-node"]["version"]),
    ]:
        if version != want:
            errors.append(f"{what} says {version}, expected {want}")

    out = {
        "crates": " ".join(crate_list),
        "gl_client_py": decide("python", "gl-client (PyPI)", gl_client_py, "PyPI",
                               published(f"https://pypi.org/pypi/gl-client/{gl_client_py}/json")),
        "gl_sdk_py": decide("python", "gl-sdk (PyPI)", gl_sdk_py, "PyPI",
                            published(f"https://pypi.org/pypi/gl-sdk/{gl_sdk_py}/json")),
    }
    npm_name, npm_version = package_json["name"], package_json["version"]
    npm_doc = f"https://registry.npmjs.org/{npm_name.replace('/', '%2F')}/{npm_version}"
    out["npm"] = decide("npm", npm_name, npm_version, "npm", published(npm_doc))
    out["swift"] = decide("swift", "GreenlightSDK", sdk, "gl-sdk-swift tag", swift_tag_exists(sdk))
    out["maven"] = decide("maven", "com.blockstream:glsdk", maven, "Maven Central", published(
        f"https://repo1.maven.org/maven2/com/blockstream/glsdk/{maven}/glsdk-{maven}.pom"))
    out["sdk_version"] = sdk

    mode = "Dry run" if args.dry_run else "Release"
    summary = [f"## {mode} plan", "", "| Group | Artifact | Version | Registry | Published | Action |",
               "| --- | --- | --- | --- | --- | --- |"]
    summary += [f"| {g} | `{a}` | {v} | {r} | {'yes' if p else 'no'} | {act} |"
                for g, a, v, r, p, act in rows]
    if errors:
        summary += ["", "### Errors", ""] + [f"- {e}" for e in errors]
    text = "\n".join(summary) + "\n"
    print(text)
    if path := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(path, "a") as f:
            f.write(text)
    if path := os.environ.get("GITHUB_OUTPUT"):
        with open(path, "a") as f:
            for k, v in out.items():
                f.write(f"{k}={str(v).lower() if isinstance(v, bool) else v}\n")
    else:
        print(json.dumps(out, indent=2))

    for e in errors:
        print(f"::error::{e}" if os.environ.get("GITHUB_ACTIONS") else f"error: {e}", file=sys.stderr)
    if errors:
        sys.exit(1)


if __name__ == "__main__":
    main()
