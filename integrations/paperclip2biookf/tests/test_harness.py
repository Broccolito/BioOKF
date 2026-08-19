import json
import os
import tempfile
import unittest
from pathlib import Path
from subprocess import CompletedProcess
from unittest.mock import patch

from paperclip_biookf.agents import _claude_schema, _subscription_env, model_catalog
from paperclip_biookf.biookf import BioOKFBuilder, validate_extraction
from paperclip_biookf.constants import EXTRACTION_SCHEMA
from paperclip_biookf.paperclip import PaperclipError, document_storage_id, parse_map_export
from paperclip_biookf.pipeline import HarnessPipeline
from paperclip_biookf.server import (
    _generated_bundles, _is_json_content_type, _is_loopback_host,
    _is_trusted_host_header,
    _resolve_bundle, _run_history, _validate_request,
)


FIXTURES = Path(__file__).parent / "fixtures"


class ExtractionTests(unittest.TestCase):
    def test_subscription_environment_strips_cloud_credentials(self):
        with patch.dict(os.environ, {
            "AWS_ACCESS_KEY_ID": "secret",
            "GOOGLE_APPLICATION_CREDENTIALS": "/tmp/key.json",
            "AZURE_CLIENT_SECRET": "secret",
            "BIOOKF_SAFE_SENTINEL": "preserved",
        }):
            env = _subscription_env()
        self.assertNotIn("AWS_ACCESS_KEY_ID", env)
        self.assertNotIn("GOOGLE_APPLICATION_CREDENTIALS", env)
        self.assertNotIn("AZURE_CLIENT_SECRET", env)
        self.assertEqual(env["BIOOKF_SAFE_SENTINEL"], "preserved")

    def test_document_storage_id_rejects_traversal_and_avoids_collisions(self):
        with self.assertRaises(PaperclipError):
            document_storage_id("../../escape")
        self.assertNotEqual(document_storage_id("study:1"), document_storage_id("study-1"))

    def test_model_catalog_exposes_provider_specific_choices(self):
        catalog = model_catalog()
        self.assertIn("codex", catalog)
        self.assertEqual([item["id"] for item in catalog["claude"]], ["sonnet", "opus", "fable"])

    def test_claude_schema_omits_only_remote_metaschema(self):
        schema = _claude_schema()
        self.assertNotIn("$schema", schema)
        self.assertIn("$schema", EXTRACTION_SCHEMA)
        self.assertEqual(schema["properties"], EXTRACTION_SCHEMA["properties"])

    def test_valid_fixture(self):
        records = json.loads((FIXTURES / "extractions.json").read_text())
        self.assertEqual(validate_extraction(records[0]["extraction"]), [])

    def test_unresolved_edge_is_rejected(self):
        records = json.loads((FIXTURES / "extractions.json").read_text())
        records[0]["extraction"]["edges"][0]["object"] = "Missing disease"
        errors = validate_extraction(records[0]["extraction"])
        self.assertTrue(any("object does not resolve" in error for error in errors))

    def test_map_export_parser_kept_for_import_compatibility(self):
        text = """Map results [m_x]\n\n--- [1] [success] Demo ---\ndoc_id: PMC1\n{\"nodes\":[],\"edges\":[]}\n"""
        records = parse_map_export(text)
        self.assertEqual(records[0]["document_id"], "PMC1")


class BundleTests(unittest.TestCase):
    def test_identifier_aliases_are_canonicalized_case_insensitively(self):
        records = [
            {"document_id": "one", "extraction": {"nodes": [
                {"identifier": "Multiple sclerosis", "type": "Disease"},
            ]}},
            {"document_id": "two", "extraction": {"nodes": [
                {"identifier": "multiple sclerosis", "type": "Disease"},
            ]}},
        ]
        aliases = BioOKFBuilder._resolve_identifier_collisions(records)
        self.assertEqual(aliases["one"][("Multiple sclerosis", "Disease")], "Multiple sclerosis")
        self.assertEqual(aliases["two"][("multiple sclerosis", "Disease")], "Multiple sclerosis")

    def test_colliding_identifier_slugs_get_distinct_node_paths(self):
        paths = BioOKFBuilder._node_output_paths({
            "IL-6": {"type": "Molecule"},
            "IL 6": {"type": "Molecule"},
        })
        self.assertNotEqual(paths["IL-6"], paths["IL 6"])
        self.assertEqual(paths["IL-6"].parent, Path("knowledge/molecule"))

    def test_opening_an_already_registered_bundle_is_idempotent(self):
        with tempfile.TemporaryDirectory() as temp:
            bundle = Path(temp) / "kb"
            bundle.mkdir()
            def fake_run(argv, **kwargs):
                if argv[-1] == "--list":
                    return CompletedProcess(argv, 0, stdout=f"demo  {bundle}\n", stderr="")
                if len(argv) > 2 and argv[1] == "register":
                    return CompletedProcess(argv, 1, stdout="", stderr="already registered")
                return CompletedProcess(argv, 0, stdout="", stderr="")
            with patch("paperclip_biookf.biookf.subprocess.run", side_effect=fake_run):
                result = BioOKFBuilder(bundle, "Demo", bokf_binary="/usr/local/bin/bokf").register_for_studio("demo")
            self.assertTrue(result["registered"])

    def test_materializes_studio_bundle_with_provenance(self):
        search = json.loads((FIXTURES / "search.json").read_text())
        records = json.loads((FIXTURES / "extractions.json").read_text())
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            run_dir = root / "runs" / "fixture"
            source = run_dir / "sources" / document_storage_id("PMC123456")
            source.mkdir(parents=True)
            (source / "source.md").write_text("# Demo\n\nL88 claim\nL120-L126 outcome\n")
            (source / "content.lines").write_text("L88 claim\nL120-L126 outcome\n")
            (source / "original.meta.json").write_text("{}\n")
            (source / "meta.yaml").write_text("source_type: paperclip_vfs_snapshot\n")
            bundle = root / "kb"
            manifest = BioOKFBuilder(bundle, "Fixture KB", bokf_binary="").build(run_dir, search, records)
            self.assertEqual(manifest["source_count"], 1)
            self.assertTrue((bundle / "SCHEMA.md").is_file())
            self.assertTrue((bundle / "knowledge" / "molecule" / "vemurafenib.md").is_file())
            concept = (bundle / "knowledge" / "molecule" / "vemurafenib.md").read_text()
            self.assertIn("primary_source", concept)
            self.assertIn("evidence_url", concept)
            self.assertIn("publication_year", (bundle / "knowledge" / "publication" / "braf-inhibitor-resistance-in-melanoma.md").read_text())
            self.assertTrue(manifest["verification"]["internal"]["ok"])


class RequestTests(unittest.TestCase):
    def test_local_ui_rejects_remote_bindings_and_non_json_posts(self):
        self.assertTrue(_is_loopback_host("127.0.0.1"))
        self.assertTrue(_is_loopback_host("::1"))
        self.assertTrue(_is_loopback_host("localhost"))
        self.assertFalse(_is_loopback_host("0.0.0.0"))
        self.assertFalse(_is_loopback_host("example.com"))
        self.assertTrue(_is_json_content_type("application/json; charset=utf-8"))
        self.assertFalse(_is_json_content_type("text/plain"))
        self.assertTrue(_is_trusted_host_header("127.0.0.1:8765", "127.0.0.1", 8765))
        self.assertTrue(_is_trusted_host_header("[::1]:8765", "::1", 8765))
        self.assertFalse(_is_trusted_host_header("attacker.example:8765", "127.0.0.1", 8765))

    def test_discovery_rejects_hostile_document_ids_before_writing(self):
        with tempfile.TemporaryDirectory() as temp:
            escape = Path(temp) / "escape"
            class FakePaperclip:
                def search(self, *args, **kwargs):
                    return {"papers": [{"document_id": str(escape), "source": "pmc"}]}

            pipeline = HarnessPipeline(Path(temp))
            pipeline.paperclip = FakePaperclip()
            with self.assertRaises(PaperclipError):
                pipeline.discover("query", ["pmc"], 1)
            self.assertFalse(escape.exists())

    def test_history_and_bundle_views_are_backed_by_workspace_files(self):
        with tempfile.TemporaryDirectory() as temp:
            workspace = Path(temp)
            run = workspace / "runs" / "20260101-demo"
            run.mkdir(parents=True)
            (run / "search.json").write_text(json.dumps({"query": "demo", "sources": ["pmc"], "count": 1}))
            (run / "extractions.json").write_text(json.dumps([{"status": "success"}]))
            bundle = workspace / "knowledge-bases" / "demo-kb"
            operations = bundle / "operations"
            operations.mkdir(parents=True)
            (bundle / "SCHEMA.md").write_text("# schema\n")
            manifest = {"query": "demo", "source_count": 1, "node_count": 3, "edge_count": 2, "verification": {"internal": {"ok": True}}}
            (operations / "latest-run.json").write_text(json.dumps(manifest))
            (operations / "20260101-demo.json").write_text(json.dumps(manifest))
            self.assertEqual(_run_history(workspace)[0]["status"], "complete")
            self.assertTrue(_generated_bundles(workspace)[0]["verified"])
            self.assertEqual(_resolve_bundle(workspace, str(bundle)), bundle.resolve())

    def test_gui_request_supports_multiple_databases_and_time(self):
        value = _validate_request({
            "query": "EGFR resistance", "sources": ["pmc", "trials/us"],
            "limit": 3, "year_min": 2018, "year_max": 2024,
            "kb_name": "EGFR KB", "agent": "claude",
        }, build=True)
        self.assertEqual(value["sources"], ["pmc", "trials/us"])
        self.assertEqual(value["year_min"], 2018)


if __name__ == "__main__":
    unittest.main()
