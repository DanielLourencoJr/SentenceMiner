#!/usr/bin/env python3
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import os

ROOT = Path(__file__).resolve().parent / "ui"

class NoCacheHandler(SimpleHTTPRequestHandler):
    def translate_path(self, path):
        # Serve from ui/ directory
        path = path.split('?', 1)[0].split('#', 1)[0]
        rel = path.lstrip('/')
        full = ROOT / rel
        return str(full)

    def end_headers(self):
        # Disable cache for HTML/JS/CSS to avoid stale UI in Tauri webview
        if self.path.endswith((".html", ".js", ".css")) or self.path.endswith("/"):
            self.send_header("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
            self.send_header("Pragma", "no-cache")
        super().end_headers()

if __name__ == "__main__":
    os.chdir(str(ROOT))
    server = ThreadingHTTPServer(("0.0.0.0", 1420), NoCacheHandler)
    print("Serving UI at http://localhost:1420 (no-cache for HTML)")
    server.serve_forever()
