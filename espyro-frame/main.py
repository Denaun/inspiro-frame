import logging
import sys
import time

import app
import machine

logging.basicConfig(level=logging.DEBUG)

frame = app.App()
while True:
    sleep = True
    start = time.ticks_ms()
    try:
        frame.led_on()
        frame.refresh()
    except KeyboardInterrupt:
        sleep = False
        break
    except Exception as e:
        sys.print_exception(e)
    finally:
        if sleep:
            frame.sleep(duration_ms=time.ticks_ms() - start)
