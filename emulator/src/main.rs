use intel8080_emulator::Emulator;

fn main() {
    let mut emulator = Emulator::new();
    emulator.load_rom("path/to/rom").expect("Failed to load ROM");
    emulator.emulate();
}
