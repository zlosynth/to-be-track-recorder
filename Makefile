.PHONY: all
all: format clippy test

.PHONY: check-format
check-format:
	make -C dsp check-format
	make -C firmware check-format
	make -C tests check-format

.PHONY: format
format:
	make -C dsp format
	make -C firmware format
	make -C tests format

.PHONY: clippy
clippy:
	make -C dsp clippy
	make -C firmware clippy
	make -C tests clippy

.PHONY: test
test:
	make -C dsp test
	make -C tests test

.PHONY: update
update:
	make -C dsp update
	make -C firmware update
	make -C tests update

.PHONY: clean
clean:
	make -C dsp clean
	make -C firmware clean
	make -C tests clean

.PHONY: flash
flash:
	make -C firmware flash
