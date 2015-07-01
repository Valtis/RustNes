
use rom::TvSystem;
use memory::Memory;
use std::rc::Rc;
use std::cell::RefCell;


#[derive(Debug)]
pub struct Cpu {
    memory: Rc<RefCell<Memory>>, // reference to memory, so that cpu can use it
    pub frequency: Frequency,
    pub program_counter:u16,
    pub stack_pointer:u8,
    pub wait_counter: u8, // used by instructions that take more than 1 cycle to complete
    pub status_flags:u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
}

impl Cpu {
    pub fn new(tv_system: &TvSystem, memory: Rc<RefCell<Memory>>) -> Cpu {
        Cpu {
            memory: memory,
            frequency: Frequency::new(&tv_system),
            program_counter: 0,
            stack_pointer: 0xFD,
            status_flags: 0x34, // unused 4 and 5 bits to 1; interrupt flag at 2 bit to 1
            wait_counter: 0,
            a: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn reset(&mut self) {
        /*self.program_counter = 0xFFFC;
        self.jump_absolute();*/
    }

    pub fn execute_instruction(&mut self) {

        let instruction = self.memory.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
        match instruction {
            1 => self.inclusive_or_indirect_x(),
            5 => self.inclusive_or_zero_page(),
            8 => self.push_status_flags_into_stack(),
            9 => self.inclusive_or_immediate(),
            13 => self.inclusive_or_absolute(),
            16 => self.branch_if_positive(),
            17 => self.inclusive_or_indirect_y(),
            21 => self.inclusive_or_zero_page_x(),
            24 => self.clear_carry_flag(),
            25 => self.inclusive_or_absolute_y(),
            29 => self.inclusive_or_absolute_x(),
            32 => self.jump_to_subroutine(),
            33 => self.and_indirect_x(),
            36 => self.bit_test_zero_page(),
            37 => self.and_zero_page(),
            40 => self.pull_status_flags_from_stack(),
            41 => self.and_immediate(),
            44 => self.bit_test_absolute(),
            45 => self.and_absolute(),
            48 => self.branch_if_negative(),
            49 => self.and_indirect_y(),
            53 => self.and_zero_page_x(),
            56 => self.set_carry_flag(),
            57 => self.and_absolute_y(),
            61 => self.and_absolute_x(),
            65 => self.exclusive_or_indirect_x(),
            69 => self.exclusive_or_zero_page(),
            72 => self.push_accumulator(),
            73 => self.exclusive_or_immediate(),
            76 => self.jump_absolute(),
            77 => self.exclusive_or_absolute(),
            80 => self.branch_if_overflow_clear(),
            81 => self.exclusive_or_indirect_y(),
            85 => self.exclusive_or_zero_page_x(),
            89 => self.exclusive_or_absolute_y(),
            93 => self.exclusive_or_absolute_x(),
            96 => self.return_from_subroutine(),
            97 => self.add_indirect_x(),
            101 => self.add_zero_page(),
            104 => self.pull_accumulator(),
            105 => self.add_immediate(),
            109 => self.add_absolute(),
            112 => self.branch_if_overflow_set(),
            113 => self.add_indirect_y(),
            117 => self.add_zero_page_x(),
            120 => self.set_interrupt_disable_flag(),
            121 => self.add_absolute_y(),
            125 => self.add_absolute_x(),
            129 => self.store_a_indirect_x(),
            133 => self.store_a_zero_page(),
            134 => self.store_x_zero_page(),
            141 => self.store_a_absolute(),
            142 => self.store_x_absolute(),
            144 => self.branch_if_carry_clear(),
            145 => self.store_a_indirect_y(),
            149 => self.store_a_zero_page_x(),
            150 => self.store_x_zero_page_y(),
            153 => self.store_a_absolute_y(),
            154 => self.transfer_x_to_stack_pointer(),
            157 => self.store_a_absolute_x(),
            161 => self.load_a_indirect_x(),
            162 => self.load_x_immediate(),
            165 => self.load_a_zero_page(),
            166 => self.load_x_zero_page(),
            169 => self.load_a_immediate(),
            173 => self.load_a_absolute(),
            174 => self.load_x_absolute(),
            176 => self.branch_if_carry_set(),
            177 => self.load_a_indirect_y(),
            181 => self.load_a_zero_page_x(),
            182 => self.load_x_zero_page_y(),
            184 => self.clear_overflow_flag(),
            185 => self.load_a_absolute_y(),
            189 => self.load_a_absolute_x(),
            190 => self.load_x_absolute_y(),
            193 => self.compare_indirect_x(),
            197 => self.compare_zero_page(),
            201 => self.compare_immediate(),
            205 => self.compare_absolute(),
            208 => self.branch_if_not_equal(),
            209 => self.compare_indirect_y(),
            213 => self.compare_zero_page_x(),
            216 => self.clear_decimal_flag(),
            217 => self.compare_absolute_y(),
            221 => self.compare_absolute_x(),
            234 => self.no_operation(),
            240 => self.branch_if_equal(),
            248 => self.set_decimal_flag(),
            _ => panic!("\n\nInvalid opcode {}\nInstruction PC: {}, \nCPU status: {:?}", instruction,
                self.program_counter - 1, self),
        }
    }
    fn set_negative_flag(&mut self, value: u8) {
        self.status_flags = (self.status_flags & 0x7F) | (value & 0x80);
    }

    fn set_zero_flag(&mut self, value: u8) {
        if value == 0 {
            // set zero flag
            self.status_flags = self.status_flags | 0x02;
        } else {
            // reset zero flag
            self.status_flags = self.status_flags & 0xFD;
        }
    }

    fn get_byte_operand(&mut self) -> u8 {
        let byte = self.memory.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
        byte
    }

    fn get_zero_page_address(&mut self) -> u16 {
        self.get_byte_operand() as u16
    }

    fn get_zero_page_address_with_offset(&mut self, offset: u16) -> u16 {
        (self.get_zero_page_address() + offset)  & 0x00FF
    }

    fn get_absolute_address(&mut self) -> u16 {
        let low_byte = self.memory.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
        let high_byte = self.memory.borrow_mut().read(self.program_counter);
        self.program_counter += 1;

         ((high_byte as u16) << 8) | low_byte as u16
    }

    fn get_absolute_address_with_offset(&mut self, offset: u16) -> u16 {
        self.get_absolute_address() + offset
    }

    fn get_indirect_x_address(&mut self) -> u16 {
        let zero_page_address = self.get_byte_operand() as u16;
        let low_byte = self.memory.borrow_mut().read((zero_page_address + self.x as u16) & 0x00FF) as u16;
        let high_byte = self.memory.borrow_mut().read((zero_page_address + self.x as u16 + 1) & 0x00FF) as u16;
        (high_byte << 8) | low_byte
    }

    fn get_indirect_y_address(&mut self) -> u16 {
        let zero_page_address =  self.get_byte_operand() as u16;

        let low_byte = self.memory.borrow_mut().read(zero_page_address) as u16;
        let high_byte = self.memory.borrow_mut().read((zero_page_address + 1) & 0x00FF) as u16;

        let base_address = (high_byte << 8) | low_byte;
        let four_byte_address = base_address as u32 + self.y as u32;

        (four_byte_address & 0xFFFF) as u16
    }

    fn read_immediate(&mut self) -> u8 {
        self.wait_counter = 2;
        self.get_byte_operand()
    }

    fn read_absolute(&mut self) -> u8 {
        self.wait_counter = 4;
        let address = self.get_absolute_address();
        self.memory.borrow_mut().read(address)
    }

    fn read_absolute_with_offset(&mut self, offset: u16) -> u8 {
        let base = self.get_absolute_address();
        let address = base + offset;
        // if page boundary is crossed, instruction takes 5 cycles. Otherwise it takes 4 cycles
        if base & 0xFF00 == address & 0xFF00 {
            self.wait_counter = 4;
        } else {
            self.wait_counter = 5;
        }
        self.memory.borrow_mut().read(address)
    }

    fn read_absolute_x(&mut self) -> u8 {
        let offset = self.x;
        self.read_absolute_with_offset(offset as u16)
    }

    fn read_absolute_y(&mut self) -> u8 {
        let offset = self.y;
        self.read_absolute_with_offset(offset as u16)
    }

    fn read_zero_page(&mut self) -> u8 {
        self.wait_counter = 3;
        let address = self.get_zero_page_address();
        self.memory.borrow_mut().read(address as u16)
    }

    fn read_zero_page_with_offset(&mut self, offset: u16) -> u8 {
        self.wait_counter = 4;
        let address = self.get_zero_page_address_with_offset(offset);
        self.memory.borrow_mut().read(address)
    }

    fn read_zero_page_x(&mut self) -> u8 {
        let offset = self.x as u16;
        self.read_zero_page_with_offset(offset)
    }

    fn read_zero_page_y(&mut self) -> u8 {
        let offset = self.y as u16;
        self.read_zero_page_with_offset(offset)
    }

    fn read_indirect_x(&mut self) -> u8 {
        self.wait_counter = 6;
        let address = self.get_indirect_x_address();
        self.memory.borrow_mut().read(address)
    }
    // duplicates get_indirect_y_address_code because timing depends on whether
    // the base address and final address are on the same page or not.
    // this probably should be fixed.
    fn read_indirect_y(&mut self) -> u8 {
        let zero_page_address =  self.get_byte_operand() as u16;

        let low_byte = self.memory.borrow_mut().read(zero_page_address) as u16;
        let high_byte = self.memory.borrow_mut().read((zero_page_address + 1) & 0x00FF) as u16;

        let base_address = (high_byte << 8) | low_byte;
        let four_byte_address =  base_address as u32 + self.y as u32;

        let final_address = (four_byte_address & 0xFFFF) as u16; // wrap around

        if final_address & 0xFF00 == base_address & 0xFF00 {
            self.wait_counter = 5;
        } else {
            self.wait_counter = 6;
        }

        self.memory.borrow_mut().read(final_address)
    }

    fn set_load_flags(&mut self, value: u8) {
        self.set_negative_flag(value);
        self.set_zero_flag(value);
    }

    fn load_a(&mut self, value: u8) {
        self.a = value;
        self.set_load_flags(value);
    }

    fn load_x(&mut self, value: u8) {
        self.x = value;
        self.set_load_flags(value);
    }

    fn load_y(&mut self, value: u8) {
        self.y = value;
        self.set_load_flags(value);
    }

    fn do_zero_page_store(&mut self, value: u8) {
        self.wait_counter = 3;
        let address = self.get_zero_page_address();
        self.memory.borrow_mut().write(address, value);
    }

    fn do_zero_page_x_store(&mut self, value: u8) {
        let offset = self.x as u16;
        self.wait_counter = 4;
        let address = self.get_zero_page_address_with_offset(offset);
        self.memory.borrow_mut().write(address, value);
    }

    fn do_zero_page_y_store(&mut self, value: u8) {
        let offset = self.y as u16;
        self.wait_counter = 4;
        let address = self.get_zero_page_address_with_offset(offset);
        self.memory.borrow_mut().write(address, value);
    }

    fn do_absolute_store(&mut self, value: u8) {
        self.wait_counter = 4;
        let address = self.get_absolute_address();
        self.memory.borrow_mut().write(address, value);
    }

    fn do_absolute_x_store(&mut self, value: u8) {
        self.wait_counter = 5;
        let offset = self.x as u16;
        let address = self.get_absolute_address_with_offset(offset);
        self.memory.borrow_mut().write(address, value);
    }

    fn do_absolute_y_store(&mut self, value: u8) {
        self.wait_counter = 5;
        let offset = self.y as u16;
        let address = self.get_absolute_address_with_offset(offset);
        self.memory.borrow_mut().write(address, value);
    }

    fn do_indirect_x_store(&mut self, value: u8) {
        self.wait_counter = 6;
        let address = self.get_indirect_x_address();
        self.memory.borrow_mut().write(address, value);
    }

    fn do_indirect_y_store(&mut self, value: u8) {
        self.wait_counter = 6;
        let address = self.get_indirect_y_address();
        self.memory.borrow_mut().write(address, value);
    }

    fn push_value_into_stack(&mut self, value: u8) {
        self.memory.borrow_mut().write(0x0100 + self.stack_pointer as u16, value);
        self.stack_pointer -= 1;
    }

    fn pop_value_from_stack(&mut self) -> u8 {
        self.stack_pointer += 1;
        self.memory.borrow_mut().read(0x0100 + self.stack_pointer as u16)
    }

    fn do_and(&mut self, operand: u8) {
        self.a = self.a & operand;
        let result = self.a;
        self.set_load_flags(result);
    }

    fn do_inclusive_or(&mut self, operand: u8) {
        self.a = self.a | operand;
        let result = self.a;
        self.set_load_flags(result);
    }

    fn do_exclusive_or(&mut self, operand: u8) {
        self.a = self.a ^ operand;
        let result = self.a;
        self.set_load_flags(result);
    }

    fn do_compare(&mut self, operand: u8) {
        // unset negative\zero\carry flags
        self.status_flags = self.status_flags & 0x7C;

        if operand > self.a {
            self.status_flags = self.status_flags | 0x80;
        } else if operand == self.a {
            self.status_flags = self.status_flags | 0x03;
        } else {
            self.status_flags = self.status_flags | 0x01;
        }
    }

    fn do_relative_jump_if(&mut self, condition: bool) {
        let offset = self.get_byte_operand() as u16;
        if  condition {
            let old_program_counter = self.program_counter;

            self.program_counter += offset;

            // the offset is signed 8 bit integer in two's complement. Thus if bit 7 is set,
            // we need to subtract 0x100 from the counter to get the correct value
            if offset & 0x80 != 0 {
                self.program_counter -= 0x100;
            }

            // timing depends on whether new address is on same or different memory page
            if old_program_counter & 0xFF00 == self.program_counter & 0xFF00 {
                self.wait_counter = 3;
            } else {
                self.wait_counter = 5;
            }
        } else {
            self.wait_counter = 2;
        }
    }

    fn do_bit_test(&mut self, operand: u8) {
        let result = self.a & operand;
        // set overflow and negative flags to correct values, unset zero flag
        self.status_flags = (self.status_flags & 0x3D) | (operand & 0xC0);
        self.set_zero_flag(result);
    }

    fn do_add(&mut self, operand: u8) {
        let mut result = self.a as u16 + operand as u16;

        // if carry is set, add 1 to result
        if self.status_flags & 0x01 == 0x01 {
            result += 1;
        }

        // clear carry, negative, overflow and zero flags
        self.status_flags = self.status_flags & 0x3C;

        // set zero flag if result is zero
        // important: do before casting to u8; if bit 9 is set
        // result is not considered to be zero
        if result == 0 {
            self.status_flags = self.status_flags | 0x02;
        }

        // if result is greater than 255, set carry flag
        if result > 255 {
            self.status_flags = self.status_flags | 0x01;
        }

        // overflow can only happen when adding two positive or two negative numbers
        // not when adding positive and negative. Therefore, if both operands have
        // same sign bit but sign bit is different than the result has, overflow
        // has happened. Thus xor both operands (a and func argument) with result
        // and mask it with 0x80. If result is nonzero, overflow has happened.
        if (operand as u16 ^ result) & (self.a as u16 ^ result) & 0x80 != 0 {
            self.status_flags = self.status_flags | 0x40;
        }

        // finally set negative flag if necessary
        self.set_negative_flag(result as u8);

        self.a = result as u8;
    }

    fn and_immediate(&mut self) {
        let operand = self.read_immediate();
        self.do_and(operand);
    }

    fn and_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.do_and(value);
    }

    fn and_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        self.do_and(value);
    }

    fn and_absolute(&mut self) {
        let value = self.read_absolute();
        self.do_and(value);
    }

    fn and_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        self.do_and(value);
    }

    fn and_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        self.do_and(value);
    }

    fn and_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        self.do_and(value);
    }

    fn and_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        self.do_and(value);
    }

    fn inclusive_or_immediate(&mut self) {
        let value = self.read_immediate();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_absolute(&mut self) {
        let value = self.read_absolute();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        self.do_inclusive_or(value);
    }

    fn inclusive_or_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        self.do_inclusive_or(value);
    }

    fn exclusive_or_immediate(&mut self) {
        let value = self.read_immediate();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_absolute(&mut self) {
        let value = self.read_absolute();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        self.do_exclusive_or(value);
    }

    fn exclusive_or_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        self.do_exclusive_or(value);
    }

    fn branch_if_carry_clear(&mut self) {
        let condition = self.status_flags & 0x01 == 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_carry_set(&mut self) {
        let condition = self.status_flags & 0x01 != 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_equal(&mut self) {
        let condition = self.status_flags & 0x02 != 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_not_equal(&mut self) {
        let condition = self.status_flags & 0x02 == 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_negative(&mut self) {
        let condition = self.status_flags & 0x80 != 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_positive(&mut self) {
        let condition = self.status_flags & 0x80 == 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_overflow_clear(&mut self) {
        let condition = self.status_flags & 0x40 == 0;
        self.do_relative_jump_if(condition);
    }

    fn branch_if_overflow_set(&mut self) {
        let condition = self.status_flags & 0x40 != 0;
        self.do_relative_jump_if(condition);
    }

    fn jump_absolute(&mut self) {
        self.wait_counter = 3;
        self.program_counter = self.get_absolute_address();
    }

    fn jump_to_subroutine(&mut self) {
        self.wait_counter = 6;
        let address = self.get_absolute_address();

        let return_address = self.program_counter - 1;
        self.push_value_into_stack(((return_address & 0xFF00) >> 8) as u8);
        self.push_value_into_stack((return_address & 0xFF) as u8);
        self.program_counter = address;
    }

    fn return_from_subroutine(&mut self) {
        self.wait_counter = 6;
        let low_byte = self.pop_value_from_stack() as u16;
        let high_byte = self.pop_value_from_stack() as u16;
        self.program_counter = ((high_byte << 8) | low_byte) + 1;
    }

    fn bit_test_zero_page(&mut self) {
        let operand = self.read_zero_page();
        self.do_bit_test(operand);
    }

    fn bit_test_absolute(&mut self) {
        let operand = self.read_absolute();
        self.do_bit_test(operand);
    }

    fn clear_carry_flag(&mut self) {
        self.wait_counter = 2;
        self.status_flags = self.status_flags & 0xFE; // clear bi 0
    }

    fn set_carry_flag(&mut self) {
        self.wait_counter = 2;
        self.status_flags = self.status_flags | 0x01;
    }

    fn clear_decimal_flag(&mut self) {
        self.wait_counter = 2;
        self.status_flags = self.status_flags & 0xF7; // clear bit 3
    }

    fn set_decimal_flag(&mut self) {
        self.wait_counter = 2;
        self.status_flags = self.status_flags | 0x08; // set bit 3
    }

    fn set_interrupt_disable_flag(&mut self) {
        self.wait_counter = 2;
        self.status_flags = self.status_flags | 0x04; // set bit 2
    }

    fn clear_overflow_flag(&mut self) {
        self.wait_counter = 2;
        self.status_flags = self.status_flags & 0xBF;
    }

    fn push_accumulator(&mut self) {
        self.wait_counter = 3;
        let value = self.a;
        self.push_value_into_stack(value);
    }

    fn pull_accumulator(&mut self) {
        self.wait_counter = 4;
        let value = self.pop_value_from_stack();
        self.a = value;
        self.set_load_flags(value);
    }

    fn push_status_flags_into_stack(&mut self) {
        // This instruction sets bits 4 & 5 to 1 for the value that gets pushed into stack.
        // In contrast, irq or nmi will set bit 4 to 0.
        self.wait_counter = 3;
        let flags = self.status_flags | 0x30;
        self.push_value_into_stack(flags);
    }

    fn pull_status_flags_from_stack(&mut self) {
        self.wait_counter = 4;
        self.status_flags = self.pop_value_from_stack() | 0x30;
    }

    fn load_a_immediate(&mut self) {
        let value = self.read_immediate();
        self.load_a(value);
    }

    fn load_a_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.load_a(value);
    }

    fn load_a_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        self.load_a(value);
    }

    fn load_a_absolute(&mut self) {
        let value = self.read_absolute();
        self.load_a(value);
    }

    fn load_a_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        self.load_a(value);
    }

    fn load_a_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        self.load_a(value);
    }

    fn load_a_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        self.load_a(value);
    }

    fn load_a_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        self.load_a(value);
    }

    fn store_a_zero_page(&mut self) {
        let value = self.a;
        self.do_zero_page_store(value);
    }

    fn store_a_zero_page_x(&mut self) {
        let value = self.a;
        self.do_zero_page_x_store(value);
    }

    fn store_a_absolute(&mut self) {
        let value = self.a;
        self.do_absolute_store(value);
    }

    fn store_a_absolute_x(&mut self) {
        let value = self.a;
        self.do_absolute_x_store(value);
    }

    fn store_a_absolute_y(&mut self) {
        let value = self.a;
        self.do_absolute_y_store(value);
    }

    fn store_a_indirect_x(&mut self) {
        let value = self.a;
        self.do_indirect_x_store(value);
    }

    fn store_a_indirect_y(&mut self) {
        let value = self.a;
        self.do_indirect_y_store(value);
    }

    fn load_x_immediate(&mut self) {
        let value = self.read_immediate();
        self.load_x(value);
    }

    fn load_x_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.load_x(value);
    }

    fn load_x_zero_page_y(&mut self) {
        let value = self.read_zero_page_y();
        self.load_x(value);
    }

    fn load_x_absolute(&mut self) {
        let value = self.read_absolute();
        self.load_x(value);
    }

    fn load_x_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        self.load_x(value);
    }

    fn store_x_zero_page(&mut self) {
        let value = self.x;
        self.do_zero_page_store(value);
    }

    fn store_x_zero_page_y(&mut self) {
        let value = self.x;
        self.do_zero_page_y_store(value);
    }

    fn store_x_absolute(&mut self) {
        let value = self.x;
        self.do_absolute_store(value);
    }

    fn transfer_x_to_stack_pointer(&mut self) {
        self.wait_counter = 2;
        self.stack_pointer = self.x;
    }

    fn compare_immediate(&mut self) {
        let operand = self.read_immediate();
        self.do_compare(operand);
    }

    fn compare_zero_page(&mut self) {
        let operand = self.read_zero_page();
        self.do_compare(operand);
    }

    fn compare_zero_page_x(&mut self) {
        let operand = self.read_zero_page_x();
        self.do_compare(operand);
    }

    fn compare_absolute(&mut self) {
        let operand = self.read_absolute();
        self.do_compare(operand);
    }

    fn compare_absolute_x(&mut self) {
        let operand = self.read_absolute_x();
        self.do_compare(operand);
    }

    fn compare_absolute_y(&mut self) {
        let operand = self.read_absolute_y();
        self.do_compare(operand);
    }

    fn compare_indirect_x(&mut self) {
        let operand = self.read_indirect_x();
        self.do_compare(operand);
    }

    fn compare_indirect_y(&mut self) {
        let operand = self.read_indirect_y();
        self.do_compare(operand);
    }

    fn add_immediate(&mut self) {
        let operand = self.read_immediate();
        self.do_add(operand);
    }

    fn add_zero_page(&mut self) {
        let operand = self.read_zero_page();
        self.do_add(operand);
    }

    fn add_zero_page_x(&mut self) {
        let operand = self.read_zero_page_x();
        self.do_add(operand);
    }

    fn add_absolute(&mut self) {
        let operand = self.read_absolute();
        self.do_add(operand);
    }

    fn add_absolute_x(&mut self) {
        let operand = self.read_absolute_x();
        self.do_add(operand);
    }

    fn add_absolute_y(&mut self) {
        let operand = self.read_absolute_y();
        self.do_add(operand);
    }

    fn add_indirect_x(&mut self) {
        let operand = self.read_indirect_x();
        self.do_add(operand);
    }

    fn add_indirect_y(&mut self) {
        let operand = self.read_indirect_y();
        self.do_add(operand);
    }

    fn no_operation(&mut self) {
        self.wait_counter = 2;
    }
}
#[derive(Debug)]
pub struct Frequency {
    color_subcarrier_frequency: f64,
    master_clock_frequency: f64,
    clock_divisor: u8,
    pub cpu_clock_frequency: f64
}


impl Frequency {
    fn new(tv_system: &TvSystem) -> Frequency {

        let mut divisor:u8;
        let mut color_freq:f64;
        let mut master_freq:f64;

        match *tv_system {
            TvSystem::Uninitialized => panic!("Uninitialized tv system type when initializing cpu"),
            TvSystem::PAL => {
                divisor = 16;
                color_freq = 4433618.75 / 1000_000.0;
            },
            TvSystem::NTSC => {
                divisor = 12;
                color_freq = 39375000.0/11.0 / 1000_000.0;
            }
        }

        master_freq = 6.0*color_freq;

        Frequency {
            color_subcarrier_frequency: color_freq,
            master_clock_frequency: master_freq,
            clock_divisor: divisor,
            cpu_clock_frequency: master_freq / divisor as f64
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use memory::Memory;
    use rom::TvSystem;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn create_test_cpu() -> Cpu {
        let memory = Rc::new(RefCell::new(Memory::new()));
        Cpu::new(&TvSystem::NTSC, memory)
    }

    #[test]
    fn set_negative_flag_sets_the_flag_if_flag_value_is_negative_and_flag_was_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.set_negative_flag(0xFF);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn set_negative_flag_does_nothing_if_value_is_negative_and_flag_was_already_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD5;
        cpu.set_negative_flag(0xFF);
        assert_eq!(0xD5, cpu.status_flags);
    }

    #[test]
    fn set_negative_flag_unsets_the_flag_if_flag_is_set_and_value_was_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD0;
        cpu.set_negative_flag(0x05);
        assert_eq!(0x50, cpu.status_flags);
    }

    #[test]
    fn set_negative_flag_does_nothing_if_flag_is_unset_and_value_is_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x7F;
        cpu.set_negative_flag(0x7F);
        assert_eq!(0x7F, cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_sets_the_flag_if_flag_value_is_zero_and_flag_was_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.set_zero_flag(0);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_does_nothing_if_value_is_zero_and_flag_was_already_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD3;
        cpu.set_zero_flag(0);
        assert_eq!(0xD3, cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_unsets_the_flag_if_flag_is_set_and_value_was_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xDF;
        cpu.set_zero_flag(0x05);
        assert_eq!(0xDD, cpu.status_flags);
    }

    #[test]
    fn set_zero_flag_does_nothing_if_flag_is_unset_and_value_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x70;
        cpu.set_zero_flag(0xFF);
        assert_eq!(0x70, cpu.status_flags);
    }

    #[test]
    fn get_byte_operand_gets_correct_value_and_updates_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 24;
        cpu.memory.borrow_mut().write(24, 0xAD);
        assert_eq!(0xAD, cpu.get_byte_operand());
        assert_eq!(25, cpu.program_counter);
    }

    #[test]
    fn push_value_to_stack_pushes_value_into_stack() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0xFF;
        cpu.push_value_into_stack(23);
        assert_eq!(23, cpu.memory.borrow_mut().read(0x01FF));
    }

    #[test]
    fn push_value_to_stack_updates_stack_pointer() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0xFF;
        cpu.push_value_into_stack(23);
        assert_eq!(0xFE, cpu.stack_pointer);
    }

    #[test]
    fn pop_value_from_stack_returns_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0xCC;
        cpu.memory.borrow_mut().write(0x0100 + 0xCD, 123);
        assert_eq!(123, cpu.pop_value_from_stack());
    }

    #[test]
    fn pop_value_from_stack_updates_stack_pointer() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0xCC;
        cpu.pop_value_from_stack();
        assert_eq!(0xCD, cpu.stack_pointer);
    }


    #[test]
    fn set_load_flags_sets_negative_flag_if_bit_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.set_load_flags(0x80);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn set_load_flags_unsets_negative_flag_if_bit_is_unset() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.set_load_flags(0x40);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn set_load_flags_set_zero_flag_if_value_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.set_load_flags(0x00);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn set_load_flags_unsets_zero_flag_if_value_is_nonzero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.set_load_flags(0x04);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn load_a_sets_a_value() {
        let mut cpu = create_test_cpu();
        cpu.load_a(0x50);
        assert_eq!(0x50, cpu.a);
    }

    #[test]
    fn load_x_sets_x_value() {
        let mut cpu = create_test_cpu();

        cpu.load_x(0x50);
        assert_eq!(0x50, cpu.x);
    }

    #[test]
    fn load_y_sets_y_value() {
        let mut cpu = create_test_cpu();
        cpu.load_y(0x50);
        assert_eq!(0x50, cpu.y);
    }

    #[test]
    fn get_zero_page_address_returns_correct_address() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.memory.borrow_mut().write(0x243, 0xAF);
        assert_eq!(0x00AF, cpu.get_zero_page_address());
    }

    #[test]
    fn get_zero_page_address_with_offset_returns_correct_address_when_value_does_not_wrap_around() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.memory.borrow_mut().write(0x243, 0xAF);
        assert_eq!(0x00AF + 0x12, cpu.get_zero_page_address_with_offset(0x12));
    }

    #[test]
    fn get_zero_page_address_with_offset_returns_correct_address_when_value_wraps_around() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.memory.borrow_mut().write(0x243, 0xFF);
        assert_eq!(0x0011, cpu.get_zero_page_address_with_offset(0x12));
    }

    #[test]
    fn get_absolute_address_returns_correct_address() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.memory.borrow_mut().write(0x243, 0xBE);
        cpu.memory.borrow_mut().write(0x244, 0xBA);
        assert_eq!(0xBABE, cpu.get_absolute_address());
    }

    #[test]
    fn get_absolute_address_with_offset_returns_correct_address() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.memory.borrow_mut().write(0x243, 0xBE);
        cpu.memory.borrow_mut().write(0x244, 0xBA);
        assert_eq!(0xBABE + 0x43, cpu.get_absolute_address_with_offset(0x43));
    }

    #[test]
    fn get_indirect_x_address_returns_correct_address() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.x = 0x25;
        cpu.memory.borrow_mut().write(0x243, 0xBE);

        cpu.memory.borrow_mut().write(0xBE + 0x25 , 0xBA);
        cpu.memory.borrow_mut().write(0xBE + 0x25 + 1, 0xAF);

        assert_eq!(0xAFBA, cpu.get_indirect_x_address());
    }

    #[test]
    fn get_indirect_x_address_returns_correct_address_if_zero_page_address_wraps_around() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.x = 0x1;
        cpu.memory.borrow_mut().write(0x243, 0xFE);

        cpu.memory.borrow_mut().write(0xFF, 0xBA);
        cpu.memory.borrow_mut().write(0x00, 0xAF);

        assert_eq!(0xAFBA, cpu.get_indirect_x_address());
    }

    #[test]
    fn get_indirect_y_address_returns_correct_address() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.y = 0x25;
        cpu.memory.borrow_mut().write(0x243, 0xBE);

        cpu.memory.borrow_mut().write(0xBE , 0xBA);
        cpu.memory.borrow_mut().write(0xBE + 1, 0xAF);

        assert_eq!(0xAFBA + 0x25, cpu.get_indirect_y_address());
    }

    #[test]
    fn get_indirect_y_address_returns_correct_address_if_zero_page_part_wraps_around() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.y = 0x25;
        cpu.memory.borrow_mut().write(0x243, 0xFF);

        cpu.memory.borrow_mut().write(0xFF, 0xBA);
        cpu.memory.borrow_mut().write(0x00, 0xAF);

        assert_eq!(0xAFBA + 0x25, cpu.get_indirect_y_address());
    }

    #[test]
    fn get_indirect_y_address_returns_correct_address_if_main_address_wraps_around() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.y = 0x25;
        cpu.memory.borrow_mut().write(0x243, 0xBE);

        cpu.memory.borrow_mut().write(0xBE , 0xFF);
        cpu.memory.borrow_mut().write(0xBE + 1, 0xFF);

        assert_eq!(0x0024, cpu.get_indirect_y_address());
    }

    #[test]
    fn read_immediate_returns_value_pointed_by_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFA);
        assert_eq!(0xFA, cpu.read_immediate());
    }

    #[test]
    fn read_immediate_sets_wait_counter_to_2() {
        let mut cpu = create_test_cpu();
        cpu.read_immediate();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn read_absolute_returns_value_pointed_by_address_at_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFA);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.memory.borrow_mut().write(0xE0FA, 0x52);
        assert_eq!(0x52, cpu.read_absolute());
    }

    #[test]
    fn read_absolute_sets_wait_counter_to_4() {
        let mut cpu = create_test_cpu();
        cpu.read_absolute();
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn read_absolute_with_offset_return_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFF);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.memory.borrow_mut().write(0xE100, 0xC5);
        assert_eq!(0xC5, cpu.read_absolute_with_offset(0x01));
    }

    #[test]
    fn read_absolute_with_offset_takes_4_cycles_if_page_boundary_is_not_crossed() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x00);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.read_absolute_with_offset(0xFA);
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn read_absolute_with_offset_takes_5_cycles_if_page_boundary_is_barely_crossed() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFF);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.read_absolute_with_offset(0x01);
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn read_absolute_with_offset_takes_5_cycles_if_page_boundary_is_crossed() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFA);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.read_absolute_with_offset(0xFE);
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn read_absolute_x_returns_value_pointed_by_16_bit_address_pointed_by_pc_and_x_register() {
        let mut cpu = create_test_cpu();
        cpu.x = 0xFA;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFA);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.memory.borrow_mut().write(0xE0FA + 0x00FA, 0x52);
        assert_eq!(0x52, cpu.read_absolute_x());
    }


    #[test]
    fn read_absolute_y_returns_value_pointed_by_16_bit_address_pointed_by_pc_and_y_register() {
        let mut cpu = create_test_cpu();
        cpu.y = 0xFA;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFA);
        cpu.memory.borrow_mut().write(0x433, 0xE0);
        cpu.memory.borrow_mut().write(0xE0FA + 0x00FA, 0x52);
        assert_eq!(0x52, cpu.read_absolute_y());
    }

    #[test]
    fn read_zero_page_returns_value_at_zero_page_pointed_by_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFA);
        cpu.memory.borrow_mut().write(0x00FA, 0xAE);
        assert_eq!(0xAE, cpu.read_zero_page());
    }

    #[test]
    fn read_zero_page_sets_wait_counter_to_3() {
        let mut cpu = create_test_cpu();
        cpu.read_zero_page();
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn read_zero_page_with_offset_returns_value_at_zero_page_with_offset() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);
        cpu.memory.borrow_mut().write(0x008F, 0xAE);
        assert_eq!(0xAE, cpu.read_zero_page_with_offset(0x0F));
    }

    #[test]
    fn read_zero_page_x_handles_wrap_around() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);
        cpu.memory.borrow_mut().write(0x007F, 0xAE);
        assert_eq!(0xAE, cpu.read_zero_page_with_offset(0xFF));
    }

    #[test]
    fn read_zero_page_with_offset_sets_wait_counter_to_4() {
        let mut cpu = create_test_cpu();
        cpu.read_zero_page_with_offset(0x00);
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn read_zero_page_x_returns_value_at_zero_page_pointed_by_program_counter_indexed_with_x() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x0F;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);
        cpu.memory.borrow_mut().write(0x008F, 0xAE);
        assert_eq!(0xAE, cpu.read_zero_page_x());
    }

    #[test]
    fn read_zero_page_y_returns_value_at_zero_page_pointed_by_program_counter_indexed_with_y() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x0F;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);
        cpu.memory.borrow_mut().write(0x008F, 0xAE);
        assert_eq!(0xAE, cpu.read_zero_page_y());
    }


    #[test]
    fn read_indirect_x_returns_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);

        cpu.memory.borrow_mut().write(0x80+0x04, 0x80);
        cpu.memory.borrow_mut().write(0x80+0x05, 0xAF);

        cpu.memory.borrow_mut().write(0xAF80, 0xAE);

        assert_eq!(0xAE, cpu.read_indirect_x());
    }

    #[test]
    fn read_indirect_x_wraps_zero_page_address_around() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFE);

        cpu.memory.borrow_mut().write(0x02, 0x80);
        cpu.memory.borrow_mut().write(0x03, 0xAF);

        cpu.memory.borrow_mut().write(0xAF80, 0xAE);

        assert_eq!(0xAE, cpu.read_indirect_x());
    }

    #[test]
    fn read_indirect_x_sets_wait_counter_to_6() {
        let mut cpu = create_test_cpu();
        cpu.read_indirect_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn read_indirect_y_returns_correct_value() {

        let mut cpu = create_test_cpu();
        cpu.y = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);

        cpu.memory.borrow_mut().write(0x80, 0x80);
        cpu.memory.borrow_mut().write(0x81, 0xAF);

        cpu.memory.borrow_mut().write(0xAF80 + 0x04, 0xAE);

        assert_eq!(0xAE, cpu.read_indirect_y());
    }

    #[test]
    fn read_indirect_y_wraps_zero_page_address_around() {

        let mut cpu = create_test_cpu();
        cpu.y = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFF);

        cpu.memory.borrow_mut().write(0xFF, 0xFF);
        cpu.memory.borrow_mut().write(0x00, 0xAB);

        cpu.memory.borrow_mut().write(0x0ABFF + 0x04, 0xAE);

        assert_eq!(0xAE, cpu.read_indirect_y());
    }

    #[test]
    fn read_indirect_y_wraps_final_address_around() {

        let mut cpu = create_test_cpu();
        cpu.y = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);

        cpu.memory.borrow_mut().write(0x80, 0xFF);
        cpu.memory.borrow_mut().write(0x81, 0xFF);

        cpu.memory.borrow_mut().write(0x0003, 0xAE);

        assert_eq!(0xAE, cpu.read_indirect_y());
    }

    #[test]
    fn read_indirect_y_takes_5_cycles_if_no_page_boundary_is_crossed() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);

        cpu.memory.borrow_mut().write(0x80, 0x80);
        cpu.memory.borrow_mut().write(0x81, 0xAF);
        cpu.read_indirect_y();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn read_indirect_takes_6_cycles_if_page_boundary_is_crossed() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x04;
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0x80);

        cpu.memory.borrow_mut().write(0x80, 0xFE);
        cpu.memory.borrow_mut().write(0x81, 0xAF);
        cpu.read_indirect_y();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn do_zero_page_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.do_zero_page_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x14));
    }

    #[test]
    fn  do_zero_page_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_zero_page_store(0x12);
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn  do_zero_page_store_takes_3_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_zero_page_store(0x12);
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn do_zero_page_x_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x24;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.do_zero_page_x_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x14 + 0x24));
    }

    #[test]
    fn  do_zero_page_x_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_zero_page_x_store(0x12);
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn  do_zero_page_x_store_takes_4_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_zero_page_x_store(0x12);
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn do_zero_page_y_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x24;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.do_zero_page_y_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x14 + 0x24));
    }

    #[test]
    fn do_zero_page_y_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_zero_page_y_store(0x12);
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn do_zero_page_y_store_takes_4_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_zero_page_y_store(0x12);
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn do_absolute_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x21);
        cpu.memory.borrow_mut().write(0x33, 0x18);

        cpu.do_absolute_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x1821));
    }

    #[test]
    fn do_absolute_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_absolute_store(0x12);
        assert_eq!(2, cpu.program_counter);
    }

    #[test]
    fn do_absolute_store_takes_4_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_absolute_store(0x12);
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn do_absolute_x_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x25;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x21);
        cpu.memory.borrow_mut().write(0x33, 0x18);

        cpu.do_absolute_x_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x1821 + 0x25));
    }

    #[test]
    fn do_absolute_x_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_absolute_x_store(0x12);
        assert_eq!(2, cpu.program_counter);
    }

    #[test]
    fn do_absolute_x_store_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_absolute_x_store(0x12);
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn do_absolute_y_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x25;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x21);
        cpu.memory.borrow_mut().write(0x33, 0x18);

        cpu.do_absolute_y_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x1821 + 0x25));
    }

    #[test]
    fn do_absolute_y_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_absolute_y_store(0x12);
        assert_eq!(2, cpu.program_counter);
    }

    #[test]
    fn do_absolute_y_store_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_absolute_y_store(0x12);
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn do_indirect_x_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x25;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x04);

        cpu.memory.borrow_mut().write(0x04 + 0x25, 0x18);
        cpu.memory.borrow_mut().write(0x04 + 0x25 + 1, 0x0B);

        cpu.do_indirect_x_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x0B18));
    }

    #[test]
    fn do_indirect_x_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_indirect_x_store(0x12);
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn do_indirect_x_store_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_indirect_x_store(0x12);
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn do_indirect_y_store_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x25;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x04);

        cpu.memory.borrow_mut().write(0x04, 0x18);
        cpu.memory.borrow_mut().write(0x04 + 1, 0x0B);

        cpu.do_indirect_y_store(0x2F);
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x0B18 + 0x25));
    }

    #[test]
    fn do_indirect_y_store_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.do_indirect_y_store(0x12);
        assert_eq!(1, cpu.program_counter);
    }

    #[test]
    fn do_indirect_y_store_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.do_indirect_y_store(0x12);
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn do_and_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.do_and(0x3E);
        assert_eq!(0x28, cpu.a);
    }

    #[test]
    fn do_and_unsets_zero_flag_if_it_was_set_before_and_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.status_flags = 0x02;
        cpu.do_and(0x3E);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_zero_flag_if_it_was_not_set_before_and_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.status_flags = 0x00;
        cpu.do_and(0x3E);
        assert_eq!(0x00, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_and_sets_zero_flag_if_result_is_zero_and_flag_was_not_set_before() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.status_flags = 0x00;
        cpu.do_and(0x3E);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_zero_flag_if_flag_is_set_and_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.status_flags = 0x02;
        cpu.do_and(0x3E);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_and_sets_negative_flag_if_result_is_negative_and_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x80;
        cpu.status_flags = 0x00;
        cpu.do_and(0xFF);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_negative_flag_if_it_is_set_and_number_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x80;
        cpu.status_flags = 0xA1;
        cpu.do_and(0xFF);
        assert_eq!(0xA1, cpu.status_flags);
    }

    #[test]
    fn do_and_unsets_negative_flag_if_flag_is_set_and_number_is_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x80;
        cpu.status_flags = 0xAF;
        cpu.do_and(0x7F);
        assert_eq!(0x2F, cpu.status_flags);
    }

    #[test]
    fn do_and_does_nothing_to_negative_flag_if_it_is_unset_and_number_is_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x80;
        cpu.status_flags = 0x3F;
        cpu.do_and(0x7F);
        assert_eq!(0x3F, cpu.status_flags);
    }

    #[test]
    fn do_and_does_not_touch_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x52;
        cpu.do_and(0xFF);
        assert_eq!(0x52, cpu.program_counter);
    }

    #[test]
    fn do_and_does_not_modify_wait_counter() {
        let mut cpu = create_test_cpu();
        cpu.do_and(0x02);
        assert_eq!(0, cpu.wait_counter);
    }

    #[test]
    fn do_inclusive_or_sets_accumulator_value_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x23;
        cpu.do_inclusive_or(0x5D);
        assert_eq!(0x7F, cpu.a);
    }

    #[test]
    fn do_inclusive_or_sets_negative_flag_if_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 0x00;
        cpu.do_inclusive_or(0x80);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_inclusive_or_unsets_negative_flag_if_result_is_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.a = 0x00;
        cpu.do_inclusive_or(0x70);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_inclusive_or_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 0x00;
        cpu.do_inclusive_or(0x00);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_inclusive_or_unsets_zero_flag_if_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.a = 0x40;
        cpu.do_inclusive_or(0x00);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_inclusive_or_does_not_touch_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x52;
        cpu.do_inclusive_or(0xFF);
        assert_eq!(0x52, cpu.program_counter);
    }

    #[test]
    fn do_inclusive_or_does_not_modify_wait_counter() {
        let mut cpu = create_test_cpu();
        cpu.do_inclusive_or(0x02);
        assert_eq!(0, cpu.wait_counter);
    }

    #[test]
    fn do_exclusive_or_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.do_exclusive_or(0x3E);
        assert_eq!(0xD7, cpu.a);
    }

    #[test]
    fn do_exclusive_or_unsets_zero_flag_if_it_was_set_before_and_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x09;
        cpu.status_flags = 0x02;
        cpu.do_exclusive_or(0x3E);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_does_nothing_to_zero_flag_if_it_was_not_set_before_and_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x29;
        cpu.status_flags = 0x00;
        cpu.do_exclusive_or(0x3E);
        assert_eq!(0x00, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_exclusive_or_sets_zero_flag_if_result_is_zero_and_flag_was_not_set_before() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xFA;
        cpu.status_flags = 0x00;
        cpu.do_exclusive_or(0xFA);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_does_nothing_to_zero_flag_if_flag_is_set_and_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x3E;
        cpu.status_flags = 0x02;
        cpu.do_exclusive_or(0x3E);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_sets_negative_flag_if_result_is_negative_and_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x72;
        cpu.status_flags = 0x00;
        cpu.do_exclusive_or(0xFF);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_does_nothing_to_negative_flag_if_it_is_set_and_number_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x72;
        cpu.status_flags = 0xA1;
        cpu.do_exclusive_or(0xFF);
        assert_eq!(0xA1, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_unsets_negative_flag_if_flag_is_set_and_number_is_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x8F;
        cpu.status_flags = 0xA0;
        cpu.do_exclusive_or(0x82);
        assert_eq!(0x20, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_does_nothing_to_negative_flag_if_it_is_unset_and_number_is_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x8A;
        cpu.status_flags = 0x39;
        cpu.do_exclusive_or(0xF9);
        assert_eq!(0x39, cpu.status_flags);
    }

    #[test]
    fn do_exclusive_or_does_not_touch_program_counter_() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x52;
        cpu.do_exclusive_or(0xFF);
        assert_eq!(0x52, cpu.program_counter);
    }

    #[test]
    fn do_exclusive_or_does_not_modify_wait_counter() {
        let mut cpu = create_test_cpu();
        cpu.do_exclusive_or(0x02);
        assert_eq!(0, cpu.wait_counter);
    }

    #[test]
    fn do_compare_does_not_modify_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x40;
        cpu.status_flags = 0x00;
        cpu.do_compare(0x20);
        assert_eq!(0x40, cpu.a);
    }


    #[test]
    fn do_compare_sets_carry_flag_if_accumulator_is_greater_than_operand() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x40;
        cpu.status_flags = 0x00;
        cpu.do_compare(0x20);
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn do_compare_sets_carry_and_zero_flags_if_accumulator_is_equal_operand() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x40;
        cpu.status_flags = 0x00;
        cpu.do_compare(0x40);
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn do_compare_sets_negative_flag_if_accumulator_is_smaller_than_operand() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x40;
        cpu.status_flags = 0x00;
        cpu.do_compare(0x60);
        assert_eq!(0x80, cpu.status_flags);
    }

    // found a bug, introduced test case triggering it
    #[test]
    fn do_compare_unsets_negative_flag_and_sets_carry_and_zero_flags_if_result_is_equal_and_negative_was_set_before() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xFF;
        cpu.status_flags = 0xEC;
        cpu.do_compare(0xFF);
        assert_eq!(0x6F, cpu.status_flags);
    }

    // testing similar case as above
    #[test]
    fn do_compare_unsets_zero_and_carry_flags_and_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x20;
        cpu.status_flags = 0x7F;
        cpu.do_compare(0xFF);
        assert_eq!(0xFC, cpu.status_flags);
    }

    #[test]
    fn do_compare_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABCD;
        cpu.do_compare(0x13);
        assert_eq!(0xABCD, cpu.program_counter);
    }


    #[test]
    fn do_relative_jump_if_jumps_if_condition_is_true() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD3;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.do_relative_jump_if(true);
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn do_relative_jump_if_does_not_jump_if_condition_is_false() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD3;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.do_relative_jump_if(false);
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn do_relative_jump_if_can_jump_backwards() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 24;
        cpu.memory.borrow_mut().write(24, 0xFC);
        cpu.do_relative_jump_if(true);
        assert_eq!(25 - 4, cpu.program_counter);
    }

    #[test]
    fn do_relative_jump_if_takes_2_cycles_if_condition_is_false() {
        let mut cpu = create_test_cpu();
        cpu.do_relative_jump_if(false);
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn do_relative_jump_takes_3_cycles_if_branching_to_same_page() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.do_relative_jump_if(true);
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn do_relative_jump_takes_5_cycles_if_branching_to_different_page() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xEF;
        cpu.memory.borrow_mut().write(0xEF, 0x7F);
        cpu.do_relative_jump_if(true);
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn do_bit_test_does_not_touch_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x08;
        cpu.a = 0xFA;
        cpu.do_bit_test(0xB2);
        assert_eq!(0xFA, cpu.a);
    }

    #[test]
    fn do_bit_test_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xCAFE;
        cpu.do_bit_test(0x12);
        assert_eq!(0xCAFE, cpu.program_counter);
    }

    #[test]
    fn do_bit_test_sets_negative_flag_if_bit_is_set_and_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x08;
        cpu.a = 0x80;
        cpu.do_bit_test(0x80);
        assert_eq!(0x88, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_sets_negative_flag_if_bit_is_set_in_memory_and_not_in_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x08;
        cpu.a = 0x01;
        cpu.do_bit_test(0x81);
        assert_eq!(0x88, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_does_nothing_if_negative_bit_is_set_and_negative_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x80;
        cpu.status_flags = 0x81;
        cpu.do_bit_test(0x80);
        assert_eq!(0x81, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_unsets_negative_flag_if_bit_is_not_set_and_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x0F;
        cpu.status_flags = 0x81;
        cpu.memory.borrow_mut().write(0x1234, 0x12);
        cpu.do_bit_test(0x0F);
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_does_nothing_if_negative_flag_is_not_set_and_bit_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x0F;
        cpu.status_flags = 0x01;
        cpu.do_bit_test(0x0F);
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_sets_overflow_flag_if_overflow_bit_is_set_and_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x04;
        cpu.a = 0x40;
        cpu.do_bit_test(0x40);
        assert_eq!(0x44, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_sets_overflow_flag_if_overflow_bit_is_set_but_accumulator_bit_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x04;
        cpu.a = 0x02;
        cpu.do_bit_test(0x42);
        assert_eq!(0x44, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_does_nothing_if_overflow_bit_is_set_and_overflow_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x74;
        cpu.a = 0x40;
        cpu.do_bit_test(0x40);
        assert_eq!(0x74, cpu.status_flags);
    }


    #[test]
    fn do_bit_test_unsets_overflow_bit_if_overflow_bit_is_not_set_and_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x44;
        cpu.a = 0x0F;
        cpu.do_bit_test(0x2F);
        assert_eq!(0x04, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_does_nothing_if_overflow_bit_is_not_set_and_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x0D;
        cpu.a = 0x0F;
        cpu.do_bit_test(0x2F);
        assert_eq!(0x0D, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x10;
        cpu.a = 0x00;
        cpu.do_bit_test(0x20);
        assert_eq!(0x12, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_sets_does_nothing_if_result_is_zero_and_zero_flag_was_already_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.a = 0x00;
        cpu.do_bit_test(0x20);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_unsets_zero_flag_if_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.a = 0x0A;
        cpu.do_bit_test(0x2F);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_bit_test_does_nothing_if_result_is_not_zero_and_zero_flag_was_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x10;
        cpu.a = 0x0A;
        cpu.do_bit_test(0x2F);
        assert_eq!(0x10, cpu.status_flags);
    }

    #[test]
    fn do_add_sets_zero_flag_if_accumulator_and_operand_are_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 0;

        cpu.do_add(0);

        assert_eq!(0x02, cpu.status_flags);
    }

     #[test]
    fn do_add_does_not_set_zero_flag_if_accumulator_and_operand_are_zero_but_carry_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 0;
        cpu.do_add(0);

        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_add_does_not_set_zero_flag_on_overflow() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 1;
        cpu.do_add(255);

        assert_eq!(0x00, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_add_does_nothing_if_zero_flag_is_set_and_accumulator_and_operand_are_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.a = 0;

        cpu.do_add(0);

        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_add_unsets_zero_flag_if_flag_is_set_and_result_is_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.a = 40;

        cpu.do_add(5);

        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_add_adds_two_small_numbers_together_and_stores_result_in_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 12;

        cpu.do_add(10);

        assert_eq!(22, cpu.a);
    }

    #[test]
    fn do_add_adds_two_small_numbers_together_and_does_not_set_any_flags() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 22;

        cpu.do_add(10);

        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn addition_works_with_positive_numbers_that_would_overflow_signed_8_bit_integer() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 90;
        // result is larger than signed 8 bit integer can hold; overflows into negative signed number
        cpu.do_add(70);

        // however we interpret the numbers as unsigned numbers and can just use positive numbers
        assert_eq!(160, cpu.a);
    }



    #[test]
    fn overflow_flag_is_not_set_when_positive_and_negative_number_are_added_and_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 20;
        cpu.do_add(220); // -36 as signed 8 bit integer

        assert_eq!(0x00, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_add_with_two_positive_numbers_sets_overflow_flag_if_result_does_not_fit_8_bit_signed_variable() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 70;
        // result is larger than signed 8 bit integer can hold; overflows into negative signed number
        cpu.do_add(90);

        // should set negative flag as well but not in scope for this one
        assert_eq!(0x40, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_add_with_two_negative_numbers_sets_overflow_flag_if_result_does_not_fit_8_bit_signed_variable() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 208; // 208 -> (208 -256 ) = - 48; this positive number represents negative number
        // result is smaller than signed 8 bit integer can hold; overflows into positive signed number
        cpu.do_add(144); // 144 -> (144 - 256) = -112
        // -48 + -112 =  -160; does not fit the signed 8 bit integer. Overflow should be set
        // should set negative flag as well but not in scope for this one
        assert_eq!(0x40, cpu.status_flags & 0x40);
    }

    #[test]
    fn overflow_flag_is_not_set_when_negative_and_positive_number_are_added_and_result_is_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 220; // -36 as signed 8 bit integer
        cpu.do_add(50);

        assert_eq!(0x00, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_add_with_unsets_overflow_flag_if_result_does_not_overflow() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x40;
        cpu.a = 10;
        cpu.do_add(90);

        // however we interpret the numbers as unsigned numbers
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_add_sets_negative_flag_if_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 10;
        cpu.do_add(180); // unsigned 180 -> (180 - 256) = -76 as signed

        assert_eq!(0x80, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_add_sets_negative_flag_if_result_is_negative_after_overflow() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 70;
        cpu.do_add(90);

        assert_eq!(0x80, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_add_unset_negative_flag_if_result_is_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.a = 70;
        cpu.do_add(200);

        assert_eq!(0x00, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_add_stores_lowest_8_bits_in_accumulator_if_result_is_too_large_to_fit_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.a = 70;
        cpu.do_add(200);

        assert_eq!(14, cpu.a);
    }

    #[test]
    fn do_add_sets_carry_flag_if_result_is_too_large_to_fit_8_bytes() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 70;
        cpu.do_add(200);

        assert_eq!(0x01, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_add_clears_carry_flag_if_result_fits_in_8_bytes() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 40;
        cpu.do_add(200);

        assert_eq!(0x00, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_add_adds_the_carry_flag_to_result_if_it_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 40;
        cpu.do_add(200);

        assert_eq!(241, cpu.a);
    }

    #[test]
    fn and_immediate_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.program_counter = 0x15;
        cpu.memory.borrow_mut().write(0x15, 0x3E);
        cpu.and_immediate();
        assert_eq!(0x28, cpu.a);
    }

    #[test]
    fn and_immediate_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x52;
        cpu.and_immediate();
        assert_eq!(0x53, cpu.program_counter);
    }

    #[test]
    fn and_zero_page_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.program_counter = 0xABCD;
        cpu.memory.borrow_mut().write(0xABCD, 0xFA);
        cpu.memory.borrow_mut().write(0xFA, 0x3E);

        cpu.and_zero_page();
        assert_eq!(0x28, cpu.a);
    }

    #[test]
    fn and_zero_page_x_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.x = 0x05;
        cpu.program_counter = 0x15;
        cpu.memory.borrow_mut().write(0x15, 0x40);
        cpu.memory.borrow_mut().write(0x40 + 0x05, 0x3E);
        cpu.and_zero_page_x();
        assert_eq!(0x28, cpu.a);
    }


    #[test]
    fn and_absolute_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.program_counter = 0x15;
        cpu.memory.borrow_mut().write(0x15, 0x40);
        cpu.memory.borrow_mut().write(0x40, 0x3E);
        cpu.and_absolute();
        assert_eq!(0x28, cpu.a);
    }

    #[test]
    fn and_absolute_x_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.x = 0x04;
        cpu.program_counter = 0x52;
        cpu.memory.borrow_mut().write(0x52, 0x00);
        cpu.memory.borrow_mut().write(0x53, 0x80);
        cpu.memory.borrow_mut().write(0x8004, 0x3E);
        cpu.and_absolute_x();
        assert_eq!(0x28, cpu.a);
    }


    #[test]
    fn and_absolute_y_sets_accumulator_value_to_the_result() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.y = 0x04;
        cpu.program_counter = 0x52;
        cpu.memory.borrow_mut().write(0x52, 0x00);
        cpu.memory.borrow_mut().write(0x53, 0x80);
        cpu.memory.borrow_mut().write(0x8004, 0x3E);
        cpu.and_absolute_y();
        assert_eq!(0x28, cpu.a);
    }

    #[test]
    fn and_indirect_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.x = 0x04;

        cpu.program_counter = 0x52;
        cpu.memory.borrow_mut().write(0x52, 0x14);

        cpu.memory.borrow_mut().write(0x14 + 0x04, 0x00);
        cpu.memory.borrow_mut().write(0x14 + 0x04 + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8000, 0x3E);
        cpu.and_indirect_x();
        assert_eq!(0x28, cpu.a);
    }


    #[test]
    fn and_indirect_y_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE9;
        cpu.y = 0x04;

        cpu.program_counter = 0x52;
        cpu.memory.borrow_mut().write(0x52, 0x14);

        cpu.memory.borrow_mut().write(0x14, 0x00);
        cpu.memory.borrow_mut().write(0x14 + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8000 + 0x04, 0x3E);
        cpu.and_indirect_y();
        assert_eq!(0x28, cpu.a);
    }

    #[test]
    fn inclusive_or_immediate_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x7A);
        cpu.inclusive_or_immediate();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_zero_page_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x45);
        cpu.memory.borrow_mut().write(0x0045, 0x7A);
        cpu.inclusive_or_zero_page();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_zero_page_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.x = 0x10;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x45);
        cpu.memory.borrow_mut().write(0x0045 + 0x10, 0x7A);
        cpu.inclusive_or_zero_page_x();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_absolute_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x45);
        cpu.memory.borrow_mut().write(0x1235, 0xAF);

        cpu.memory.borrow_mut().write(0xAF45 , 0x7A);
        cpu.inclusive_or_absolute();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_absolute_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.x = 0x15;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x45);
        cpu.memory.borrow_mut().write(0x1235, 0xAF);

        cpu.memory.borrow_mut().write(0xAF45 + 0x15, 0x7A);
        cpu.inclusive_or_absolute_x();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_absolute_y_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.y = 0x15;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x45);
        cpu.memory.borrow_mut().write(0x1235, 0xAF);

        cpu.memory.borrow_mut().write(0xAF45 + 0x15, 0x7A);
        cpu.inclusive_or_absolute_y();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_indirect_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.x = 0x15;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x20);

        cpu.memory.borrow_mut().write(0x20 + 0x15, 0x45);
        cpu.memory.borrow_mut().write(0x20 + 0x15 + 1, 0xAF);

        cpu.memory.borrow_mut().write(0xAF45, 0x7A);
        cpu.inclusive_or_indirect_x();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn inclusive_or_indirect_y_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.y = 0x15;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x20);

        cpu.memory.borrow_mut().write(0x20, 0x45);
        cpu.memory.borrow_mut().write(0x20 + 1, 0xAF);

        cpu.memory.borrow_mut().write(0xAF45 + 0x15, 0x7A);
        cpu.inclusive_or_indirect_y();
        assert_eq!(0xFB, cpu.a);
    }

    #[test]
    fn exclusive_or_immediate_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0xAF);

        cpu.exclusive_or_immediate();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn exclusive_or_zero_page_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);
        cpu.memory.borrow_mut().write(0x29, 0xAF);

        cpu.exclusive_or_zero_page();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn exclusive_or_zero_page_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.x = 0x25;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);
        cpu.memory.borrow_mut().write(0x29 + 0x25, 0xAF);

        cpu.exclusive_or_zero_page_x();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn exclusive_or_absolute_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);
        cpu.memory.borrow_mut().write(0x100, 0xEF);
        cpu.memory.borrow_mut().write(0xEF29, 0xAF);

        cpu.exclusive_or_absolute();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn exclusive_or_absolute_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.x = 0xFA;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);
        cpu.memory.borrow_mut().write(0x100, 0xEF);
        cpu.memory.borrow_mut().write(0xEF29 + 0xFA, 0xAF);

        cpu.exclusive_or_absolute_x();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn exclusive_or_absolute_y_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.y = 0xFA;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);
        cpu.memory.borrow_mut().write(0x100, 0xEF);
        cpu.memory.borrow_mut().write(0xEF29 + 0xFA, 0xAF);

        cpu.exclusive_or_absolute_y();
        assert_eq!(0x2E, cpu.a);
    }


    #[test]
    fn exclusive_or_indirect_x_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.x = 0x04;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);

        cpu.memory.borrow_mut().write(0x29 + 0x04, 0x29);
        cpu.memory.borrow_mut().write(0x29 + 0x04 + 1, 0xEF);

        cpu.memory.borrow_mut().write(0xEF29 , 0xAF);

        cpu.exclusive_or_indirect_x();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn exclusive_or_indirect_y_sets_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x81;
        cpu.y = 0x04;

        cpu.program_counter = 0xFF;
        cpu.memory.borrow_mut().write(0xFF, 0x29);

        cpu.memory.borrow_mut().write(0x29, 0x29);
        cpu.memory.borrow_mut().write(0x29 + 1, 0xEF);

        cpu.memory.borrow_mut().write(0xEF29 + 0x04, 0xAF);

        cpu.exclusive_or_indirect_y();
        assert_eq!(0x2E, cpu.a);
    }

    #[test]
    fn branch_if_carry_clear_branches_if_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_carry_clear();
        // 0x21 as the instruction reads the offset, thus modifying the pc
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_clear_does_not_branch_and_updates_pc_correctly_if_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x43;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_carry_clear();
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_set_branches_if_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_carry_set();
        // 0x21 as the instruction reads the offset, thus modifying the pc
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_carry_set_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_carry_set();
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn branch_if_equal_branches_if_zero_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD3;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_equal();
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_equal_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_equal();
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn branch_if_not_equal_branches_if_zero_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD4;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_not_equal();
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_negative_branches_if_zero_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_negative();
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_negative_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x7F;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_negative();
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn branch_if_positive_jumps_to_relative_address_on_nonzero_positive_number() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 24;
        cpu.memory.borrow_mut().write(24, 0x6C);
        cpu.set_negative_flag(0x32);
        cpu.branch_if_positive();
        assert_eq!(25 + 0x6C, cpu.program_counter);
    }

    #[test]
    fn branch_if_positive_jumps_to_address_on_zero() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 24;
        cpu.memory.borrow_mut().write(24, 0x02);
        cpu.set_negative_flag(0x00);
        cpu.branch_if_positive();

        assert_eq!(25 + 0x02, cpu.program_counter);
    }

    #[test]
    fn branch_if_positive_does_not_jump_on_negative_number() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 24;
        cpu.memory.borrow_mut().write(24, 0xBC);
        cpu.set_negative_flag(0xff);
        cpu.branch_if_positive();
        assert_eq!(25, cpu.program_counter);
    }

    #[test]
    fn branch_if_overflow_clear_branches_if_overflow_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xBF;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_overflow_clear();
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_overflow_clear_does_not_branch_and_updates_pc_correctly_if_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x40;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_overflow_clear();
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn branch_if_overflow_set_branches_if_overflow_flag_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD0;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_overflow_set();
        assert_eq!(0x21 + 0x10, cpu.program_counter);
    }

    #[test]
    fn branch_if_overflow_set_does_not_branch_and_updates_pc_correctly_if_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x20;
        cpu.memory.borrow_mut().write(0x20, 0x10);
        cpu.branch_if_overflow_set();
        assert_eq!(0x21, cpu.program_counter);
    }

    #[test]
    fn jump_absolute_sets_program_counter_to_new_value() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0;
        cpu.memory.borrow_mut().write(0, 0x15);
        cpu.memory.borrow_mut().write(1, 0xF0);
        cpu.jump_absolute();
        assert_eq!(0xf015, cpu.program_counter);
    }

    #[test]
    fn jump_absolute_sets_wait_counter_correctly() {
        let mut cpu = create_test_cpu();

        cpu.jump_absolute();
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn jump_to_subroutine_pushes_return_address_into_stack() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABCD;
        cpu.stack_pointer = 0xFF;
        cpu.memory.borrow_mut().write(0xABCD, 0x09);
        cpu.memory.borrow_mut().write(0xABCD + 1, 0xFC);
        cpu.jump_to_subroutine();
        // return address - 1 is pushed into stack in little endian form.
        // in this case, it's 0xABCE as the instruction takes two values from the instruction stream
        assert_eq!(0xCE, cpu.pop_value_from_stack());
        assert_eq!(0xAB, cpu.pop_value_from_stack());
    }

    #[test]
    fn jump_to_subroutine_changes_program_counter_value() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABCD;
        cpu.stack_pointer = 0xFF;
        cpu.memory.borrow_mut().write(0xABCD, 0x09);
        cpu.memory.borrow_mut().write(0xABCD + 1, 0xFC);
        cpu.jump_to_subroutine();
        assert_eq!(0xFC09, cpu.program_counter);
    }

    #[test]
    fn jump_to_subroutine_does_not_affect_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 15;
        cpu.stack_pointer = 0xFF;
        cpu.status_flags = 0xAD;
        cpu.jump_to_subroutine();
        assert_eq!(0xAD, cpu.status_flags);
    }

    #[test]
    fn jump_to_subroutine_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 15;
        cpu.stack_pointer = 0xFF;
        cpu.jump_to_subroutine();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn return_from_subroutine_sets_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x1234;
        // push high byte
        cpu.push_value_into_stack(0xFA);
        // push low byte
        cpu.push_value_into_stack(0x0B);
        cpu.return_from_subroutine();
        assert_eq!(0xFA0B + 1, cpu.program_counter);
    }

    #[test]
    fn return_from_subroutine_increments_stack_pointer() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x10;
        cpu.return_from_subroutine();
        assert_eq!(0x10 + 2, cpu.stack_pointer);
    }

    #[test]
    fn return_from_subroutine_does_not_touch_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xFA;
        cpu.return_from_subroutine();
        assert_eq!(0xFA, cpu.status_flags);
    }

    #[test]
    fn return_from_subroutine_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.return_from_subroutine();
        assert_eq!(6, cpu.wait_counter);
    }

    // to a large degree, these bit_test test the same things that some more general tests
    // above. This is however necessary to make sure that the desired function
    // has actually been called

    #[test]
    fn bit_test_zero_page_sets_flags_correctly() {

        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 0xCA;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0x07);
        cpu.memory.borrow_mut().write(0x07, 0xF0);
        cpu.bit_test_zero_page();
        assert_eq!(0xC0, cpu.status_flags);
    }

    #[test]
    fn bit_test_zero_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x1234;
        cpu.bit_test_zero_page();
        assert_eq!(0x1234+1, cpu.program_counter);
    }

    #[test]
    fn bit_test_zero_page_takes_3_cycles() {
        let mut cpu = create_test_cpu();
        cpu.bit_test_zero_page();
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn bit_test_absolute_sets_flags_correctly() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 0xCA;
        cpu.program_counter = 0x1234;
        cpu.memory.borrow_mut().write(0x1234, 0xFE);
        cpu.memory.borrow_mut().write(0x1235, 0xCA);

        cpu.memory.borrow_mut().write(0xCAFE, 0xF0);
        cpu.bit_test_absolute();
        assert_eq!(0xC0, cpu.status_flags);
    }

    #[test]
    fn bit_test_absolute_increments_pc_correctly() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x1234;
        cpu.bit_test_absolute();
        assert_eq!(0x1234+2, cpu.program_counter);
    }

    #[test]
    fn bit_test_absolute_takes_4_cycles() {
        let mut cpu = create_test_cpu();
        cpu.bit_test_absolute();
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn clear_carry_flag_clears_the_flag_if_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xC5;
        cpu.clear_carry_flag();
        assert_eq!(0xC4, cpu.status_flags);
    }

    #[test]
    fn clear_carry_does_nothing_if_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD6;
        cpu.clear_carry_flag();
        assert_eq!(0xD6, cpu.status_flags);
    }

    #[test]
    fn clear_carry_flag_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.clear_carry_flag();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn set_carry_flag_sets_the_flag_if_it_was_not_set_before() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 15;
        cpu.status_flags = 0x86;
        cpu.set_carry_flag();
        assert_eq!(0x87, cpu.status_flags);
    }

    #[test]
    fn set_carry_flag_does_nothing_if_flag_is_already_set() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 15;
        cpu.status_flags = 0x86;
        cpu.set_carry_flag();
        assert_eq!(0x87, cpu.status_flags);
    }

    #[test]
    fn set_carry_flag_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 15;
        cpu.stack_pointer = 0x86;
        cpu.set_carry_flag();
        assert_eq!(15, cpu.program_counter);
    }

    #[test]
    fn set_carry_flag_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 15;
        cpu.stack_pointer = 0xFF;
        cpu.set_carry_flag();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn clear_decimal_flags_clears_the_flag_and_does_not_touch_other_flags() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xCF;
        cpu.clear_decimal_flag();
        assert_eq!(0xC7, cpu.status_flags);
    }

    #[test]
    fn clear_decimal_flags_does_nothing_if_flag_is_already_cleared() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD6;
        cpu.clear_decimal_flag();
        assert_eq!(0xD6, cpu.status_flags);
    }

    #[test]
    fn clear_decimal_flags_sets_wait_counter_correctly() {
        let mut cpu = create_test_cpu();
        cpu.clear_decimal_flag();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn set_decimal_flag_sets_the_flag_if_it_was_unset() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x07;
        cpu.set_decimal_flag();
        assert_eq!(0x0F, cpu.status_flags);
    }

    #[test]
    fn set_decimal_flag_does_nothing_if_flag_was_already_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x0A;
        cpu.set_decimal_flag();
        assert_eq!(0x0A, cpu.status_flags);
    }

    #[test]
    fn set_decimal_flag_does_not_touch_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xAB12;
        cpu.set_decimal_flag();
        assert_eq!(0xAB12, cpu.program_counter);
    }

    #[test]
    fn set_decimal_flag_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xAB12;
        cpu.set_decimal_flag();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn setting_interrupt_disable_flag_does_nothing_if_flag_is_already_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xD5;
        cpu.set_interrupt_disable_flag();
        assert_eq!(0xD5, cpu.status_flags);
    }

    #[test]
    fn setting_interrupt_disable_flag_sets_the_flag_and_does_not_touch_other_flags() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xC3;
        cpu.set_interrupt_disable_flag();
        assert_eq!(0xC7, cpu.status_flags);
    }

    #[test]
    fn setting_interrupt_disable_flag_sets_wait_counter_correctly() {
        let mut cpu = create_test_cpu();
        cpu.set_interrupt_disable_flag();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn clear_overflow_flag_clears_the_flag() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xFF;
        cpu.clear_overflow_flag();
        assert_eq!(0xBF, cpu.status_flags);
    }

    #[test]
    fn clear_overflow_flag_does_nothing_if_the_flag_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xBF;
        cpu.clear_overflow_flag();
        assert_eq!(0xBF, cpu.status_flags);
    }

    #[test]
    fn clear_overflow_flag_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.clear_overflow_flag();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn push_accumulator_pushes_accumulator_into_stack() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x34;
        cpu.push_accumulator();
        assert_eq!(0x34, cpu.pop_value_from_stack());
    }

    #[test]
    fn push_accumulator_does_not_modify_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x34;
        cpu.push_accumulator();
        assert_eq!(0x34, cpu.a);
    }

    #[test]
    fn push_accumulator_decrements_stack_pointer() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0xEF;
        cpu.push_accumulator();
        assert_eq!(0xEF - 1, cpu.stack_pointer);
    }

    #[test]
    fn push_accumulator_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x123;
        cpu.push_accumulator();
        assert_eq!(0x123, cpu.program_counter);
    }

    #[test]
    fn push_accumulator_takes_3_cycles() {
        let mut cpu = create_test_cpu();
        cpu.push_accumulator();
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn pull_accumulator_sets_accumulator_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.push_value_into_stack(0xFA);
        cpu.pull_accumulator();
        assert_eq!(0xFA, cpu.a);
    }

    #[test]
    fn pull_accumulator_increments_stack_pointer() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x24;
        cpu.pull_accumulator();
        assert_eq!(0x24 + 1, cpu.stack_pointer);
    }

    #[test]
    fn pull_accumulator_sets_zero_flag_if_value_pulled_was_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xAA;
        cpu.status_flags = 0x78;
        cpu.push_value_into_stack(0x00);
        cpu.pull_accumulator();
        assert_eq!(0x7A, cpu.status_flags);
    }

    #[test]
    fn pull_accumulator_unsets_zero_flag_if_value_pulled_was_not_zero() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.status_flags = 0xAA;
        cpu.push_value_into_stack(0xBA);
        cpu.pull_accumulator();
        assert_eq!(0xA8, cpu.status_flags);
    }

    #[test]
    fn pull_accumulator_sets_negative_flag_if_value_pulled_was_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xAA;
        cpu.status_flags = 0x00;
        cpu.push_value_into_stack(0xFF);
        cpu.pull_accumulator();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn pull_accumulator_unsets_negative_flag_if_value_pulled_was_not_negative() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xAA;
        cpu.status_flags = 0x80;
        cpu.push_value_into_stack(0x7F);
        cpu.pull_accumulator();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn pull_accumulator_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x20;
        cpu.pull_accumulator();
        assert_eq!(0x20, cpu.program_counter);
    }

    #[test]
    fn pull_accumulator_takes_4_cycles() {
        let mut cpu = create_test_cpu();
        cpu.pull_accumulator();
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn push_status_flags_into_stack_pushes_flags_to_stack_and_sets_bits_4_and_5_to_1() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x8A;
        cpu.push_status_flags_into_stack();
        assert_eq!(0xBA, cpu.pop_value_from_stack());
    }

    #[test]
    fn push_status_flags_into_stack_does_not_increment_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x4000;
        cpu.push_status_flags_into_stack();
        assert_eq!(0x4000, cpu.program_counter);
    }

    #[test]
    fn push_status_flags_into_stack_takes_3_cycles() {
        let mut cpu = create_test_cpu();
        cpu.push_status_flags_into_stack();
        assert_eq!(3, cpu.wait_counter);

    }

    #[test]
    fn pull_status_flags_sets_status_flags_correctly() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x1A;
        cpu.push_value_into_stack(0xFE);
        cpu.pull_status_flags_from_stack();
        assert_eq!(0xFE, cpu.status_flags);
    }

    // hardwired to 1
    #[test]
    fn pull_status_flags_always_sets_4_and_5_bits() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xAE;
        cpu.push_value_into_stack(0x00);
        cpu.pull_status_flags_from_stack();
        assert_eq!(0x30, cpu.status_flags);
    }

    #[test]
    fn pull_status_flags_from_stack_increments_stack_pointer() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x3f;
        cpu.pull_status_flags_from_stack();
        assert_eq!(0x3f + 1, cpu.stack_pointer);
    }

    #[test]
    fn pull_status_flags_from_stack_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x1253;
        cpu.pull_status_flags_from_stack();
        assert_eq!(0x1253, cpu.program_counter);
    }

    #[test]
    fn pull_status_flags_from_stack_takes_4_cycles() {
        let mut cpu = create_test_cpu();
        cpu.pull_status_flags_from_stack();
        assert_eq!(4, cpu.wait_counter);
    }

    #[test]
    fn load_a_immediate_sets_a_to_the_value_given_in_next_byte() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0x23);
        cpu.load_a_immediate();
        assert_eq!(0x23, cpu.a);
    }

    #[test]
    fn load_a_zero_page_sets_a_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0x23);
        cpu.memory.borrow_mut().write(0x23, 0xFA);
        cpu.load_a_zero_page();
        assert_eq!(0xFA, cpu.a);
    }

    #[test]
    fn load_a_zero_page_x_sets_a_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.x = 0x12;
        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0x23);
        cpu.memory.borrow_mut().write(0x23 + 0x12, 0xFA);
        cpu.load_a_zero_page_x();
        assert_eq!(0xFA, cpu.a);
    }

    #[test]
    fn load_a_absolute_loads_correct_value_from_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0xB1);
        cpu.memory.borrow_mut().write(26, 0xF0);
        cpu.memory.borrow_mut().write(0xF0B1, 42);

        cpu.load_a_absolute();
        assert_eq!(42, cpu.a);
    }

    #[test]
    fn load_a_absolute_x_loads_correct_value_from_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x14;
        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0xB1);
        cpu.memory.borrow_mut().write(26, 0xF0);
        cpu.memory.borrow_mut().write(0xF0B1 + 0x14, 42);

        cpu.load_a_absolute_x();
        assert_eq!(42, cpu.a);
    }

    #[test]
    fn load_a_absolute_y_loads_correct_value_from_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x14;
        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0xB1);
        cpu.memory.borrow_mut().write(26, 0xF0);
        cpu.memory.borrow_mut().write(0xF0B1 + 0x14, 42);

        cpu.load_a_absolute_y();
        assert_eq!(42, cpu.a);
    }

    #[test]
    fn load_a_indirect_x_loads_correct_value_from_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x14;

        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0xB1);

        cpu.memory.borrow_mut().write(0xB1 + 0x14, 0xEF);
        cpu.memory.borrow_mut().write(0xB1 + 0x14 + 1, 0x02);

        cpu.memory.borrow_mut().write(0x02EF, 0xAF);

        cpu.load_a_indirect_x();
        assert_eq!(0xAF, cpu.a);
    }

    #[test]
    fn load_a_indirect_y_loads_correct_value_from_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x14;

        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0xB1);

        cpu.memory.borrow_mut().write(0xB1, 0xEF);
        cpu.memory.borrow_mut().write(0xB1 + 1, 0x02);

        cpu.memory.borrow_mut().write(0x02EF + 0x14, 0xAF);

        cpu.load_a_indirect_y();
        assert_eq!(0xAF, cpu.a);
    }

    #[test]
    fn store_a_zero_page_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.store_a_zero_page();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x14));
    }

    #[test]
    fn store_a_zero_page_x_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.x = 0xBF;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.store_a_zero_page_x();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x14 + 0xBF));
    }

    #[test]
    fn store_a_absolute_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;

        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0x33, 0x07);

        cpu.store_a_absolute();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x07AF));
    }

    #[test]
    fn store_a_absolute_x_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.x = 0x14;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0x33, 0x07);

        cpu.store_a_absolute_x();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x07AF + 0x14));
    }

    #[test]
    fn store_a_absolute_y_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.y = 0x14;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0x33, 0x07);

        cpu.store_a_absolute_y();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x07AF + 0x14));
    }

    #[test]
    fn store_a_indirect_x_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.x = 0x14;
        cpu.program_counter = 0x32;

        cpu.memory.borrow_mut().write(0x32, 0xAF);


        cpu.memory.borrow_mut().write(0xAF + 0x14 , 0x07);
        cpu.memory.borrow_mut().write(0xAF + 0x14 + 1 , 0x20);

        cpu.store_a_indirect_x();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x2007));
    }

    #[test]
    fn store_a_indirect_y_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.y = 0x14;
        cpu.program_counter = 0x32;

        cpu.memory.borrow_mut().write(0x32, 0xAF);

        cpu.memory.borrow_mut().write(0xAF, 0x07);
        cpu.memory.borrow_mut().write(0xAF + 1 , 0x20);

        cpu.store_a_indirect_y();
        assert_eq!(0x2F, cpu.memory.borrow_mut().read(0x2007 + 0x14));
    }

    #[test]
    fn load_x_immediate_sets_x_to_the_value_given_in_next_byte() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 25;
        cpu.memory.borrow_mut().write(25, 0x23);
        cpu.load_x_immediate();
        assert_eq!(0x23, cpu.x);
    }

    #[test]
    fn load_x_zero_page_sets_x_to_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x25;
        cpu.memory.borrow_mut().write(0x25, 0xBE);

        cpu.memory.borrow_mut().write(0xBE, 0x09);

        cpu.load_x_zero_page();
        assert_eq!(0x09, cpu.x);
    }

    #[test]
    fn load_x_zero_page_y_sets_x_to_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.y = 0x13;
        cpu.program_counter = 0x25;
        cpu.memory.borrow_mut().write(0x25, 0xBE);

        cpu.memory.borrow_mut().write(0xBE + 0x13, 0x09);

        cpu.load_x_zero_page_y();
        assert_eq!(0x09, cpu.x);
    }

    #[test]
    fn load_x_absolute_sets_x_to_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x25;
        cpu.memory.borrow_mut().write(0x25, 0xBE);
        cpu.memory.borrow_mut().write(0x26, 0xAB);

        cpu.memory.borrow_mut().write(0xABBE, 0x09);

        cpu.load_x_absolute();
        assert_eq!(0x09, cpu.x);
    }

    #[test]
    fn load_x_absolute_y_sets_x_to_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.y = 0x13;
        cpu.program_counter = 0x25;
        cpu.memory.borrow_mut().write(0x25, 0xBE);
        cpu.memory.borrow_mut().write(0x26, 0xAB);

        cpu.memory.borrow_mut().write(0xABBE + 0x13, 0x09);

        cpu.load_x_absolute_y();
        assert_eq!(0x09, cpu.x);
    }

    #[test]
    fn store_x_zero_page_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x2f;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.store_x_zero_page();
        assert_eq!(0x2f, cpu.memory.borrow_mut().read(0x14));
    }

    #[test]
    fn store_x_zero_page_y_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x2f;
        cpu.y = 0x53;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.store_x_zero_page_y();
        assert_eq!(0x2f, cpu.memory.borrow_mut().read(0x14 + 0x53));
    }

    #[test]
    fn store_x_absolute_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x2f;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.memory.borrow_mut().write(0x33, 0x08);

        cpu.store_x_absolute();
        assert_eq!(0x2f, cpu.memory.borrow_mut().read(0x0814));
    }

    #[test]
    fn transfer_x_to_stack_pointer_sets_stack_pointer_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.x = 0xFC;
        cpu.transfer_x_to_stack_pointer();
        assert_eq!(0xFC, cpu.stack_pointer);
    }

    #[test]
    fn transfer_x_to_stack_pointer_does_not_touch_flags() {
        let mut cpu = create_test_cpu();
        cpu.x = 0xFC;
        cpu.status_flags = 0xAB;
        cpu.transfer_x_to_stack_pointer();
        assert_eq!(0xAB, cpu.status_flags);
    }

    #[test]
    fn transfer_x_to_stack_pointer_sets_wait_counter_correct() {
        let mut cpu = create_test_cpu();
        cpu.transfer_x_to_stack_pointer();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn compare_immediate_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_immediate();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_immediate_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_immediate();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_immediate_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x00;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x40);
        cpu.a = 0x39;

        cpu.compare_immediate();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_zero_page_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_zero_page();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_zero_page_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_zero_page();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_zero_page_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x00;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x40);
        cpu.a = 0x39;

        cpu.compare_zero_page();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_zero_page_x_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.x = 0x25;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50 + 0x25, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_zero_page_x();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_zero_page_x_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.x = 0x25;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50 + 0x25, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_zero_page_x();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_zero_page_x_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x00;
        cpu.x = 0x25;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50 + 0x25, 0x40);
        cpu.a = 0x39;

        cpu.compare_zero_page_x();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_absolute();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_absolute();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x00;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.a = 0x39;

        cpu.compare_absolute();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_x_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xFA;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050 + 0xFA, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_absolute_x();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_x_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xFA;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050 + 0xFA, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_absolute_x();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_x_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xFA;
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050 + 0xFA, 0x40);
        cpu.a = 0x39;

        cpu.compare_absolute_x();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_y_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xFA;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050 + 0xFA, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_absolute_y();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_y_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xFA;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050 + 0xFA, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_absolute_y();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_absolute_y_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xFA;
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050 + 0xFA, 0x40);
        cpu.a = 0x39;

        cpu.compare_absolute_y();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_indirect_x_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xBA;
        cpu.program_counter = 0x1010;
        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x0E + 0xBA, 0x50);
        cpu.memory.borrow_mut().write(0x0E + 0xBA + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8050, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_indirect_x();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_indirect_x_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xBA;
        cpu.program_counter = 0x1010;
        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x0E + 0xBA, 0x50);
        cpu.memory.borrow_mut().write(0x0E + 0xBA + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_indirect_x();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_indirect_x_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xBA;
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x1010;
        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x0E + 0xBA, 0x50);
        cpu.memory.borrow_mut().write(0x0E + 0xBA + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.a = 0x39;

        cpu.compare_indirect_x();
        assert_eq!(0x80, cpu.status_flags);
    }
    #[test]

    fn compare_indirect_y_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xBA;
        cpu.program_counter = 0x1010;
        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x0E, 0x50);
        cpu.memory.borrow_mut().write(0x0E + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8050 + 0xBA, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0xFF;

        cpu.compare_indirect_y();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_indirect_y_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xBA;
        cpu.program_counter = 0x1010;
        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x0E, 0x50);
        cpu.memory.borrow_mut().write(0x0E + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8050 + 0xBA, 0x40);
        cpu.status_flags = 0x00;
        cpu.a = 0x40;

        cpu.compare_indirect_y();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_indirect_y_sets_negative_flag_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xBA;
        cpu.status_flags = 0x00;
        cpu.program_counter = 0x1010;
        cpu.memory.borrow_mut().write(0x1010, 0x0E);

        cpu.memory.borrow_mut().write(0x0E, 0x50);
        cpu.memory.borrow_mut().write(0x0E + 1, 0x80);

        cpu.memory.borrow_mut().write(0x8050 + 0xBA, 0x40);
        cpu.a = 0x39;

        cpu.compare_indirect_y();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn add_immediate_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;

        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 20);
        cpu.add_immediate();
        assert_eq!(69, cpu.a);
    }

    #[test]
    fn add_zero_page_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;

        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);
        cpu.memory.borrow_mut().write(0x20, 29);

        cpu.add_zero_page();
        assert_eq!(78, cpu.a);
    }

    #[test]
    fn add_zero_page_x_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.x = 0x40;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);
        cpu.memory.borrow_mut().write(0x20 + 0x40, 29);

        cpu.add_zero_page_x();
        assert_eq!(78, cpu.a);
    }

    #[test]
    fn add_absolute_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);
        cpu.memory.borrow_mut().write(0x31, 0xDE);
        cpu.memory.borrow_mut().write(0xDE20, 29);

        cpu.add_absolute();
        assert_eq!(78, cpu.a);
    }

    #[test]
    fn add_absolute_x_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.x = 0x42;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);
        cpu.memory.borrow_mut().write(0x31, 0xDE);
        cpu.memory.borrow_mut().write(0xDE20 + 0x42, 29);

        cpu.add_absolute_x();
        assert_eq!(78, cpu.a);
    }

    #[test]
    fn add_absolute_y_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.y = 0x42;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);
        cpu.memory.borrow_mut().write(0x31, 0xDE);
        cpu.memory.borrow_mut().write(0xDE20 + 0x42, 29);

        cpu.add_absolute_y();
        assert_eq!(78, cpu.a);
    }

    #[test]
    fn add_indirect_x_stores_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.x = 0x42;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);

        cpu.memory.borrow_mut().write(0x20 + 0x42, 0xDE);
        cpu.memory.borrow_mut().write(0x20 + 0x42 + 1, 0x29);

        cpu.memory.borrow_mut().write(0x29DE, 29);

        cpu.add_indirect_x();
        assert_eq!(78, cpu.a);

    }

    #[test]
    fn add_indirect_y_stores_correct_value_into_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.y = 0x42;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 0x20);

        cpu.memory.borrow_mut().write(0x20, 0xDE);
        cpu.memory.borrow_mut().write(0x20 + 1, 0x29);

        cpu.memory.borrow_mut().write(0x29DE + 0x42, 29);

        cpu.add_indirect_y();
        assert_eq!(78, cpu.a);

    }

    #[test]
    fn no_operation_waits_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.no_operation();
        assert_eq!(2, cpu.wait_counter);
    }
}
