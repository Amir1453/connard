EXECUTABLE = bxc.exe

all: build

build:
	cargo build --release
	cp target/release/$(shell basename `pwd`) $(EXECUTABLE)

clean:
	cargo clean
	rm -f $(EXECUTABLE)

run: build
	./$(EXECUTABLE)

.PHONY: all build clean run

