
# Makefile

VENV_DIR := env
OUT_DIR := build

MD := mkdir
RM := rm
PYTHON := python3
PIP := $(VENV_DIR)/bin/pip
MPY_CROSS := $(VENV_DIR)/bin/mpy-cross
MPREMOTE := $(VENV_DIR)/bin/mpremote

DEPS := \
	logging \

SOURCES := \
	app.py \
	config.py \
	epd_12in48b.py \

OBJS = $(SOURCES:%.py=$(OUT_DIR)/%.mpy)

.PHONY: all venv flash

all: venv flash

venv:
	@$(PYTHON) -m venv $(VENV_DIR)
	@$(PIP) install mpy-cross
	@$(PIP) install mpremote

dirs:
	$(MD) -p $(OUT_DIR)

$(OBJS): $(OUT_DIR)/%.mpy: %.py | dirs
	$(MPY_CROSS) $< -o $@

mpy: $(OBJS)

deps:
	$(MPREMOTE) mip install $(DEPS)

flash: main.py $(OBJS) | deps
	$(MPREMOTE) cp $^ :

clean:
	$(RM) -rf $(OUT_DIR)