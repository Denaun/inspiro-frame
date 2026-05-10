import logging
import socket
from typing import Callable, Literal

logger = logging.getLogger(__name__)


def _parse_host_port(endpoint: str) -> tuple[str, int, str]:
    """Parse 'http://host:port' or 'http://host' into (host, port, base_path)."""
    if not endpoint.startswith("http://"):
        raise ValueError(f"Only http:// supported, got: {endpoint}")

    rest = endpoint[len("http://") :]
    host_port, *_ = rest.split("/", 1)
    if ":" in host_port:
        host, port = host_port.rsplit(":", 1)
        port = int(port)
    else:
        host, port = host_port, 80
    return host, port, host_port


class HttpClient:
    """
    Open, persistent HTTP/1.1 connection. Not instantiated directly —
    use HttpConnection as a context manager to obtain one.
    The socket is guaranteed to be open for the lifetime of this object.
    """

    def __init__(self, sock: socket.socket, host_header: str):
        self._sock = sock
        self._host_header = host_header

    def post(
        self, path: str, body: bytes, content_type: str = "application/json"
    ) -> bytes:
        self._send_headers(
            "POST",
            path,
            extra_headers=[
                f"Content-Type: {content_type}",
                f"Content-Length: {len(body)}",
            ],
        )
        self._sock.write(body)
        status, length = self._read_response_headers()
        _raise_for_status(status)
        if length is None:
            length = 0
        buf = bytearray(length)
        _read_exactly_into(self._sock, buf)
        return bytes(buf)

    def get_into(
        self,
        path: str,
        buf: bytearray,
        callbacks: list[Callable[[bytearray], None]],
    ) -> None:
        expected = len(buf) * len(callbacks)
        self._send_headers("GET", path)
        status, length = self._read_response_headers()
        _raise_for_status(status)
        if length != expected:
            raise ConnectionError(f"expected {expected} bytes, got {length}")
        for callback in callbacks:
            _read_exactly_into(self._sock, buf)
            callback(buf)

    def _send_headers(
        self,
        method: Literal["GET", "POST"],
        path: str,
        extra_headers: list[str] | None = None,
    ) -> None:
        lines = [
            f"{method} {path} HTTP/1.1",
            f"Host: {self._host_header}",
            "Connection: keep-alive",
        ]
        if extra_headers:
            lines.extend(extra_headers)
        lines.append("\r\n")
        self._sock.write("\r\n".join(lines).encode())

    def _read_response_headers(self) -> tuple[int, int | None]:
        """Returns (status_code, content_length)."""
        status_line = self._sock.readline()
        status = int(status_line.split(b" ", 2)[1])
        content_length = None
        while (line := self._sock.readline()) not in (b"\r\n", b"\n"):
            if line.lower().startswith(b"content-length:"):
                content_length = int(line.split(b":", 1)[1].strip())
        return status, content_length


class HttpConnection:
    """
    Context manager that opens a TCP connection and yields a HttpClient.
    The socket exists only within the `with` block.

    Usage:
        with HttpConnection("http://192.168.1.10:8080") as client:
            client.post(...)
    """

    def __init__(self, endpoint: str):
        self._host, self._port, self._host_header = _parse_host_port(endpoint)

    def __enter__(self) -> HttpClient:
        addr = socket.getaddrinfo(self._host, self._port, 0, socket.SOCK_STREAM)[0][-1]
        self._sock = socket.socket()
        self._sock.settimeout(15)
        self._sock.connect(addr)
        logger.info("connected to %s:%d", self._host, self._port)
        return HttpClient(self._sock, self._host_header)

    def __exit__(self, *_) -> None:
        self._sock.close()


def _raise_for_status(status: int) -> None:
    if not (200 <= status <= 299):
        raise ValueError(f"HTTP error {status}")


def _read_exactly_into(sock: socket.socket, buf: bytearray) -> None:
    """Read exactly len(buf) bytes from sock into buf, in-place."""
    view = memoryview(buf)
    pos = 0
    n = len(buf)
    while pos < n:
        chunk = sock.readinto(view[pos:], n - pos)
        if not chunk:
            raise ConnectionError("connection closed early")
        pos += chunk
