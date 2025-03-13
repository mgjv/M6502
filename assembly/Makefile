PROGRAM_SOURCES := $(wildcard *.program.s)
PROGRAM_TARGETS := $(PROGRAM_SOURCES:%.program.s=%.program.bin)

ROM_SOURCES := $(wildcard *.rom.s)
ROM_TARGETS := $(ROM_SOURCES:.s=)

TEST_SOURCES := $(wildcard *.test.s)
TEST_TARGETS := $(TEST_SOURCES:.s=.bin)

CA65 := ca65
CL65 := cl65
LD65 := ld65

all: $(TEST_TARGETS) $(PROGRAM_TARGETS) $(ROM_TARGETS)

%.program.bin: %.p.s
	$(CL65) -t none -o $@ $<

%.test.bin: %.test.s test.inc test.cfg
	$(CL65) -t none -C test.cfg -o $@ $<

%.rom: %.rom.s %.rom.cfg
	$(CA65) $<
	$(LD65) $(<:.s=.o) -o $(<:.s=) -C $(<:.s=.cfg) -m $(<:.s=.map)

test:
	@echo program targets: $(PROGRAM_TARGETS)
	@echo program sources: $(PROGRAM_SOURCES)
	@echo rom targets: $(ROM_TARGETS)
	@echo rom sources: $(ROM_SOURCES)
	@echo test targets: $(TEST_TARGETS)
	@echo test sources: $(TEST_SOURCES)

clean:
	rm -f $(PROGRAM_TARGETS) $(ROM_TARGETS) $(TEST_TARGETS)
	rm -f $(PROGRAM_SOURCES:.s=.o) $(ROM_SOURCES:.s=.o) $(TEST_SOURCES:.s=.o)
	rm -f $(ROM_SOURCES:.s=.map)


