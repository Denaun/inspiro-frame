import json
import logging

import config
import epd_12in48b as epd
import esp32
import machine
import network
import requests
from micropython import const

logger = logging.getLogger(__name__)

MS_PER_DAY = const(86_400_000)


class App:
    def __init__(self):
        self.epd = epd.EPD(
            spi=machine.SPI(
                config.SPI,
                baudrate=config.BAUD_RATE,
                sck=config.Pins.SCK,
                mosi=config.Pins.MOSI,
                miso=None,
            ),
            m1_cs=config.Pins.M1_CS,
            s1_cs=config.Pins.S1_CS,
            m2_cs=config.Pins.M2_CS,
            s2_cs=config.Pins.S2_CS,
            m1s1_dc=config.Pins.M1S1_DC,
            m2s2_dc=config.Pins.M2S2_DC,
            m1s1_rst=config.Pins.M1S1_RST,
            m2s2_rst=config.Pins.M2S2_RST,
            m1_busy=config.Pins.M1_BUSY,
            s1_busy=config.Pins.S1_BUSY,
            m2_busy=config.Pins.M2_BUSY,
            s2_busy=config.Pins.S2_BUSY,
        )

    def led_on(self):
        if config.LED is not None:
            machine.Pin(config.LED, machine.Pin.OUT).on()

    def daily_clear(self):
        nvs = esp32.NVS("espiro_frame")
        try:
            refreshes = nvs.get_i32("refreshes")
        except OSError:
            refreshes = 0
        logger.info("refresh %s", refreshes)
        if refreshes == 0:
            logger.info("daily clear")
            self.epd.init()
            try:
                self.epd.clear()
                self.epd.turn_on()
            finally:
                self.epd.sleep()
        nvs.set_i32("refreshes", (refreshes + 1) % config.REFRESHES_PER_DAY)
        nvs.commit()

    def refresh(self):
        self.epd.init()
        try:
            self._fetch_and_display()
            self.epd.turn_on()
        finally:
            self.epd.sleep()

    def sleep(self, duration_ms: int):
        if config.REFRESHES_PER_DAY <= 0:
            return
        sleep_ms = MS_PER_DAY // config.REFRESHES_PER_DAY - duration_ms
        logger.info("sleeping for %sms", sleep_ms)
        machine.deepsleep(sleep_ms)

    def _fetch_and_display(self):
        wlan = self._connect_wifi()
        try:
            screenshot_id = self._take_screenshot()
            logger.info("created screenshot %s", screenshot_id)
            self._display(screenshot_id)
        finally:
            wlan.active(False)

    def _connect_wifi(self) -> network.WLAN:
        wlan = network.WLAN(network.WLAN.IF_STA)
        wlan.active(True)
        wlan.config(reconnects=10)
        if not wlan.isconnected():
            logger.info("connecting to network...")
            wlan.connect(config.WIFI_SSID, config.WIFI_PASSWORD)
            while not wlan.isconnected():
                pass
        logger.info("network config: %s", wlan.ipconfig("addr4"))
        return wlan

    def _take_screenshot(self) -> str:
        uri = f"{config.MATE_ENDPOINT}/screenshots"
        logger.info("taking screenshot of %s on %s", config.DASHBOARD_URL, uri)
        response = requests.post(
            uri,
            json={
                "url": config.DASHBOARD_URL,
                "width": epd.WIDTH,
                "height": epd.HEIGHT,
            },
            timeout=15,
        )
        _raise_for_status(response)
        return response.text

    def _display(self, screenshot_id: str):
        logger.info("s1")
        uri = self._quadrant_uri(
            screenshot_id,
            x=epd.LEFT_WIDTH,
            y=epd.HALF_HEIGHT,
            width=epd.RIGHT_WIDTH,
            height=epd.HALF_HEIGHT,
        )
        buf_size = epd.RIGHT_WIDTH * epd.HALF_HEIGHT >> 3
        response = _fetch_quadrant(uri, expected=2 * buf_size)
        buf = response.raw.read(buf_size)
        self.epd.s1_display_white(buf)
        buf = response.raw.read(buf_size)
        self.epd.s1_display_red(buf)

        logger.info("m2")
        uri = self._quadrant_uri(
            screenshot_id,
            x=epd.LEFT_WIDTH,
            y=0,
            width=epd.RIGHT_WIDTH,
            height=epd.HALF_HEIGHT,
        )
        buf_size = epd.RIGHT_WIDTH * epd.HALF_HEIGHT >> 3
        response = _fetch_quadrant(uri, expected=2 * buf_size)
        buf = response.raw.read(buf_size)
        self.epd.m2_display_white(buf)
        buf = response.raw.read(buf_size)
        self.epd.m2_display_red(buf)

        logger.info("m1")
        uri = self._quadrant_uri(
            screenshot_id,
            x=0,
            y=epd.HALF_HEIGHT,
            width=epd.LEFT_WIDTH,
            height=epd.HALF_HEIGHT,
        )
        buf_size = epd.LEFT_WIDTH * epd.HALF_HEIGHT >> 3
        response = _fetch_quadrant(uri, expected=2 * buf_size)
        buf = response.raw.read(buf_size)
        self.epd.m1_display_white(buf)
        buf = response.raw.read(buf_size)
        self.epd.m1_display_red(buf)

        logger.info("s2")
        uri = self._quadrant_uri(
            screenshot_id,
            x=0,
            y=0,
            width=epd.LEFT_WIDTH,
            height=epd.HALF_HEIGHT,
        )
        buf_size = epd.LEFT_WIDTH * epd.HALF_HEIGHT >> 3
        response = _fetch_quadrant(uri, expected=2 * buf_size)
        buf = response.raw.read(buf_size)
        self.epd.s2_display_white(buf)
        buf = response.raw.read(buf_size)
        self.epd.s2_display_red(buf)

    def _quadrant_uri(
        self, screenshot_id: str, x: int, y: int, width: int, height: int
    ) -> str:
        return f"{config.MATE_ENDPOINT}/screenshots/{screenshot_id}?x={x}&y={y}&width={width}&height={height}&format=bwr-raw"


def _fetch_quadrant(uri: str, expected: int):
    logger.info("fetching quadrant from %s", uri)
    response = requests.get(uri, timeout=5)
    _raise_for_status(response)
    length = int(response.headers["Content-Length"])
    assert length == expected, f"expected {expected} bytes, got {length}"
    return response


def _raise_for_status(response: requests.Response):
    if not 200 <= response.status_code <= 299:
        raise ValueError(
            f"unexpected response code {response.status_code}: {response.reason}"
        )
