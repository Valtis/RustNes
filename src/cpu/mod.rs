
use rom::TvSystem;
use memory::Memory;
use std::rc::Rc;
use std::cell::RefCell;

// official opcodes: http://www.obelisk.demon.co.uk/6502/reference.html
// addressing modes: http://www.obelisk.demon.co.uk/6502/addressing.html

// unofficial opcodes: http://www.ataripreservation.org/websites/freddy.offenga/illopc31.txt
// and http://www.oxyron.de/html/opcodes02.html.
// and http://www.ffd2.com/fridge/docs/6502-NMOS.extra.opcodes

// The documentation on behaviour of unofficial opcodes is somewhat inconsistent.
// Conflicts have been solved by observing existing emulator behaviour (hopefully they got it right)

#[derive(Debug)]
pub struct Cpu {
    memory: Rc<RefCell<Box<Memory>>>, // reference to memory, so that cpu can use it
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
    pub fn new(tv_system: &TvSystem, memory: Rc<RefCell<Box<Memory>>>) -> Cpu {
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
        self.program_counter = 0xFFFC;
        self.jump_absolute();
    }


    pub fn execute_instruction(&mut self) {
        let instruction = self.memory.borrow_mut().read(self.program_counter);

        println!("{:04X} Opcode:{:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.program_counter,
            instruction,
            self.a,
            self.x,
            self.y,
            self.status_flags & 0xEF,
            self.stack_pointer,
            );

        self.program_counter += 1;
        match instruction {
            0 => self.force_interrupt(),
            1 => self.inclusive_or_indirect_x(),
            3 => self.unofficial_shift_left_memory_inclusive_or_acc_indirect_x(),
            4 => self.unofficial_double_no_operation(3),
            5 => self.inclusive_or_zero_page(),
            6 => self.arithmetic_shift_left_zero_page(),
            7 => self.unofficial_shift_left_memory_inclusive_or_acc_zero_page(),
            8 => self.push_status_flags_into_stack(),
            9 => self.inclusive_or_immediate(),
            10 => self.arithmetic_shift_left_accumulator(),
            12 => self.unofficial_triple_no_operation_no_page_penalty(4),
            13 => self.inclusive_or_absolute(),
            14 => self.arithmetic_shift_left_absolute(),
            15 => self.unofficial_shift_left_memory_inclusive_or_acc_absolute(),
            16 => self.branch_if_positive(),
            17 => self.inclusive_or_indirect_y(),
            19 => self.unofficial_shift_left_memory_inclusive_or_acc_indirect_y(),
            20 => self.unofficial_double_no_operation(4),
            21 => self.inclusive_or_zero_page_x(),
            22 => self.arithmetic_shift_left_zero_page_x(),
            23 => self.unofficial_shift_left_memory_inclusive_or_acc_zero_page_x(),
            24 => self.clear_carry_flag(),
            25 => self.inclusive_or_absolute_y(),
            26 => self.unofficial_nop(),
            27 => self.unofficial_shift_left_memory_inclusive_or_acc_absolute_y(),
            28 => self.unofficial_triple_no_operation_page_penalty(4),
            29 => self.inclusive_or_absolute_x(),
            30 => self.arithmetic_shift_left_absolute_x(),
            31 => self.unofficial_shift_left_memory_inclusive_or_acc_absolute_x(),
            32 => self.jump_to_subroutine(),
            33 => self.and_indirect_x(),
            35 => self.unofficial_rotate_left_memory_bitwise_and_acc_indirect_x(),
            36 => self.bit_test_zero_page(),
            37 => self.and_zero_page(),
            38 => self.rotate_left_zero_page(),
            39 => self.unofficial_rotate_left_memory_bitwise_and_acc_zero_page(),
            40 => self.pull_status_flags_from_stack(),
            41 => self.and_immediate(),
            42 => self.rotate_left_accumulator(),
            44 => self.bit_test_absolute(),
            45 => self.and_absolute(),
            46 => self.rotate_left_absolute(),
            47 => self.unofficial_rotate_left_memory_bitwise_and_acc_absolute(),
            48 => self.branch_if_negative(),
            49 => self.and_indirect_y(),
            51 => self.unofficial_rotate_left_memory_bitwise_and_acc_indirect_y(),
            52 => self.unofficial_double_no_operation(4),
            53 => self.and_zero_page_x(),
            54 => self.rotate_left_zero_page_x(),
            55 => self.unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x(),
            56 => self.set_carry_flag(),
            57 => self.and_absolute_y(),
            58 => self.unofficial_nop(),
            59 => self.unofficial_rotate_left_memory_bitwise_and_acc_absolute_y(),
            60 => self.unofficial_triple_no_operation_page_penalty(4),
            61 => self.and_absolute_x(),
            62 => self.rotate_left_absolute_x(),
            63 => self.unofficial_rotate_left_memory_bitwise_and_acc_absolute_x(),
            64 => self.return_from_interrupt(),
            65 => self.exclusive_or_indirect_x(),
            67 => self.unofficial_shift_right_memory_xor_acc_indirect_x(),
            68 => self.unofficial_double_no_operation(3),
            69 => self.exclusive_or_zero_page(),
            70 => self.logical_shift_right_zero_page(),
            71 => self.unofficial_shift_right_memory_xor_acc_zero_page(),
            72 => self.push_accumulator(),
            73 => self.exclusive_or_immediate(),
            74 => self.logical_shift_right_accumulator(),
            76 => self.jump_absolute(),
            77 => self.exclusive_or_absolute(),
            78 => self.logical_shift_right_absolute(),
            79 => self.unofficial_shift_right_memory_xor_acc_absolute(),
            80 => self.branch_if_overflow_clear(),
            81 => self.exclusive_or_indirect_y(),
            83 => self.unofficial_shift_right_memory_xor_acc_indirect_y(),
            84 => self.unofficial_double_no_operation(4),
            85 => self.exclusive_or_zero_page_x(),
            86 => self.logical_shift_right_zero_page_x(),
            87 => self.unofficial_shift_right_memory_xor_acc_zero_page_x(),
            89 => self.exclusive_or_absolute_y(),
            90 => self.unofficial_nop(),
            91 => self.unofficial_shift_right_memory_xor_acc_absolute_y(),
            92 => self.unofficial_triple_no_operation_page_penalty(4),
            93 => self.exclusive_or_absolute_x(),
            94 => self.logical_shift_right_absolute_x(),
            95 => self.unofficial_shift_right_memory_xor_acc_absolute_x(),
            96 => self.return_from_subroutine(),
            97 => self.add_indirect_x(),
            99 => self.unofficial_rotate_right_memory_add_acc_indirect_x(),
            100 => self.unofficial_double_no_operation(3),
            101 => self.add_zero_page(),
            102 => self.rotate_right_zero_page(),
            103 => self.unofficial_rotate_right_memory_add_acc_zero_page(),
            104 => self.pull_accumulator(),
            105 => self.add_immediate(),
            106 => self.rotate_right_accumulator(),
            108 => self.jump_indirect(),
            109 => self.add_absolute(),
            110 => self.rotate_right_absolute(),
            111 => self.unofficial_rotate_right_memory_add_acc_absolute(),
            112 => self.branch_if_overflow_set(),
            113 => self.add_indirect_y(),
            115 => self.unofficial_rotate_right_memory_add_acc_indirect_y(),
            116 => self.unofficial_double_no_operation(4),
            117 => self.add_zero_page_x(),
            118 => self.rotate_right_zero_page_x(),
            119 => self.unofficial_rotate_right_memory_add_acc_zero_page_x(),
            120 => self.set_interrupt_disable_flag(),
            121 => self.add_absolute_y(),
            122 => self.unofficial_nop(),
            123 => self.unofficial_rotate_right_memory_add_acc_absolute_y(),
            124 => self.unofficial_triple_no_operation_page_penalty(4),
            125 => self.add_absolute_x(),
            126 => self.rotate_right_absolute_x(),
            127 => self.unofficial_rotate_right_memory_add_acc_absolute_x(),
            128 => self.unofficial_double_no_operation(2),
            129 => self.store_a_indirect_x(),
            130 => self.unofficial_double_no_operation(2),
            131 => self.unofficial_and_a_with_x_store_result_indirect_x(),
            132 => self.store_y_zero_page(),
            133 => self.store_a_zero_page(),
            134 => self.store_x_zero_page(),
            135 => self.unofficial_and_a_with_x_store_result_zero_page(),
            136 => self.decrease_y(),
            137 => self.unofficial_double_no_operation(2),
            138 => self.transfer_x_to_accumulator(),
            140 => self.store_y_absolute(),
            141 => self.store_a_absolute(),
            142 => self.store_x_absolute(),
            143 => self.unofficial_and_a_with_x_store_result_absolute(),
            144 => self.branch_if_carry_clear(),
            145 => self.store_a_indirect_y(),
            148 => self.store_y_zero_page_x(),
            149 => self.store_a_zero_page_x(),
            150 => self.store_x_zero_page_y(),
            151 => self.unofficial_and_a_with_x_store_result_zero_page_y(),
            152 => self.transfer_y_to_accumulator(),
            153 => self.store_a_absolute_y(),
            154 => self.transfer_x_to_stack_pointer(),
            157 => self.store_a_absolute_x(),
            160 => self.load_y_immediate(),
            161 => self.load_a_indirect_x(),
            162 => self.load_x_immediate(),
            163 => self.unofficial_load_a_and_x_indirect_x(),
            164 => self.load_y_zero_page(),
            165 => self.load_a_zero_page(),
            166 => self.load_x_zero_page(),
            167 => self.unofficial_load_a_and_x_zero_page(),
            168 => self.transfer_accumulator_to_y(),
            169 => self.load_a_immediate(),
            170 => self.transfer_accumulator_to_x(),
            172 => self.load_y_absolute(),
            173 => self.load_a_absolute(),
            174 => self.load_x_absolute(),
            175 => self.unofficial_load_a_and_x_absolute(),
            176 => self.branch_if_carry_set(),
            177 => self.load_a_indirect_y(),
            179 => self.unofficial_load_a_and_x_indirect_y(),
            180 => self.load_y_zero_page_x(),
            181 => self.load_a_zero_page_x(),
            182 => self.load_x_zero_page_y(),
            183 => self.unofficial_load_a_and_x_zero_page_y(),
            184 => self.clear_overflow_flag(),
            185 => self.load_a_absolute_y(),
            186 => self.transfer_stack_pointer_to_x(),
            188 => self.load_y_absolute_x(),
            189 => self.load_a_absolute_x(),
            190 => self.load_x_absolute_y(),
            191 => self.unofficial_load_a_and_x_absolute_y(),
            192 => self.compare_y_immediate(),
            193 => self.compare_indirect_x(),
            194 => self.unofficial_double_no_operation(2),
            195 => self.unofficial_decrement_memory_and_compare_with_acc_indirect_x(),
            196 => self.compare_y_zero_page(),
            197 => self.compare_zero_page(),
            198 => self.decrement_memory_zero_page(),
            199 => self.unofficial_decrement_memory_and_compare_with_acc_zero_page(),
            200 => self.increase_y(),
            201 => self.compare_immediate(),
            202 => self.decrease_x(),
            204 => self.compare_y_absolute(),
            205 => self.compare_absolute(),
            206 => self.decrement_memory_absolute(),
            207 => self.unofficial_decrement_memory_and_compare_with_acc_absolute(),
            208 => self.branch_if_not_equal(),
            209 => self.compare_indirect_y(),
            211 => self.unofficial_decrement_memory_and_compare_with_acc_indirect_y(),
            212 => self.unofficial_double_no_operation(4),
            213 => self.compare_zero_page_x(),
            214 => self.decrement_memory_zero_page_x(),
            215 => self.unofficial_decrement_memory_and_compare_with_acc_zero_page_x(),
            216 => self.clear_decimal_flag(),
            217 => self.compare_absolute_y(),
            218 => self.unofficial_nop(),
            219 => self.unofficial_decrement_memory_and_compare_with_acc_absolute_y(),
            220 => self.unofficial_triple_no_operation_page_penalty(4),
            221 => self.compare_absolute_x(),
            222 => self.decrement_memory_absolute_x(),
            223 => self.unofficial_decrement_memory_and_compare_with_acc_absolute_x(),
            224 => self.compare_x_immediate(),
            225 => self.subtract_indirect_x(),
            226 => self.unofficial_double_no_operation(2),
            227 => self.unofficial_increment_memory_subtract_acc_indirect_x(),
            228 => self.compare_x_zero_page(),
            229 => self.subtract_zero_page(),
            230 => self.increment_memory_zero_page(),
            231 => self.unofficial_increment_memory_subtract_acc_zero_page(),
            232 => self.increase_x(),
            233 => self.subtract_immediate(),
            234 => self.no_operation(),
            235 => self.unofficial_subtract_immediate(),
            236 => self.compare_x_absolute(),
            237 => self.subtract_absolute(),
            238 => self.increment_memory_absolute(),
            239 => self.unofficial_increment_memory_subtract_acc_absolute(),
            240 => self.branch_if_equal(),
            241 => self.subtract_indirect_y(),
            243 => self.unofficial_increment_memory_subtract_acc_indirect_y(),
            244 => self.unofficial_double_no_operation(4),
            245 => self.subtract_zero_page_x(),
            246 => self.increment_memory_zero_page_x(),
            247 => self.unofficial_increment_memory_subtract_acc_zero_page_x(),
            248 => self.set_decimal_flag(),
            249 => self.subtract_absolute_y(),
            250 => self.unofficial_nop(),
            251 => self.unofficial_increment_memory_subtract_acc_absolute_y(),
            252 => self.unofficial_triple_no_operation_page_penalty(4),
            253 => self.subtract_absolute_x(),
            254 => self.increment_memory_absolute_x(),
            255 => self.unofficial_increment_memory_subtract_acc_absolute_x(),
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

    // must handle pc wrapping, as 0xFFFE\0xFFFF stores interrupt vector
    fn get_absolute_address(&mut self) -> u16 {
        let low_byte = self.memory.borrow_mut().read(self.program_counter);
        self.program_counter += 1;
        let high_byte = self.memory.borrow_mut().read(self.program_counter);
        self.program_counter = ((self.program_counter as u32 + 1) & 0xFFFF) as u16;

         ((high_byte as u16) << 8) | low_byte as u16
    }

    fn get_absolute_address_with_offset(&mut self, offset: u16) -> u16 {
        ((self.get_absolute_address() as u32 + offset as u32) & 0xFFFF) as u16
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
        let address = ((base as u32 + offset as u32) & 0xFFFF) as u16;
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

    fn set_zero_negative_flags(&mut self, value: u8) {
        self.set_negative_flag(value);
        self.set_zero_flag(value);
    }

    fn load_a(&mut self, value: u8) {
        self.a = value;
        self.set_zero_negative_flags(value);
    }

    fn load_x(&mut self, value: u8) {
        self.x = value;
        self.set_zero_negative_flags(value);
    }

    fn load_y(&mut self, value: u8) {
        self.y = value;
        self.set_zero_negative_flags(value);
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
        self.set_zero_negative_flags(result);
    }

    fn do_inclusive_or(&mut self, operand: u8) {
        self.a = self.a | operand;
        let result = self.a;
        self.set_zero_negative_flags(result);
    }

    fn do_exclusive_or(&mut self, operand: u8) {
        self.a = self.a ^ operand;
        let result = self.a;
        self.set_zero_negative_flags(result);
    }

    fn do_compare(&mut self, register: u8, operand: u8) {
        // unset negative\zero\carry flags
        self.status_flags = self.status_flags & 0x7C;
        let result = register as i16 - operand as i16;

        if result < 0 {
            self.status_flags = self.status_flags | (result as u16 & 0x80) as u8;
        } else if result == 0 {
            self.status_flags = self.status_flags | 0x03;
        } else {
            self.status_flags = self.status_flags | 0x01 | (result as u16 & 0x80) as u8;
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
        let result = self.a as u16 + operand as u16 + (self.status_flags & 0x01) as u16;

        // clear carry, negative, overflow and zero flags
        self.status_flags = self.status_flags & 0x3C;

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

        // finally set negative and zero flags if necessary
        self.set_zero_negative_flags(result as u8);

        self.a = result as u8;
    }

    fn do_subtract(&mut self, operand: u8) {
        self.do_add(255 - operand);
    }

    fn do_rotate_right(&mut self, operand: u8) -> u8 {
        let result = operand >> 1 | ((self.status_flags & 0x01 ) << 7);
        self.set_zero_negative_flags(result);
        self.status_flags = (self.status_flags & 0xFE) | (operand & 0x01);
        result
    }

    fn do_logical_shift_right(&mut self, operand: u8) -> u8 {
        let result = self.do_rotate_right(operand)  & 0x7F;
        self.set_zero_negative_flags(result);
        result
    }

    fn do_rotate_left(&mut self, operand: u8) -> u8 {
        let result = operand << 1 | self.status_flags & 0x01;
        self.set_zero_negative_flags(result);
        self.status_flags = (self.status_flags & 0xFE) | ((operand & 0x80) >> 7);
        result
    }

    fn do_arithmetic_shift_left(&mut self, operand: u8) -> u8 {
        let result = self.do_rotate_left(operand) & 0xFE;
        self.set_zero_negative_flags(result);
        result
    }

    fn do_increment(&mut self, value: u8) -> u8 {
        let result = (value as u16) + 1;
        self.set_zero_negative_flags((result & 0xFF) as u8);
        (result & 0xFF) as u8
    }

    fn do_decrement(&mut self, value: u8) -> u8 {
        let result = (value as i16) - 1;
        self.set_zero_negative_flags((result & 0xFF) as u8);
        (result & 0xFF) as u8
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

    fn jump_indirect(&mut self) {
        self.wait_counter = 5;
        let indirect_address = self.get_absolute_address();

        // 6502 has a bug where high byte is fetched incorrectly when low byte resides
        // at xxFF. The high byte is incorrectly fetched from xx00 instead of
        // the beginning of the next page
        let low_byte = self.memory.borrow_mut().read(indirect_address) as u16;
        let high_byte = if indirect_address & 0x00FF == 0x00FF {
            self.memory.borrow_mut().read(indirect_address - 255) as u16
        } else {
            self.memory.borrow_mut().read(indirect_address + 1) as u16
        };

        let address = (high_byte << 8) | low_byte;
        self.program_counter = address;
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

    fn force_interrupt(&mut self) {
        let return_address = self.program_counter;
        self.push_value_into_stack(((return_address & 0xFF00) >> 8) as u8);
        self.push_value_into_stack((return_address & 0xFF) as u8);

        let flags = self.status_flags | 0x30; // bit 5 and 4 must be set
        self.push_value_into_stack(flags);
        self.program_counter = 0xFFFE;

        self.jump_absolute();
        self.wait_counter = 7;
    }

    fn return_from_interrupt(&mut self) {
        self.wait_counter = 6;

        let flags = self.pop_value_from_stack();
        let low_byte = self.pop_value_from_stack() as u16;
        let high_byte = self.pop_value_from_stack() as u16;

        self.program_counter = (high_byte << 8) | low_byte;
        self.status_flags = flags & 0xCF | (self.status_flags & 0x30); // flags 4 & 5 are ignored
    }


    fn bit_test_zero_page(&mut self) {
        let operand = self.read_zero_page();
        self.do_bit_test(operand);
    }

    fn bit_test_absolute(&mut self) {
        let operand = self.read_absolute();
        self.do_bit_test(operand);
    }

    fn rotate_right_accumulator(&mut self) {
        self.wait_counter = 2;
        let value = self.a;
        self.a = self.do_rotate_right(value);
    }

    fn rotate_right_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_rotate_right(value);
        // decrement PC so that store works
        self.program_counter -= 1;
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn rotate_right_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_rotate_right(value);
        // decrement PC so that store works
        self.program_counter -= 1;
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn rotate_right_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_rotate_right(value);
        // decrement PC so that store works
        self.program_counter -= 2;
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn rotate_right_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_rotate_right(value);
        // decrement PC so that store works
        self.program_counter -= 2;
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn logical_shift_right_accumulator(&mut self) {
        self.wait_counter = 2;
        let value = self.a;
        self.a = self.do_logical_shift_right(value);
    }

    fn logical_shift_right_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_logical_shift_right(value);
        // decrement PC so that store works
        self.program_counter -= 1;
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn logical_shift_right_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_logical_shift_right(value);
        // decrement PC so that store works
        self.program_counter -= 1;
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn logical_shift_right_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_logical_shift_right(value);
        // decrement PC so that store works
        self.program_counter -= 2;
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn logical_shift_right_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_logical_shift_right(value);
        // decrement PC so that store works
        self.program_counter -= 2;
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn rotate_left_accumulator(&mut self) {
        self.wait_counter = 2;
        let value = self.a;
        self.a = self.do_rotate_left(value);
    }

    fn rotate_left_zero_page(&mut self) {
        let value = self.read_zero_page();
        // move program counter back so that store works
        self.program_counter -= 1;
        let result = self.do_rotate_left(value);
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn rotate_left_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        // move program counter back so that store works
        self.program_counter -= 1;
        let result = self.do_rotate_left(value);
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn rotate_left_absolute(&mut self) {
        let value = self.read_absolute();
        // move program counter back so that store works
        self.program_counter -= 2;
        let result = self.do_rotate_left(value);
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn rotate_left_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        // move program counter back so that store works
        self.program_counter -= 2;
        let result = self.do_rotate_left(value);
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn arithmetic_shift_left_accumulator(&mut self) {
        self.wait_counter = 2;
        let value = self.a;
        self.a = self.do_arithmetic_shift_left(value);
    }

    fn arithmetic_shift_left_zero_page(&mut self) {
        let value = self.read_zero_page();
        // move program counter back so that store works
        self.program_counter -= 1;
        let result = self.do_arithmetic_shift_left(value);
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn arithmetic_shift_left_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        // move program counter back so that store works
        self.program_counter -= 1;
        let result = self.do_arithmetic_shift_left(value);
        self.do_zero_page_x_store(result);
        self.wait_counter = 5;
    }

    fn arithmetic_shift_left_absolute(&mut self) {
        let value = self.read_absolute();
        // move program counter back so that store works
        self.program_counter -= 2;
        let result = self.do_arithmetic_shift_left(value);
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn arithmetic_shift_left_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        // move program counter back so that store works
        self.program_counter -= 2;
        let result = self.do_arithmetic_shift_left(value);
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
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
        self.set_zero_negative_flags(value);
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

    fn load_y_immediate(&mut self) {
        let value = self.read_immediate();
        self.load_y(value);
    }

    fn load_y_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.load_y(value);
    }

    fn load_y_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        self.load_y(value);
    }

    fn load_y_absolute(&mut self) {
        let value = self.read_absolute();
        self.load_y(value);
    }

    fn load_y_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        self.load_y(value);
    }

    fn store_y_zero_page(&mut self) {
        let value = self.y;
        self.do_zero_page_store(value);
    }

    fn store_y_zero_page_x(&mut self) {
        let value = self.y;
        self.do_zero_page_x_store(value);
    }

    fn store_y_absolute(&mut self) {
        let value = self.y;
        self.do_absolute_store(value);
    }

    // unofficial opcodes
    fn unofficial_load_a_and_x_zero_page(&mut self) {
        let value = self.read_zero_page();
        self.load_a(value);
        self.load_x(value);
    }

    fn unofficial_load_a_and_x_zero_page_y(&mut self) {
        let value = self.read_zero_page_y();
        self.load_a(value);
        self.load_x(value);
    }

    fn unofficial_load_a_and_x_absolute(&mut self) {
        let value = self.read_absolute();
        self.load_a(value);
        self.load_x(value);
    }

    fn unofficial_load_a_and_x_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        self.load_a(value);
        self.load_x(value);
    }

    fn unofficial_load_a_and_x_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        self.load_a(value);
        self.load_x(value);
    }

    fn unofficial_load_a_and_x_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        self.load_a(value);
        self.load_x(value);
    }

    fn transfer_x_to_stack_pointer(&mut self) {
        self.wait_counter = 2;
        self.stack_pointer = self.x;
    }

    fn transfer_stack_pointer_to_x(&mut self) {
        self.wait_counter = 2;
        self.x = self.stack_pointer;
        let value = self.x;
        self.set_zero_negative_flags(value);
    }

    fn transfer_x_to_accumulator(&mut self) {
        self.wait_counter = 2;
        self.a = self.x;
        let value = self.a;
        self.set_zero_negative_flags(value);
    }

    fn transfer_accumulator_to_x(&mut self) {
        self.wait_counter = 2;
        self.x = self.a;
        let value = self.x;
        self.set_zero_negative_flags(value);
    }

    fn transfer_y_to_accumulator(&mut self) {
        self.wait_counter = 2;
        self.a = self.y;
        let value = self.a;
        self.set_zero_negative_flags(value);
    }

    fn transfer_accumulator_to_y(&mut self) {
        self.wait_counter = 2;
        self.y = self.a;
        let value = self.y;
        self.set_zero_negative_flags(value);
    }

    fn compare_immediate(&mut self) {
        let register = self.a;
        let operand = self.read_immediate();
        self.do_compare(register, operand);
    }

    fn compare_zero_page(&mut self) {
        let register = self.a;
        let operand = self.read_zero_page();
        self.do_compare(register, operand);
    }

    fn compare_zero_page_x(&mut self) {
        let register = self.a;
        let operand = self.read_zero_page_x();
        self.do_compare(register, operand);
    }

    fn compare_absolute(&mut self) {
        let register = self.a;
        let operand = self.read_absolute();
        self.do_compare(register, operand);
    }

    fn compare_absolute_x(&mut self) {
        let register = self.a;
        let operand = self.read_absolute_x();
        self.do_compare(register, operand);
    }

    fn compare_absolute_y(&mut self) {
        let register = self.a;
        let operand = self.read_absolute_y();
        self.do_compare(register, operand);
    }

    fn compare_indirect_x(&mut self) {
        let register = self.a;
        let operand = self.read_indirect_x();
        self.do_compare(register, operand);
    }

    fn compare_indirect_y(&mut self) {
        let register = self.a;
        let operand = self.read_indirect_y();
        self.do_compare(register, operand);
    }

    fn compare_x_immediate(&mut self) {
        let register = self.x;
        let operand = self.read_immediate();
        self.do_compare(register, operand);
    }

    fn compare_x_zero_page(&mut self) {
        let register = self.x;
        let operand = self.read_zero_page();
        self.do_compare(register, operand);
    }

    fn compare_x_absolute(&mut self) {
        let register = self.x;
        let operand = self.read_absolute();
        self.do_compare(register, operand);
    }

    fn compare_y_immediate(&mut self) {
        let register = self.y;
        let operand = self.read_immediate();
        self.do_compare(register, operand);
    }

    fn compare_y_zero_page(&mut self) {
        let register = self.y;
        let operand = self.read_zero_page();
        self.do_compare(register, operand);

    }

    fn compare_y_absolute(&mut self) {
        let register = self.y;
        let operand = self.read_absolute();
        self.do_compare(register, operand);
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

    // for unofficial opcode $EB. Different function for documentation\readability
    // purposes (immediately obvious that subtract_immediate isn't accidentally added twice to
    // instruction decoding )
    fn unofficial_subtract_immediate(&mut self) {
        self.subtract_immediate();
    }

    fn subtract_immediate(&mut self) {
        let operand = self.read_immediate();
        self.do_subtract(operand);
    }

    fn subtract_zero_page(&mut self) {
        let operand = self.read_zero_page();
        self.do_subtract(operand);
    }

    fn subtract_zero_page_x(&mut self) {
        let operand = self.read_zero_page_x();
        self.do_subtract(operand);
    }

    fn subtract_absolute(&mut self) {
        let operand = self.read_absolute();
        self.do_subtract(operand);
    }

    fn subtract_absolute_x(&mut self) {
        let operand = self.read_absolute_x();
        self.do_subtract(operand);
    }

    fn subtract_absolute_y(&mut self) {
        let operand = self.read_absolute_y();
        self.do_subtract(operand);
    }

    fn subtract_indirect_x(&mut self) {
        let operand = self.read_indirect_x();
        self.do_subtract(operand);
    }

    fn subtract_indirect_y(&mut self) {
        let operand = self.read_indirect_y();
        self.do_subtract(operand);
    }

    fn increase_x(&mut self) {
        self.wait_counter = 2;
        let value = self.x;
        self.x = self.do_increment(value);
    }


    fn decrease_x(&mut self) {
        self.wait_counter = 2;
        let value = self.x;
        self.x = self.do_decrement(value);
    }

    fn increase_y(&mut self) {
        self.wait_counter = 2;
        let value = self.y;
        self.y = self.do_increment(value);
    }

    fn decrease_y(&mut self) {
        self.wait_counter = 2;
        let value = self.y;
        self.y = self.do_decrement(value);
    }

    fn increment_memory_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_increment(value);
        self.program_counter -= 1; // need the zero page address again
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn increment_memory_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_increment(value);
        self.program_counter -= 1; // need the zero page address again
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn increment_memory_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_increment(value);
        self.program_counter -= 2; // need the zero page address again
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn increment_memory_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_increment(value);
        self.program_counter -= 2; // need the zero page address again
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn decrement_memory_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_decrement(value);
        self.program_counter -= 1; // need the zero page address again
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn decrement_memory_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_decrement(value);
        self.program_counter -= 1; // need the zero page address again
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn decrement_memory_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_decrement(value);
        self.program_counter -= 2; // need the zero page address again
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn decrement_memory_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_decrement(value);
        self.program_counter -= 2; // need the zero page address again
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn no_operation(&mut self) {
        self.wait_counter = 2;
    }

    fn unofficial_and_a_with_x_store_result_zero_page(&mut self) {
         let result = self.a & self.x;
         self.do_zero_page_store(result);
    }

    fn unofficial_and_a_with_x_store_result_zero_page_y(&mut self) {
        let result = self.a & self.x;
        self.do_zero_page_y_store(result);
    }

    fn unofficial_and_a_with_x_store_result_absolute(&mut self) {
        let result = self.a & self.x;
        self.do_absolute_store(result);
    }

    fn unofficial_and_a_with_x_store_result_indirect_x(&mut self) {
        let result = self.a & self.x;
        self.do_indirect_x_store(result);
    }


    fn unofficial_decrement_memory_and_compare_with_acc_zero_page(&mut self) {
        self.decrement_memory_zero_page();
        self.program_counter -= 1;
        let result = self.read_zero_page();
        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 5;
    }

    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_x(&mut self) {
        self.decrement_memory_zero_page_x();
        self.program_counter -= 1;
        let result = self.read_zero_page_x();
        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 6;
    }

    fn unofficial_decrement_memory_and_compare_with_acc_absolute(&mut self) {
        self.decrement_memory_absolute();
        self.program_counter -= 2;
        let result = self.read_absolute();
        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 6;
    }

    fn unofficial_decrement_memory_and_compare_with_acc_absolute_x(&mut self) {
        self.decrement_memory_absolute_x();
        self.program_counter -= 2;
        let result = self.read_absolute_x();
        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 7;
    }

    fn unofficial_decrement_memory_and_compare_with_acc_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        let result = self.do_decrement(value);
        self.program_counter -= 2;
        self.do_absolute_y_store(result);

        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 7;
    }

    fn unofficial_decrement_memory_and_compare_with_acc_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        let result = self.do_decrement(value);
        self.program_counter -= 1;
        self.do_indirect_x_store(result);

        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 8;
    }

    fn unofficial_decrement_memory_and_compare_with_acc_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        let result = self.do_decrement(value);
        self.program_counter -= 1;
        self.do_indirect_y_store(result);

        let register = self.a;
        self.do_compare(register, result);
        self.wait_counter = 8;
    }

    fn unofficial_increment_memory_subtract_acc_zero_page(&mut self) {
        self.increment_memory_zero_page();
        self.program_counter -= 1;
        let result = self.read_zero_page();
        self.do_subtract(result);
        self.wait_counter = 5;
    }

    fn unofficial_increment_memory_subtract_acc_zero_page_x(&mut self) {
        self.increment_memory_zero_page_x();
        self.program_counter -= 1;
        let result = self.read_zero_page_x();
        self.do_subtract(result);
        self.wait_counter = 6;
    }

    fn unofficial_increment_memory_subtract_acc_absolute(&mut self) {
        self.increment_memory_absolute();
        self.program_counter -= 2;
        let result = self.read_absolute();
        self.do_subtract(result);
        self.wait_counter = 6;
    }

    fn unofficial_increment_memory_subtract_acc_absolute_x(&mut self) {
        self.increment_memory_absolute_x();
        self.program_counter -= 2;
        let result = self.read_absolute_x();
        self.do_subtract(result);
        self.wait_counter = 7;
    }

    fn unofficial_increment_memory_subtract_acc_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        let result = self.do_increment(value);
        self.program_counter -= 2; // need the zero page address again
        self.do_absolute_y_store(result);

        self.do_subtract(result);
        self.wait_counter = 7;
    }

    fn unofficial_increment_memory_subtract_acc_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        let result = self.do_increment(value);
        self.program_counter -= 1; // need the zero page address again
        self.do_indirect_x_store(result);

        self.do_subtract(result);
        self.wait_counter = 8;
    }

    fn unofficial_increment_memory_subtract_acc_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        let result = self.do_increment(value);
        self.program_counter -= 1; // need the zero page address again
        self.do_indirect_y_store(result);

        self.do_subtract(result);
        self.wait_counter = 8;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 1;
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 1;
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 2;
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 2;
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 2;
        self.do_absolute_y_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 1;
        self.do_indirect_x_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        let result = self.do_arithmetic_shift_left(value);
        self.do_inclusive_or(result);
        self.program_counter -= 1;
        self.do_indirect_y_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_rotate_left(value);

        self.do_and(result);
        self.program_counter -= 1;
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_rotate_left(value);
        self.do_and(result);
        self.program_counter -= 1;
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_rotate_left(value);
        self.do_and(result);
        self.program_counter -= 2;
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_rotate_left(value);
        self.do_and(result);
        self.program_counter -= 2;
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        let result = self.do_rotate_left(value);
        self.do_and(result);
        self.program_counter -= 2;
        self.do_absolute_y_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        let result = self.do_rotate_left(value);
        self.do_and(result);
        self.program_counter -= 1;
        self.do_indirect_x_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        let result = self.do_rotate_left(value);
        self.do_and(result);
        self.program_counter -= 1;
        self.do_indirect_y_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_shift_right_memory_xor_acc_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_logical_shift_right(value);

        self.do_exclusive_or(result);
        self.program_counter -= 1;
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn  unofficial_shift_right_memory_xor_acc_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_logical_shift_right(value);
        self.do_exclusive_or(result);
        self.program_counter -= 1;
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn  unofficial_shift_right_memory_xor_acc_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_logical_shift_right(value);
        self.do_exclusive_or(result);
        self.program_counter -= 2;
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn  unofficial_shift_right_memory_xor_acc_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_logical_shift_right(value);
        self.do_exclusive_or(result);
        self.program_counter -= 2;
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_shift_right_memory_xor_acc_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        let result = self.do_logical_shift_right(value);
        self.do_exclusive_or(result);
        self.program_counter -= 2;
        self.do_absolute_y_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_shift_right_memory_xor_acc_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        let result = self.do_logical_shift_right(value);
        self.do_exclusive_or(result);
        self.program_counter -= 1;
        self.do_indirect_x_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_shift_right_memory_xor_acc_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        let result = self.do_logical_shift_right(value);
        self.do_exclusive_or(result);
        self.program_counter -= 1;
        self.do_indirect_y_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_rotate_right_memory_add_acc_zero_page(&mut self) {
        let value = self.read_zero_page();
        let result = self.do_rotate_right(value);

        self.do_add(result);
        self.program_counter -= 1;
        self.do_zero_page_store(result);
        self.wait_counter = 5;
    }

    fn unofficial_rotate_right_memory_add_acc_zero_page_x(&mut self) {
        let value = self.read_zero_page_x();
        let result = self.do_rotate_right(value);
        self.do_add(result);
        self.program_counter -= 1;
        self.do_zero_page_x_store(result);
        self.wait_counter = 6;
    }

    fn unofficial_rotate_right_memory_add_acc_absolute(&mut self) {
        let value = self.read_absolute();
        let result = self.do_rotate_right(value);
        self.do_add(result);
        self.program_counter -= 2;
        self.do_absolute_store(result);
        self.wait_counter = 6;
    }

    fn unofficial_rotate_right_memory_add_acc_absolute_x(&mut self) {
        let value = self.read_absolute_x();
        let result = self.do_rotate_right(value);
        self.do_add(result);
        self.program_counter -= 2;
        self.do_absolute_x_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_rotate_right_memory_add_acc_absolute_y(&mut self) {
        let value = self.read_absolute_y();
        let result = self.do_rotate_right(value);
        self.do_add(result);
        self.program_counter -= 2;
        self.do_absolute_y_store(result);
        self.wait_counter = 7;
    }

    fn unofficial_rotate_right_memory_add_acc_indirect_x(&mut self) {
        let value = self.read_indirect_x();
        let result = self.do_rotate_right(value);
        self.do_add(result);
        self.program_counter -= 1;
        self.do_indirect_x_store(result);
        self.wait_counter = 8;
    }

    fn unofficial_rotate_right_memory_add_acc_indirect_y(&mut self) {
        let value = self.read_indirect_y();
        let result = self.do_rotate_right(value);
        self.do_add(result);
        self.program_counter -= 1;
        self.do_indirect_y_store(result);
        self.wait_counter = 8;
    }


    // unofficial\illegal instructions may basically just do a read without
    // doing anything else with the result

    // for consistency
    fn unofficial_nop(&mut self) {
        self.no_operation();
    }

    fn unofficial_double_no_operation(&mut self, cycles: u8) {
        self.wait_counter = cycles;
        self.program_counter += 1;
    }

    fn unofficial_triple_no_operation_no_page_penalty(&mut self, cycles: u8) {
        self.wait_counter = cycles;
        self.program_counter += 2;
    }
    // add a cycle if page boundary is crossed
    fn unofficial_triple_no_operation_page_penalty(&mut self, cycles: u8) {
        let first = self.program_counter;
        let second = self.program_counter + 1;
        self.program_counter += 2;
        self.wait_counter = cycles;
        if first & 0xFF00 != second & 0xFF00 {
            self.wait_counter += 1;
        }
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

    // 64 kilobytes of memory, no mapped addresses
    struct MockMemory {
        ram: Vec<u8>
    }

    impl MockMemory {
        fn new() -> MockMemory {
            MockMemory {
                ram: vec![0;0xFFFF + 1],
            }
        }
    }

    impl Memory for MockMemory {
        fn read(&mut self, address: u16) -> u8 {
            self.ram[address as usize]
        }

        fn write(&mut self, address: u16, value: u8) {
            self.ram[address as usize] = value;
        }
    }

    fn create_test_cpu() -> Cpu {
        let memory = Rc::new(RefCell::new(Box::new(MockMemory::new()) as Box<Memory>));
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
    fn set_negative_flag_clears_the_flag_if_flag_is_set_and_value_was_positive() {
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
    fn set_zero_flag_clears_the_flag_if_flag_is_set_and_value_was_not_zero() {
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
    fn set_zero_negative_flags_sets_negative_flag_if_bit_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.set_zero_negative_flags(0x80);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn set_zero_negative_flags_clears_negative_flag_if_bit_is_unset() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.set_zero_negative_flags(0x40);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn set_zero_negative_flags_set_zero_flag_if_value_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.set_zero_negative_flags(0x00);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn set_zero_negative_flags_clears_zero_flag_if_value_is_nonzero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.set_zero_negative_flags(0x04);
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
    fn get_absolute_address_handles_program_counter_wrapping() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFFFE;
        cpu.memory.borrow_mut().write(0xFFFE, 0xBE);
        cpu.memory.borrow_mut().write(0xFFFF, 0xBA);
        assert_eq!(0xBABE, cpu.get_absolute_address());
        assert_eq!(0, cpu.program_counter);
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

    fn get_absolute_address_with_offset_hans() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x243;
        cpu.memory.borrow_mut().write(0x243, 0xFF);
        cpu.memory.borrow_mut().write(0x244, 0xFF);
        assert_eq!(0x42, cpu.get_absolute_address_with_offset(0x43));
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
    fn read_absolute_with_offset_handles_wrapping() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x432;
        cpu.memory.borrow_mut().write(0x432, 0xFF);
        cpu.memory.borrow_mut().write(0x433, 0xFF);
        cpu.memory.borrow_mut().write(0x0033, 0xC5);
        assert_eq!(0xC5, cpu.read_absolute_with_offset(0x34));
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
    fn do_and_clears_zero_flag_if_it_was_set_before_and_result_is_not_zero() {
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
    fn do_and_clears_negative_flag_if_flag_is_set_and_number_is_not_negative() {
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
    fn do_inclusive_or_clears_negative_flag_if_result_is_not_negative() {
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
    fn do_inclusive_or_clears_zero_flag_if_result_is_not_zero() {
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
    fn do_exclusive_or_clears_zero_flag_if_it_was_set_before_and_result_is_not_zero() {
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
    fn do_exclusive_or_clears_negative_flag_if_flag_is_set_and_number_is_not_negative() {
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
    fn do_compare_does_not_modify_registers() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;

        cpu.a = 0x40;
        cpu.x = 0x51;
        cpu.y = 0x62;

        cpu.do_compare(0x40, 0x20);
        assert_eq!(0x40, cpu.a);
        assert_eq!(0x51, cpu.x);
        assert_eq!(0x62, cpu.y);
    }


    #[test]
    fn do_compare_sets_carry_flag_if_register_is_greater_than_operand_and_result_has_no_sign_bit_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_compare(0x40, 0x20);
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn do_compare_sets_carry_flag_and_negative_flag_if_register_is_greater_but_subtraction_result_has_sign_bit_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_compare(0x80, 0x00);
        assert_eq!(0x81, cpu.status_flags);
    }


    #[test]
    fn do_compare_sets_carry_and_zero_flags_if_register_is_equal_operand() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_compare(0x40, 0x40);
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn do_compare_clears_carry_and_zero_flag_if_register_is_smaller_and_subtraction_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x03;
        cpu.do_compare(0x83, 0x90);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_compare_clears_carry_zero_and_negative_flag_if_register_is_smaller_and_subtraction_result_is_positive_due_to_overflow() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x83;
        cpu.do_compare(0x00, 0xFF);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_compare_clears_negative_flag_and_sets_carry_and_zero_flags_if_result_is_equal_and_negative_was_set_before() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0xEC;
        cpu.do_compare(0xFF, 0xFF);
        assert_eq!(0x6F, cpu.status_flags);
    }


    #[test]
    fn do_compare_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABCD;
        cpu.do_compare(0x40, 0x13);
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
    fn do_bit_test_clears_negative_flag_if_bit_is_not_set_and_flag_is_set() {
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
    fn do_bit_test_clears_overflow_bit_if_overflow_bit_is_not_set_and_flag_is_set() {
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
    fn do_bit_test_clears_zero_flag_if_result_is_not_zero() {
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
    fn do_add_sets_zero_flag_if_negative_and_positive_number_end_up_as_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 27;
        cpu.do_add(229);

        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_add_sets_zero_flag_if_result_is_zero_after_adding_carry() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 0x7F;
        cpu.do_add(0x80);

        assert_eq!(0x02, cpu.status_flags & 0x02);
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
    fn do_add_clears_zero_flag_if_flag_is_set_and_result_is_not_zero() {
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
    fn do_add_with_clears_overflow_flag_if_result_does_not_overflow() {
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
    fn do_subtract_subtracts_two_positive_numbers_correctly_when_result_is_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 40;
        cpu.do_subtract(10);
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn do_subtract_subtracts_borrow_if_carry_is_not_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 40;
        cpu.do_subtract(10);
        assert_eq!(29, cpu.a);
    }

    #[test]
    fn do_subtract_gives_correct_value_if_result_wraps_around() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 40;
        cpu.do_subtract(60);
        assert_eq!(236, cpu.a);
    }

    #[test]
    fn do_subtract_sets_clears_carry_flag_if_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 40;
        cpu.do_subtract(60);
        assert_eq!(0x00, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_subtract_sets_carry_if_result_is_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.a = 40;
        cpu.do_subtract(20);
        assert_eq!(0x01, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_subtract_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 40;
        cpu.do_subtract(40);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_subtract_clears_zero_flag_if_result_is_non_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x03;
        cpu.a = 40;
        cpu.do_subtract(10);
        assert_eq!(0x00, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_subtract_sets_overflow_flag_if_subtraction_is_too_small_to_be_represented_as_8_bit_signed() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x03;
        cpu.a = 208; // - 48
        cpu.do_subtract(112);
        assert_eq!(0x40, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_subtract_sets_overflow_flag_if_subtraction_is_too_big_to_be_represented_as_8_bit_signed() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x03;
        cpu.a = 80;
        cpu.do_subtract(176); // -80;
        assert_eq!(0x40, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_subtract_clears_overflow_flag_if_accumulator_was_positive_and_result_is_positive_and_fits() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x43;
        cpu.a = 80;
        cpu.do_subtract(40);
        assert_eq!(0x00, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_subtract_clears_overflow_flag_if_accumulator_was_positive_and_result_is_negative_and_fits() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x43;
        cpu.a = 80;
        cpu.do_subtract(100);
        assert_eq!(0x00, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_subtract_clears_overflow_flag_if_accumulator_was_negative_and_result_is_negative_and_fits() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x43;
        cpu.a = 0xFF;
        cpu.do_subtract(1);
        assert_eq!(0x00, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_subtract_clears_overflow_flag_if_accumulator_was_negative_and_result_is_positive_and_fits() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x43;
        cpu.a = 0xFF;
        cpu.do_subtract(0xFE);
        assert_eq!(0x00, cpu.status_flags & 0x40);
    }

    #[test]
    fn do_subtract_sets_negative_flag_if_end_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x03;
        cpu.a = 4;
        cpu.do_subtract(6);
        assert_eq!(0x80, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_subtract_clears_negative_flag_if_end_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x03;
        cpu.a = 4;
        cpu.do_subtract(6);
        assert_eq!(0x80, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_rotate_right_moves_bits_right_and_sets_bit_7_with_zero_when_carry_is_0() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        assert_eq!(0x3D, cpu.do_rotate_right(0x7B));
    }

    #[test]
    fn do_rotate_right_moves_bits_right_and_sets_bit_7_with_1_when_carry_is_1() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        assert_eq!(0xBD, cpu.do_rotate_right(0x7B));
    }

    #[test]
    fn do_rotate_right_sets_carry_to_1_if_old_bit_0_was_1() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_rotate_right(0x7B);
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn do_rotate_right_sets_carry_to_0_if_old_bit_0_was_0() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.do_rotate_right(0x72);
        assert_eq!(0x00, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_rotate_right_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_rotate_right(0x01);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_rotate_right_clears_zero_flag_if_result_is_non_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.do_rotate_right(0xF0);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_rotate_right_clears_negative_flag_if_original_was_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_rotate_right(0x70);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_rotate_right_clears_negative_flag_if_original_was_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_rotate_right(0xF0);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_rotate_right_sets_negative_flag_if_carry_is_1() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x81;
        cpu.do_rotate_right(0xF0);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_right_bitshift_moves_bits_right_and_fills_bit_7_with_zero_when_it_was_0() {
        let mut cpu = create_test_cpu();
        assert_eq!(0x3D, cpu.do_logical_shift_right(0x7A));
    }

    #[test]
    fn do_right_bitshift_moves_bits_right_and_fills_bit_7_with_zero_when_it_was_1() {
        let mut cpu = create_test_cpu();
        assert_eq!(0x7D, cpu.do_logical_shift_right(0xFA));
    }

    #[test]
    fn do_right_bitshift_moves_bits_right_and_fills_bit_7_with_zero_when_it_was_0_and_bit_0_was_1() {
        let mut cpu = create_test_cpu();
        assert_eq!(0x3D, cpu.do_logical_shift_right(0x7B));
    }

    #[test]
    fn do_right_bitshift_moves_bits_right_and_fills_bit_7_with_zero_when_it_was_1_and_bit_0_was_1() {
        let mut cpu = create_test_cpu();
        assert_eq!(0x7D, cpu.do_logical_shift_right(0xFB));
    }

    #[test]
    fn do_right_bitshift_sets_bit_7_0_even_if_carry_is_1() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        assert_eq!(0x00, cpu.do_logical_shift_right(0x00));
    }

    #[test]
    fn do_right_bitshift_sets_carry_to_1_if_old_bit_0_was_1() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_logical_shift_right(0x7B);
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn do_right_bitshift_sets_carry_to_0_if_old_bit_0_was_0() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.do_logical_shift_right(0x72);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_right_bitshift_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_logical_shift_right(0x01);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_right_bitshift_sets_zero_flag_if_result_is_zero_and_carry_flag_was_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.do_logical_shift_right(0x01);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_right_bitshift_clears_zero_flag_if_result_is_non_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.do_logical_shift_right(0xF0);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_right_bitshift_clears_negative_flag_if_original_was_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_logical_shift_right(0x70);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_right_bitshift_clears_negative_flag_if_original_was_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_logical_shift_right(0xF0);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_rotate_left_moves_bits_left() {
        let mut cpu = create_test_cpu();
        assert_eq!(0xE6, cpu.do_rotate_left(0x73));
    }

    #[test]
    fn do_rotate_left_sets_bit_0_to_1_if_carry_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        assert_eq!(0xE7, cpu.do_rotate_left(0x73));
    }

    #[test]
    fn do_rotate_left_clears_carry_if_bit_7_is_0_before_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.do_rotate_left(0x73);
        assert_eq!(0x00, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_rotate_left_sets_carry_if_bit_7_is_1_before_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_rotate_left(0xF3);
        assert_eq!(0x01, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_rotate_left_sets_negative_flag_if_bit_7_is_set_after_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_rotate_left(0x73);
        assert_eq!(0x80, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_rotate_left_clears_negative_flag_if_bit_7_is_not_set_after_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_rotate_left(0x13);
        assert_eq!(0x00, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_rotate_left_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_rotate_left(0x80);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_rotate_left_clears_zero_flag_if_result_is_non_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.do_rotate_left(0x32);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_arithmetic_shift_left_moves_bits_left() {
        let mut cpu = create_test_cpu();
        assert_eq!(0xE6, cpu.do_arithmetic_shift_left(0x73));
    }

    #[test]
    fn do_arithmetic_shift_left_does_not_set_bit_0_even_if_carry_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        assert_eq!(0xE6, cpu.do_arithmetic_shift_left(0x73));
    }

    #[test]
    fn do_arithmetic_shift_left_clears_carry_if_bit_7_is_0_before_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.do_arithmetic_shift_left(0x73);
        assert_eq!(0x00, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_arithmetic_shift_left_sets_carry_if_bit_7_is_1_before_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_arithmetic_shift_left(0xF3);
        assert_eq!(0x01, cpu.status_flags & 0x01);
    }

    #[test]
    fn do_arithmetic_shift_left_sets_negative_flag_if_bit_7_is_set_after_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_arithmetic_shift_left(0x73);
        assert_eq!(0x80, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_arithmetic_shift_left_clears_negative_flag_if_bit_7_is_not_set_after_shift() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_arithmetic_shift_left(0x13);
        assert_eq!(0x00, cpu.status_flags & 0x80);
    }

    #[test]
    fn do_arithmetic_shift_left_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_arithmetic_shift_left(0x80);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_arithmetic_shift_left_sets_zero_flag_if_result_is_zero_and_carry_is_set() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.do_arithmetic_shift_left(0x80);
        assert_eq!(0x02, cpu.status_flags & 0x02);
    }

    #[test]
    fn do_arithmetic_shift_left_clears_zero_flag_if_result_is_non_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.do_arithmetic_shift_left(0x32);
        assert_eq!(0x00, cpu.status_flags);
    }


    #[test]
    fn do_increment_increases_value_by_one() {
        let mut cpu = create_test_cpu();
        assert_eq!(21, cpu.do_increment(20));
    }

    #[test]
    fn do_increment_handles_overflow() {
        let mut cpu = create_test_cpu();
        assert_eq!(0, cpu.do_increment(255));
    }

    #[test]
    fn do_increment_negative_flag_if_result_is_negative() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_increment(0x7F);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_increment_clears_negative_flag_if_result_is_positive() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_increment(0x5);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_increment_sets_zero_flag_if_result_is_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_increment(0xFF);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_increment_clears_zero_flag_if_result_is_non_zero() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.do_increment(0x05);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_decrement_decreases_y_by_one() {
        let mut cpu = create_test_cpu();
        assert_eq!(20, cpu.do_decrement(21));
    }

    #[test]
    fn do_decrement_handles_overflow() {
        let mut cpu = create_test_cpu();
        assert_eq!(255, cpu.do_decrement(0));
    }

    #[test]
    fn do_decrement_sets_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_decrement(255);
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn do_decrement_clears_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x80;
        cpu.do_decrement(80);
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn do_decrement_sets_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x00;
        cpu.do_decrement(1);
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn do_decrement_clears_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x02;
        cpu.do_decrement(80);
        assert_eq!(0x00, cpu.status_flags);
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
        assert_eq!(0xF015, cpu.program_counter);
    }

    #[test]
    fn jump_absolute_sets_wait_counter_correctly() {
        let mut cpu = create_test_cpu();

        cpu.jump_absolute();
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn jump_indirect_sets_program_counter_to_new_value() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 5;
        cpu.memory.borrow_mut().write(5, 0x15);
        cpu.memory.borrow_mut().write(6, 0xF0);

        cpu.memory.borrow_mut().write(0xF015, 0xBA);
        cpu.memory.borrow_mut().write(0xF016, 0x0D);

        cpu.jump_indirect();
        assert_eq!(0x0DBA, cpu.program_counter);
    }

    #[test]
    fn jump_indirect_handles_6502_indirect_bug() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 5;
        cpu.memory.borrow_mut().write(5, 0xFF);
        cpu.memory.borrow_mut().write(6, 0xF0);

        cpu.memory.borrow_mut().write(0xF0FF, 0xBA);
        cpu.memory.borrow_mut().write(0xF100, 0x0D);
        cpu.memory.borrow_mut().write(0xF000, 0xDB);

        cpu.jump_indirect();
        assert_eq!(0xDBBA, cpu.program_counter);
    }

    #[test]
    fn jump_indirect_sets_wait_counter_correctly() {
        let mut cpu = create_test_cpu();

        cpu.jump_indirect();
        assert_eq!(5, cpu.wait_counter);
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
    fn force_interrupt_jumps_to_address_stored_at_interrupt_vector() {

        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x40;
        cpu.program_counter = 0x40;

        cpu.memory.borrow_mut().write(0xFFFE, 0x20);
        cpu.memory.borrow_mut().write(0xFFFF, 0xA3);

        cpu.force_interrupt();
        assert_eq!(0xA320, cpu.program_counter);
    }

    #[test]
    fn force_interrupt_pushes_3_values_into_stack() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x40;
        cpu.force_interrupt();
        assert_eq!(0x3D, cpu.stack_pointer);
    }

    #[test]
    fn force_interrupt_pushes_status_flags_to_top_of_stack_with_bits_4_and_5_set() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x40;
        cpu.status_flags = 0x82;
        cpu.force_interrupt();
        assert_eq!(0x82 | 0x30, cpu.pop_value_from_stack());
    }

    #[test]
    fn force_interrupt_pushes_old_pc_before_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xA0EF;
        cpu.stack_pointer = 0x40;
        cpu.status_flags = 0x82;
        cpu.force_interrupt();

        cpu.pop_value_from_stack();

        assert_eq!(0xEF, cpu.pop_value_from_stack());
        assert_eq!(0xA0, cpu.pop_value_from_stack());
    }

    #[test]
    fn force_interrupt_does_not_modify_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x80;
        cpu.status_flags = 0x0A;
        cpu.force_interrupt();
        assert_eq!(0x0A, cpu.status_flags);
    }

    #[test]
    fn force_interrupt_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x80;
        cpu.force_interrupt();
        assert_eq!(7, cpu.wait_counter);
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

    #[test]
    fn return_from_interrupt_sets_the_program_counter_correctly() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x10;
        cpu.program_counter = 0x10;
        cpu.push_value_into_stack(0xD8); // high byte
        cpu.push_value_into_stack(0xBE); // low byte
        cpu.push_value_into_stack(0x13);

        cpu.return_from_interrupt();

        assert_eq!(0xD8BE, cpu.program_counter);
    }

    #[test]
    fn return_from_interrupt_increments_stack_pointer_by_3() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x10;
        cpu.return_from_interrupt();
        assert_eq!(0x10 + 3, cpu.stack_pointer);
    }

    #[test]
    fn return_from_interrupt_sets_status_flags_to_value_from_stack_but_ignore_bits_4_and_5() {
        let mut cpu = create_test_cpu();

        cpu.stack_pointer = 0x10;
        cpu.status_flags = 0x01;

        cpu.push_value_into_stack(0xFE);

        cpu.return_from_interrupt();
        assert_eq!(0xCE, cpu.status_flags);
    }

    #[test]
    fn return_from_interrupt_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x10;
        cpu.return_from_interrupt();
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
    fn  rotate_right_accumulator_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE6;
        cpu.status_flags = 0x01;
        cpu.rotate_right_accumulator();
        assert_eq!(0xF3, cpu.a);
    }

    #[test]
    fn rotate_right_accumulator_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.rotate_right_accumulator();
        assert_eq!(0x2442, cpu.program_counter);
    }

    #[test]
    fn rotate_right_accumulator_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_right_accumulator();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn rotate_right_zero_page_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x0F40;
        cpu.status_flags = 0x01;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x70, 0xE6);
        cpu.rotate_right_zero_page();
        assert_eq!(0xF3, cpu.memory.borrow_mut().read(0x70));
    }

    #[test]
    fn rotate_right_zero_page_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.rotate_right_zero_page();
        assert_eq!(0x2443, cpu.program_counter);
    }

    #[test]
    fn rotate_right_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_right_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn rotate_right_zero_page_x_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x20;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x0F40;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x70 + 0x20, 0xE6);
        cpu.rotate_right_zero_page_x();
        assert_eq!(0xF3, cpu.memory.borrow_mut().read(0x70 + 0x20));
    }

    #[test]
    fn rotate_right_zero_page_x_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.rotate_right_zero_page_x();
        assert_eq!(0x2443, cpu.program_counter);
    }

    #[test]
    fn rotate_right_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_right_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn rotate_right_absolute_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x0F40;
        cpu.status_flags = 0x01;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x0F41, 0xB1);
        cpu.memory.borrow_mut().write(0xB170, 0xE6);
        cpu.rotate_right_absolute();
        assert_eq!(0xF3, cpu.memory.borrow_mut().read(0xB170));
    }

    #[test]
    fn rotate_right_absolute_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.rotate_right_absolute();
        assert_eq!(0x2444, cpu.program_counter);
    }

    #[test]
    fn rotate_right_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_right_absolute();
        assert_eq!(6, cpu.wait_counter);
    }


    #[test]
    fn rotate_right_absolute_x_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x20;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x0F40;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x0F41, 0xB1);
        cpu.memory.borrow_mut().write(0xB170 + 0x20, 0xE6);
        cpu.rotate_right_absolute_x();
        assert_eq!(0xF3, cpu.memory.borrow_mut().read(0xB170 + 0x20));
    }

    #[test]
    fn rotate_right_absolute_x_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.rotate_right_absolute_x();
        assert_eq!(0x2444, cpu.program_counter);
    }

    #[test]
    fn rotate_right_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_right_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn logical_right_shift_accumulator_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xE6;
        cpu.logical_shift_right_accumulator();
        assert_eq!(0x73, cpu.a);
    }

    #[test]
    fn logical_right_shift_accumulator_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.logical_shift_right_accumulator();
        assert_eq!(0x2442, cpu.program_counter);
    }

    #[test]
    fn logical_right_shift_accumulator_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.logical_shift_right_accumulator();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn logical_right_shift_zero_page_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x0F40;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x70, 0xE6);
        cpu.logical_shift_right_zero_page();
        assert_eq!(0x73, cpu.memory.borrow_mut().read(0x70));
    }

    #[test]
    fn logical_right_shift_zero_page_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.logical_shift_right_zero_page();
        assert_eq!(0x2443, cpu.program_counter);
    }

    #[test]
    fn logical_right_shift_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.logical_shift_right_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn logical_right_shift_zero_page_x_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x20;
        cpu.program_counter = 0x0F40;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x70 + 0x20, 0xE6);
        cpu.logical_shift_right_zero_page_x();
        assert_eq!(0x73, cpu.memory.borrow_mut().read(0x70 + 0x20));
    }

    #[test]
    fn logical_right_shift_zero_page_x_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.logical_shift_right_zero_page_x();
        assert_eq!(0x2443, cpu.program_counter);
    }

    #[test]
    fn logical_right_shift_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.logical_shift_right_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn logical_right_shift_absolute_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x0F40;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x0F41, 0xB1);
        cpu.memory.borrow_mut().write(0xB170, 0xE6);
        cpu.logical_shift_right_absolute();
        assert_eq!(0x73, cpu.memory.borrow_mut().read(0xB170));
    }

    #[test]
    fn logical_right_shift_absolute_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.logical_shift_right_absolute();
        assert_eq!(0x2444, cpu.program_counter);
    }

    #[test]
    fn logical_right_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.logical_shift_right_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn logical_right_shift_absolute_x_modifies_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x20;
        cpu.program_counter = 0x0F40;
        cpu.memory.borrow_mut().write(0x0F40, 0x70);
        cpu.memory.borrow_mut().write(0x0F41, 0xB1);
        cpu.memory.borrow_mut().write(0xB170 + 0x20, 0xE6);
        cpu.logical_shift_right_absolute_x();
        assert_eq!(0x73, cpu.memory.borrow_mut().read(0xB170 + 0x20));
    }

    #[test]
    fn logical_right_shift_absolute_x_does_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x2442;
        cpu.logical_shift_right_absolute_x();
        assert_eq!(0x2444, cpu.program_counter);
    }

    #[test]
    fn logical_right_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.logical_shift_right_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn rotate_left_accumulator_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 0x35;
        cpu.rotate_left_accumulator();
        assert_eq!(0x6B, cpu.a);
    }

    #[test]
    fn rotate_left_accumulator_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.rotate_left_accumulator();
        assert_eq!(0x643, cpu.program_counter);
    }

    #[test]
    fn rotate_left_accumulator_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_left_accumulator();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn rotate_left_zero_page_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xFE);
        cpu.memory.borrow_mut().write(0xFE, 0x35);

        cpu.rotate_left_zero_page();
        assert_eq!(0x6B, cpu.memory.borrow_mut().read(0xFE));
    }

    #[test]
    fn rotate_left_zero_page_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.rotate_left_zero_page();
        assert_eq!(0x644, cpu.program_counter);
    }

    #[test]
    fn rotate_left_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_left_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn rotate_left_zero_page_x_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xAE);
        cpu.memory.borrow_mut().write(0xAE + 0x13, 0x35);

        cpu.rotate_left_zero_page_x();
        assert_eq!(0x6B, cpu.memory.borrow_mut().read(0xAE + 0x13));
    }

    #[test]
    fn rotate_left_zero_page_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.rotate_left_zero_page_x();
        assert_eq!(0x644, cpu.program_counter);
    }

    #[test]
    fn rotate_left_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_left_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn rotate_left_absolute_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xAE);
        cpu.memory.borrow_mut().write(0x235, 0xF1);
        cpu.memory.borrow_mut().write(0xF1AE, 0x35);

        cpu.rotate_left_absolute();
        assert_eq!(0x6B, cpu.memory.borrow_mut().read(0xF1AE));
    }

    #[test]
    fn rotate_left_absolute_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.rotate_left_absolute();
        assert_eq!(0x645, cpu.program_counter);
    }

    #[test]
    fn rotate_left_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_left_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn rotate_left_absolute_x_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.x = 0x20;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xAE);
        cpu.memory.borrow_mut().write(0x235, 0xF1);
        cpu.memory.borrow_mut().write(0xF1AE + 0x20, 0x35);

        cpu.rotate_left_absolute_x();
        assert_eq!(0x6B, cpu.memory.borrow_mut().read(0xF1AE + 0x20));
    }

    #[test]
    fn rotate_left_absolute_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.rotate_left_absolute_x();
        assert_eq!(0x645, cpu.program_counter);
    }

    #[test]
    fn rotate_left_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.rotate_left_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn arithmetic_shift_left_accumulator_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.a = 0x35;
        cpu.arithmetic_shift_left_accumulator();
        assert_eq!(0x6A, cpu.a);
    }

    #[test]
    fn arithmetic_shift_left_accumulator_does_not_modify_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.arithmetic_shift_left_accumulator();
        assert_eq!(0x643, cpu.program_counter);
    }

    #[test]
    fn arithmetic_shift_left_accumulator_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.arithmetic_shift_left_accumulator();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn arithmetic_shift_left_zero_page_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xFE);
        cpu.memory.borrow_mut().write(0xFE, 0x35);

        cpu.arithmetic_shift_left_zero_page();
        assert_eq!(0x6A, cpu.memory.borrow_mut().read(0xFE));
    }

    #[test]
    fn arithmetic_shift_left_zero_page_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.arithmetic_shift_left_zero_page();
        assert_eq!(0x644, cpu.program_counter);
    }

    #[test]
    fn arithmetic_shift_left_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.arithmetic_shift_left_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn arithmetic_shift_left_zero_page_x_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.x = 0x24;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xAE);
        cpu.memory.borrow_mut().write(0xAE + 0x24, 0x35);

        cpu.arithmetic_shift_left_zero_page_x();
        assert_eq!(0x6A, cpu.memory.borrow_mut().read(0xAE + 0x24));
    }

    #[test]
    fn arithmetic_shift_left_zero_x_page_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.arithmetic_shift_left_zero_page_x();
        assert_eq!(0x644, cpu.program_counter);
    }

    #[test]
    fn arithmetic_shift_left_zero_x_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.arithmetic_shift_left_zero_page_x();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn arithmetic_shift_left_absolute_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xFE);
        cpu.memory.borrow_mut().write(0x235, 0xCA);
        cpu.memory.borrow_mut().write(0xCAFE, 0x35);

        cpu.arithmetic_shift_left_absolute();
        assert_eq!(0x6A, cpu.memory.borrow_mut().read(0xCAFE));
    }

    #[test]
    fn arithmetic_shift_left_absolute_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.arithmetic_shift_left_absolute();
        assert_eq!(0x645, cpu.program_counter);
    }

    #[test]
    fn arithmetic_shift_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.arithmetic_shift_left_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn arithmetic_shift_left_absolute_x_stores_correct_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.status_flags = 0x01;
        cpu.x = 0x65;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0xFE);
        cpu.memory.borrow_mut().write(0x235, 0xCA);
        cpu.memory.borrow_mut().write(0xCAFE + 0x65, 0x35);

        cpu.arithmetic_shift_left_absolute_x();
        assert_eq!(0x6A, cpu.memory.borrow_mut().read(0xCAFE + 0x65));
    }

    #[test]
    fn arithmetic_shift_left_absolute_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x643;
        cpu.arithmetic_shift_left_absolute_x();
        assert_eq!(0x645, cpu.program_counter);
    }

    #[test]
    fn arithmetic_shift_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.arithmetic_shift_left_absolute_x();
        assert_eq!(7, cpu.wait_counter);
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
    fn pull_accumulator_clears_zero_flag_if_value_pulled_was_not_zero() {
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
    fn pull_accumulator_clears_negative_flag_if_value_pulled_was_not_negative() {
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
    fn load_y_immediate_sets_y_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x70;
        cpu.memory.borrow_mut().write(0x70, 0x2F);

        cpu.load_y_immediate();
        assert_eq!(0x2F, cpu.y);
    }

    #[test]
    fn load_y_zero_page_sets_y_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x720;
        cpu.memory.borrow_mut().write(0x720, 0x40);
        cpu.memory.borrow_mut().write(0x40, 0x2F);

        cpu.load_y_zero_page();
        assert_eq!(0x2F, cpu.y);
    }

    #[test]
    fn load_y_zero_page_x_sets_y_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.x = 0x14;
        cpu.program_counter = 0x720;
        cpu.memory.borrow_mut().write(0x720, 0x40);
        cpu.memory.borrow_mut().write(0x40 + 0x14, 0x2F);

        cpu.load_y_zero_page_x();
        assert_eq!(0x2F, cpu.y);
    }

    #[test]
    fn load_y_absolute_sets_y_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x720;
        cpu.memory.borrow_mut().write(0x720, 0x40);
        cpu.memory.borrow_mut().write(0x721, 0xBE);
        cpu.memory.borrow_mut().write(0xBE40, 0x2F);

        cpu.load_y_absolute();
        assert_eq!(0x2F, cpu.y);
    }

    #[test]
    fn load_y_absolute_x_sets_y_to_a_correct_value() {
        let mut cpu = create_test_cpu();

        cpu.x = 0x25;
        cpu.program_counter = 0x720;
        cpu.memory.borrow_mut().write(0x720, 0x40);
        cpu.memory.borrow_mut().write(0x721, 0xBE);
        cpu.memory.borrow_mut().write(0xBE40 + 0x25, 0x2F);

        cpu.load_y_absolute_x();
        assert_eq!(0x2F, cpu.y);
    }


    #[test]
    fn store_y_zero_page_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x2f;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.store_y_zero_page();
        assert_eq!(0x2f, cpu.memory.borrow_mut().read(0x14));
    }

    #[test]
    fn store_y_zero_page_x_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x2f;
        cpu.x = 0x53;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.store_y_zero_page_x();
        assert_eq!(0x2f, cpu.memory.borrow_mut().read(0x14 + 0x53));
    }

    #[test]
    fn store_y_absolute_stores_value_into_memory_correctly() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x2f;
        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.memory.borrow_mut().write(0x33, 0x08);

        cpu.store_y_absolute();
        assert_eq!(0x2f, cpu.memory.borrow_mut().read(0x0814));
    }

    #[test]
    fn unofficial_load_a_and_x_zero_page_loads_correct_values() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.x = 0;
        cpu.a = 0;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.memory.borrow_mut().write(0x14, 0xDF);
        cpu.unofficial_load_a_and_x_zero_page();
        assert_eq!(0xDF, cpu.a);
        assert_eq!(0xDF, cpu.x);
    }

    #[test]
    fn unofficial_load_a_and_x_zero_page_y_loads_correct_values() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.x = 0;
        cpu.a = 0;
        cpu.y = 0x42;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.memory.borrow_mut().write(0x14 + 0x42, 0xDF);
        cpu.unofficial_load_a_and_x_zero_page_y();
        assert_eq!(0xDF, cpu.a);
        assert_eq!(0xDF, cpu.x);
    }

    #[test]
    fn unofficial_load_a_and_x_absolute_loads_correct_values() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.x = 0;
        cpu.a = 0;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.memory.borrow_mut().write(0x33, 0xAF);
        cpu.memory.borrow_mut().write(0xAF14, 0xDF);

        cpu.unofficial_load_a_and_x_absolute();
        assert_eq!(0xDF, cpu.a);
        assert_eq!(0xDF, cpu.x);
    }

    #[test]
    fn unofficial_load_a_and_x_absolute_y_loads_correct_values() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.x = 0;
        cpu.a = 0;
        cpu.y = 0x12;
        cpu.memory.borrow_mut().write(0x32, 0x14);
        cpu.memory.borrow_mut().write(0x33, 0xAF);
        cpu.memory.borrow_mut().write(0xAF14 + 0x12, 0xDF);

        cpu.unofficial_load_a_and_x_absolute_y();
        assert_eq!(0xDF, cpu.a);
        assert_eq!(0xDF, cpu.x);
    }

    #[test]
    fn unofficial_load_a_and_x_indirect_x_loads_correct_values() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.x = 0x23;
        cpu.a = 0;
        cpu.memory.borrow_mut().write(0x32, 0x20);

        cpu.memory.borrow_mut().write(0x20 + 0x23, 0x14);
        cpu.memory.borrow_mut().write(0x21 + 0x23, 0xAF);

        cpu.memory.borrow_mut().write(0xAF14, 0xDF);

        cpu.unofficial_load_a_and_x_indirect_x();
        assert_eq!(0xDF, cpu.a);
        assert_eq!(0xDF, cpu.x);
    }

    #[test]
    fn unofficial_load_a_and_x_indirect_y_loads_correct_values() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.x = 0x23;
        cpu.y = 0x5D;
        cpu.a = 0;
        cpu.memory.borrow_mut().write(0x32, 0x20);

        cpu.memory.borrow_mut().write(0x20, 0x14);
        cpu.memory.borrow_mut().write(0x21, 0xAF);

        cpu.memory.borrow_mut().write(0xAF14 + 0x5D, 0xDF);

        cpu.unofficial_load_a_and_x_indirect_y();
        assert_eq!(0xDF, cpu.a);
        assert_eq!(0xDF, cpu.x);
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
    fn transfer_stack_pointer_to_x_sets_x_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x2F;
        cpu.x = 0x4B;
        cpu.transfer_stack_pointer_to_x();
        assert_eq!(0x2F, cpu.x);
    }

    #[test]
    fn transfer_x_to_stack_pointer_sets_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x00;
        cpu.x = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_stack_pointer_to_x();
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn transfer_stack_pointer_to_x_clears_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x05;
        cpu.x = 0x01;
        cpu.status_flags = 0x02;
        cpu.transfer_stack_pointer_to_x();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_stack_pointer_to_x_sets_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0xF0;
        cpu.x = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_stack_pointer_to_x();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn transfer_stack_pointer_to_x_clears_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.stack_pointer = 0x05;
        cpu.x = 0x01;
        cpu.status_flags = 0x80;
        cpu.transfer_stack_pointer_to_x();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_stack_pointer_to_x_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.transfer_stack_pointer_to_x();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn transfer_x_to_accumulator_sets_accumulator_value_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x2F;
        cpu.a = 0x01;
        cpu.transfer_x_to_accumulator();
        assert_eq!(0x2F, cpu.a);
    }

    #[test]
    fn transfer_x_to_accumulator_sets_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x00;
        cpu.a = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_x_to_accumulator();
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn transfer_x_to_accumulator_clears_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x05;
        cpu.a = 0x01;
        cpu.status_flags = 0x02;
        cpu.transfer_x_to_accumulator();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_x_to_accumulator_sets_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.x = 0xF0;
        cpu.a = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_x_to_accumulator();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn transfer_x_to_accumulator_clears_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x05;
        cpu.a = 0x01;
        cpu.status_flags = 0x80;
        cpu.transfer_x_to_accumulator();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_x_to_accumulator_takes_2_cycles() {
        let mut cpu = create_test_cpu();

        cpu.transfer_x_to_accumulator();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn transfer_accumulator_to_x_sets_x_value_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.x = 0x01;
        cpu.transfer_accumulator_to_x();
        assert_eq!(0x2F, cpu.x);
    }

    #[test]
    fn transfer_accumulator_to_x_sets_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.x = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_accumulator_to_x();
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_x_clears_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x05;
        cpu.x = 0x01;
        cpu.status_flags = 0x02;
        cpu.transfer_accumulator_to_x();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_x_sets_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xF0;
        cpu.x = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_accumulator_to_x();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_x_clears_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x05;
        cpu.x = 0x01;
        cpu.status_flags = 0x80;
        cpu.transfer_accumulator_to_x();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_x_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.transfer_accumulator_to_x();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn transfer_y_to_accumulator_sets_accumulator_value_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x2F;
        cpu.a = 0x01;
        cpu.transfer_y_to_accumulator();
        assert_eq!(0x2F, cpu.a);
    }

    #[test]
    fn transfer_y_to_accumulator_sets_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x00;
        cpu.a = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_y_to_accumulator();
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn transfer_y_to_accumulator_clears_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x05;
        cpu.a = 0x01;
        cpu.status_flags = 0x02;
        cpu.transfer_y_to_accumulator();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_y_to_accumulator_sets_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.y = 0xF0;
        cpu.a = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_y_to_accumulator();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn transfer_y_to_accumulator_clears_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x05;
        cpu.a = 0x01;
        cpu.status_flags = 0x80;
        cpu.transfer_y_to_accumulator();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_y_to_accumulator_takes_2_cycles() {
        let mut cpu = create_test_cpu();

        cpu.transfer_y_to_accumulator();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn transfer_accumulator_to_y_sets_y_value_to_correct_value() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x2F;
        cpu.y = 0x01;
        cpu.transfer_accumulator_to_y();
        assert_eq!(0x2F, cpu.y);
    }

    #[test]
    fn transfer_accumulator_to_y_sets_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.y = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_accumulator_to_y();
        assert_eq!(0x02, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_y_clears_zero_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x05;
        cpu.y = 0x01;
        cpu.status_flags = 0x02;
        cpu.transfer_accumulator_to_y();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_y_sets_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xF0;
        cpu.y = 0x01;
        cpu.status_flags = 0x00;
        cpu.transfer_accumulator_to_y();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_y_clears_negative_flag() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x05;
        cpu.y = 0x01;
        cpu.status_flags = 0x80;
        cpu.transfer_accumulator_to_y();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn transfer_accumulator_to_y_takes_2_cycles() {
        let mut cpu = create_test_cpu();

        cpu.transfer_accumulator_to_y();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn compare_immediate_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x12);
        cpu.status_flags = 0x00;
        cpu.a = 0x4F;

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
    fn compare_immediate_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
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
        cpu.a = 0x4F;

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
    fn compare_zero_page_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
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
        cpu.a = 0x4F;

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
    fn compare_zero_page_x_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
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
        cpu.a = 0x2F;

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
    fn compare_absolute_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
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
        cpu.a = 0x4F;

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
    fn compare_absolute_x_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xFA;
        cpu.status_flags = 0x103;
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
        cpu.a = 0x4F;

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
    fn compare_absolute_y_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xFA;
        cpu.status_flags = 0x03;
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
        cpu.a = 0x4F;

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
    fn compare_indirect_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.x = 0xBA;
        cpu.status_flags = 0x03;
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
        cpu.a = 0x40;

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
    fn compare_indirect_y_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.y = 0xBA;
        cpu.status_flags = 0x03;
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
    fn compare_x_immediate_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x12);
        cpu.status_flags = 0x00;
        cpu.x = 0x4F;

        cpu.compare_x_immediate();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_x_immediate_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x40);
        cpu.status_flags = 0x00;
        cpu.x = 0x40;

        cpu.compare_x_immediate();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_x_immediate_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x40);
        cpu.x = 0x39;

        cpu.compare_x_immediate();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_x_zero_page_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x12);
        cpu.status_flags = 0x00;
        cpu.x = 0x4F;

        cpu.compare_x_zero_page();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_x_zero_page_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x40);
        cpu.status_flags = 0x00;
        cpu.x = 0x40;

        cpu.compare_x_zero_page();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_x_zero_page_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x40);
        cpu.x = 0x39;

        cpu.compare_x_zero_page();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_x_absolute_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x12);
        cpu.status_flags = 0x00;
        cpu.x = 0x2F;

        cpu.compare_x_absolute();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_x_absolute_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.status_flags = 0x00;
        cpu.x = 0x40;

        cpu.compare_x_absolute();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_x_absolute_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.x = 0x39;

        cpu.compare_x_absolute();
        assert_eq!(0x80, cpu.status_flags);
    }


    #[test]
    fn compare_y_immediate_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x12);
        cpu.status_flags = 0x00;
        cpu.y = 0x4F;

        cpu.compare_y_immediate();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_y_immediate_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x40);
        cpu.status_flags = 0x00;
        cpu.y = 0x40;

        cpu.compare_y_immediate();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_y_immediate_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x40);
        cpu.y = 0x39;

        cpu.compare_y_immediate();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_y_zero_page_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x12);
        cpu.status_flags = 0x00;
        cpu.y = 0x4F;

        cpu.compare_y_zero_page();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_y_zero_page_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x40);
        cpu.status_flags = 0x00;
        cpu.y = 0x40;

        cpu.compare_y_zero_page();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_y_zero_page_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x50, 0x40);
        cpu.y = 0x39;

        cpu.compare_y_zero_page();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn compare_y_absolute_sets_carry_flag_if_accumulator_is_greater() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x12);
        cpu.status_flags = 0x00;
        cpu.y = 0x2F;

        cpu.compare_y_absolute();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn compare_y_absolute_sets_carry_flag_and_zero_flag_if_accumulator_is_equal() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.status_flags = 0x00;
        cpu.y = 0x40;

        cpu.compare_y_absolute();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn compare_y_absolute_clears_carry_zero_flags_and_sets_negative_if_accumulator_is_smaller() {
        let mut cpu = create_test_cpu();

        cpu.status_flags = 0x03;
        cpu.program_counter = 0x123;
        cpu.memory.borrow_mut().write(0x123, 0x50);
        cpu.memory.borrow_mut().write(0x124, 0x80);
        cpu.memory.borrow_mut().write(0x8050, 0x40);
        cpu.y = 0x39;

        cpu.compare_y_absolute();
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
    fn subtract_immediate_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x30;
        cpu.memory.borrow_mut().write(0x30, 19);
        cpu.subtract_immediate();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_zero_page_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x30);
        cpu.memory.borrow_mut().write(0x30, 19);

        cpu.subtract_zero_page();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_zero_page_x_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.x = 0x20;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x30);
        cpu.memory.borrow_mut().write(0x30 + 0x20, 19);

        cpu.subtract_zero_page_x();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_absolute_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x30);
        cpu.memory.borrow_mut().write(0x08F1, 0xB0);

        cpu.memory.borrow_mut().write(0xB030, 19);

        cpu.subtract_absolute();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_absolute_x_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.x = 0x70;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x30);
        cpu.memory.borrow_mut().write(0x08F1, 0xB0);

        cpu.memory.borrow_mut().write(0xB030 + 0x70, 19);

        cpu.subtract_absolute_x();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_absolute_y_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.y = 0x70;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x30);
        cpu.memory.borrow_mut().write(0x08F1, 0xB0);

        cpu.memory.borrow_mut().write(0xB030 + 0x70, 19);

        cpu.subtract_absolute_y();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_indirect_x_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.x = 0x05;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x70);

        cpu.memory.borrow_mut().write(0x70 + 0x05, 0x30);
        cpu.memory.borrow_mut().write(0x71 + 0x05, 0xB0);

        cpu.memory.borrow_mut().write(0xB030, 19);

        cpu.subtract_indirect_x();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn subtract_indirect_y_stores_correct_value_in_accumulator() {
        let mut cpu = create_test_cpu();

        cpu.a = 49;
        cpu.y = 0x05;
        cpu.status_flags = 0x01;
        cpu.program_counter = 0x08F0;
        cpu.memory.borrow_mut().write(0x08F0, 0x70);

        cpu.memory.borrow_mut().write(0x70, 0x30);
        cpu.memory.borrow_mut().write(0x71, 0xB0);

        cpu.memory.borrow_mut().write(0xB030 + 0x05, 19);

        cpu.subtract_indirect_y();
        assert_eq!(30, cpu.a);
    }

    #[test]
    fn increase_x_increases_value_by_one() {
        let mut cpu = create_test_cpu();

        cpu.x = 20;
        cpu.increase_x();
        assert_eq!(21, cpu.x);
    }

    #[test]
    fn increase_x_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.increase_x();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn decrease_x_decreases_x_by_one() {
        let mut cpu = create_test_cpu();
        cpu.x = 21;
        cpu.decrease_x();
        assert_eq!(20, cpu.x);
    }

    #[test]
    fn increase_y_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.increase_y();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn decrease_y_decreases_y_by_one() {
        let mut cpu = create_test_cpu();
        cpu.y = 21;
        cpu.decrease_y();
        assert_eq!(20, cpu.y);
    }

    #[test]
    fn decrease_y_takes_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.decrease_y();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn increment_memory_zero_page_increases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0x70, 43);
        cpu.increment_memory_zero_page();
        assert_eq!(44, cpu.memory.borrow_mut().read(0x70));
    }

    #[test]
    fn increment_memory_zero_page_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.increment_memory_zero_page();
        assert_eq!(0xABD, cpu.program_counter);
    }

    #[test]
    fn increment_memory_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.increment_memory_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn increment_memory_zero_page_x_increases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x24;
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0x70 + 0x24, 43);
        cpu.increment_memory_zero_page_x();
        assert_eq!(44, cpu.memory.borrow_mut().read(0x70 + 0x24));
    }

    #[test]
    fn increment_memory_zero_page_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.increment_memory_zero_page_x();
        assert_eq!(0xABD, cpu.program_counter);
    }

    #[test]
    fn increment_memory_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.increment_memory_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn increment_memory_absolute_increases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0xABD, 0x02);

        cpu.memory.borrow_mut().write(0x0270, 43);
        cpu.increment_memory_absolute();
        assert_eq!(44, cpu.memory.borrow_mut().read(0x0270));
    }

    #[test]
    fn increment_memory_absolute_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.increment_memory_absolute();
        assert_eq!(0xABE, cpu.program_counter);
    }

    #[test]
    fn increment_memory_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.increment_memory_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn increment_memory_absolute_x_increases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.x = 0x53;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0xABD, 0x02);

        cpu.memory.borrow_mut().write(0x0270 + 0x53, 43);
        cpu.increment_memory_absolute_x();
        assert_eq!(44, cpu.memory.borrow_mut().read(0x0270 + 0x53));
    }

    #[test]
    fn increment_memory_absolute_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.increment_memory_absolute_x();
        assert_eq!(0xABE, cpu.program_counter);
    }

    #[test]
    fn increment_memory_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.increment_memory_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn decrement_memory_zero_page_decreases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0x70, 43);
        cpu.decrement_memory_zero_page();
        assert_eq!(42, cpu.memory.borrow_mut().read(0x70));
    }

    #[test]
    fn decrement_memory_zero_page_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.decrement_memory_zero_page();
        assert_eq!(0xABD, cpu.program_counter);
    }

    #[test]
    fn decrement_memory_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.decrement_memory_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn decrement_memory_zero_page_x_decreases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x24;
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0x70 + 0x24, 43);
        cpu.decrement_memory_zero_page_x();
        assert_eq!(42, cpu.memory.borrow_mut().read(0x70 + 0x24));
    }

    #[test]
    fn decrement_memory_zero_page_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.decrement_memory_zero_page_x();
        assert_eq!(0xABD, cpu.program_counter);
    }

    #[test]
    fn decrement_memory_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.decrement_memory_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn decrement_memory_absolute_decreases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0xABD, 0x02);

        cpu.memory.borrow_mut().write(0x0270, 43);
        cpu.decrement_memory_absolute();
        assert_eq!(42, cpu.memory.borrow_mut().read(0x0270));
    }

    #[test]
    fn decrement_memory_absolute_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.decrement_memory_absolute();
        assert_eq!(0xABE, cpu.program_counter);
    }

    #[test]
    fn decrement_memory_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.decrement_memory_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn decrement_memory_absolute_x_decreases_value_in_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.x = 0x53;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.memory.borrow_mut().write(0xABD, 0x02);

        cpu.memory.borrow_mut().write(0x0270 + 0x53, 43);
        cpu.decrement_memory_absolute_x();
        assert_eq!(42, cpu.memory.borrow_mut().read(0x0270 + 0x53));
    }

    #[test]
    fn decrement_memory_absolute_x_increments_program_counter() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xABC;
        cpu.decrement_memory_absolute_x();
        assert_eq!(0xABE, cpu.program_counter);
    }

    #[test]
    fn decrement_memory_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.decrement_memory_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_zero_page_does_not_modify_registers() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x42;
        cpu.x = 0xFA;
        cpu.unofficial_and_a_with_x_store_result_zero_page();
        assert_eq!(0x42, cpu.a);
        assert_eq!(0xFA, cpu.x);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_zero_page_does_not_modify_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x0;
        cpu.x = 0xFA;
        cpu.status_flags = 0x80;
        cpu.unofficial_and_a_with_x_store_result_zero_page();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_zero_page_page_writes_result_to_memory() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x43;
        cpu.x = 0xFA;
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.unofficial_and_a_with_x_store_result_zero_page();
        assert_eq!(0x42, cpu.memory.borrow_mut().read(0x70));
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_zero_page_y_does_not_modify_registers() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x42;
        cpu.x = 0xFA;
        cpu.unofficial_and_a_with_x_store_result_zero_page_y();
        assert_eq!(0x42, cpu.a);
        assert_eq!(0xFA, cpu.x);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_zero_page_y_does_not_modify_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x0;
        cpu.x = 0xFA;
        cpu.status_flags = 0x80;
        cpu.unofficial_and_a_with_x_store_result_zero_page_y();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_zero_page_y_writes_result_to_memory() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x43;
        cpu.x = 0xFA;
        cpu.y = 0x5D;
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x70);
        cpu.unofficial_and_a_with_x_store_result_zero_page_y();
        assert_eq!(0x42, cpu.memory.borrow_mut().read(0x70 + 0x5D));
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_absolute_does_not_modify_registers() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x42;
        cpu.x = 0xFA;
        cpu.unofficial_and_a_with_x_store_result_absolute();
        assert_eq!(0x42, cpu.a);
        assert_eq!(0xFA, cpu.x);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_absolute_does_not_modify_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x0;
        cpu.x = 0xFA;
        cpu.status_flags = 0x80;
        cpu.unofficial_and_a_with_x_store_result_absolute();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_absolute_writes_result_to_memory() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x43;
        cpu.x = 0xFA;
        cpu.y = 0x5D;
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x02);
        cpu.memory.borrow_mut().write(0xABD, 0x7F);

        cpu.unofficial_and_a_with_x_store_result_absolute();
        assert_eq!(0x42, cpu.memory.borrow_mut().read(0x7F02));
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_indirect_x_does_not_modify_registers() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x42;
        cpu.x = 0xFA;
        cpu.unofficial_and_a_with_x_store_result_indirect_x();
        assert_eq!(0x42, cpu.a);
        assert_eq!(0xFA, cpu.x);
    }


    #[test]
    fn unofficial_and_a_with_x_store_result_indirect_x_does_not_modify_status_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x0;
        cpu.x = 0xFA;
        cpu.status_flags = 0x80;
        cpu.unofficial_and_a_with_x_store_result_indirect_x();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn unofficial_and_a_with_x_store_result_indirect_x_writes_result_to_memory() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x43;
        cpu.x = 0xFA;
        cpu.y = 0x5D;
        cpu.program_counter = 0xABC;
        cpu.memory.borrow_mut().write(0xABC, 0x02);
        cpu.memory.borrow_mut().write(0x02 + 0xFA, 0xAF);
        cpu.memory.borrow_mut().write(0x03 + 0xFA, 0xEF);

        cpu.unofficial_and_a_with_x_store_result_indirect_x();
        assert_eq!(0x42, cpu.memory.borrow_mut().read(0xEFAF));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0xAF, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0xAF));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0xF0;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x00;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0xAF, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page();
        assert_eq!(0x81, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_a_zero_page_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page();
        assert_eq!(0x33, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_x_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.x = 0x10;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0xAF + 0x10, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page_x();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0xAF + 0x10));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_x_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x15;
        cpu.x = 0x10;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x83;
        cpu.memory.borrow_mut().write(0x32, 0xAF);
        cpu.memory.borrow_mut().write(0xAF + 0x10, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page_x();
        assert_eq!(0x80, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_x_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page_x();
        assert_eq!(0x33, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;

        cpu.memory.borrow_mut().write(0x32, 0x8F);
        cpu.memory.borrow_mut().write(0x33, 0x09);
        cpu.memory.borrow_mut().write(0x098F, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0x098F));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_absolute_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x00;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x83;
        cpu.memory.borrow_mut().write(0x32, 0x8F);
        cpu.memory.borrow_mut().write(0x33, 0x09);
        cpu.memory.borrow_mut().write(0x098F, 0xFE);

        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute();
        assert_eq!(0x00, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute();
        assert_eq!(0x34, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_x_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.x = 0x42;

        cpu.memory.borrow_mut().write(0x32, 0x8F);
        cpu.memory.borrow_mut().write(0x33, 0x09);
        cpu.memory.borrow_mut().write(0x098F + 0x42, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_x();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0x098F + 0x42));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_absolute_x_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x90;
        cpu.x = 0x20;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x83;
        cpu.memory.borrow_mut().write(0x32, 0x8F);
        cpu.memory.borrow_mut().write(0x33, 0x09);
        cpu.memory.borrow_mut().write(0x098F + 0x20, 0x3E);

        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_x();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_x_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_x();
        assert_eq!(0x34, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_y_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.y = 0x42;

        cpu.memory.borrow_mut().write(0x32, 0x8F);
        cpu.memory.borrow_mut().write(0x33, 0x09);
        cpu.memory.borrow_mut().write(0x098F + 0x42, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_y();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0x098F + 0x42));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_absolute_y_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x90;
        cpu.y = 0x20;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x00;
        cpu.memory.borrow_mut().write(0x32, 0x8F);
        cpu.memory.borrow_mut().write(0x33, 0x09);
        cpu.memory.borrow_mut().write(0x098F + 0x20, 0x3E);

        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_y();
        assert_eq!(0x01, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_y_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_y();
        assert_eq!(0x34, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_absolute_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_absolute_y();
        assert_eq!(7, cpu.wait_counter);
    }


    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_x_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.x = 0x20;
        cpu.memory.borrow_mut().write(0x32, 0x3E);

        cpu.memory.borrow_mut().write(0x3E + 0x20, 0x07);
        cpu.memory.borrow_mut().write(0x3F + 0x20, 0x3F);
        cpu.memory.borrow_mut().write(0x3F07, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_x();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0x3F07));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_x_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x4E;
        cpu.x = 0x20;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x00;
        cpu.memory.borrow_mut().write(0x32, 0x3E);

        cpu.memory.borrow_mut().write(0x3E + 0x20, 0x07);
        cpu.memory.borrow_mut().write(0x3F + 0x20, 0x3F);
        cpu.memory.borrow_mut().write(0x3F07, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_x();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_x_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_x();
        assert_eq!(0x33, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_x_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_x();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_y_decrements_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x32;
        cpu.y = 0x20;
        cpu.memory.borrow_mut().write(0x32, 0x3E);

        cpu.memory.borrow_mut().write(0x3E, 0x07);
        cpu.memory.borrow_mut().write(0x3F, 0x3F);
        cpu.memory.borrow_mut().write(0x3F07 + 0x20, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_y();
        assert_eq!(0x4E, cpu.memory.borrow_mut().read(0x3F07 + 0x20));
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_y_sets_flags() {
        let mut cpu = create_test_cpu();
        cpu.a = 0x4E;
        cpu.y = 0x20;
        cpu.program_counter = 0x32;
        cpu.status_flags = 0x00;
        cpu.memory.borrow_mut().write(0x32, 0x3E);

        cpu.memory.borrow_mut().write(0x3E, 0x07);
        cpu.memory.borrow_mut().write(0x3F, 0x3F);
        cpu.memory.borrow_mut().write(0x3F07 + 0x20, 0x4F);

        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_y();
        assert_eq!(0x03, cpu.status_flags);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_y_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x32;
        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_y();
        assert_eq!(0x33, cpu.program_counter);
    }

    #[test]
    fn unofficial_decrement_memory_and_compare_with_acc_indirect_y_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_decrement_memory_and_compare_with_acc_indirect_y();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_zero_page_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_zero_page();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0x4F));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_zero_page_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x71;
        cpu.a = 0x80;
        cpu.status_flags = 0x01;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_zero_page();
        assert_eq!(0x80 - 0x51, cpu.a); // carry not set === borrow set
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_zero_page_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_zero_page();
        assert_eq!(0xFEE1, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_zero_page_x_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x25;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x25, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_zero_page_x();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0x4F + 0x25));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_zero_page_x_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x71;
        cpu.a = 0x80;
        cpu.status_flags = 0x01;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x71, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_zero_page_x();
        assert_eq!(0x80 - 0x51, cpu.a); // carry not set === borrow set
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_zero_page_x_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_zero_page_x();
        assert_eq!(0xFEE1, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_zero_page_x();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x235, 0x12);
        cpu.memory.borrow_mut().write(0x124F, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_absolute();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0x124F));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_absolute_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x71;
        cpu.a = 0x80;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x235, 0x12);
        cpu.memory.borrow_mut().write(0x124F, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_absolute();
        assert_eq!(0x80 - 0x51 - 1, cpu.a); // carry not set === borrow set
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_absolute();
        assert_eq!(0xFEE2, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_x_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x71;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x235, 0x12);
        cpu.memory.borrow_mut().write(0x124F + 0x71, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_absolute_x();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0x124F + 0x71));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_absolute_x_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x71;
        cpu.a = 0x80;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x235, 0x12);
        cpu.memory.borrow_mut().write(0x124F + 0x71, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_absolute_x();
        assert_eq!(0x80 - 0x51 - 1, cpu.a); // carry not set === borrow set
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_x_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_absolute_x();
        assert_eq!(0xFEE2, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_y_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.y = 0x71;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x235, 0x12);
        cpu.memory.borrow_mut().write(0x124F + 0x71, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_absolute_y();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0x124F + 0x71));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_absolute_y_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.y = 0x71;
        cpu.a = 0x80;
        cpu.status_flags = 0x01;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x235, 0x12);
        cpu.memory.borrow_mut().write(0x124F + 0x71, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_absolute_y();
        assert_eq!(0x80 - 0x51, cpu.a);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_y_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_absolute_y();
        assert_eq!(0xFEE2, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_absolute_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_absolute_y();
        assert_eq!(7, cpu.wait_counter);
    }


    #[test]
    fn unofficial_increment_memory_subtract_acc_indirect_x_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.x = 0x30;

        cpu.memory.borrow_mut().write(0x234, 0x4F);

        cpu.memory.borrow_mut().write(0x4F + 0x30, 0xA1);
        cpu.memory.borrow_mut().write(0x50 + 0x30, 0xB2);

        cpu.memory.borrow_mut().write(0xB2A1, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_indirect_x();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0xB2A1));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_indirect_x_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;

        cpu.a = 0x80;
        cpu.x = 0x30;
        cpu.memory.borrow_mut().write(0x234, 0x4F);

        cpu.memory.borrow_mut().write(0x4F + 0x30, 0xA1);
        cpu.memory.borrow_mut().write(0x50 + 0x30, 0xB2);
        cpu.memory.borrow_mut().write(0xB2A1, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_indirect_x();
        assert_eq!(0x80 - 0x51 - 1, cpu.a);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_indirect_x_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_indirect_x();
        assert_eq!(0xFEE1, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_indirect_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_indirect_x();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_indirect_y_increments_memory() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.y = 0x30;

        cpu.memory.borrow_mut().write(0x234, 0x4F);

        cpu.memory.borrow_mut().write(0x4F, 0xA1);
        cpu.memory.borrow_mut().write(0x50, 0xB2);

        cpu.memory.borrow_mut().write(0xB2A1 + 0x30, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_indirect_y();
        assert_eq!(0x51, cpu.memory.borrow_mut().read(0xB2A1 + 0x30));
    }

    #[test]
    fn unofficial_increment_memory_subtracts_acc_indirect_y_subtracts_result_from_a() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;

        cpu.a = 0x80;
        cpu.y = 0x30;
        cpu.memory.borrow_mut().write(0x234, 0x4F);

        cpu.memory.borrow_mut().write(0x4F, 0xA1);
        cpu.memory.borrow_mut().write(0x50, 0xB2);
        cpu.memory.borrow_mut().write(0xB2A1 + 0x30, 0x50);

        cpu.unofficial_increment_memory_subtract_acc_indirect_y();
        assert_eq!(0x80 - 0x51 - 1, cpu.a);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_indirect_y_updates_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0xFEE0;
        cpu.unofficial_increment_memory_subtract_acc_indirect_y();
        assert_eq!(0xFEE1, cpu.program_counter);
    }

    #[test]
    fn unofficial_increment_memory_subtract_acc_indirect_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_increment_memory_subtract_acc_indirect_y();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_changes_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_zero_page();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x4F));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_zero_page();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_zero_page_x();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x4F + 0x13));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_x_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_zero_page_x();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x431F));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute_x();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_x_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute_x();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute_y();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_y_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute_y();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_absolute_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_absolute_y();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_shift_left_memory_inclusive_or_acc_indirect_x();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x0A02));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_x_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_shift_left_memory_inclusive_or_acc_indirect_x();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_x_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_indirect_x();
        assert_eq!(8, cpu.wait_counter);
    }


    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_shift_left_memory_inclusive_or_acc_indirect_y();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x0A02 + 0x13));
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_y_changes_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_shift_left_memory_inclusive_or_acc_indirect_y();
        assert_eq!(0xF6, cpu.a);
    }

    #[test]
    fn unofficial_shift_left_memory_inclusive_or_acc_indirect_y_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_left_memory_inclusive_or_acc_indirect_y();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_changes_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_zero_page();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x4F));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_zero_page();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x4F + 0x13));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x431F));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute_x();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute_x();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute_y();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_y_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute_y();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_absolute_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_absolute_y();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_rotate_left_memory_bitwise_and_acc_indirect_x();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x0A02));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_rotate_left_memory_bitwise_and_acc_indirect_x();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_x_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_indirect_x();
        assert_eq!(8, cpu.wait_counter);
    }


    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_rotate_left_memory_bitwise_and_acc_indirect_y();
        assert_eq!(0xB4, cpu.memory.borrow_mut().read(0x0A02 + 0x13));
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_y_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_rotate_left_memory_bitwise_and_acc_indirect_y();
        assert_eq!(0x30, cpu.a);
    }

    #[test]
    fn unofficial_rotate_left_memory_bitwise_and_acc_indirect_y_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_left_memory_bitwise_and_acc_indirect_y();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_zero_page_changes_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_zero_page();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x4F));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_zero_page_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_zero_page();
        assert_eq!(0x5F,  cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_zero_page_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_zero_page_x();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x4F + 0x13));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_zero_page_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_zero_page_x();
        assert_eq!(0x5F, cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_absolute();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x431F));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_absolute();
        assert_eq!(0x5F, cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_absolute_x();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_absolute_x();
        assert_eq!(0x5F, cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_absolute_y();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_y_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_shift_right_memory_xor_acc_absolute_y();
        assert_eq!(0x5F, cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_absolute_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_absolute_y();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_indirect_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_shift_right_memory_xor_acc_indirect_x();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x0A02));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_indirect_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_shift_right_memory_xor_acc_indirect_x();
        assert_eq!(0x5F, cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_indirect_x_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_indirect_x();
        assert_eq!(8, cpu.wait_counter);
    }


    #[test]
    fn unofficial_shift_right_memory_xor_acc_indirect_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_shift_right_memory_xor_acc_indirect_y();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x0A02 + 0x13));
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_indirect_y_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_shift_right_memory_xor_acc_indirect_y();
        assert_eq!(0x5F, cpu.a);
    }

    #[test]
    fn unofficial_shift_right_memory_xor_acc_indirect_y_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_shift_right_memory_xor_acc_indirect_y();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_zero_page_changes_memory() {
        let mut cpu = create_test_cpu();

        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_zero_page();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x4F));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_zero_page_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;

        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_zero_page();
        assert_eq!(0x72 + 0x2D,  cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_zero_page_takes_5_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_zero_page();
        assert_eq!(5, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_zero_page_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_zero_page_x();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x4F + 0x13));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_zero_page_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x4F);
        cpu.memory.borrow_mut().write(0x4F + 0x13, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_zero_page_x();
        assert_eq!(0x72 + 0x2D, cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_zero_page_x_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_absolute();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x431F));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_absolute();
        assert_eq!(0x72 + 0x2D, cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_takes_6_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_absolute();
        assert_eq!(6, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_absolute_x();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_absolute_x();
        assert_eq!(0x72 + 0x2D, cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_x_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_absolute_x();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_absolute_y();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x431F + 0x13));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_y_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x235, 0x43);
        cpu.memory.borrow_mut().write(0x431F + 0x13, 0x5A);
        cpu.unofficial_rotate_right_memory_add_acc_absolute_y();
        assert_eq!(0x72 + 0x2D, cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_absolute_y_takes_7_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_absolute_y();
        assert_eq!(7, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_indirect_x_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.x = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_rotate_right_memory_add_acc_indirect_x();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x0A02));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_indirect_x_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.x = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F + 0x13, 0x02);
        cpu.memory.borrow_mut().write(0x20 + 0x13, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02, 0x5A);

        cpu.unofficial_rotate_right_memory_add_acc_indirect_x();
        assert_eq!(0x72 + 0x2D, cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_indirect_x_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_indirect_x();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_indirect_y_changes_memory() {
        let mut cpu = create_test_cpu();
        cpu.y = 0x13;
        cpu.program_counter = 0x234;

        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_rotate_right_memory_add_acc_indirect_y();
        assert_eq!(0x2D, cpu.memory.borrow_mut().read(0x0A02 + 0x13));
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_indirect_y_changes_and_accumulator() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x234;
        cpu.a = 0x72;
        cpu.y = 0x13;
        cpu.memory.borrow_mut().write(0x234, 0x1F);
        cpu.memory.borrow_mut().write(0x1F, 0x02);
        cpu.memory.borrow_mut().write(0x20, 0x0A);
        cpu.memory.borrow_mut().write(0x0A02 + 0x13, 0x5A);

        cpu.unofficial_rotate_right_memory_add_acc_indirect_y();
        assert_eq!(0x72 + 0x2D, cpu.a);
    }

    #[test]
    fn unofficial_rotate_right_memory_add_acc_indirect_y_takes_8_cycles() {
        let mut cpu = create_test_cpu();
        cpu.unofficial_rotate_right_memory_add_acc_indirect_y();
        assert_eq!(8, cpu.wait_counter);
    }

    #[test]
    fn no_operation_waits_2_cycles() {
        let mut cpu = create_test_cpu();
        cpu.no_operation();
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn unofficial_double_no_operation_increments_pc() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x13;
        cpu.unofficial_double_no_operation(2);
        assert_eq!(0x14, cpu.program_counter);
    }

    #[test]
    fn unofficial_triple_no_operation_page_penalty_does_not_add_a_cycle_when_page_boundary_is_not_crossed() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x0023;
        cpu.unofficial_triple_no_operation_page_penalty(2);
        assert_eq!(2, cpu.wait_counter);
    }

    #[test]
    fn unofficial_triple_no_operation_page_penalty_adds_a_cycle_when_page_boundary_is_crossed() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x00FF;
        cpu.unofficial_triple_no_operation_page_penalty(2);
        assert_eq!(3, cpu.wait_counter);
    }

    #[test]
    fn unofficial_triple_no_operation_no_page_penalty_increments_pc_twice() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x13;
        cpu.unofficial_triple_no_operation_no_page_penalty(3);
        assert_eq!(0x15, cpu.program_counter);
    }

    #[test]
    fn unofficial_triple_no_operation_page_penalty_increments_pc_twice() {
        let mut cpu = create_test_cpu();
        cpu.program_counter = 0x13;
        cpu.unofficial_triple_no_operation_page_penalty(3);
        assert_eq!(0x15, cpu.program_counter);
    }

}
