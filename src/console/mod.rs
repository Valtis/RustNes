extern crate time;

use cpu::Cpu;
use cpu::get_cpu;
use memory::Memory;
use rom::read_rom;

pub struct Console {
    cpu: Cpu,
    memory: Memory
}

impl Console {
    pub fn new (rom_path: &str) -> Console {
        let rom = read_rom(rom_path);

        let mut console = Console {
            cpu: get_cpu(&rom.header.tv_system),
            memory: Memory::new(),
        };

        console.memory.set_rom(rom);
        console
    }

    pub fn execute(&mut self) {
        let mut avg_cycle = 0.0;
        let cycle_time_in_nanoseconds = (1.0/(self.cpu.frequency.cpu_clock_frequency/1000.0)) as u64;
        println!("CPU frequency: {}", self.cpu.frequency.cpu_clock_frequency);
        println!("Cycle time in nanoseconds: {}", cycle_time_in_nanoseconds);

        let mut cycles = 0;
        let max_cycles = (self.cpu.frequency.cpu_clock_frequency*1000_000.0) as u32;
        let cpu_cycles_per_frame = 100;

        // 0xFFFC contains  location of first instruction
        self.cpu.program_counter = 0xFFFC;
        self.jump_absolute();


        // FOR NES CPU TEST
        self.cpu.program_counter = 0xC000;
        println!("\nPC: {}\n", self.cpu.program_counter);

        let mut time = time::precise_time_ns();
        loop {
            let current_time = time::precise_time_ns();
            let time_taken = current_time - time;

            // execute cpu_cycles_per_frame cycles every cpu_cycle_per_frame * cycle_time nanoseconds.
            // the 6502 has frequency around ~2 MHZ whics means that a cycle needs to be
            // executed every ~500ns. This however is not really possible even with high precision
            // timers. However, executing, say, 10 cycles every 5000ns is far more achievable.

            if time_taken > cycle_time_in_nanoseconds * cpu_cycles_per_frame {
                avg_cycle += time_taken as f64;
                for _ in 0..cpu_cycles_per_frame {
                    // ensure instruction timing
                    if self.cpu.wait_counter > 0 {
                        self.cpu.wait_counter -= 1;
                    } else {
                        let instruction = self.memory.read(self.cpu.program_counter);
                        self.cpu.program_counter += 1;
                        self.execute_instruction(instruction);
                    }
                    cycles += 1;
                }

                time = current_time;
            }
            if cycles >= max_cycles {
                break;
            }
        }

        println!("Avg cycle length: {}", avg_cycle/max_cycles as f64);
        println!("Duration: {}", avg_cycle as f64/ 1000_000_000.0)
    }

    fn execute_instruction(&mut self, instruction: u8) {
        match instruction {
            16 => { self.branch_if_positive(); }
            32 => { self.jump_to_subroutine();}
            76 => { self.jump_absolute(); }
            120 => { self.set_interrupt_disable_flag(); }
            134 => { self.store_x_zero_page(); }
            154 => { self.transfer_x_to_stack_pointer(); }
            162 => { self.load_x_immediate(); }
            173 => { self.load_a_absolute(); }
            216 => { self.clear_decimal_flag(); }
            _ => panic!("Invalid opcode {} (PC: {})", instruction, self.cpu.program_counter - 1),
        }
    }

    fn set_negative_flag(&mut self, value: u8) {
        self.cpu.status_flags = (self.cpu.status_flags & 0x7F) | (value & 0x80);
    }

    fn set_zero_flag(&mut self, value: u8) {
        if value == 0 {
            // set zero flag
            self.cpu.status_flags = self.cpu.status_flags | 0x02;
        } else {
            // reset zero flag
            self.cpu.status_flags = self.cpu.status_flags & 0xFD;
        }
    }

    fn get_2_byte_operand(&mut self) -> u16 {
        let low_byte = self.memory.read(self.cpu.program_counter);
        self.cpu.program_counter += 1;
        let high_byte = self.memory.read(self.cpu.program_counter);
        self.cpu.program_counter += 1;

         ((high_byte as u16) << 8) | low_byte as u16
    }

    fn get_byte_operand(&mut self) -> u8 {
        let byte = self.memory.read(self.cpu.program_counter);
        self.cpu.program_counter += 1;
        byte
    }

    fn push_value_into_stack(&mut self, value: u8) {
        self.memory.write(0x0100 + self.cpu.stack_pointer as u16, value);
        self.cpu.stack_pointer -= 1;
    }

    fn pop_value_from_stack(&mut self) -> u8 {
        self.cpu.stack_pointer += 1;
        self.memory.read(0x0100 + self.cpu.stack_pointer as u16)
    }

    fn branch_if_positive(&mut self) {

        // This needs to be removed from instruction stream even if we do not jump.
        // Get the value as u16 as pc is u16.
        let offset = self.get_byte_operand() as u16;

        // check if negative flag is zero and if so, branch
        if self.cpu.status_flags & 0x80 == 0 {
            let old_program_counter = self.cpu.program_counter;

            self.cpu.program_counter += offset;

            // the offset is signed 8 bit integer in two's complement. Thus if bit 7 is set,
            // we need to subtract 0x100 from the counter to get the correct value
            if offset & 0x80 != 0 {
                self.cpu.program_counter -= 0x100;
            }

            // timing depends on whether new address is on same or different memory page
            if old_program_counter & 0xFF00 == self.cpu.program_counter & 0xFF00 {
                self.cpu.wait_counter = 3;
            } else {
                self.cpu.wait_counter = 5;
            }
        } else {
            self.cpu.wait_counter = 2;
        }
    }

    fn jump_to_subroutine(&mut self) {
        self.cpu.wait_counter = 6;
        let address = self.get_2_byte_operand();

        let return_address = self.cpu.program_counter - 1;
        self.push_value_into_stack(((return_address & 0xFF00) >> 8) as u8);
        self.push_value_into_stack((return_address & 0xFF) as u8);
        self.cpu.program_counter = address;
    }

    fn jump_absolute(&mut self) {
        self.cpu.wait_counter = 3;
        self.cpu.program_counter = self.get_2_byte_operand();
    }

    fn set_interrupt_disable_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags | 0x04; // set second bit
    }

    fn store_x_zero_page(&mut self) {
        self.cpu.wait_counter = 3;
        let address = self.get_byte_operand();
        self.memory.write(address as u16, self.cpu.x);
    }

    fn transfer_x_to_stack_pointer(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.stack_pointer = self.cpu.x;
    }

    fn load_x_immediate(&mut self) {
        self.cpu.wait_counter = 2;
        let operand = self.get_byte_operand();
        self.cpu.x = operand;

        self.set_negative_flag(operand);
        self.set_zero_flag(operand);
    }

    fn load_a_absolute(&mut self) {
        self.cpu.wait_counter = 4;
        let address = self.get_2_byte_operand();
        let operand = self.memory.read(address);
        self.cpu.a = operand;
        self.set_negative_flag(operand);
        self.set_zero_flag(operand);

    }

    fn clear_decimal_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags & 0xF7; // clear third bit
    }

}




#[cfg(test)]
mod tests {

    use super::*;
    use cpu::get_cpu;
    use memory::Memory;
    use rom::TvSystem;

    fn create_test_console() -> Console {
        Console {
            memory: Memory::new(),
            cpu: get_cpu(&TvSystem::NTSC),
        }
    }
    #[test]
    fn set_negative_flag_sets_the_flag_if_flag_value_is_negative_and_flag_was_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.set_negative_flag(0xFF);
        assert_eq!(0x80, console.cpu.status_flags);
    }

    #[test]
    fn set_negative_flag_does_nothing_if_value_is_negative_and_flag_was_already_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD5;
        console.set_negative_flag(0xFF);
        assert_eq!(0xD5, console.cpu.status_flags);
    }

    #[test]
    fn set_negative_flag_unsets_the_flag_if_flag_is_set_and_value_was_positive() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD0;
        console.set_negative_flag(0x05);
        assert_eq!(0x50, console.cpu.status_flags);
    }

    #[test]
    fn set_negative_flag_does_nothing_if_flag_is_unset_and_value_is_positive() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x7F;
        console.set_negative_flag(0x7F);
        assert_eq!(0x7F, console.cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_sets_the_flag_if_flag_value_is_zero_and_flag_was_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.set_zero_flag(0);
        assert_eq!(0x02, console.cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_does_nothing_if_value_is_zero_and_flag_was_already_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD3;
        console.set_zero_flag(0);
        assert_eq!(0xD3, console.cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_unsets_the_flag_if_flag_is_set_and_value_was_not_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xDF;
        console.set_zero_flag(0x05);
        assert_eq!(0xDD, console.cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_does_nothing_if_flag_is_unset_and_value_is_not_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x70;
        console.set_zero_flag(0xFF);
        assert_eq!(0x70, console.cpu.status_flags);
    }

    #[test]
    fn read_2_bytes_reads_values_correctly_and_updates_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0xAD);
        console.memory.write(25, 0x04);
        assert_eq!(0x04AD, console.get_2_byte_operand());
        assert_eq!(26, console.cpu.program_counter);
    }

    #[test]
    fn get_byte_operand_gets_correct_value_and_updates_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0xAD);
        assert_eq!(0xAD, console.get_byte_operand());
        assert_eq!(25, console.cpu.program_counter);
    }

    #[test]
    fn push_value_to_stack_pushes_value_into_stack() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0xFF;
        console.push_value_into_stack(23);
        assert_eq!(23, console.memory.read(0x01FF));
    }

    #[test]
    fn push_value_to_stack_updates_stack_pointer() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0xFF;
        console.push_value_into_stack(23);
        assert_eq!(0xFE, console.cpu.stack_pointer);
    }

    #[test]
    fn pop_value_from_stack_returns_correct_value() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0xCC;
        console.memory.write(0x0100 + 0xCD, 123);
        assert_eq!(123, console.pop_value_from_stack());
    }

    #[test]
    fn pop_value_from_stack_updates_stack_pointer() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0xCC;
        console.pop_value_from_stack();
        assert_eq!(0xCD, console.cpu.stack_pointer);
    }

    #[test]
    fn branch_if_positive_jumps_to_relative_address_on_nonzero_positive_number() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0x6C);
        console.set_negative_flag(0x32);
        console.branch_if_positive();
        assert_eq!(25 + 0x6C, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_positive_can_jump_backwards() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0xFC);
        console.set_negative_flag(0x32);
        console.branch_if_positive();
        assert_eq!(25 - 4, console.cpu.program_counter);
    }
    #[test]
    fn branch_if_positive_jumps_to_address_on_zero() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0x02);
        console.set_negative_flag(0x00);
        console.branch_if_positive();

        assert_eq!(25 + 0x02, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_positive_does_not_jump_on_negative_number() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0xBC);
        console.set_negative_flag(0xff);
        console.branch_if_positive();
        assert_eq!(25, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_positive_does_not_change_flags_if_negative_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0xBC);
        console.set_negative_flag(0xff);
        let flags = console.cpu.status_flags;
        console.branch_if_positive();
        assert_eq!(flags, console.cpu.status_flags);
    }

    #[test]
    fn branch_if_positive_does_not_change_flags_if_negative_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0x0C);
        console.set_negative_flag(0x00);
        let flags = console.cpu.status_flags;
        console.branch_if_positive();
        assert_eq!(flags, console.cpu.status_flags);
    }

    #[test]
    fn branch_if_positive_waits_two_cycles_if_negative_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 24;
        console.memory.write(24, 0xBC);
        console.set_negative_flag(0xFF);
        console.branch_if_positive();
        assert_eq!(2, console.cpu.wait_counter);
    }
    #[test]
    fn branch_if_positive_waits_three_cycles_if_value_is_positive_and_on_same_page() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x030;
        console.memory.write(0x030, 0x005);
        console.set_negative_flag(0x00);
        console.branch_if_positive();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_positive_waits_five_cycles_if_value_is_positive_and_on_previous_page() {
        let mut console = create_test_console();
        console.cpu.program_counter = 540;
        console.memory.write(540, 0x80);
        console.set_negative_flag(0x00);
        console.branch_if_positive();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_positive_waits_five_cycles_if_value_is_positive_and_barely_on_previous_page() {
        let mut console = create_test_console();
        console.cpu.program_counter = 512;
        console.memory.write(512, 0xfe);
        console.set_negative_flag(0x00);
        console.branch_if_positive();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_positive_waits_five_cycles_if_value_is_positive_and_barely_on_next_page() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x0FE;
        console.memory.write(0x0FE, 0x01);
        console.set_negative_flag(0x00);
        console.branch_if_positive();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_positive_waits_five_cycles_if_value_is_positive_and_on_next_page() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0xF0;
        console.memory.write(0xF0, 0x7F);
        console.set_negative_flag(0x00);
        console.branch_if_positive();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn jump_to_subroutine_pushes_return_address_into_stack() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0xABCD;
        console.cpu.stack_pointer = 0xFF;
        console.memory.write(0xABCD, 0x09);
        console.memory.write(0xABCD + 1, 0xFC);
        console.jump_to_subroutine();
        // return address - 1 is pushed into stack in little endian form.
        // in this case, it's 0xABCE as the instruction takes two values from the instruction stream
        assert_eq!(0xCE, console.pop_value_from_stack());
        assert_eq!(0xAB, console.pop_value_from_stack());
    }

    #[test]
    fn jump_to_subroutine_changes_program_counter_value() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0xABCD;
        console.cpu.stack_pointer = 0xFF;
        console.memory.write(0xABCD, 0x09);
        console.memory.write(0xABCD + 1, 0xFC);
        console.jump_to_subroutine();
        assert_eq!(0xFC09, console.cpu.program_counter);
    }

    #[test]
    fn jump_to_subroutine_does_not_affect_status_flags() {
        let mut console = create_test_console();
        console.cpu.program_counter = 15;
        console.cpu.stack_pointer = 0xFF;
        console.cpu.status_flags = 0xAD;
        console.jump_to_subroutine();
        assert_eq!(0xAD, console.cpu.status_flags);
    }

    #[test]
    fn jump_to_subroutine_takes_6_cycles() {
        let mut console = create_test_console();
        console.cpu.program_counter = 15;
        console.cpu.stack_pointer = 0xFF;
        console.jump_to_subroutine();
        assert_eq!(6, console.cpu.wait_counter);
    }

    #[test]
    fn jump_absolute_sets_program_counter_to_new_value() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0;
        console.memory.write(0, 0x15);
        console.memory.write(1, 0xF0);
        console.jump_absolute();
        assert_eq!(0xf015, console.cpu.program_counter);
    }

    #[test]
    fn jump_absolute_sets_wait_counter_correctly() {
        let mut console = create_test_console();

        console.jump_absolute();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn setting_interrupt_disable_flag_does_nothing_if_flag_is_already_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD5;
        console.set_interrupt_disable_flag();
        assert_eq!(0xD5, console.cpu.status_flags);
    }


    #[test]
    fn setting_interrupt_disable_flag_sets_the_flag_and_does_not_touch_other_flags() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xC3;
        console.set_interrupt_disable_flag();
        assert_eq!(0xC7, console.cpu.status_flags);
    }

    #[test]
    fn setting_interrupt_disable_flag_sets_wait_counter_correctly() {
        let mut console = create_test_console();
        console.set_interrupt_disable_flag();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn store_x_zero_page_stores_value_into_memory_correctly() {
        let mut console = create_test_console();
        console.cpu.x = 0x2f;
        console.cpu.program_counter = 0x32;
        console.memory.write(0x32, 0x14);
        console.store_x_zero_page();
        assert_eq!(0x2f, console.memory.read(0x14));
    }

    #[test]
    fn store_x_zero_page_does_not_affect_flags() {
        let mut console = create_test_console();
        console.cpu.x = 0x2f;
        console.cpu.program_counter = 0x32;
        console.memory.write(0x32, 0x14);
        console.cpu.status_flags = 0xE0;
        console.store_x_zero_page();
        assert_eq!(0xE0, console.cpu.status_flags);
    }
    #[test]
    fn store_x_zero_page_increments_pc_correctly() {
        let mut console = create_test_console();
        console.store_x_zero_page();
        assert_eq!(1, console.cpu.program_counter);
    }
    #[test]
    fn store_x_zero_page_takes_3_cycles() {
        let mut console = create_test_console();
        console.store_x_zero_page();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn transfer_x_to_stack_pointer_sets_stack_pointer_to_correct_value() {
        let mut console = create_test_console();
        console.cpu.x = 0xFC;
        console.transfer_x_to_stack_pointer();
        assert_eq!(0xFC, console.cpu.stack_pointer);
    }

    #[test]
    fn transfer_x_to_stack_pointer_does_not_touch_flags() {
        let mut console = create_test_console();
        console.cpu.x = 0xFC;
        console.cpu.status_flags = 0xAB;
        console.transfer_x_to_stack_pointer();
        assert_eq!(0xAB, console.cpu.status_flags);
    }
    #[test]

    fn transfer_x_to_stack_pointer_sets_wait_counter_correct() {
        let mut console = create_test_console();
        console.transfer_x_to_stack_pointer();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn load_x_immediate_sets_x_to_the_value_given_in_next_byte() {
        let mut console = create_test_console();

        console.cpu.program_counter = 25;
        console.memory.write(25, 0x23);
        console.load_x_immediate();
        assert_eq!(0x23, console.cpu.x);
    }

    #[test]
    fn load_x_immediate_sets_negative_flag_and_does_not_touch_other_flags_if_value_is_negative() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x4C;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.load_x_immediate();
        assert_eq!(0xCC, console.cpu.status_flags);
    }

    #[test]
    fn load_x_immediate_sets_zero_flag_and_does_not_touch_other_flags_if_value_is_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x4C;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0);
        console.load_x_immediate();
        assert_eq!(0x4E, console.cpu.status_flags);
    }

    #[test]
    fn load_x_immediate_resets_zero_flag_when_value_is_negative() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x72;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.load_x_immediate();
        assert_eq!(0xF0, console.cpu.status_flags);
    }

    #[test]
    fn load_x_immediate_resets_negative_flag_when_value_is_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xFC;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0);
        console.load_x_immediate();
        assert_eq!(0x7E, console.cpu.status_flags);
    }

    #[test]
    fn load_x_immediate_does_nothing_to_flags_with_negative_value_if_negative_flag_was_set_before() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xFC;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.load_x_immediate();
        assert_eq!(0xFC, console.cpu.status_flags);
    }

    #[test]
    fn load_x_immediate_does_nothing_to_flags_with_zero_value_if_zero_flag_was_set_before() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x43;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0);
        console.load_x_immediate();
        assert_eq!(0x43, console.cpu.status_flags);
    }


    #[test]
    fn load_x_immediate_sets_wait_counter_correctly() {
        let mut console = create_test_console();
        console.load_x_immediate();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn load_a_absolute_loads_correct_value_from_memory() {
        let mut console = create_test_console();
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 42);

        console.load_a_absolute();
        assert_eq!(42, console.cpu.a);
    }

    #[test]
    fn load_a_absolute_sets_negative_flag_if_value_was_negative() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 0xFF);

        console.load_a_absolute();
        assert_eq!(0x80, console.cpu.status_flags);
    }

    #[test]
    fn load_a_absolute_does_nothing_to_flags_if_value_was_negative_and_flag_was_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xF0;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 0xFF);

        console.load_a_absolute();
        assert_eq!(0xF0, console.cpu.status_flags);
    }

    #[test]
    fn load_a_absolute_resets_zero_flag_if_it_was_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xF2;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 0xFF);

        console.load_a_absolute();
        assert_eq!(0xF0, console.cpu.status_flags);
    }

    #[test]
    fn load_a_sets_zero_flag_if_value_was_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x20;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 0);

        console.load_a_absolute();
        assert_eq!(0x22, console.cpu.status_flags);
    }

    #[test]
    fn load_a_does_nothing_to_flags_if_value_was_zero_and_flag_was_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x2F;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 0);

        console.load_a_absolute();
        assert_eq!(0x2F, console.cpu.status_flags);
    }

    #[test]
    fn load_a_resets_negative_flag_if_value_was_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xF0;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.memory.write(26, 0xF0);
        console.memory.write(0xF0B1, 0);

        console.load_a_absolute();
        assert_eq!(0x72, console.cpu.status_flags);
    }

    #[test]
    fn load_a_absolute_sets_wait_counter_correctly() {
        let mut console = create_test_console();
        console.load_a_absolute();
        assert_eq!(4, console.cpu.wait_counter);
    }

    #[test]
    fn clear_decimal_flags_clears_the_flag_and_does_not_touch_other_flags() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xCF;
        console.clear_decimal_flag();
        assert_eq!(0xC7, console.cpu.status_flags);
    }

    #[test]
    fn clear_decimal_flags_does_nothing_if_flag_is_already_cleared() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD6;
        console.clear_decimal_flag();
        assert_eq!(0xD6, console.cpu.status_flags);
    }

    #[test]
    fn clear_decimal_flags_sets_wait_counter_correctly() {
        let mut console = create_test_console();
        console.clear_decimal_flag();
        assert_eq!(2, console.cpu.wait_counter);
    }
}
