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
            8 => self.push_status_flags_into_stack(),
            16 => self.branch_if_positive(),
            24 => self.clear_carry_flag(),
            32 => self.jump_to_subroutine(),
            36 => self.bit_test_zero_page(),
            37 => self.and_zero_page(),
            40 => self.pull_status_flags_from_stack(),
            41 => self.and_immediate(),
            48 => self.branch_if_minus_set(),
            53 => self.and_zero_page_x(),
            56 => self.set_carry_flag(),
            72 => self.push_accumulator(),
            76 => self.jump_absolute(),
            80 => self.branch_if_overflow_clear(),
            96 => self.return_from_subroutine(),
            104 => self.pull_accumulator(),
            112 => self.branch_if_overflow_set(),
            120 => self.set_interrupt_disable_flag(),
            133 => self.store_a_zero_page(),
            134 => self.store_x_zero_page(),
            144 => self.branch_if_carry_clear(),
            154 => self.transfer_x_to_stack_pointer(),
            162 => self.load_x_immediate(),
            169 => self.load_a_immediate(),
            173 => self.load_a_absolute(),
            176 => self.branch_if_carry_set(),
            201 => self.compare_immediate(),
            208 => self.branch_if_not_equal(),
            216 => self.clear_decimal_flag(),
            234 => self.no_operation(),
            240 => self.branch_if_equal(),
            248 => self.set_decimal_flag(),
            _ => panic!("\n\nInvalid opcode {}\nInstruction PC: {}, \nCPU status: {:?}", instruction,
                self.cpu.program_counter - 1, self.cpu),
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

    fn read_immediate(&mut self) -> u8 {
        self.cpu.wait_counter = 2;
        self.get_byte_operand()
    }

    fn read_absolute(&mut self) -> u8 {
        self.cpu.wait_counter = 4;
        let address = self.get_2_byte_operand();
        self.memory.read(address)
    }

    fn read_absolute_x(&mut self) -> u8 {
        let base = self.get_2_byte_operand();
        let address = base + self.cpu.x as u16;
        // if page boundary is crossed, instruction takes 5 cycles. Otherwise it takes 4 cycles
        if base & 0xFF00 == address & 0xFF00 {
            self.cpu.wait_counter = 4;
        } else {
            self.cpu.wait_counter = 5;
        }
        self.memory.read(address)
    }

    fn read_zero_page(&mut self) -> u8 {
        self.cpu.wait_counter = 3;
        let address = self.get_byte_operand();
        self.memory.read(address as u16)
    }

    fn read_zero_page_x(&mut self) -> u8 {
        self.cpu.wait_counter = 4;
        let address = self.get_byte_operand() as u16 + self.cpu.x as u16;
        self.memory.read(address % 256)
    }

    fn set_load_flags(&mut self, value: u8) {
        self.set_negative_flag(value);
        self.set_zero_flag(value);
    }

    fn load_a(&mut self, value: u8) {
        self.cpu.a = value;
        self.set_load_flags(value);
    }

    fn load_x(&mut self, value: u8) {
        self.cpu.x = value;
        self.set_load_flags(value);
    }

    fn load_y(&mut self, value: u8) {
        self.cpu.y = value;
        self.set_load_flags(value);
    }

    fn do_zero_page_store(&mut self, value: u8) {
        self.cpu.wait_counter = 3;
        let address = self.get_byte_operand();
        self.memory.write(address as u16, value);
    }

    fn push_value_into_stack(&mut self, value: u8) {
        self.memory.write(0x0100 + self.cpu.stack_pointer as u16, value);
        self.cpu.stack_pointer -= 1;
    }

    fn pop_value_from_stack(&mut self) -> u8 {
        self.cpu.stack_pointer += 1;
        self.memory.read(0x0100 + self.cpu.stack_pointer as u16)
    }

    fn do_and(&mut self, operand: u8) {
        self.cpu.a = self.cpu.a & operand;
        let result = self.cpu.a;
        self.set_negative_flag(result);
        self.set_zero_flag(result);
    }

    fn do_relative_jump_if(&mut self, condition: bool) {
        let offset = self.get_byte_operand() as u16;
        if  condition {
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

    fn push_status_flags_into_stack(&mut self) {
        // This instruction sets bits 4 & 5 to 1 for the value that gets pushed into stack.
        // In contrast, irq or nmi will set bit 4 to 0.
        self.cpu.wait_counter = 3;
        let flags = self.cpu.status_flags | 0x30;
        self.push_value_into_stack(flags);
    }

    fn branch_if_positive(&mut self) {
        let condition = self.cpu.status_flags & 0x80 == 0;
        self.do_relative_jump_if(condition);
    }

    fn clear_carry_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags & 0xFE; // clear bi 0
    }

    fn jump_to_subroutine(&mut self) {
        self.cpu.wait_counter = 6;
        let address = self.get_2_byte_operand();

        let return_address = self.cpu.program_counter - 1;
        self.push_value_into_stack(((return_address & 0xFF00) >> 8) as u8);
        self.push_value_into_stack((return_address & 0xFF) as u8);
        self.cpu.program_counter = address;
    }

    fn bit_test_zero_page(&mut self) {
        let operand = self.read_zero_page();
        let result = self.cpu.a & operand;
        // set overflow and negative flags to correct values, unset zero flag
        self.cpu.status_flags = (self.cpu.status_flags & 0x3D) | (result & 0xC0);
        self.set_zero_flag(result);
    }

    fn and_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.do_and(value);
    }

    fn pull_status_flags_from_stack(&mut self) {
        self.cpu.wait_counter = 4;
        self.cpu.status_flags = self.pop_value_from_stack() | 0x30;
    }

    fn and_immediate(&mut self) {
        let operand = self.read_immediate();
        self.do_and(operand);
    }

    fn branch_if_minus_set(&mut self) {
        let condition = self.cpu.status_flags & 0x80 != 0;
        self.do_relative_jump_if(condition);
    }

    fn and_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        self.do_and(value);
    }

    fn set_carry_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags | 0x01;
    }

    fn push_accumulator(&mut self) {
        self.cpu.wait_counter = 3;
        let value = self.cpu.a;
        self.push_value_into_stack(value);
    }

    fn jump_absolute(&mut self) {
        self.cpu.wait_counter = 3;
        self.cpu.program_counter = self.get_2_byte_operand();
    }

    fn branch_if_overflow_clear(&mut self) {
        let condition = self.cpu.status_flags & 0x40 == 0;
        self.do_relative_jump_if(condition);
    }

    fn return_from_subroutine(&mut self) {
        self.cpu.wait_counter = 6;
        let low_byte = self.pop_value_from_stack() as u16;
        let high_byte = self.pop_value_from_stack() as u16;
        self.cpu.program_counter = ((high_byte << 8) | low_byte) + 1;
    }

    fn pull_accumulator(&mut self) {
        self.cpu.wait_counter = 4;
        let value = self.pop_value_from_stack();
        self.cpu.a = value;
        self.set_zero_flag(value);
    }

    fn branch_if_overflow_set(&mut self) {
        let condition = self.cpu.status_flags & 0x40 != 0;
        self.do_relative_jump_if(condition);
    }

    fn set_interrupt_disable_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags | 0x04; // set bit 2
    }

    fn store_a_zero_page(&mut self) {
        let value = self.cpu.a;
        self.do_zero_page_store(value);
    }

    fn store_x_zero_page(&mut self) {
        let value = self.cpu.x;
        self.do_zero_page_store(value);
    }

    fn branch_if_carry_clear(&mut self) {
        let condition = self.cpu.status_flags & 0x01 == 0;
        self.do_relative_jump_if(condition);
    }

    fn transfer_x_to_stack_pointer(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.stack_pointer = self.cpu.x;
    }

    fn load_x_immediate(&mut self) {
        let value = self.read_immediate();
        self.load_x(value);
    }

    fn load_a_immediate(&mut self) {
        let value = self.read_immediate();
        self.load_a(value);
    }

    fn load_a_absolute(&mut self) {
        let value = self.read_absolute();
        self.load_a(value);
    }

    fn branch_if_carry_set(&mut self) {
        let condition = self.cpu.status_flags & 0x01 != 0;
        self.do_relative_jump_if(condition);
    }

    fn compare_immediate(&mut self) {
        let operand = self.read_immediate();
        if operand > self.cpu.a {
            self.cpu.status_flags = self.cpu.status_flags | 0x80;
        } else if operand == self.cpu.a {
            self.cpu.status_flags = self.cpu.status_flags | 0x03;
        } else {
            self.cpu.status_flags = self.cpu.status_flags | 0x01;
        }

    }

    fn branch_if_not_equal(&mut self) {
        let condition = self.cpu.status_flags & 0x02 == 0;
        self.do_relative_jump_if(condition);
    }

    fn clear_decimal_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags & 0xF7; // clear bit 3
    }

    fn no_operation(&mut self) {
        self.cpu.wait_counter = 2;
    }

    fn branch_if_equal(&mut self) {
        let condition = self.cpu.status_flags & 0x02 != 0;
        self.do_relative_jump_if(condition);
    }

    fn set_decimal_flag(&mut self) {
        self.cpu.wait_counter = 2;
        self.cpu.status_flags = self.cpu.status_flags | 0x08; // set bit 3
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
    fn read_immediate_returns_value_pointed_by_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0xFA);
        assert_eq!(0xFA, console.read_immediate());
    }

    #[test]
    fn read_immediate_sets_wait_counter_to_2() {
        let mut console = create_test_console();
        console.read_immediate();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn read_absolute_returns_value_pointed_by_address_at_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0xFA);
        console.memory.write(0x433, 0xE0);
        console.memory.write(0xE0FA, 0x52);
        assert_eq!(0x52, console.read_absolute());
    }

    #[test]
    fn read_absolute_sets_wait_counter_to_4() {
        let mut console = create_test_console();
        console.read_absolute();
        assert_eq!(4, console.cpu.wait_counter);
    }

    #[test]
    fn read_absolute_x_returns_value_pointed_by_16_bit_address_pointed_by_pc_and_x_register() {
        let mut console = create_test_console();
        console.cpu.x = 0xFA;
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0xFA);
        console.memory.write(0x433, 0xE0);
        console.memory.write(0xE0FA + 0x00FA, 0x52);
        assert_eq!(0x52, console.read_absolute_x());
    }

    #[test]
    fn read_absolute_x_takes_4_cycles_if_page_boundary_is_not_crossed() {
        let mut console = create_test_console();
        console.cpu.x = 0xFA;
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0x00);
        console.memory.write(0x433, 0xE0);
        console.read_absolute_x();
        assert_eq!(4, console.cpu.wait_counter);
    }


    #[test]
    fn read_absolute_x_takes_5_cycles_if_page_boundary_is_barely_crossed() {
        let mut console = create_test_console();
        console.cpu.x = 0x01;
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0xFF);
        console.memory.write(0x433, 0xE0);
        console.read_absolute_x();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn read_absolute_x_takes_5_cycles_if_page_boundary_is__crossed() {
        let mut console = create_test_console();
        console.cpu.x = 0xFE;
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0xFA);
        console.memory.write(0x433, 0xE0);
        console.read_absolute_x();
        assert_eq!(5, console.cpu.wait_counter);
    }


    #[test]
    fn read_zero_page_returns_value_at_zero_page_pointed_by_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0xFA);
        console.memory.write(0x00FA, 0xAE);
        assert_eq!(0xAE, console.read_zero_page());
    }

    #[test]
    fn read_zero_page_sets_wait_counter_to_3() {
        let mut console = create_test_console();
        console.read_zero_page();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn read_zero_page_x_returns_value_at_zero_page_pointed_by_program_counter_indexed_with_x() {
        let mut console = create_test_console();
        console.cpu.x = 0x0F;
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0x80);
        console.memory.write(0x008F, 0xAE);
        assert_eq!(0xAE, console.read_zero_page_x());
    }

    #[test]
    fn read_zero_page_x_handles_wrap_around() {
        let mut console = create_test_console();
        console.cpu.x = 0xFF;
        console.cpu.program_counter = 0x432;
        console.memory.write(0x432, 0x80);
        console.memory.write(0x007F, 0xAE);
        assert_eq!(0xAE, console.read_zero_page_x());
    }

    #[test]
    fn read_zero_page_x_sets_wait_counter_to_4() {
        let mut console = create_test_console();
        console.read_zero_page_x();
        assert_eq!(4, console.cpu.wait_counter);
    }


    #[test]
    fn do_and_sets_accumulator_value_to_the_result() {
        let mut console = create_test_console();
        console.cpu.a = 0xE9;
        console.do_and(0x3E);
        assert_eq!(0x28, console.cpu.a);
    }

    #[test]
    fn do_and_unsets_zero_flag_if_it_was_set_before_and_result_is_not_zero() {
        let mut console = create_test_console();
        console.cpu.a = 0xE9;
        console.cpu.status_flags = 0x02;
        console.do_and(0x3E);
        assert_eq!(0x00, console.cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_zero_flag_if_it_was_not_set_before_and_result_is_not_zero() {
        let mut console = create_test_console();
        console.cpu.a = 0xE9;
        console.cpu.status_flags = 0x00;
        console.do_and(0x3E);
        assert_eq!(0x00, console.cpu.status_flags & 0x02);
    }

    #[test]
    fn do_and_sets_zero_flag_if_result_is_zero_and_flag_was_not_set_before() {
        let mut console = create_test_console();
        console.cpu.a = 0x00;
        console.cpu.status_flags = 0x00;
        console.do_and(0x3E);
        assert_eq!(0x02, console.cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_zero_flag_if_flag_is_set_and_result_is_zero() {
        let mut console = create_test_console();
        console.cpu.a = 0x00;
        console.cpu.status_flags = 0x02;
        console.do_and(0x3E);
        assert_eq!(0x02, console.cpu.status_flags);
    }

    #[test]
    fn do_and_sets_negative_flag_if_result_is_negative_and_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.a = 0x80;
        console.cpu.status_flags = 0x00;
        console.do_and(0xFF);
        assert_eq!(0x80, console.cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_negative_flag_if_it_is_set_and_number_is_negative() {
        let mut console = create_test_console();
        console.cpu.a = 0x80;
        console.cpu.status_flags = 0xA1;
        console.do_and(0xFF);
        assert_eq!(0xA1, console.cpu.status_flags);
    }

    #[test]
    fn do_and_unsets_negative_flag_if_flag_is_set_and_number_is_not_negative() {
        let mut console = create_test_console();
        console.cpu.a = 0x80;
        console.cpu.status_flags = 0xAF;
        console.do_and(0x7F);
        assert_eq!(0x2F, console.cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_negative_flag_if_it_is_unset_and_number_is_not_negative() {
        let mut console = create_test_console();
        console.cpu.a = 0x80;
        console.cpu.status_flags = 0x3F;
        console.do_and(0x7F);
        assert_eq!(0x3F, console.cpu.status_flags);
    }

    #[test]
    fn do_and_does_not_touch_program_counter_increments_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x52;
        console.do_and(0xFF);
        assert_eq!(0x52, console.cpu.program_counter);
    }

    #[test]
    fn do_and_does_not_modify_wait_counter() {
        let mut console = create_test_console();
        console.do_and(0x02);
        assert_eq!(0, console.cpu.wait_counter);
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
    fn push_status_flags_into_stack_pushes_flags_to_stack_and_sets_bits_4_and_5_to_1() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x8A;
        console.push_status_flags_into_stack();
        assert_eq!(0xBA, console.pop_value_from_stack());
    }

    #[test]
    fn push_status_flags_into_stack_does_not_increment_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x4000;
        console.push_status_flags_into_stack();
        assert_eq!(0x4000, console.cpu.program_counter);
    }

    #[test]
    fn push_status_flags_into_stack_takes_3_cycles() {
        let mut console = create_test_console();
        console.push_status_flags_into_stack();
        assert_eq!(3, console.cpu.wait_counter);
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
    fn clear_carry_flag_clears_the_flag_if_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xC5;
        console.clear_carry_flag();
        assert_eq!(0xC4, console.cpu.status_flags);
    }

    #[test]
    fn clear_carry_does_nothing_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD6;
        console.clear_carry_flag();
        assert_eq!(0xD6, console.cpu.status_flags);
    }

    #[test]
    fn clear_carry_flag_takes_2_cycles() {
        let mut console = create_test_console();
        console.clear_carry_flag();
        assert_eq!(2, console.cpu.wait_counter);
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
    fn bit_test_zero_page_does_not_touch_accumulator() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x08;
        console.cpu.a = 0xFA;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0xB2);
        console.bit_test_zero_page();
        assert_eq!(0xFA, console.cpu.a);
    }

    #[test]
    fn bit_test_zero_page_sets_negative_flag_if_bit_is_set_and_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x08;
        console.cpu.a = 0x80;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x80);
        console.bit_test_zero_page();
        assert_eq!(0x88, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_page_does_nothing_if_negative_bit_is_set_and_negative_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.a = 0x80;
        console.cpu.status_flags = 0x81;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x80);
        console.bit_test_zero_page();
        assert_eq!(0x81, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_page_unsets_negative_flag_if_bit_is_not_set_and_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.a = 0x0F;
        console.cpu.status_flags = 0x81;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0xFF);
        console.bit_test_zero_page();
        assert_eq!(0x01, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_page_does_nothing_if_negative_flag_is_not_set_and_bit_is_not_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.a = 0x0F;
        console.cpu.status_flags = 0x01;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0xFF);
        console.bit_test_zero_page();
        assert_eq!(0x01, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_page_sets_overflow_flag_if_overflow_bit_is_set_and_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x04;
        console.cpu.a = 0x40;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x40);
        console.bit_test_zero_page();
        assert_eq!(0x44, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_page_does_nothing_if_overflow_bit_is_set_and_overflow_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x74;
        console.cpu.a = 0x40;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x40);
        console.bit_test_zero_page();
        assert_eq!(0x74, console.cpu.status_flags);
    }


    #[test]
    fn bit_test_zero_page_unsets_overflow_bit_if_overflow_bit_is_not_set_and_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x44;
        console.cpu.a = 0x0F;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x4F);
        console.bit_test_zero_page();
        assert_eq!(0x04, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_page_does_nothing_if_overflow_bit_is_not_set_and_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x0D;
        console.cpu.a = 0x0F;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x4F);
        console.bit_test_zero_page();
        assert_eq!(0x0D, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_sets_zero_flag_if_result_is_zero() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x10;
        console.cpu.a = 0x00;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x40);
        console.bit_test_zero_page();
        assert_eq!(0x12, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_sets_does_nothing_if_result_is_zero_and_zero_flag_was_already_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x02;
        console.cpu.a = 0x00;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x40);
        console.bit_test_zero_page();
        assert_eq!(0x02, console.cpu.status_flags);
    }


    #[test]
    fn bit_test_zero_unsets_zero_flag_if_result_is_not_zero() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x02;
        console.cpu.a = 0x0A;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x4F);
        console.bit_test_zero_page();
        assert_eq!(0x00, console.cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_does_nothing_if_result_is_not_zero_and_zero_flag_was_not_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.cpu.status_flags = 0x10;
        console.cpu.a = 0x0A;
        // pc points to address to zero page that contains actual operand
        console.memory.write(0x1234, 0x12);
        console.memory.write(0x12, 0x4F);
        console.bit_test_zero_page();
        assert_eq!(0x10, console.cpu.status_flags);
    }


    #[test]
    fn bit_test_zero_increments_pc_correctly() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        console.bit_test_zero_page();
        assert_eq!(0x1234+1, console.cpu.program_counter);
    }

    #[test]
    fn bit_test_zero_page_takes_3_cycles() {
        let mut console = create_test_console();
        console.bit_test_zero_page();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn and_zero_page_sets_accumulator_value_to_the_result() {
        let mut console = create_test_console();
        console.cpu.a = 0xE9;
        console.cpu.program_counter = 0xABCD;
        console.memory.write(0xABCD, 0xFA);
        console.memory.write(0xFA, 0x3E);

        console.and_zero_page();
        assert_eq!(0x28, console.cpu.a);
    }

    #[test]
    fn and_zero_page_increments_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x52;
        console.and_zero_page();
        assert_eq!(0x53, console.cpu.program_counter);
    }

    #[test]
    fn and_zero_page_takes_3_cycles() {
        let mut console = create_test_console();
        console.and_zero_page();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn pull_status_flags_sets_status_flags_correctly() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x1A;
        console.push_value_into_stack(0xFE);
        console.pull_status_flags_from_stack();
        assert_eq!(0xFE, console.cpu.status_flags);
    }

    // hardwired to 1
    #[test]
    fn pull_status_flags_always_sets_4_and_5_bits() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xAE;
        console.push_value_into_stack(0x00);
        console.pull_status_flags_from_stack();
        assert_eq!(0x30, console.cpu.status_flags);
    }

    #[test]
    fn pull_status_flags_from_stack_increments_stack_pointer() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0x3f;
        console.pull_status_flags_from_stack();
        assert_eq!(0x3f + 1, console.cpu.stack_pointer);
    }

    #[test]
    fn pull_status_flags_from_stack_does_not_modify_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1253;
        console.pull_status_flags_from_stack();
        assert_eq!(0x1253, console.cpu.program_counter);
    }

    #[test]
    fn pull_status_flags_from_stack_takes_4_cycles() {
        let mut console = create_test_console();
        console.pull_status_flags_from_stack();
        assert_eq!(4, console.cpu.wait_counter);
    }

    #[test]
    fn and_immediate_sets_accumulator_value_to_the_result() {
        let mut console = create_test_console();
        console.cpu.a = 0xE9;
        console.cpu.program_counter = 0x15;
        console.memory.write(0x15, 0x3E);
        console.and_immediate();
        assert_eq!(0x28, console.cpu.a);
    }

    #[test]
    fn and_immediate_increments_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x52;
        console.and_immediate();
        assert_eq!(0x53, console.cpu.program_counter);
    }

    #[test]
    fn and_immediate_takes_2_cycles() {
        let mut console = create_test_console();
        console.and_immediate();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_minus_set_branches_if_zero_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x80;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_minus_set();
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_minus_set_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x7F;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_minus_set();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_minus_set_takes_2_cycles_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x7F;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_minus_set();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_minus_set_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x80;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_minus_set();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_minus_set_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x80;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_minus_set();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn and_zero_page_x_sets_accumulator_value_to_the_result() {
        let mut console = create_test_console();
        console.cpu.a = 0xE9;
        console.cpu.x = 0x05;
        console.cpu.program_counter = 0x15;
        console.memory.write(0x15, 0x40);
        console.memory.write(0x40 + 0x05, 0x3E);
        console.and_zero_page_x();
        assert_eq!(0x28, console.cpu.a);
    }

    #[test]
    fn and_zero_page_x_increments_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x52;
        console.and_zero_page_x();
        assert_eq!(0x53, console.cpu.program_counter);
    }

    #[test]
    fn and_zero_page_x_takes_4_cycles() {
        let mut console = create_test_console();
        console.and_zero_page_x();
        assert_eq!(4, console.cpu.wait_counter);
    }

    #[test]
    fn set_carry_flag_sets_the_flag_if_it_was_not_set_before() {
        let mut console = create_test_console();
        console.cpu.program_counter = 15;
        console.cpu.status_flags = 0x86;
        console.set_carry_flag();
        assert_eq!(0x87, console.cpu.status_flags);
    }

    #[test]
    fn set_carry_flag_does_nothing_if_flag_is_already_set() {
        let mut console = create_test_console();
        console.cpu.program_counter = 15;
        console.cpu.status_flags = 0x86;
        console.set_carry_flag();
        assert_eq!(0x87, console.cpu.status_flags);
    }

    #[test]
    fn set_carry_flag_does_not_modify_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 15;
        console.cpu.stack_pointer = 0x86;
        console.set_carry_flag();
        assert_eq!(15, console.cpu.program_counter);
    }

    #[test]
    fn set_carry_flag_takes_2_cycles() {
        let mut console = create_test_console();
        console.cpu.program_counter = 15;
        console.cpu.stack_pointer = 0xFF;
        console.set_carry_flag();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn push_accumulator_pushes_accumulator_into_stack() {
        let mut console = create_test_console();
        console.cpu.a = 0x34;
        console.push_accumulator();
        assert_eq!(0x34, console.pop_value_from_stack());
    }

    #[test]
    fn push_accumulator_does_not_modify_accumulator() {
        let mut console = create_test_console();
        console.cpu.a = 0x34;
        console.push_accumulator();
        assert_eq!(0x34, console.cpu.a);
    }

    #[test]
    fn push_accumulator_decrements_stack_pointer() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0xEF;
        console.push_accumulator();
        assert_eq!(0xEF - 1, console.cpu.stack_pointer);
    }

    #[test]
    fn push_accumulator_does_not_modify_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x123;
        console.push_accumulator();
        assert_eq!(0x123, console.cpu.program_counter);
    }

    #[test]
    fn push_accumulator_takes_3_cycles() {
        let mut console = create_test_console();
        console.push_accumulator();
        assert_eq!(3, console.cpu.wait_counter);
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
    fn branch_if_overflow_clear_branches_if_overflow_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xBF;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_clear();
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_overflow_clear_does_not_branch_and_updates_pc_correctly_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x40;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_clear();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_oveflow_clear_takes_2_cycles_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x040;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_clear();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_overflow_clear_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xBF;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_clear();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_overflow_clear_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xBF;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_overflow_clear();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn return_from_subroutine_sets_pc_correctly() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x1234;
        // push high byte
        console.push_value_into_stack(0xFA);
        // push low byte
        console.push_value_into_stack(0x0B);
        console.return_from_subroutine();
        assert_eq!(0xFA0B + 1, console.cpu.program_counter);
    }

    #[test]
    fn return_from_subroutine_increments_stack_pointer() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0x10;
        console.return_from_subroutine();
        assert_eq!(0x10 + 2, console.cpu.stack_pointer);
    }

    #[test]
    fn return_from_subroutine_does_not_touch_status_flags() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xFA;
        console.return_from_subroutine();
        assert_eq!(0xFA, console.cpu.status_flags);
    }

    #[test]
    fn return_from_subroutine_takes_6_cycles() {
        let mut console = create_test_console();
        console.return_from_subroutine();
        assert_eq!(6, console.cpu.wait_counter);
    }

    #[test]
    fn pull_accumulator_sets_accumulator_to_correct_value() {
        let mut console = create_test_console();
        console.cpu.a = 0x00;
        console.push_value_into_stack(0xFA);
        console.pull_accumulator();
        assert_eq!(0xFA, console.cpu.a);
    }

    #[test]
    fn pull_accumulator_increments_stack_pointer() {
        let mut console = create_test_console();
        console.cpu.stack_pointer = 0x24;
        console.pull_accumulator();
        assert_eq!(0x24 + 1, console.cpu.stack_pointer);
    }

    #[test]
    fn pull_accumulator_sets_zero_flag_if_value_pulled_was_zero() {
        let mut console = create_test_console();
        console.cpu.a = 0xAA;
        console.cpu.status_flags = 0xF8;
        console.push_value_into_stack(0x00);
        console.pull_accumulator();
        assert_eq!(0xFA, console.cpu.status_flags);
    }

    #[test]
    fn pull_accumulator_unsets_zero_flag_if_value_pulled_was_not_zero() {
        let mut console = create_test_console();
        console.cpu.a = 0x00;
        console.cpu.status_flags = 0xAA;
        console.push_value_into_stack(0xBA);
        console.pull_accumulator();
        assert_eq!(0xA8, console.cpu.status_flags);
    }

    #[test]
    fn pull_accumulator_does_not_modify_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0x20;
        console.pull_accumulator();
        assert_eq!(0x20, console.cpu.program_counter);
    }

    #[test]
    fn pull_accumulator_takes_4_cycles() {
        let mut console = create_test_console();
        console.pull_accumulator();
        assert_eq!(4, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_overflow_set_branches_if_overflow_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD0;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_set();
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_overflow_set_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_set();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_oveflow_set_takes_2_cycles_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xBF;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_set();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_overflow_set_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xEF;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_overflow_set();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_overflow_set_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x40;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_overflow_set();
        assert_eq!(5, console.cpu.wait_counter);
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
    fn branch_if_carry_clear_branches_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x80;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_clear();
        // 0x21 as the instruction reads the offset, thus modifying the pc
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_clear_does_not_branch_and_updates_pc_correctly_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x43;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_clear();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_clear_takes_2_cycles_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xB9;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_clear();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_carry_clear_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_clear();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_carry_clear_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xB6;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_carry_clear();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn store_a_zero_page_stores_value_into_memory_correctly() {
        let mut console = create_test_console();
        console.cpu.a = 0x2f;
        console.cpu.program_counter = 0x32;
        console.memory.write(0x32, 0x14);
        console.store_a_zero_page();
        assert_eq!(0x2f, console.memory.read(0x14));
    }

    #[test]
    fn store_a_zero_page_does_not_affect_flags() {
        let mut console = create_test_console();
        console.cpu.a = 0x2f;
        console.cpu.program_counter = 0x32;
        console.memory.write(0x32, 0x14);
        console.cpu.status_flags = 0xE0;
        console.store_a_zero_page();
        assert_eq!(0xE0, console.cpu.status_flags);
    }
    #[test]
    fn store_a_zero_page_increments_pc_correctly() {
        let mut console = create_test_console();
        console.store_a_zero_page();
        assert_eq!(1, console.cpu.program_counter);
    }

    #[test]
    fn store_a_zero_page_takes_3_cycles() {
        let mut console = create_test_console();
        console.store_a_zero_page();
        assert_eq!(3, console.cpu.wait_counter);
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
    fn load_a_immediate_sets_a_to_the_value_given_in_next_byte() {
        let mut console = create_test_console();

        console.cpu.program_counter = 25;
        console.memory.write(25, 0x23);
        console.load_a_immediate();
        assert_eq!(0x23, console.cpu.a);
    }

    #[test]
    fn load_a_immediate_sets_negative_flag_and_does_not_touch_other_flags_if_value_is_negative() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x4C;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.load_a_immediate();
        assert_eq!(0xCC, console.cpu.status_flags);
    }

    #[test]
    fn load_a_immediate_sets_zero_flag_and_does_not_touch_other_flags_if_value_is_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x4C;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0);
        console.load_a_immediate();
        assert_eq!(0x4E, console.cpu.status_flags);
    }

    #[test]
    fn load_a_immediate_resets_zero_flag_when_value_is_negative() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x72;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.load_a_immediate();
        assert_eq!(0xF0, console.cpu.status_flags);
    }

    #[test]
    fn load_a_immediate_resets_negative_flag_when_value_is_zero() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xFC;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0);
        console.load_a_immediate();
        assert_eq!(0x7E, console.cpu.status_flags);
    }

    #[test]
    fn load_a_immediate_does_nothing_to_flags_with_negative_value_if_negative_flag_was_set_before() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xFC;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0xB1);
        console.load_a_immediate();
        assert_eq!(0xFC, console.cpu.status_flags);
    }

    #[test]
    fn load_a_immediate_does_nothing_to_flags_with_zero_value_if_zero_flag_was_set_before() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x43;
        console.cpu.program_counter = 25;
        console.memory.write(25, 0);
        console.load_a_immediate();
        assert_eq!(0x43, console.cpu.status_flags);
    }


    #[test]
    fn load_a_immediate_sets_wait_counter_correctly() {
        let mut console = create_test_console();
        console.load_a_immediate();
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
    fn branch_if_carry_set_branches_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x01;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_set();
        // 0x21 as the instruction reads the offset, thus modifying the pc
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_set_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_set();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_set_takes_2_cycles_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_set();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_carry_set_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x01;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_carry_set();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_carry_set_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x01;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_carry_set();
        assert_eq!(5, console.cpu.wait_counter);
    }

     #[test]
    fn compare_immediate_does_not_modify_accumulator() {
        let mut console = create_test_console();
        console.cpu.a = 0x40;
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0xABCD;
        console.memory.write(0xABCD, 0x20);
        console.compare_immediate();
        assert_eq!(0x40, console.cpu.a);
    }


    #[test]
    fn compare_immediate_sets_carry_flag_if_accumulator_is_greater_than_operand() {
        let mut console = create_test_console();
        console.cpu.a = 0x40;
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0xABCD;
        console.memory.write(0xABCD, 0x20);
        console.compare_immediate();
        assert_eq!(0x01, console.cpu.status_flags);
    }

    #[test]
    fn compare_immediate_sets_carry_and_zero_flags_if_accumulator_is_equal_operand() {
        let mut console = create_test_console();
        console.cpu.a = 0x40;
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0xABCD;
        console.memory.write(0xABCD, 0x40);
        console.compare_immediate();
        assert_eq!(0x03, console.cpu.status_flags);
    }

    #[test]
    fn compare_immediate_sets_negative_flag_if_accumulator_is_smaller_than_operand() {
        let mut console = create_test_console();
        console.cpu.a = 0x40;
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0xABCD;
        console.memory.write(0xABCD, 0x60);
        console.compare_immediate();
        assert_eq!(0x80, console.cpu.status_flags);
    }

    #[test]
    fn compare_immediate_increments_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0xABCD;
        console.compare_immediate();
        assert_eq!(0xABCD + 1, console.cpu.program_counter);
    }

    #[test]
    fn compare_immediate_takes_2_cycles() {
        let mut console = create_test_console();
        console.compare_immediate();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_not_equal_branches_if_zero_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD4;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_not_equal();
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_not_equal_does_not_branch_and_updates_pc_correctly_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x03;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_not_equal();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_not_equal_takes_2_cycles_if_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x02;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_not_equal();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_not_equal_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x01;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_not_equal();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_not_equal_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_not_equal();
        assert_eq!(5, console.cpu.wait_counter);
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
    #[test]
    fn no_operation_waits_2_cycles() {
        let mut console = create_test_console();
        console.no_operation();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_equal_branches_if_zero_flag_is_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0xD3;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_equal();
        assert_eq!(0x21 + 0x10, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_equal_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_equal();
        assert_eq!(0x21, console.cpu.program_counter);
    }

    #[test]
    fn branch_if_equal_takes_2_cycles_if_flag_is_not_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x00;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_equal();
        assert_eq!(2, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_equal_takes_3_cycles_if_branching_to_same_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x06;
        console.cpu.program_counter = 0x20;
        console.memory.write(0x20, 0x10);
        console.branch_if_equal();
        assert_eq!(3, console.cpu.wait_counter);
    }

    #[test]
    fn branch_if_equal_takes_5_cycles_if_branching_to_different_page() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x07;
        console.cpu.program_counter = 0xEF;
        console.memory.write(0xEF, 0x7F);
        console.branch_if_equal();
        assert_eq!(5, console.cpu.wait_counter);
    }

    #[test]
    fn set_decimal_flag_sets_the_flag_if_it_was_unset() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x07;
        console.set_decimal_flag();
        assert_eq!(0x0F, console.cpu.status_flags);
    }

    #[test]
    fn set_decimal_flag_does_nothing_if_flag_was_already_set() {
        let mut console = create_test_console();
        console.cpu.status_flags = 0x0A;
        console.set_decimal_flag();
        assert_eq!(0x0A, console.cpu.status_flags);
    }

    #[test]
    fn set_decimal_flag_does_not_touch_program_counter() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0xAB12;
        console.set_decimal_flag();
        assert_eq!(0xAB12, console.cpu.program_counter);
    }

    #[test]
    fn set_decimal_flag_takes_2_cycles() {
        let mut console = create_test_console();
        console.cpu.program_counter = 0xAB12;
        console.set_decimal_flag();
        assert_eq!(2, console.cpu.wait_counter);
    }
}
