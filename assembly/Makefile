SOURCES := $(wildcard *.s)
TARGETS := $(SOURCES:.s=.rom)

$(TARGETS):

%.rom: %.s
	cl65 -t none -o $@ $<

clean:
	rm -f $(TARGETS)
	rm -f $(SOURCES:.s=.o)


