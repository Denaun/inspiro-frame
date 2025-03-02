# 12.48""" 3-color

import logging
import time

import machine
from micropython import const

WIDTH = const(1304)
HEIGHT = const(984)

LEFT_WIDTH = const(648)
RIGHT_WIDTH = const(WIDTH - LEFT_WIDTH)
HALF_HEIGHT = const(HEIGHT >> 1)

LEFT_BYTES = const(LEFT_WIDTH >> 3)
RIGHT_BYTES = const(RIGHT_WIDTH >> 3)

logger = logging.getLogger(__name__)


class EPD:
    def __init__(
        self,
        spi: machine.SPI,
        m1_cs: machine.Pin,
        s1_cs: machine.Pin,
        m2_cs: machine.Pin,
        s2_cs: machine.Pin,
        m1s1_dc: machine.Pin,
        m2s2_dc: machine.Pin,
        m1s1_rst: machine.Pin,
        m2s2_rst: machine.Pin | None,
        m1_busy: machine.Pin,
        s1_busy: machine.Pin,
        m2_busy: machine.Pin,
        s2_busy: machine.Pin,
    ) -> EPD:
        self.spi = spi
        self.m1_cs = m1_cs
        self.s1_cs = s1_cs
        self.m2_cs = m2_cs
        self.s2_cs = s2_cs
        self.m1s1_dc = m1s1_dc
        self.m2s2_dc = m2s2_dc
        self.m1s1_rst = m1s1_rst
        self.m2s2_rst = m2s2_rst
        self.m1_busy = m1_busy
        self.s1_busy = s1_busy
        self.m2_busy = m2_busy
        self.s2_busy = s2_busy

    def waveshare() -> EPD:
        return EPD(
            spi=machine.SPI(
                3,
                baudrate=200_000,
                sck=machine.Pin(14),
                mosi=machine.Pin(13),
                miso=None,
            ),
            m1_cs=machine.Pin(23, machine.Pin.OUT),
            s1_cs=machine.Pin(22, machine.Pin.OUT),
            m2_cs=machine.Pin(16, machine.Pin.OUT),
            s2_cs=machine.Pin(19, machine.Pin.OUT),
            m1s1_dc=machine.Pin(25, machine.Pin.OUT),
            m2s2_dc=machine.Pin(17, machine.Pin.OUT),
            m1s1_rst=machine.Pin(33, machine.Pin.OUT),
            m2s2_rst=machine.Pin(5, machine.Pin.OUT),
            m1_busy=machine.Pin(32, machine.Pin.IN),
            s1_busy=machine.Pin(26, machine.Pin.IN),
            m2_busy=machine.Pin(18, machine.Pin.IN),
            s2_busy=machine.Pin(4, machine.Pin.IN),
        )

    def init(self) -> None:
        self.reset()
        self.m1_cs.on()
        self.s1_cs.on()
        self.m2_cs.on()
        self.s2_cs.on()
        self._init_v1()

    def _init_v1(self) -> None:
        logger.info("Init V1")
        # panel setting
        # KW-3f    KWR-2F   BWROTP 0f   BWOTP 1f
        self._m1s1m2s2_send_command(b"\x00")
        self._m1s1_send_data(b"\x2F")
        self._m2s2_send_data(b"\x23")

        # POWER SETTING
        # VGH=20V,VGL=-20V
        # VDH=15V
        # VDL=-15V
        self._m1m2_send_command(b"\x01")
        self._m1m2_send_data(b"\x07\x17\x3F\x3F\x0D")

        # booster soft start
        self._m1m2_send_command(b"\x06")
        self._m1m2_send_data(b"\x17\x17\x39\x17")

        # resolution setting
        self._m1s1m2s2_send_command(b"\x61")
        # source 648
        # gate 492
        self._m1s2_send_data(b"\x02\x88\x01\xEC")
        # source 656
        # gate 492
        self._s1m2_send_data(b"\x02\x90\x01\xEC")

        # DUSPI
        self._m1s1m2s2_send_command(b"\x15")
        self._m1s1m2s2_send_data(b"\x20")

        # PLL
        self._m1s1m2s2_send_command(b"\x30")
        self._m1s1m2s2_send_data(b"\x08")

        # Vcom and data interval setting
        self._m1s1m2s2_send_command(b"\x50")
        self._m1s1m2s2_send_data(b"\x31\x07")

        # TCON
        self._m1s1m2s2_send_command(b"\x60")
        self._m1s1m2s2_send_data(b"\x22")

        # POWER SETTING
        self._m1m2_send_command(b"\xE0")
        self._m1m2_send_data(b"\x01")

        self._m1s1m2s2_send_command(b"\xE3")
        self._m1s1m2s2_send_data(b"\x00")

        self._m1m2_send_command(b"\x82")
        self._m1m2_send_data(b"\x1C")

        self._set_lut()

    def clear(self) -> None:
        # M1 part 648*492
        # S1 part 656*492
        # M2 part 656*492
        # S2 part 648*492

        self._m1s1m2s2_send_command(b"\x10")
        self._s2_send_data(b"\xFF" * LEFT_BYTES * HALF_HEIGHT)
        self._m2_send_data(b"\xFF" * RIGHT_BYTES * HALF_HEIGHT)
        self._m1_send_data(b"\xFF" * LEFT_BYTES * HALF_HEIGHT)
        self._s1_send_data(b"\xFF" * RIGHT_BYTES * HALF_HEIGHT)

        self._m1s1m2s2_send_command(b"\x13")
        self._s2_send_data(b"\x00" * LEFT_BYTES * HALF_HEIGHT)
        self._m2_send_data(b"\x00" * RIGHT_BYTES * HALF_HEIGHT)
        self._m1_send_data(b"\x00" * LEFT_BYTES * HALF_HEIGHT)
        self._s1_send_data(b"\x00" * RIGHT_BYTES * HALF_HEIGHT)

    def m1_display_white(self, white) -> None:
        """Write the bottom left white buffer."""
        self._m1_send_command(b"\x10")
        self._m1_send_data(white)

    def m1_display_red(self, red) -> None:
        """Write the bottom left red buffer."""
        self._m1_send_command(b"\x13")
        self._m1_send_data(red)

    def s1_display_white(self, white) -> None:
        """Write the bottom right white buffer."""
        self._s1_send_command(b"\x10")
        self._s1_send_data(white)

    def s1_display_red(self, red) -> None:
        """Write the bottom right red buffer."""
        self._s1_send_command(b"\x13")
        self._s1_send_data(red)

    def m2_display_white(self, white) -> None:
        """Write the top right white buffer."""
        self._m2_send_command(b"\x10")
        self._m2_send_data(white)

    def m2_display_red(self, red) -> None:
        """Write the top right red buffer."""
        self._m2_send_command(b"\x13")
        self._m2_send_data(red)

    def s2_display_white(self, white) -> None:
        """Write the top left white buffer."""
        self._s2_send_command(b"\x10")
        self._s2_send_data(white)

    def s2_display_red(self, red) -> None:
        """Write the top left red buffer."""
        self._s2_send_command(b"\x13")
        self._s2_send_data(red)

    def turn_on(self) -> None:
        self._m1m2_send_command(b"\x04")  # power on
        time.sleep_ms(300)
        self._m1s1m2s2_send_command(b"\x12")  # Display Refresh

        logger.info("Busy")
        self._m1s1m2s2_send_command(b"\x71")
        _wait_for_high(self.m1_busy)
        _wait_for_high(self.s1_busy)
        _wait_for_high(self.m2_busy)
        _wait_for_high(self.s2_busy)
        logger.info("Busy free")

    def sleep(self) -> None:
        # power off
        self._m1s1m2s2_send_command(b"\x02")
        time.sleep_ms(300)

        # deep sleep
        self._m1s1m2s2_send_command(b"\x07")
        self._m1s1m2s2_send_data(b"\xA5")
        time.sleep_ms(300)

    def reset(self) -> None:
        self.m1s1_rst.on()
        if pin := self.m2s2_rst:
            pin.on()
        time.sleep_ms(200)
        self.m1s1_rst.off()
        if pin := self.m2s2_rst:
            pin.off()
        time.sleep_ms(5)
        self.m1s1_rst.on()
        if pin := self.m2s2_rst:
            pin.on()
        time.sleep_ms(200)

    def _m1_send_command(self, reg: bytes) -> None:
        self.m1s1_dc.off()
        self.m1_cs.off()
        self.spi.write(reg)
        self.m1_cs.on()

    def _m1_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.m1_cs.off()
        self.spi.write(data)
        self.m1_cs.on()

    def _s1_send_command(self, reg: bytes) -> None:
        self.m1s1_dc.off()
        self.s1_cs.off()
        self.spi.write(reg)
        self.s1_cs.on()

    def _s1_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.s1_cs.off()
        self.spi.write(data)
        self.s1_cs.on()

    def _m2_send_command(self, reg: bytes) -> None:
        self.m2s2_dc.off()
        self.m2_cs.off()
        self.spi.write(reg)
        self.m2_cs.on()

    def _m2_send_data(self, data: bytes) -> None:
        self.m2s2_dc.on()
        self.m2_cs.off()
        self.spi.write(data)
        self.m2_cs.on()

    def _s2_send_command(self, reg: bytes) -> None:
        self.m2s2_dc.off()
        self.s2_cs.off()
        self.spi.write(reg)
        self.s2_cs.on()

    def _s2_send_data(self, data: bytes) -> None:
        self.m2s2_dc.on()
        self.s2_cs.off()
        self.spi.write(data)
        self.s2_cs.on()

    def _m1s2_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.m2s2_dc.on()
        self.m1_cs.off()
        self.s2_cs.off()
        self.spi.write(data)
        self.m1_cs.on()
        self.s2_cs.on()

    def _s1m2_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.m2s2_dc.on()
        self.s1_cs.off()
        self.m1_cs.off()
        self.spi.write(data)
        self.s1_cs.on()
        self.m1_cs.on()

    def _m1s1_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.m1_cs.off()
        self.s1_cs.off()
        self.spi.write(data)
        self.m1_cs.on()
        self.s1_cs.on()

    def _m2s2_send_data(self, data: bytes) -> None:
        self.m2s2_dc.on()
        self.m2_cs.off()
        self.s2_cs.off()
        self.spi.write(data)
        self.m2_cs.on()
        self.s2_cs.on()

    def _m1m2_send_command(self, reg: bytes) -> None:
        self.m1s1_dc.off()
        self.m2s2_dc.off()
        self.m1_cs.off()
        self.m2_cs.off()
        self.spi.write(reg)
        self.m1_cs.on()
        self.m2_cs.on()

    def _m1m2_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.m2s2_dc.on()
        self.m1_cs.off()
        self.m2_cs.off()
        self.spi.write(data)
        self.m1_cs.on()
        self.m2_cs.on()

    def _m1s1m2s2_send_command(self, reg: bytes) -> None:
        self.m1s1_dc.off()
        self.m2s2_dc.off()
        self.m1_cs.off()
        self.s1_cs.off()
        self.m2_cs.off()
        self.s2_cs.off()
        self.spi.write(reg)
        self.m1_cs.on()
        self.s1_cs.on()
        self.m2_cs.on()
        self.s2_cs.on()

    def _m1s1m2s2_send_data(self, data: bytes) -> None:
        self.m1s1_dc.on()
        self.m2s2_dc.on()
        self.m1_cs.off()
        self.s1_cs.off()
        self.m2_cs.off()
        self.s2_cs.off()
        self.spi.write(data)
        self.m1_cs.on()
        self.s1_cs.on()
        self.m2_cs.on()
        self.s2_cs.on()

    def _set_lut(self) -> None:
        self._m1s1m2s2_send_command(b"\x20")  # vcom
        self._m1s1m2s2_send_data(LUT_VCOM1)

        self._m1s1m2s2_send_command(b"\x21")  # red not use
        self._m1s1m2s2_send_data(LUT_WW1)

        self._m1s1m2s2_send_command(b"\x22")  # bw r
        self._m1s1m2s2_send_data(LUT_BW1)  # bw=r

        self._m1s1m2s2_send_command(b"\x23")  # wb w
        self._m1s1m2s2_send_data(LUT_WB1)  # wb=w

        self._m1s1m2s2_send_command(b"\x24")  # bb b
        self._m1s1m2s2_send_data(LUT_BB1)  # bb=b

        self._m1s1m2s2_send_command(b"\x25")  # bb b
        self._m1s1m2s2_send_data(LUT_WW1)  # bb=b


def _wait_for_high(pin: machine.Pin):
    while pin.value() == 0:
        continue


LUT_VCOM1 = b"""\
\x00\x10\x10\x01\x08\x01\
\x00\x06\x01\x06\x01\x05\
\x00\x08\x01\x08\x01\x06\
\x00\x06\x01\x06\x01\x05\
\x00\x05\x01\x1E\x0F\x06\
\x00\x05\x01\x1E\x0F\x01\
\x00\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
LUT_WW1 = b"""\
\x91\x10\x10\x01\x08\x01\
\x04\x06\x01\x06\x01\x05\
\x84\x08\x01\x08\x01\x06\
\x80\x06\x01\x06\x01\x05\
\x00\x05\x01\x1E\x0F\x06\
\x00\x05\x01\x1E\x0F\x01\
\x08\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
LUT_BW1 = b"""\
\xA8\x10\x10\x01\x08\x01\
\x84\x06\x01\x06\x01\x05\
\x84\x08\x01\x08\x01\x06\
\x86\x06\x01\x06\x01\x05\
\x8C\x05\x01\x1E\x0F\x06\
\x8C\x05\x01\x1E\x0F\x01\
\xF0\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
LUT_WB1 = b"""\
\x91\x10\x10\x01\x08\x01\
\x04\x06\x01\x06\x01\x05\
\x84\x08\x01\x08\x01\x06\
\x80\x06\x01\x06\x01\x05\
\x00\x05\x01\x1E\x0F\x06\
\x00\x05\x01\x1E\x0F\x01\
\x08\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
LUT_BB1 = b"""\
\x92\x10\x10\x01\x08\x01\
\x80\x06\x01\x06\x01\x05\
\x84\x08\x01\x08\x01\x06\
\x04\x06\x01\x06\x01\x05\
\x00\x05\x01\x1E\x0F\x06\
\x00\x05\x01\x1E\x0F\x01\
\x01\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
