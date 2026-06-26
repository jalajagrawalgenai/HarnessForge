"""
Forge SDK for Python — AI Agent Harness

Wrap any AI agent with 12-dimension observation, 16 detectors,
and 14 autonomous intervention strategies.

Usage:
    from forge_sdk import create_harness, quick_run
    result = quick_run("Write a function to validate emails")
    print(result.detection_count)
"""

import os

# Allow PyO3 to load on Python versions newer than what it explicitly supports
# (e.g. Python 3.14+ with PyO3 0.23.5)
os.environ.setdefault("PYO3_USE_ABI3_FORWARD_COMPATIBILITY", "1")

from .forge_sdk import (
    HarnessRunResult,
    PyHarness,
    create_harness,
    quick_run,
    list_presets,
    list_detectors,
    list_strategies,
    list_observers,
    get_version,
)

__all__ = [
    "HarnessRunResult",
    "PyHarness",
    "create_harness",
    "quick_run",
    "list_presets",
    "list_detectors",
    "list_strategies",
    "list_observers",
    "get_version",
]
