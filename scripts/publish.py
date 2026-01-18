#!/usr/bin/env python3
"""Publish a release by creating a draft GitHub release with generated notes."""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).parent.parent


def run(cmd: list[str], capture: bool = False, input_text: str | None = None) -> str:
    result = subprocess.run(
        cmd, capture_output=capture, text=True, cwd=REPO_ROOT, input=input_text
    )
    if result.returncode != 0:
        if capture and result.stderr:
            print(result.stderr, file=sys.stderr)
        sys.exit(1)
    return result.stdout.strip() if capture else ""


def parse_pep440(version: str) -> tuple[str, str | None, int | None]:
    """Parse PEP 440 version, return (base, prerelease_type, prerelease_num).

    Requires numeric suffix for prerelease markers (e.g., 1.0.0a1, not 1.0.0a).
    """
    # Try prerelease format first (requires digits after a/b/rc)
    match = re.match(r"^(\d+\.\d+\.\d+)(a|b|rc)(\d+)$", version)
    if match:
        return (match.group(1), match.group(2), int(match.group(3)))
    # Try stable format (no prerelease marker)
    match = re.match(r"^(\d+\.\d+\.\d+)$", version)
    if match:
        return (match.group(1), None, None)
    return ("", None, None)


def pep440_to_semver(version: str) -> str:
    """Convert PEP 440 version to semver."""
    base, pre_type, pre_num = parse_pep440(version)
    if not base:
        return ""
    if pre_type is None:
        return base
    type_map = {"a": "alpha", "b": "beta", "rc": "rc"}
    return f"{base}-{type_map[pre_type]}.{pre_num}"


def find_version_in_section(content: str, section: str) -> str | None:
    """Find version in a TOML section, handling arrays correctly."""
    # Find section header
    section_pattern = re.escape(f"[{section}]")
    section_match = re.search(f"^{section_pattern}$", content, re.MULTILINE)
    if not section_match:
        return None

    # Find where this section ends (next section header at start of line)
    section_start = section_match.end()
    next_section = re.search(r"^\[", content[section_start:], re.MULTILINE)
    section_end = section_start + next_section.start() if next_section else len(content)
    section_content = content[section_start:section_end]

    # Find version = "..." within this section
    version_match = re.search(r'^version = "([^"]+)"', section_content, re.MULTILINE)
    if not version_match:
        return None
    return version_match.group(1)


def check_branch():
    print("Verifying on main branch...", end=" ", flush=True)
    branch = run(["git", "branch", "--show-current"], capture=True)
    if branch != "main":
        print(f"no ({branch})")
        print("Error: Not on main branch", file=sys.stderr)
        sys.exit(1)
    print("ok")


def check_working_tree():
    print("Checking working tree...", end=" ", flush=True)
    status = run(["git", "status", "--porcelain"], capture=True)
    if status:
        print("dirty")
        print("Error: Working tree is not clean", file=sys.stderr)
        sys.exit(1)
    print("clean")


def check_origin_sync():
    print("Fetching origin...")
    run(["git", "fetch", "origin"])
    print("Verifying HEAD matches origin/main...", end=" ", flush=True)
    local_head = run(["git", "rev-parse", "HEAD"], capture=True)
    remote_head = run(["git", "rev-parse", "origin/main"], capture=True)
    if local_head != remote_head:
        print("mismatch")
        print(
            "Error: Local HEAD does not match origin/main. Run: git pull",
            file=sys.stderr,
        )
        sys.exit(1)
    print("ok")


def find_last_tag() -> str:
    print("Finding last tag...", end=" ", flush=True)
    # Find latest v* tag reachable from HEAD (ancestor-aware)
    result = subprocess.run(
        ["git", "describe", "--tags", "--abbrev=0", "--match", "v*"],
        capture_output=True,
        text=True,
        cwd=REPO_ROOT,
    )
    if result.returncode != 0 or not result.stdout.strip():
        print("none")
        print(
            "Error: No previous tag found. First release must be created manually.",
            file=sys.stderr,
        )
        sys.exit(1)
    tag = result.stdout.strip()
    print(tag)
    return tag


def verify_cargo_version(expected_semver: str):
    print("Verifying versions in Cargo.toml...", end=" ", flush=True)
    content = (REPO_ROOT / "Cargo.toml").read_text()
    actual = find_version_in_section(content, "workspace.package") or ""
    if actual != expected_semver:
        print("mismatch")
        print(
            f"Error: Version mismatch in Cargo.toml. Found {actual}, expected {expected_semver}",
            file=sys.stderr,
        )
        sys.exit(1)
    print(f"{actual} ok")


def verify_pyproject_version(expected_pep440: str):
    print("Verifying versions in pyproject.toml...", end=" ", flush=True)
    content = (REPO_ROOT / "pyproject.toml").read_text()
    actual = find_version_in_section(content, "project") or ""
    if actual != expected_pep440:
        print("mismatch")
        print(
            f"Error: Version mismatch in pyproject.toml. Found {actual}, expected {expected_pep440}",
            file=sys.stderr,
        )
        sys.exit(1)
    print(f"{actual} ok")


def verify_readme_version(expected_semver: str):
    print("Verifying versions in README.md...", end=" ", flush=True)
    content = (REPO_ROOT / "README.md").read_text()
    lines = content.split("\n")
    found_rev = None
    for i, line in enumerate(lines):
        if "repo: https://github.com/jakekaplan/loq" in line:
            # Search for rev: within the next few lines (handles comments/blank lines)
            for j in range(i + 1, min(i + 5, len(lines))):
                match = re.search(r"rev: (v[^\s]+)", lines[j])
                if match:
                    found_rev = match.group(1)
                    break
                # Stop if we hit another repo: or end of YAML block
                if "repo:" in lines[j] or lines[j].strip() == "```":
                    break
            break
    expected_tag = f"v{expected_semver}"
    if found_rev != expected_tag:
        print("mismatch")
        print(
            f"Error: Version mismatch in README.md. Found {found_rev}, expected {expected_tag}",
            file=sys.stderr,
        )
        sys.exit(1)
    print(f"{found_rev} ok")


def get_commits_since_tag(tag: str) -> str:
    commits = run(["git", "log", f"{tag}..HEAD", "--oneline"], capture=True)
    count = len(commits.split("\n")) if commits else 0
    print(f"Collecting {count} commits since {tag}...")
    return commits


def generate_release_notes(commits: str) -> str:
    print("Generating release notes...")
    prompt = (
        "Generate concise release notes for loq (a file line limit enforcer). "
        "Group by: Features, Fixes, Other. Be brief. Input is git commits:"
    )
    notes = run(["claude", "--print", prompt], capture=True, input_text=commits)
    return notes


def create_draft_release(tag: str, notes: str):
    print(f"Creating draft release {tag}...")
    with tempfile.NamedTemporaryFile(mode="w", suffix=".md", delete=False) as f:
        f.write(notes)
        notes_file = f.name

    result = run(
        [
            "gh",
            "release",
            "create",
            tag,
            "--draft",
            "--title",
            tag,
            "--notes-file",
            notes_file,
        ],
        capture=True,
    )
    print(f"Done: {result}")


def main():
    if len(sys.argv) != 2:
        print(f"Usage: {sys.argv[0]} <version>", file=sys.stderr)
        print("Example: python scripts/publish.py 0.1.0a7", file=sys.stderr)
        sys.exit(1)

    pep440_version = sys.argv[1]
    semver = pep440_to_semver(pep440_version)
    if not semver:
        print(
            "Error: Invalid version format. Expected PEP 440 (e.g., 0.1.0a7, 1.0.0, 2.0.0rc1)",
            file=sys.stderr,
        )
        sys.exit(1)

    check_branch()
    check_working_tree()
    check_origin_sync()
    last_tag = find_last_tag()

    verify_cargo_version(semver)
    verify_pyproject_version(pep440_version)
    verify_readme_version(semver)

    commits = get_commits_since_tag(last_tag)
    notes = generate_release_notes(commits)
    create_draft_release(f"v{semver}", notes)


if __name__ == "__main__":
    main()
