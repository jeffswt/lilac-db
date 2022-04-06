import argparse
import dataclasses
import enum
import sys
import re
from typing import List, Optional


class CommitType(enum.Enum):
    """Types of conventional commits."""

    feat = "feat"
    fix = "fix"
    docs = "docs"
    style = "style"
    refactor = "refactor"
    perf = "perf"
    test = "test"
    build = "build"
    ci = "ci"
    chore = "chore"
    merge = "merge"
    revert = "revert"


class CommitScope(enum.Enum):
    """Project scopes accepted by Keqing."""

    any = "*"
    keqing = "keqing"  # Version Control Utility
    kleestor = "kleestor"  # Storage engine
    kleestor_memtable = "memtable"  # Memory-only part of storage engine
    kleestor_bloomfilter = "kbloomf"  # Bloom filter for storage engine


@dataclasses.dataclass
class CommitHeader:
    """Conventional commit header description."""

    typ: CommitType
    scope: CommitScope
    # Whether this is a breaking change.
    breaking: bool
    description: str


@dataclasses.dataclass
class Commit:
    """Conventional commit."""

    header: CommitHeader
    body: List[str]


def parse_commit(commit: str) -> Optional[Commit]:
    """Parse conventional commit. Returns None on fail."""
    header, *_body = commit.split("\n")
    header = parse_header(header)
    if not header:
        return None
    # validate body
    body: List[str] = []
    for line in _body:
        # no trailing whitespaces
        if line.rstrip() != line:
            return None
        # must not exceed 80 col limit
        if len(line) > 80:
            return None
        # contain only ascii and no tabs
        if "".join(filter(lambda ch: ord(ch) < 256 and ch != "\t", line)) != line:
            return None
        body.append(line)
    # no double-whitelines or trailing whitelines
    _body = "\n".join(body)
    if "\n\n\n" in _body:
        return None
    if _body.endswith("\n"):
        return None
    # collect
    return Commit(header=header, body=body)


def parse_header(header: str) -> Optional[CommitHeader]:
    """Parses conventional commits header. Returns None on fail."""
    matches = re.findall(r"^([a-z]+)\((.*?)\)([!]?): (.*?)$", header)
    if not matches:
        return None
    typ, scope, breaking, description = matches[0]
    # validate components
    try:
        typ = CommitType(typ)
        scope = CommitScope(scope)
    except ValueError:
        return None
    breaking = breaking == "!"
    if rectify_string(description) != description:
        return None
    # collect
    return CommitHeader(
        typ=typ, scope=scope, breaking=breaking, description=description
    )


def rectify_string(s: str) -> str:
    """Format string to a sleek style."""
    # must only contain ascii characters
    s = "".join(filter(lambda ch: ord(ch) < 256, s))
    # must not begin with capital case letters
    s = s[0].lower() + s[1:]
    # may not have preleading or trailing whitespaces
    s = s.strip()
    # never contain multiple whitespaces in a row
    s = re.sub(r"[ \t\r\n]+", r" ", s)
    # must not end with a punctuation
    s = re.sub(r"[^a-zA-Z0-9]+$", r"", s)
    return s


def main() -> None:
    """Console worker."""
    parser = argparse.ArgumentParser(description="Conventional Commit Linter")
    parser.add_argument("--lint", type=str, dest="lint", action="store", required=True)
    parser.add_argument("--requires-type", type=str, dest="requires_type", action="store")
    args = parser.parse_args()

    # lint commit
    msg = args.lint
    commit = parse_commit(msg)
    if not commit:
        print(f"{repr(msg)} is not a conventional commit.")
        exit(1)
    # type requirements
    if args.requires_type:
        the_type = args.requires_type
        if commit.header.typ != CommitType(the_type):
            print(f"{repr(msg)} must have type '{the_type}'.")
            exit(1)
    # all ok
    return


if __name__ == "__main__":
    main()
