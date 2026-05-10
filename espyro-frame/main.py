import logging
import sys
import time

import app

logging.basicConfig(level=logging.WARNING, filename="espyro.log", filemode="w")

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
        duration_ms = time.ticks_diff(time.ticks_ms(), start)
        logging.debug("duration: %dms", duration_ms)
        if sleep:
            frame.sleep(duration_ms)
