use intel8080_core::Cpu8080;

pub struct Emulator {
   cpu :Cpu8080,
}

impl Emulator {
   pub fn new() -> Self {
       Emulator {
           cpu: Cpu8080::new(),
       }
   }

   pub fn load_rom(&mut self, path: &str) -> anyhow::Result<()> {
        let rom_data = std::fs::read(path)?;
        self.cpu.load_program(&rom_data, 0x0000);
        Ok(())
   }

   pub fn emulate(&mut self) {
        while true {
            self.cpu.print_debug();
            self.cpu.step(); 

            if self.cpu.is_halted() {
                break;
            }
        }
   }
}