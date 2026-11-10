#!/usr/bin/env python3
"""Record the on-demand example's real token counting, compaction, and generation calls.

Build --bin on-demand first. The example loads .env; authentication is forwarded only in
memory by record_live. Captures contain request/response bodies, never credentials.
"""
import argparse
import datetime
import json
import pathlib
import subprocess
import threading
from http.server import ThreadingHTTPServer

from record_live import Recorder, ROOT, DESTINATION


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("case", choices=["claude", "openai", "custom", "claude-stream"])
    parser.add_argument("--destination", type=pathlib.Path, default=DESTINATION / "results")
    args = parser.parse_args()
    args.destination.mkdir(parents=True, exist_ok=True)
    mode = "claude" if args.case == "claude-stream" else args.case
    provider = "openai" if mode == "openai" else "claude"
    server = ThreadingHTTPServer(("127.0.0.1", 0), Recorder)
    server.exchanges = []
    threading.Thread(target=server.serve_forever, daemon=True).start()
    report = args.destination / f"on-demand-{args.case}-report.json"
    report.unlink(missing_ok=True)
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    try:
        command = [str(DESTINATION / "target/debug/on-demand"), mode,
                   "--api-base", f"http://127.0.0.1:{server.server_port}/{provider}/v1",
                   "--report", str(report)]
        if args.case == "claude-stream":
            command.append("--stream")
        result = subprocess.run(command, cwd=ROOT, timeout=300)
        return_code = result.returncode
    finally:
        server.shutdown()
        server.server_close()
        capture = args.destination / f"on-demand-{args.case}-capture.json"
        capture.write_text(json.dumps({
            "captured_at": started,
            "case": args.case,
            "report": json.loads(report.read_text()) if report.exists() else None,
            "exchanges": server.exchanges,
        }, indent=2) + "\n")
        print(f"Saved {len(server.exchanges)} API exchanges to {capture}", flush=True)
    raise SystemExit(return_code)


if __name__ == "__main__":
    main()
