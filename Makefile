EXECUTABLE = bxc.exe

all: build

build:
	cargo build --release
	cp target/release/connard $(EXECUTABLE)

clean:
	cargo clean
	rm -f $(EXECUTABLE)

run: build
	./$(EXECUTABLE)

.PHONY: all build clean run

