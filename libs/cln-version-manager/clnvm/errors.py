class UnrunnableVersion(Exception):

    def __init__(self, tag: str):
        self.tag = tag

    def __str__(self) -> str:
        return f"Failed to run Core Lightning {self.tag}"

    def __repr__(self) -> str:
        return f"UnrunnableVersion(tag={self.tag})"


class UnknownVersion(Exception):

    def __init__(self, tag: str):
        self.tag = tag

    def __str__(self) -> str:
        return f"Unknown version {self.tag}"

    def __repr__(self) -> str:
        return f"UnknownVersoin(tag={self.tag})"


class VersionMismatch(Exception):

    def __init__(self, expected: str, actual: str):
        self.expected = expected
        self.actual = actual

    def __str__(self) -> str:
        return f"Unexpected version of `lightningd`. Downloaded {self.actual} but expected {self.expected}"

    def __repr__(self) -> str:
        return f"VersionMismatch(expected={self.expected}, actual={self.actual})"


class HashMismatch(Exception):

    def __init__(self, tag: str, expected: str, actual: str):
        self.tag = tag
        self.actual = actual
        self.expected = expected

    def __str__(self) -> str:
        return self.__repr__()

    def __repr__(self) -> str:
        return f"HashMismatch(tag={self.tag}, actual={self.actual}, expected={self.expected})"


class SignatureVerificationFailed(Exception):

    def __init__(self, tag: str, reason: str):
        self.tag = tag
        self.reason = reason

    def __str__(self) -> str:
        return self.__repr__()

    def __repr__(self) -> str:
        return f"SignatureVerificationFailed(tag={self.tag}, reason={self.reason})"
