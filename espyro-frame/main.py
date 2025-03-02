import logging
import sys

import app
import machine

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
        frame.sleep()
