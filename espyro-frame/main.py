import logging
import sys
import time

import app
import machine

start = time.ticks_us()
logging.basicConfig(level=logging.DEBUG)

frame = app.App()
frame.led_on()

sleep = True
try:
    frame.refresh()
except KeyboardInterrupt:
    sleep = False
except Exception as e:
    sys.print_exception(e)
finally:
    if sleep:
        frame.sleep(duration_ms=(time.ticks_us() - start) // 1000)
