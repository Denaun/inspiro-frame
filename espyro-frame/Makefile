PORT ?= auto
BOARD ?= esp32_generic_c6

VENV_DIR := env
OUT_DIR := out
LIB_DIR := lib
TYPINGS_DIR := typings

MD := mkdir
RM := rm
PYTHON := python3
PIP := $(VENV_DIR)/bin/pip
MPY_CROSS := $(VENV_DIR)/bin/mpy-cross
MPREMOTE := $(VENV_DIR)/bin/mpremote

MPREMOTE_CMD := $(MPREMOTE) connect $(PORT)
CURL := curl -fsSL

MICROPYTHON_LIB = https://raw.githubusercontent.com/micropython/micropython-lib/master
STDLIB_DEPS := \
	logging
STDLIB_FILES = $(STDLIB_DEPS:%=$(LIB_DIR)/%.py)
MICROPYTHON_PYI = $(TYPINGS_DIR)/micropython.pyi

SOURCES := \
	app.py \
	config.py \
	epd_12in48b.py \
	http_client.py \
	mate_client.py
OBJS = $(SOURCES:%.py=$(OUT_DIR)/%.mpy)

.PHONY: all dev build device-deps deploy flash clean distclean

all: dev flash

$(PIP):
	$(PYTHON) -m venv $(VENV_DIR)

$(MPY_CROSS): $(PIP)
	$(PIP) install mpy-cross

$(MPREMOTE): $(PIP)
	$(PIP) install mpremote

$(MICROPYTHON_PYI): %/micropython.pyi: $(PIP)
	$(PIP) install -U micropython-esp32-$(BOARD)-stubs --target $*

$(OUT_DIR):
	$(MD) -p $@

$(LIB_DIR):
	$(MD) -p $@

$(STDLIB_FILES): $(LIB_DIR)/%.py: | $(LIB_DIR)
	$(CURL) -o $@ $(MICROPYTHON_LIB)/python-stdlib/$*/$*.py

$(OBJS): $(OUT_DIR)/%.mpy: %.py $(MPY_CROSS) | $(OUT_DIR)
	$(MPY_CROSS) $< -o $@

dev: $(STDLIB_FILES) $(MICROPYTHON_PYI)

build: $(OBJS)

device-deps: | $(MPREMOTE)
	$(MPREMOTE_CMD) mip install $(STDLIB_DEPS)
	$(MPREMOTE_CMD) mip install github:josverl/micropython-stubs/mip/typing_mpy.json

deploy: main.py $(OBJS) | $(MPREMOTE)
	$(MPREMOTE_CMD) cp $^ :

flash: device-deps deploy

logs: | $(MPREMOTE)
	$(MPREMOTE_CMD) cat espyro.log

clean:
	$(RM) -rf $(OUT_DIR)
	$(RM) -rf $(LIB_DIR)
	$(RM) -rf $(TYPINGS_DIR)

distclean: clean
	$(RM) -rf $(VENV_DIR)
