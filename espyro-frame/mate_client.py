import json
import logging
from typing import Callable

import http_client

logger = logging.getLogger(__name__)


class MateClient:
    """
    API client for the inspiro-mate server.
    Wraps a HttpClient and speaks the Mate HTTP API.
    """

    def __init__(self, client: http_client.HttpClient):
        self._client = client

    def take_screenshot(self, url: str, width: int, height: int) -> str:
        """POST /screenshots, return the screenshot ID."""
        logger.info("taking screenshot of %s", url)
        body = json.dumps({"url": url, "width": width, "height": height})
        logger.debug("body: %s", body)
        return self._client.post("/screenshots", body.encode()).decode()

    def fetch_quadrant(
        self,
        screenshot_id: str,
        x: int,
        y: int,
        width: int,
        height: int,
        white_consumer: Callable[[bytearray], None],
        red_consumer: Callable[[bytearray], None],
    ) -> None:
        logger.info("fetching quadrant x=%d y=%d %dx%d", x, y, width, height)
        path = (
            f"/screenshots/{screenshot_id}"
            f"?x={x}&y={y}&width={width}&height={height}&format=bwr-raw"
        )
        logger.debug("path: %s", path)
        buf = bytearray(width * height >> 3)
        self._client.get_into(path, buf, [white_consumer, red_consumer])
