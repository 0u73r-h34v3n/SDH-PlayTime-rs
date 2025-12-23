import sys
import shutil
from pathlib import Path

# Add the release folder to Python path to import the compiled library
project_root = Path(__file__).parent.parent.parent.parent
release_lib_path = project_root / "target" / "release"

# Python needs the .so file without the 'lib' prefix to import it
lib_source = release_lib_path / "libplaytime_rs.so"
lib_target = release_lib_path / "playtime_rs.so"

if lib_source.exists() and not lib_target.exists():
    shutil.copy2(lib_source, lib_target)
    print(f"Copied {lib_source.name} to {lib_target.name}")
elif not lib_source.exists():
    raise FileNotFoundError(
        f"Library not found at {lib_source}. "
        f"Please build it first: cargo build --release --package playtime-pyo3"
    )

sys.path.insert(0, str(release_lib_path))

import playtime_rs


def test_import():
    """Test that the library imports successfully"""
    assert playtime_rs is not None
    print("✓ Library imported successfully")


def test_basic_functionality():
    """Test basic functionality of the library"""

    assert hasattr(playtime_rs, "PlayTime")
    assert hasattr(playtime_rs, "UserManager")
    assert hasattr(playtime_rs, "clear_db_cache")

    print("✓ PlayTime class available")
    print("✓ UserManager class available")
    print("✓ clear_db_cache function available")


if __name__ == "__main__":
    print(f"Python path: {sys.path}")
    print(f"Looking for library in: {release_lib_path}")
    print(f"Library exists: {(release_lib_path / 'libplaytime_rs.so').exists()}")

    test_import()
    print("\nRunning tests...")
    test_basic_functionality()
    print("\n✓ All tests passed!")
