"""Relay between Claude Code and a Gibo data server (MCP over stdio).

On top of passing messages through, the relay:
1. answers the `server/discover` probe itself. Claude Code 2.1.274 sends it
   together with `initialize`, and FastMCP 4 servers (OpenBB) then reject the
   handshake with -32022. Remove this once Claude Code waits for the probe reply;
2. saves an oversized tool response to a file under the project's 자료/응답/
   folder and returns a summary with the path, so one call cannot flood the
   conversation (threshold: GIBO_SPILL_TOKENS, default 20000 estimated tokens);
3. adds Gibo tools that run on top of the server (relay_<name>.py);
4. appends Gibo's usage notes to the server instructions; and
5. with --lazy, starts the server on the first tool call, answering the
   handshake and tool list from a cache written on the first full start.

Usage: python3 -I relay.py --name NAME [--lazy --cache FILE] -- COMMAND [ARGS...]
"""
import importlib
import json
import os
import re
import subprocess
import sys
import threading
import time

sys.dont_write_bytecode = True  # keep the plugin folder free of __pycache__
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

RELAY_VERSION = "1"
SPILL_TOKENS = int(os.environ.get("GIBO_SPILL_TOKENS") or 20000)
SPILL_KEEP_DAYS = 7


def estimate_tokens(text):
    """Rough token count: ~4 ASCII characters per token, ~1 token per other character."""
    ascii_chars = sum(1 for ch in text if ord(ch) < 128)
    return round(ascii_chars / 4 + (len(text) - ascii_chars))


def files_dir(*parts):
    """Folder for saved files: GIBO_FILES_DIR, else 자료/ in the Claude Code project."""
    base = os.environ.get("GIBO_FILES_DIR") or os.path.join(
        os.environ.get("CLAUDE_PROJECT_DIR") or os.getcwd(), "자료")
    path = os.path.join(base, *parts)
    os.makedirs(path, exist_ok=True)
    return path


def safe_name(text):
    return re.sub(r"[^\w.-]+", "_", text).strip("_")[:80] or "response"


def short(value, limit=300):
    text = value if isinstance(value, str) else json.dumps(value, ensure_ascii=False)
    return text if len(text) <= limit else text[:limit] + f"…(+{len(text) - limit}자)"


def describe(value, depth=0):
    """A few lines describing the shape of a JSON value."""
    pad = "  " * depth
    if isinstance(value, list):
        lines = [f"{pad}목록 {len(value)}개"]
        if value and isinstance(value[0], dict):
            lines[0] += f", 항목 키: {', '.join(list(value[0])[:20])}"
        for label, item in (("첫째", value[0] if value else None),
                            ("둘째", value[1] if len(value) > 1 else None),
                            ("마지막", value[-1] if len(value) > 2 else None)):
            if item is not None:
                lines.append(f"{pad}  {label}: {short(item)}")
        return lines
    if isinstance(value, dict):
        lines = []
        for key, item in list(value.items())[:30]:
            if isinstance(item, (list, dict)) and depth < 1:
                lines.append(f"{pad}{key}:")
                lines.extend(describe(item, depth + 1))
            elif isinstance(item, (list, dict)):
                lines.append(f"{pad}{key}: {type(item).__name__} {len(item)}개")
            else:
                lines.append(f"{pad}{key}: {short(item, 120)}")
        if len(value) > 30:
            lines.append(f"{pad}…키 {len(value) - 30}개 더")
        return lines
    return [pad + short(value)]


def headings(text, limit=250):
    """Markdown headings with their line numbers."""
    found = [f"{n}: {line}" for n, line in enumerate(text.splitlines(), 1)
             if line.startswith("#") and line.lstrip("#").startswith(" ")]
    if len(found) > limit:
        found = found[:limit] + [f"…제목 {len(found) - limit}개 더"]
    return found


def write_json_lines(path, value):
    """Write JSON so that one record sits on one line (readable with Read and Grep)."""
    with open(path, "w", encoding="utf-8") as out:
        if isinstance(value, list):
            out.write("[\n" + ",\n".join(json.dumps(v, ensure_ascii=False) for v in value) + "\n]\n")
        else:
            json.dump(value, out, ensure_ascii=False, indent=1)


def prune_spills(folder):
    cutoff = time.time() - SPILL_KEEP_DAYS * 86400
    for entry in os.scandir(folder):
        if re.match(r"\d{8}-\d{6}_", entry.name) and entry.stat().st_mtime < cutoff:
            os.remove(entry.path)


def spill(server_name, tool, arguments, text, tokens):
    """Save an oversized response and return the summary that replaces it."""
    folder = files_dir("응답")
    prune_spills(folder)
    stem = os.path.join(folder, f"{time.strftime('%Y%m%d-%H%M%S')}_{int(time.time() * 1000) % 1000:03d}_{server_name}_{safe_name(tool)}")
    try:
        value = json.loads(text)
    except ValueError:
        value = None
    lines = []
    big_field = None
    if isinstance(value, dict):
        # A document wrapped in JSON (DART download_document): save the document itself.
        for key, item in value.items():
            if isinstance(item, str) and len(item) > len(text) / 2:
                big_field = key
    if big_field:
        path = stem + ".md"
        with open(path, "w", encoding="utf-8") as out:
            out.write(value[big_field])
        meta = {k: v for k, v in value.items() if k != big_field}
        lines += ["응답 정보:"] + describe(meta, 1)
        marks = headings(value[big_field], 120)
        if marks:
            lines += [f"문서 제목(줄 번호, {path} 기준):"] + marks
    elif value is not None:
        path = stem + ".json"
        write_json_lines(path, value)
        lines += ["구조 (목록은 한 줄에 한 항목):"] + describe(value, 1)
    else:
        path = stem + ".txt"
        with open(path, "w", encoding="utf-8") as out:
            out.write(text)
        marks = headings(text, 120)
        lines += ["제목(줄 번호):"] + marks if marks else ["앞부분:", short(text, 2000)]
    return "\n".join([
        f"[Gibo] 응답이 약 {tokens:,}토큰이라 대화에 싣지 않고 파일로 저장했습니다.",
        f"파일: {path}",
        f"요청: {tool} {short(arguments, 400)}",
        "전체를 한 번에 읽지 말고 Grep으로 찾거나 Read의 offset/limit으로 필요한 부분만 읽으세요.",
        "",
        *lines,
    ])


class Relay:
    def __init__(self, name, command, lazy=False, cache_path=None):
        self.name = name
        self.command = command
        self.lazy = lazy and bool(cache_path)
        self.cache_path = cache_path
        try:
            self.extras = importlib.import_module(f"relay_{name}")
        except ModuleNotFoundError:
            self.extras = None
        self.extra_names = {t["name"] for t in getattr(self.extras, "TOOLS", [])}
        self.server = None
        self.start_lock = threading.Lock()
        self.client_lock = threading.Lock()
        self.server_lock = threading.Lock()
        self.pending = {}
        self.client_calls = {}
        self.client_lists = set()
        self.client_init_id = None
        self.init_params = None
        self.initialized_sent = False
        self.next_id = 0
        self.cache = self.load_cache() if self.cache_path else None
        self.cache_tools_saved = False

    # --- transport -------------------------------------------------------
    def to_client(self, message):
        data = (json.dumps(message, ensure_ascii=False) + "\n").encode()
        with self.client_lock:
            sys.stdout.buffer.write(data)
            sys.stdout.buffer.flush()

    def raw_to_client(self, line):
        with self.client_lock:
            sys.stdout.buffer.write(line)
            sys.stdout.buffer.flush()

    def to_server(self, message):
        data = message if isinstance(message, bytes) else (json.dumps(message, ensure_ascii=False) + "\n").encode()
        with self.server_lock:
            self.server.stdin.write(data)
            self.server.stdin.flush()

    def reply(self, request_id, result=None, error=None):
        message = {"jsonrpc": "2.0", "id": request_id}
        message.update({"error": error} if error else {"result": result})
        self.to_client(message)

    # --- cache for --lazy ------------------------------------------------
    def cache_key(self):
        try:
            stamp = os.stat(self.command[0]).st_mtime
        except OSError:
            stamp = 0
        return f"{RELAY_VERSION}:{self.command[0]}:{stamp}:{' '.join(self.command[1:])}"

    def load_cache(self):
        try:
            with open(self.cache_path, encoding="utf-8") as f:
                cache = json.load(f)
        except (OSError, ValueError):
            return None
        if cache.get("key") != self.cache_key() or "initialize" not in cache or "tools" not in cache:
            return None
        return cache

    def save_cache(self, field, value):
        if self.cache is not None and self.cache.get("key") == self.cache_key() and field in self.cache:
            return
        try:
            with open(self.cache_path, encoding="utf-8") as f:
                current = json.load(f)
            if current.get("key") != self.cache_key():
                current = {}
        except (OSError, ValueError):
            current = {}
        current["key"] = self.cache_key()
        current[field] = value
        tmp = self.cache_path + ".tmp"
        try:
            with open(tmp, "w", encoding="utf-8") as f:
                json.dump(current, f, ensure_ascii=False)
            os.replace(tmp, self.cache_path)
        except OSError:
            pass  # no cache: the next session starts the server eagerly again

    # --- server lifecycle --------------------------------------------------
    def start_server(self):
        self.server = subprocess.Popen(self.command, stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        threading.Thread(target=self.read_server, daemon=True).start()

    def ensure_started(self):
        """Start a lazily deferred server and run the handshake for the client."""
        with self.start_lock:
            if self.server is not None:
                return
            self.start_server()
            reply = self.request("initialize", self.init_params, timeout=180)
            if reply is None or "error" in reply:
                raise RuntimeError(f"{self.name} server failed to start: {reply}")
            self.to_server({"jsonrpc": "2.0", "method": "notifications/initialized"})
            self.initialized_sent = True

    def request(self, method, params, timeout=300):
        """Send a request of the relay's own to the server and wait for the reply."""
        with self.server_lock:
            self.next_id += 1
            request_id = f"gibo-{self.next_id}"
        slot = [threading.Event(), None]
        self.pending[request_id] = slot
        self.to_server({"jsonrpc": "2.0", "id": request_id, "method": method, "params": params})
        slot[0].wait(timeout)
        self.pending.pop(request_id, None)
        return slot[1]

    def call_tool(self, tool, arguments, timeout=300):
        """Call a server tool for a Gibo tool. Returns the response text or raises."""
        self.ensure_started()
        reply = self.request("tools/call", {"name": tool, "arguments": arguments}, timeout)
        if reply is None:
            raise RuntimeError(f"{tool}: no reply within {timeout}s")
        if "error" in reply:
            raise RuntimeError(f"{tool}: {reply['error'].get('message')}")
        result = reply["result"]
        text = "\n".join(c.get("text", "") for c in result.get("content", []) if c.get("type") == "text")
        if result.get("isError"):
            raise RuntimeError(f"{tool}: {text[:500]}")
        return text

    def read_server(self):
        for line in iter(self.server.stdout.readline, b""):
            try:
                message = json.loads(line)
            except ValueError:
                self.raw_to_client(line)
                continue
            if not isinstance(message, dict) or "method" in message or "id" not in message:
                self.raw_to_client(line)
                continue
            request_id = message["id"]
            slot = self.pending.get(request_id) if isinstance(request_id, str) else None
            if slot is not None:
                slot[1] = message
                slot[0].set()
                continue
            key = json.dumps(request_id)
            if key == self.client_init_id and "result" in message:
                if self.cache_path:
                    self.save_cache("initialize", message["result"])
                message["result"] = self.with_instructions(message["result"])
            elif key in self.client_lists and "result" in message:
                self.client_lists.discard(key)
                if self.cache_path and not self.cache_tools_saved:
                    self.cache_tools_saved = True
                    self.save_cache("tools", message["result"].get("tools", []))
                message["result"] = self.with_extra_tools(message["result"])
            elif key in self.client_calls:
                tool, arguments = self.client_calls.pop(key)
                message = self.limit_size(message, tool, arguments)
            self.to_client(message)
        # The server exited: end the relay so Claude Code sees the disconnect.
        os._exit(self.server.wait() or 0)

    # --- additions -------------------------------------------------------
    def with_instructions(self, result):
        notes = getattr(self.extras, "INSTRUCTIONS", "")
        if notes:
            result = dict(result)
            result["instructions"] = ((result.get("instructions") or "") + "\n\n" + notes).strip()
        return result

    def with_extra_tools(self, result):
        tools = getattr(self.extras, "TOOLS", [])
        if tools:
            result = dict(result)
            result["tools"] = list(result.get("tools", [])) + tools
        return result

    def limit_size(self, message, tool, arguments):
        result = message.get("result")
        if not isinstance(result, dict) or result.get("isError"):
            return message
        texts = [c.get("text", "") for c in result.get("content", []) if c.get("type") == "text"]
        text = "\n".join(texts)
        tokens = estimate_tokens(text) if len(text) > SPILL_TOKENS else 0
        if tokens <= SPILL_TOKENS:
            return message
        try:
            summary = spill(self.name, tool, arguments, text, tokens)
        except OSError as error:
            summary = (f"[Gibo] 응답이 약 {tokens:,}토큰인데 파일로 저장하지 못했습니다({error}). "
                       "기간이나 항목을 좁혀 다시 요청하세요.")
        result = {k: v for k, v in result.items() if k not in ("content", "structuredContent")}
        result["content"] = [{"type": "text", "text": summary}]
        return {**message, "result": result}

    def run_extra(self, request_id, tool, arguments):
        try:
            text = self.extras.handle(self, tool, arguments or {})
            self.reply(request_id, {"content": [{"type": "text", "text": text}]})
        except Exception as error:  # report every failure to the client as a tool error
            self.reply(request_id, {"content": [{"type": "text", "text": f"[Gibo] {tool} 실패: {error}"}],
                                    "isError": True})

    # --- client side -----------------------------------------------------
    def handle_client(self, line):
        try:
            message = json.loads(line)
        except ValueError:
            message = None
        if not isinstance(message, dict):
            self.forward(line)
            return
        method = message.get("method")
        request_id = message.get("id")
        if method == "server/discover":
            if "id" in message:
                self.reply(request_id, error={"code": -32601, "message": "Method not found"})
            return
        if method == "initialize":
            self.init_params = message.get("params") or {}
            self.client_init_id = json.dumps(request_id)
            if self.lazy and self.cache:
                self.reply(request_id, self.with_instructions(self.cache["initialize"]))
                return
            if self.server is None:
                self.start_server()
            self.forward(line)
            return
        if method == "tools/list" and "id" in message:
            if self.server is None and self.cache:
                self.reply(request_id, self.with_extra_tools({"tools": self.cache["tools"]}))
                return
            self.client_lists.add(json.dumps(request_id))
        elif method == "tools/call" and "id" in message:
            params = message.get("params") or {}
            tool = params.get("name")
            if tool in self.extra_names:
                threading.Thread(target=self.run_extra, args=(request_id, tool, params.get("arguments")),
                                 daemon=True).start()
                return
            self.client_calls[json.dumps(request_id)] = (tool, params.get("arguments"))
        elif self.server is None:
            if method == "notifications/initialized" or "id" not in message:
                return
            local = {"ping": {}, "logging/setLevel": {}, "resources/list": {"resources": []},
                     "resources/templates/list": {"resourceTemplates": []}, "prompts/list": {"prompts": []}}
            if method in local:
                self.reply(request_id, local[method])
                return
        if self.server is None:
            try:
                self.ensure_started()
            except RuntimeError as error:
                if "id" in message:
                    self.reply(request_id, error={"code": -32603, "message": str(error)})
                return
        self.forward(line)

    def forward(self, line):
        if self.server is None:
            return
        self.to_server(line if line.endswith(b"\n") else line + b"\n")

    def run(self):
        for line in iter(sys.stdin.buffer.readline, b""):
            if line.strip():
                self.handle_client(line)
        if self.server is not None:
            self.server.stdin.close()
            try:
                self.server.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.server.kill()


def main(argv):
    if "--" not in argv:
        sys.exit("usage: relay.py --name NAME [--lazy --cache FILE] -- COMMAND [ARGS...]")
    split = argv.index("--")
    options, command = argv[:split], argv[split + 1:]
    name = options[options.index("--name") + 1] if "--name" in options else "server"
    cache = options[options.index("--cache") + 1] if "--cache" in options else None
    Relay(name, command, lazy="--lazy" in options, cache_path=cache).run()


if __name__ == "__main__":
    main(sys.argv[1:])
