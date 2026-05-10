import binascii
import logging
import time

import config
import epd_12in48b as epd
import esp32
import http_client
import machine
import mate_client
import network
from micropython import const

logger = logging.getLogger(__name__)

MS_PER_DAY = const(86_400_000)


def _timed(f, *args, **kwargs):
    myname = str(f).split(" ")[1]

    def new_func(*args, **kwargs):
        t = time.ticks_us()
        result = f(*args, **kwargs)
        delta = time.ticks_diff(time.ticks_us(), t)
        logger.info("%s time = %6.3fms", myname, delta / 1000)
        return result

    return new_func


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
        self._wifi_cache = esp32.NVS("wifi_cache")
        self._conn = http_client.HttpConnection(config.MATE_ENDPOINT)

    def led_on(self):
        if config.LED is not None:
            machine.Pin(config.LED, machine.Pin.OUT).on()

    def led_off(self):
        if config.LED is not None:
            machine.Pin(config.LED, machine.Pin.OUT).off()

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

    @_timed
    def _fetch_and_display(self):
        wlan = self._connect_wifi()
        try:
            with self._conn as client:
                mate = mate_client.MateClient(client)
                screenshot_id = mate.take_screenshot(
                    config.DASHBOARD_URL, width=epd.WIDTH, height=epd.HEIGHT
                )
                logger.info("created screenshot %s", screenshot_id)
                self._display(mate, screenshot_id)
        finally:
            wlan.active(False)

    @_timed
    def _connect_wifi(self) -> network.WLAN:
        wlan = network.WLAN(network.WLAN.IF_STA)
        wlan.active(True)
        wlan.config(reconnects=3)

        if wlan.isconnected():
            logger.info("network config: %s", wlan.ipconfig("addr4"))
            return wlan

        # Try to load cached BSSID + channel from NVS
        bssid = None
        try:
            bssid_hex = bytearray(6)
            self._wifi_cache.get_blob("bssid", bssid_hex)
            channel = self._wifi_cache.get_i32("channel")
            bssid = bytes(bssid_hex)
            logger.info(
                "fast connect: channel=%s bssid=%s",
                channel,
                binascii.hexlify(bssid),
            )
            wlan.connect(config.WIFI_SSID, config.WIFI_PASSWORD, bssid=bssid)
        except OSError:
            # No cache yet, or NVS read failed — do a normal connect
            logger.info("connecting to network (no cache)...")
            wlan.connect(config.WIFI_SSID, config.WIFI_PASSWORD)

        # Wait for connection with timeout
        deadline = time.ticks_add(time.ticks_ms(), 10_000)
        while not wlan.isconnected():
            if time.ticks_diff(deadline, time.ticks_ms()) <= 0:
                # Fast connect may have failed (AP moved channel etc.) — retry without BSSID
                if bssid is not None:
                    logger.info("fast connect failed, retrying without cache...")
                    wlan.disconnect()
                    wlan.connect(config.WIFI_SSID, config.WIFI_PASSWORD)
                    bssid = None
                    deadline = time.ticks_add(time.ticks_ms(), 15_000)
                else:
                    raise OSError("WiFi connection timed out")
            time.sleep_ms(50)

        # Cache BSSID + channel to NVS if we didn't use them (cache miss or fast-connect failed)
        if bssid is None:
            try:
                scan_results = wlan.scan()
                ssid_bytes = config.WIFI_SSID.encode()
                best = None
                for ssid, bssid, ch, rssi, *_ in scan_results:
                    if ssid == ssid_bytes:
                        if best is None or rssi > best[2]:
                            best = (bssid, ch, rssi)

                if best:
                    self._wifi_cache.set_blob("bssid", best[0])
                    self._wifi_cache.set_i32("channel", best[1])
                    self._wifi_cache.commit()
                    logger.info(
                        "cached: ch=%d bssid=%s", best[1], binascii.hexlify(best[0])
                    )
            except Exception as e:
                logger.warning("failed to cache WiFi info: %s", e)

        logger.info("network config: %s", wlan.ipconfig("addr4"))
        return wlan

    def _display(self, mate: mate_client.MateClient, screenshot_id: str):
        logger.info("s1")
        mate.fetch_quadrant(
            screenshot_id,
            x=epd.LEFT_WIDTH,
            y=epd.HALF_HEIGHT,
            width=epd.RIGHT_WIDTH,
            height=epd.HALF_HEIGHT,
            white_consumer=self.epd.s1_display_white,
            red_consumer=self.epd.s1_display_red,
        )

        logger.info("m2")
        mate.fetch_quadrant(
            screenshot_id,
            x=epd.LEFT_WIDTH,
            y=0,
            width=epd.RIGHT_WIDTH,
            height=epd.HALF_HEIGHT,
            white_consumer=self.epd.m2_display_white,
            red_consumer=self.epd.m2_display_red,
        )

        logger.info("m1")
        mate.fetch_quadrant(
            screenshot_id,
            x=0,
            y=epd.HALF_HEIGHT,
            width=epd.LEFT_WIDTH,
            height=epd.HALF_HEIGHT,
            white_consumer=self.epd.m1_display_white,
            red_consumer=self.epd.m1_display_red,
        )

        logger.info("s2")
        mate.fetch_quadrant(
            screenshot_id,
            x=0,
            y=0,
            width=epd.LEFT_WIDTH,
            height=epd.HALF_HEIGHT,
            white_consumer=self.epd.s2_display_white,
            red_consumer=self.epd.s2_display_red,
        )
