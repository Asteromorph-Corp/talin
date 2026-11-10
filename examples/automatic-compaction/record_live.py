#!/usr/bin/env python3
"""Run the real example binaries and record HTTP bodies before the Rust SDK decodes them.

Build the automatic-compaction application first. Run this script from the repository root. The examples load .env;
this recorder forwards authentication headers in memory and never saves them. API calls cost money.
"""

import datetime
import argparse
import json
import pathlib
import subprocess
import threading
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


ROOT = pathlib.Path(__file__).resolve().parents[2]
DESTINATION = ROOT / "examples/automatic-compaction"


class Recorder(BaseHTTPRequestHandler):
    def log_message(self, *_):
        pass

    def do_POST(self):
        provider, path = self.path.lstrip("/").split("/", 1)
        origins = {"openai": "https://api.openai.com", "claude": "https://api.anthropic.com"}
        if provider not in origins or path not in ("v1/responses", "v1/messages", "v1/responses/compact", "v1/responses/input_tokens", "v1/messages/count_tokens"):
            self.send_error(404)
            return
        body = self.rfile.read(int(self.headers["Content-Length"]))
        # Only these headers go upstream; none are written to the capture.
        allowed = {"authorization", "x-api-key", "anthropic-version", "anthropic-beta", "content-type"}
        headers = {k: v for k, v in self.headers.items() if k.lower() in allowed}
        request = urllib.request.Request(origins[provider] + "/" + path, data=body, headers=headers)
        try:
            response = urllib.request.urlopen(request, timeout=180)
        except urllib.error.HTTPError as error:
            response = error
        with response:
            raw = response.read()
            content_type = response.headers.get("content-type", "application/json")
            self.server.exchanges.append({
                "path": "/" + path,
                "request": json.loads(body),
                "status": response.status,
                "content_type": content_type,
                "request_id": response.headers.get("request-id") or response.headers.get("x-request-id"),
                "body": raw.decode(),
            })
            self.send_response(response.status)
            self.send_header("content-type", content_type)
            self.send_header("content-length", str(len(raw)))
            self.end_headers()
            self.wfile.write(raw)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--case", choices=["openai", "claude", "claude-stream"])
    args = parser.parse_args()
    server = ThreadingHTTPServer(("127.0.0.1", 0), Recorder)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_port}"
    cases = [
        ("openai", "openai", []),
        ("claude", "claude", []),
        ("claude-stream", "claude", ["--stream"]),
    ]
    try:
        for case, provider, extra in cases:
            if args.case and case != args.case:
                continue
            server.exchanges = []
            report = DESTINATION / "results" / f"typed-{case}.json"
            started = datetime.datetime.now(datetime.timezone.utc).isoformat()
            result = subprocess.run([
                str(DESTINATION / "target/debug/automatic-compaction"), provider,
                "--api-base", f"{base}/{provider}/v1", "--report", str(report), *extra,
            ], cwd=ROOT, timeout=240)
            fixture = ROOT / "crates/talin/tests/fixtures/auto-compaction" / f"typed-{case}.json"
            fixture.parent.mkdir(parents=True, exist_ok=True)
            fixture.write_text(json.dumps({
                "source": origins_for(provider),
                "captured_at": started,
                "workload": {k: v for k, v in json.loads(report.read_text()).items()
                             if k in ("input_facts", "archive_rows", "threshold")},
                "exchanges": server.exchanges,
            }, indent=2) + "\n")
            if result.returncode:
                raise SystemExit(f"{case} failed; the raw exchange and failed report were saved")
            print(f"Recorded {case}: {len(server.exchanges)} real API exchanges", flush=True)
    finally:
        server.shutdown()
        server.server_close()


def origins_for(provider):
    return "https://api.openai.com/v1/responses" if provider == "openai" else "https://api.anthropic.com/v1/messages"


if __name__ == "__main__":
    main()
