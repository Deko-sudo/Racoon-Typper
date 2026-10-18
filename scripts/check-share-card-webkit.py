#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
"""Run the built PNG fixture in ephemeral system WebKitGTK, without user data.

Requires existing system PyGObject, GTK3 and WebKit2 4.1; installs nothing.
Usage: /usr/bin/python3 scripts/check-share-card-webkit.py BUILD_DIR OUTPUT_DIR
"""
import base64
import functools
import http.server
import json
from pathlib import Path
import sys
import threading

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
from gi.repository import GLib, Gtk, WebKit2

build, output = map(lambda arg: Path(arg).resolve(), sys.argv[1:3])
output.mkdir(parents=True, exist_ok=True)
page = next(build.rglob('share-card.html'))
requests = []


class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, fmt, *args):
        requests.append(fmt % args)


server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Handler, directory=str(build)))
threading.Thread(target=server.serve_forever, daemon=True).start()
origin = f'http://127.0.0.1:{server.server_port}'
context = WebKit2.WebContext.new_with_website_data_manager(WebKit2.WebsiteDataManager.new_ephemeral())
view = WebKit2.WebView.new_with_context(context)
# Offscreen GTK windows have no native GL surface; Canvas 2D uses software here.
view.get_settings().set_hardware_acceleration_policy(WebKit2.HardwareAccelerationPolicy.NEVER)
window = Gtk.OffscreenWindow()
window.set_default_size(1000, 700)
window.add(view)
window.show_all()
result = {'ok': False, 'error': 'Timed out'}


def evaluated(webview, task, _data=None):
    global result
    try:
        state = json.loads(webview.evaluate_javascript_finish(task).to_string())
        if state.get('status') not in ('PASS', 'FAIL'):
            return
        result = {'ok': state['status'] == 'PASS', 'webkit': [WebKit2.get_major_version(), WebKit2.get_minor_version(), WebKit2.get_micro_version()], 'details': state['details']}
        if result['ok']:
            details = json.loads(state['details'])
            assert all(url.startswith(origin + '/') for url in details['resources']), 'Non-local resource'
            (output / 'webkit-card.png').write_bytes(base64.b64decode(state['png']))
        Gtk.main_quit()
    except Exception as error:
        result = {'ok': False, 'error': str(error)}
        Gtk.main_quit()


def poll():
    view.evaluate_javascript("JSON.stringify({status:document.querySelector('#status')?.textContent,details:document.querySelector('#results')?.textContent,png:document.querySelector('#png-base64')?.textContent})", -1, None, None, None, evaluated, None)
    return True


def timeout():
    Gtk.main_quit()
    return False


view.load_uri(origin + '/' + page.relative_to(build).as_posix())
GLib.timeout_add(250, poll)
GLib.timeout_add_seconds(30, timeout)
try:
    Gtk.main()
finally:
    window.destroy()
    server.shutdown()
    result['requests'] = requests
    (output / 'webkit-result.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result, indent=2))
sys.exit(0 if result['ok'] else 1)
