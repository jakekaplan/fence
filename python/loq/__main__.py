"""Binary wrapper for loq.

Finds and executes the loq binary, handling various installation scenarios.
Based on the pattern used by ruff and uv.
"""

from __future__ import annotations

import os
import sys
import sysconfig


def find_loq_bin() -> str:
    """Find the loq binary."""
    loq_exe = "loq.exe" if sys.platform == "win32" else "loq"

    # First, check the scripts directory (standard pip install)
    scripts_dir = sysconfig.get_path("scripts")
    if scripts_dir:
        candidate = os.path.join(scripts_dir, loq_exe)
        if os.path.isfile(candidate):
            return candidate

    # Check user scripts directory
    user_scheme = "nt_user" if os.name == "nt" else "posix_user"
    user_scripts = sysconfig.get_path("scripts", scheme=user_scheme)
    if user_scripts:
        candidate = os.path.join(user_scripts, loq_exe)
        if os.path.isfile(candidate):
            return candidate

    # Check sibling bin directory (pip install --target)
    package_dir = os.path.dirname(__file__)
    candidate = os.path.join(package_dir, "..", "..", "bin", loq_exe)
    if os.path.isfile(candidate):
        return os.path.abspath(candidate)

    # Check pip build environment overlay
    if sys.prefix != sys.base_prefix:
        scripts = "Scripts" if os.name == "nt" else "bin"
        candidate = os.path.join(sys.prefix, scripts, loq_exe)
        if os.path.isfile(candidate):
            return candidate

    raise FileNotFoundError(f"Could not find {loq_exe}")


def main() -> None:
    """Run the loq CLI."""
    loq = find_loq_bin()

    if sys.platform == "win32":
        import subprocess

        result = subprocess.run([loq, *sys.argv[1:]])
        sys.exit(result.returncode)
    else:
        os.execvp(loq, [loq, *sys.argv[1:]])


if __name__ == "__main__":
    main()
