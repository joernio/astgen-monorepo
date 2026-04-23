"""Setup configuration for astgen-regression.

This setup.py exists for compatibility with tools that don't support pyproject.toml.
All configuration is defined in pyproject.toml - this file simply delegates to setuptools.
"""

from setuptools import setup

# All configuration is in pyproject.toml
# This setup.py exists for:
# 1. Editable installs with older pip versions (pip install -e .)
# 2. Compatibility with tools that don't support PEP 517/518
# 3. Building distributions with older setuptools versions
setup()
