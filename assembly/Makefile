PROGRAM_SOURCES := $(wildcard *.p.s)
PROGRAM_TARGETS := $(PROGRAM_SOURCES:%.p.s=%.program)
ROM_SOURCES := $(wildcard *.rom.s)
ROM_TARGETS := $(ROM_SOURCES:.s=)

all: $(PROGRAM_TARGETS) $(ROM_TARGETS)

%.program: %.p.s
	cl65 -t none -o $@ $<

%.rom: %.rom.s %.rom.cfg
	ca65 $<
	ld65 $(<:.s=.o) -o $(<:.s=) -C $(<:.s=.cfg) -m $(<:.s=.map)

test:
	@echo program targets: $(PROGRAM_TARGETS)
	@echo program sources: $(PROGRAM_SOURCES)
	@echo rom targets: $(ROM_TARGETS)
	@echo rom sources: $(ROM_SOURCES)

clean:
	rm -f $(PROGRAM_TARGETS) $(ROM_TARGETS)
	rm -f $(PROGRAM_SOURCES:.s=.o) $(ROM_SOURCES:.s=.o)
	rm -f $(ROM_SOURCES:.s=.map)


