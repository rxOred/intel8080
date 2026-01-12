setup-test:
	@echo "Setting up test environment..."
	ruby ./assembler/assembler.rb ./test/sample.asm ./test/sample.bin
	@echo "Test environment setup complete."

build: setup-test
	@echo "building the app..."
	cargo build
	@echo "build completed."

run: build
	@echo "running the app..."
	cargo run ./test/sample.bin
	@echo "app run completed."