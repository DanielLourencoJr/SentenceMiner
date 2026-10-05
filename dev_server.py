#!/usr/bin/env python3
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Event, Thread
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
    parent_pid = os.getppid()
    os.chdir(str(ROOT))
    server = ThreadingHTTPServer(("0.0.0.0", 1420), NoCacheHandler)

    def watch_parent(stop=Event()):
        # Se o processo pai (ex.: `cargo tauri dev`) morrer de forma abrupta
        # (SIGKILL, crash), o SO nos re-adota e o ppid muda: desligamos em vez
        # de virar um orfao segurando a porta 1420.
        while not stop.wait(2.0):
            if os.getppid() != parent_pid:
                server.shutdown()
                return

    Thread(target=watch_parent, daemon=True).start()
    print("Serving UI at http://localhost:1420 (no-cache for HTML)")
    server.serve_forever()
