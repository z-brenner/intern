#!/usr/bin/env python3
"""A stand-in for llama-server that answers from a file of chosen replies. Laboratory tooling.

    python3 -I fake_server.py --port 18091 --capture requests.jsonl [--answers answers.json]

It speaks just enough of the OpenAI chat-completions API for `intern-bench run`:
GET /health and POST /v1/chat/completions. Every request body is appended to --capture (one
JSON per line), so the exact system turn, user turn and grammar the engine sends are kept.
When --answers maps the SHA-256 of a request's last user message to a reply string, that
string is returned with every token at probability 1, so the engine parses, validates,
composes and scores it exactly as it would a model's reply. Unknown requests get an empty
JSON object, which the engine rejects; nothing about them is invented.

No model, no network beyond 127.0.0.1.
"""
import argparse
import hashlib
import json
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def user_text(body):
    for message in reversed(body.get("messages", [])):
        if message.get("role") == "user":
            content = message.get("content")
            if isinstance(content, list):
                return "".join(part.get("text", "") for part in content if isinstance(part, dict))
            return content or ""
    return ""


def tokens(text):
    # Pieces of a few characters, as a tokenizer would cut them; every one certain.
    pieces = []
    at = 0
    while at < len(text):
        piece = text[at:at + 3]
        pieces.append({"token": piece, "logprob": 0.0, "bytes": list(piece.encode("utf-8")), "top_logprobs": []})
        at += 3
    return pieces


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=18091)
    parser.add_argument("--capture", required=True)
    parser.add_argument("--answers")
    args = parser.parse_args()
    answers = {}
    if args.answers:
        with open(args.answers, encoding="utf-8") as file:
            answers = json.load(file)
    lock = threading.Lock()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def send_json(self, status, payload):
            data = json.dumps(payload).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            if self.path.startswith("/health"):
                self.send_json(200, {"status": "ok"})
            elif self.path.startswith("/v1/models"):
                self.send_json(200, {"data": [{"id": "intern-local"}]})
            else:
                self.send_json(404, {"error": "not found"})

        def do_POST(self):
            length = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(length)
            try:
                body = json.loads(raw)
            except ValueError:
                self.send_json(400, {"error": "bad json"})
                return
            prompt = user_text(body)
            key = hashlib.sha256(prompt.encode("utf-8")).hexdigest()
            with lock, open(args.capture, "a", encoding="utf-8") as file:
                file.write(json.dumps({"key": key, "body": body}) + "\n")
            content = answers.get(key, "{}")
            self.send_json(200, {
                "id": "fake",
                "object": "chat.completion",
                "model": "intern-local",
                "choices": [{
                    "index": 0,
                    "finish_reason": "stop",
                    "message": {"role": "assistant", "content": content},
                    "logprobs": {"content": tokens(content)},
                }],
                "usage": {"prompt_tokens": len(prompt) // 4, "completion_tokens": len(content) // 3},
                "timings": {"prompt_n": len(prompt) // 4, "cache_n": 0, "prompt_ms": 1.0,
                            "predicted_n": len(content) // 3, "predicted_ms": 1.0},
            })

    ThreadingHTTPServer(("127.0.0.1", args.port), Handler).serve_forever()


if __name__ == "__main__":
    main()
