# 12.48""" 3-color

import errno
import logging
import time
from typing import Self

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


def _timed(f, *args, **kwargs):
    myname = str(f).split(" ")[1]

    def new_func(*args, **kwargs):
        t = time.ticks_us()
        result = f(*args, **kwargs)
        delta = time.ticks_diff(time.ticks_us(), t)
        logger.info("%s time = %6.3fms", myname, delta / 1000)
        return result

    return new_func


class EPD:
    def __init__(
        self,
        spi: machine.SPI,
        m1_cs: int,
        s1_cs: int,
        m2_cs: int,
        s2_cs: int,
        m1s1_dc: int,
        m2s2_dc: int,
        m1s1_rst: int,
        m2s2_rst: int | None,
        m1_busy: int,
        s1_busy: int,
        m2_busy: int,
        s2_busy: int,
    ):
        self.spi = spi
        self.m1_cs = machine.Pin(m1_cs, machine.Pin.OUT)
        self.s1_cs = machine.Pin(s1_cs, machine.Pin.OUT)
        self.m2_cs = machine.Pin(m2_cs, machine.Pin.OUT)
        self.s2_cs = machine.Pin(s2_cs, machine.Pin.OUT)
        self.m1s1_dc = machine.Pin(m1s1_dc, machine.Pin.OUT)
        self.m2s2_dc = machine.Pin(m2s2_dc, machine.Pin.OUT)
        self.m1s1_rst = machine.Pin(
            m1s1_rst,
            machine.Pin.OUT,
            pull=machine.Pin.PULL_UP,
            hold=True,
        )
        if m2s2_rst is not None:
            self.m2s2_rst = machine.Pin(
                m2s2_rst,
                machine.Pin.OUT,
                pull=machine.Pin.PULL_UP,
                hold=True,
            )
        else:
            self.m2s2_rst = None
        self.m1_busy = machine.Pin(m1_busy, machine.Pin.IN)
        self.s1_busy = machine.Pin(s1_busy, machine.Pin.IN)
        self.m2_busy = machine.Pin(m2_busy, machine.Pin.IN)
        self.s2_busy = machine.Pin(s2_busy, machine.Pin.IN)

    @classmethod
    def waveshare(cls) -> Self:
        return cls(
            spi=machine.SPI(
                3,
                baudrate=200_000,
                sck=14,
                mosi=13,
                miso=None,
            ),
            m1_cs=23,
            s1_cs=22,
            m2_cs=16,
            s2_cs=19,
            m1s1_dc=25,
            m2s2_dc=17,
            m1s1_rst=33,
            m2s2_rst=5,
            m1_busy=32,
            s1_busy=26,
            m2_busy=18,
            s2_busy=4,
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
        self._m1s1_send_data(b"\x2f")
        self._m2s2_send_data(b"\x23")

        # POWER SETTING
        # VGH=20V,VGL=-20V
        # VDH=15V
        # VDL=-15V
        self._m1m2_send_command(b"\x01")
        self._m1m2_send_data(b"\x07\x17\x3f\x3f\x0d")

        # booster soft start
        self._m1m2_send_command(b"\x06")
        self._m1m2_send_data(b"\x17\x17\x39\x17")

        # resolution setting
        self._m1s1m2s2_send_command(b"\x61")
        # source 648
        # gate 492
        self._m1s2_send_data(b"\x02\x88\x01\xec")
        # source 656
        # gate 492
        self._s1m2_send_data(b"\x02\x90\x01\xec")

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
        self._m1m2_send_command(b"\xe0")
        self._m1m2_send_data(b"\x01")

        self._m1s1m2s2_send_command(b"\xe3")
        self._m1s1m2s2_send_data(b"\x00")

        self._m1m2_send_command(b"\x82")
        self._m1m2_send_data(b"\x1c")

        self._set_lut()

    @_timed
    def clear(self) -> None:
        # M1 part 648*492
        # S1 part 656*492
        # M2 part 656*492
        # S2 part 648*492

        self._m1s1m2s2_send_command(b"\x10")
        self._s2_send_data(b"\xff" * LEFT_BYTES * HALF_HEIGHT)
        self._m2_send_data(b"\xff" * RIGHT_BYTES * HALF_HEIGHT)
        self._m1_send_data(b"\xff" * LEFT_BYTES * HALF_HEIGHT)
        self._s1_send_data(b"\xff" * RIGHT_BYTES * HALF_HEIGHT)

        self._m1s1m2s2_send_command(b"\x13")
        self._s2_send_data(b"\x00" * LEFT_BYTES * HALF_HEIGHT)
        self._m2_send_data(b"\x00" * RIGHT_BYTES * HALF_HEIGHT)
        self._m1_send_data(b"\x00" * LEFT_BYTES * HALF_HEIGHT)
        self._s1_send_data(b"\x00" * RIGHT_BYTES * HALF_HEIGHT)

    @_timed
    def m1_display_white(self, white) -> None:
        """Write the bottom left white buffer."""
        self._m1_send_command(b"\x10")
        self._m1_send_data(white)

    @_timed
    def m1_display_red(self, red) -> None:
        """Write the bottom left red buffer."""
        self._m1_send_command(b"\x13")
        self._m1_send_data(red)

    @_timed
    def s1_display_white(self, white) -> None:
        """Write the bottom right white buffer."""
        self._s1_send_command(b"\x10")
        self._s1_send_data(white)

    @_timed
    def s1_display_red(self, red) -> None:
        """Write the bottom right red buffer."""
        self._s1_send_command(b"\x13")
        self._s1_send_data(red)

    @_timed
    def m2_display_white(self, white) -> None:
        """Write the top right white buffer."""
        self._m2_send_command(b"\x10")
        self._m2_send_data(white)

    @_timed
    def m2_display_red(self, red) -> None:
        """Write the top right red buffer."""
        self._m2_send_command(b"\x13")
        self._m2_send_data(red)

    @_timed
    def s2_display_white(self, white) -> None:
        """Write the top left white buffer."""
        self._s2_send_command(b"\x10")
        self._s2_send_data(white)

    @_timed
    def s2_display_red(self, red) -> None:
        """Write the top left red buffer."""
        self._s2_send_command(b"\x13")
        self._s2_send_data(red)

    def turn_on(self) -> None:
        self._m1m2_send_command(b"\x04")  # power on
        self._wait_ready(timeout=1)
        self._m1s1m2s2_send_command(b"\x12")  # Display Refresh

        logger.info("Busy")
        self._m1s1m2s2_send_command(b"\x71")
        self._wait_ready(timeout=15)
        logger.info("Busy free")

    def sleep(self) -> None:
        # power off
        self._m1s1m2s2_send_command(b"\x02")
        self._wait_ready(timeout=1)

        # deep sleep
        self._m1s1m2s2_send_command(b"\x07")
        self._m1s1m2s2_send_data(b"\xa5")
        time.sleep_ms(300)

    def reset(self) -> None:
        self.m1s1_rst.init(value=1, hold=False)
        if pin := self.m2s2_rst:
            pin.init(value=1, hold=False)
        time.sleep_ms(200)
        self.m1s1_rst.off()
        if pin := self.m2s2_rst:
            pin.off()
        time.sleep_ms(5)
        self.m1s1_rst.init(value=1, hold=True)
        if pin := self.m2s2_rst:
            pin.init(value=1, hold=True)
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

    @_timed
    def _wait_ready(self, timeout):
        m1 = self.m1_busy
        s1 = self.s1_busy
        m2 = self.m2_busy
        s2 = self.s2_busy
        last = 0
        start = time.ticks_us()
        while (busy := (s2() << 3) | (m2() << 2) | (s1() << 1) | m1()) != 0b1111:
            if busy != last:
                logger.debug("Busy: %x", last)
                last = busy
            if time.ticks_diff(time.ticks_us(), start) >= timeout * 1_000_000:
                raise OSError(errno.ETIMEDOUT)


LUT_VCOM1 = b"""\
\x00\x10\x10\x01\x08\x01\
\x00\x06\x01\x06\x01\x05\
\x00\x08\x01\x08\x01\x06\
\x00\x06\x01\x06\x01\x05\
\x00\x05\x01\x1e\x0f\x06\
\x00\x05\x01\x1e\x0f\x01\
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
\x00\x05\x01\x1e\x0f\x06\
\x00\x05\x01\x1e\x0f\x01\
\x08\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
LUT_BW1 = b"""\
\xa8\x10\x10\x01\x08\x01\
\x84\x06\x01\x06\x01\x05\
\x84\x08\x01\x08\x01\x06\
\x86\x06\x01\x06\x01\x05\
\x8c\x05\x01\x1e\x0f\x06\
\x8c\x05\x01\x1e\x0f\x01\
\xf0\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
LUT_WB1 = b"""\
\x91\x10\x10\x01\x08\x01\
\x04\x06\x01\x06\x01\x05\
\x84\x08\x01\x08\x01\x06\
\x80\x06\x01\x06\x01\x05\
\x00\x05\x01\x1e\x0f\x06\
\x00\x05\x01\x1e\x0f\x01\
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
\x00\x05\x01\x1e\x0f\x06\
\x00\x05\x01\x1e\x0f\x01\
\x01\x04\x05\x08\x08\x01\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
\x00\x00\x00\x00\x00\x00\
"""
