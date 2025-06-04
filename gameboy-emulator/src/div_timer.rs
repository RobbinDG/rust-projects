pub struct DivTimer {
    write_reg: u8,
    read_reg: u8,
    div_counter: u8,
    pub div_apu: u8,
}

impl DivTimer {
    pub fn new() -> Self {
        Self {
            write_reg: 0,
            read_reg: 0,
            div_counter: 0,
            div_apu: 0,
        }
    }

    /// DIV register: counts up at 16384 Hz. Clock freq is 4.19 MHz. So, DIV counts up
    /// every 4190000 / 16384 = 255.76 ~= 256 clock cycles. We use a standard u8 with
    /// wrapping_add to count these DIV cycles.
    pub fn tick(&mut self, m_cycles: u8) -> bool {
        let mut div_apu_ticked = false;
        let last_div_ctr = self.div_counter;

        self.div_counter = self.div_counter.wrapping_add(4 * m_cycles);
        if self.div_counter < last_div_ctr { // Check if DIV counter overflowed.
            // self.div_counter += 32;
            let last_div = self.read_reg;
            let new_div = last_div.wrapping_add(1);
            // Check for falling edge of 4th bit to update DIV-APU.
            if new_div & 0b0001_0000 == 0 && last_div & 0b0001_0000 != 0 {
                self.div_apu = self.div_apu.wrapping_add(1);
                div_apu_ticked = true;
            }
            self.read_reg = new_div;
        }
        div_apu_ticked
    }

    pub fn read(&self) -> &u8 {
        &self.read_reg
    }

    pub fn write(&mut self) -> &mut u8 {
        self.read_reg = 0;
        &mut self.write_reg
    }
}