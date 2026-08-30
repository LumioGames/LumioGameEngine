#!/usr/bin/env python3
"""Focused regression tests for the GAS contract gate."""

from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from decimal import Decimal
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import lumio_contract  # noqa: E402


class GasContractRegressionTests(unittest.TestCase):
    def test_replication_non_string_component_and_field_are_rejected_without_traceback(self) -> None:
        base = lumio_contract.load_json(ROOT / "fixtures" / "valid" / "gas-replication-hashes.json")
        malformed_values = ([], {}, 1, True, None)
        for member in ("component", "field"):
            for malformed in malformed_values:
                record = copy.deepcopy(base)
                record["fields"][0][member] = malformed
                errors = lumio_contract._gas_replication_errors(record)
                self.assertTrue(errors)
                self.assertTrue(any(error.startswith("GAS.REPLICATION.FIELD_TYPE:") for error in errors))

    def test_prediction_non_list_steps_are_rejected_without_traceback(self) -> None:
        base = lumio_contract.load_json(ROOT / "fixtures" / "valid" / "gas-prediction-rollback.json")
        for malformed in (True, 1, {}):
            record = copy.deepcopy(base)
            record["rollback"]["steps"] = malformed
            errors = lumio_contract._gas_prediction_errors(record)
            self.assertTrue(errors)
            self.assertTrue(any(error.startswith("GAS.PREDICTION.ROLLBACK_STEPS:") for error in errors))

    def test_prediction_step_shape_fixtures_are_registered_with_exact_rule(self) -> None:
        fixture_ids = (
            "gas/prediction-rollback-steps-null",
            "gas/prediction-rollback-steps-string",
            "gas/prediction-rollback-steps-array",
            "gas/prediction-rollback-steps-missing",
        )
        for fixture_id in fixture_ids:
            with self.subTest(fixture_id=fixture_id):
                result = subprocess.run(
                    [
                        sys.executable,
                        str(ROOT / "tools" / "lumio_contract.py"),
                        "validate",
                        "--fixture",
                        fixture_id,
                        "--json",
                    ],
                    cwd=ROOT,
                    text=True,
                    capture_output=True,
                )
                self.assertEqual(result.returncode, 0, msg=result.stderr)
                payload = json.loads(result.stdout)
                self.assertTrue(payload["passed"])
                self.assertTrue(
                    any(
                        error.startswith("GAS.PREDICTION.ROLLBACK_STEPS:")
                        for error in payload["fixtureResults"][0]["errors"]
                    )
                )

    def test_empty_replay_is_rejected_by_the_published_cardinality_policy(self) -> None:
        record = lumio_contract.load_json(ROOT / "fixtures" / "valid" / "gas-prediction-rollback.json")
        self.assertGreaterEqual(len(record["rollback"]["replayInputFrames"]), 1)
        record["rollback"]["replayInputFrames"] = []
        errors = lumio_contract._gas_prediction_errors(record)
        self.assertTrue(any(error.startswith("GAS.PREDICTION.REPLAY_CARDINALITY:") for error in errors))

    def test_canonical_preserves_decimal_numbers_and_existing_integer_form(self) -> None:
        self.assertEqual(lumio_contract.canonical_json({"b": 1, "a": "x"}), '{"a":"x","b":1}')
        collision = {"marker": "\x00lumio-decimal-0", "number": Decimal("1.2500")}
        collision_text = lumio_contract.canonical_json(collision)
        self.assertEqual(json.loads(collision_text, parse_float=Decimal), collision)
        paths = [
            ROOT / "fixtures" / "valid" / "gas-evaluation-decimal-rounding.json",
            ROOT / "fixtures" / "valid" / "gas-evaluation-subnormal-exponent.json",
            ROOT / "fixtures" / "valid" / "gas-evaluation-large-exponent.json",
            ROOT / "fixtures" / "valid" / "gas-evaluation-fractional-result.json",
        ]
        for path in paths:
            value = lumio_contract.load_json(path)
            text = lumio_contract.canonical_json(value)
            self.assertNotIn('"base":"', text)
            round_trip = json.loads(text, parse_float=Decimal)
            self.assertEqual(round_trip, value)

    def test_canonical_cli_accepts_every_positive_gas_evaluation_fixture(self) -> None:
        for path in sorted((ROOT / "fixtures" / "valid").glob("gas-evaluation-*.json")):
            result = subprocess.run(
                [sys.executable, str(ROOT / "tools" / "lumio_contract.py"), "canonical", str(path)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
            self.assertEqual(result.returncode, 0, msg=f"{path.name}: {result.stderr}")
            self.assertNotIn("Traceback", result.stderr)
            self.assertEqual(json.loads(result.stdout, parse_float=Decimal), lumio_contract.load_json(path))

    def test_decimal_bounds_use_adjusted_exponent_for_trailing_zero_and_zero_values(self) -> None:
        accepted = (
            Decimal("1.0e-6176"),
            Decimal("1.00e-6176"),
            Decimal("9.999999999999999999999999999999999e-6176"),
            Decimal("0e-6176"),
            Decimal("-0.00e-6174"),
            Decimal("0e6144"),
            Decimal("1.0e6144"),
            Decimal("9.99e6144"),
        )
        rejected = (
            Decimal("1e-6177"),
            Decimal("1.0e-6177"),
            Decimal("0e-6177"),
            Decimal("0e6145"),
            Decimal("1e6145"),
        )
        for value in accepted:
            with self.subTest(value=value):
                self.assertIsNotNone(lumio_contract._gas_decimal(value))
        for value in rejected:
            with self.subTest(value=value):
                self.assertIsNone(lumio_contract._gas_decimal(value))

    def test_canonical_cli_preserves_decimal_lexemes_for_out_of_tree_files(self) -> None:
        source = ROOT / "fixtures" / "valid" / "gas-evaluation-decimal-rounding.json"
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "renamed-evaluation.json"
            target.write_bytes(source.read_bytes())
            result = subprocess.run(
                [sys.executable, str(ROOT / "tools" / "lumio_contract.py"), "canonical", str(target)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
        self.assertEqual(result.returncode, 0, msg=result.stderr)
        canonical = json.loads(result.stdout, parse_float=Decimal)
        self.assertEqual(canonical["base"], Decimal("0.12345678901234567890123456789012345"))
        self.assertEqual(canonical["result"], Decimal("0.1234567890123456789012345678901234"))

    def test_canonical_cli_rejects_nonfinite_constants_without_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "nonfinite.json"
            target.write_text('{"a":NaN,"b":Infinity}\n', encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(ROOT / "tools" / "lumio_contract.py"), "canonical", str(target)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("error:", result.stderr)

    def test_canonical_cli_rejects_duplicate_members_without_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / "duplicate.json"
            target.write_text('{"a":1,"a":2}\n', encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(ROOT / "tools" / "lumio_contract.py"), "canonical", str(target)],
                cwd=ROOT,
                text=True,
                capture_output=True,
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("duplicate JSON member", result.stderr)


if __name__ == "__main__":
    unittest.main()
