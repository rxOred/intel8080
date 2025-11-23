use intel8080_emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    emulator.load_rom("./test/sample.bin").expect("Failed to load ROM");
    emulator.emulate();
}
