use std::collections::HashMap; 
use std::io;

const MEM_MAX: u16 = 32767;
const LITERAL_MAX: u16 = 32767;
const REG_ADDR_MAX: u16 = 32775;
const REG_N: usize = 8;

type VMResult<T> = Result<T, String>;

// histogram of memory positions for executed instructions
// designed for getting started with teleporter analysis
pub struct InstructionAccumulator {
	acc:HashMap<u16, usize>,
}

impl InstructionAccumulator {
	pub fn new() -> InstructionAccumulator {
		InstructionAccumulator { acc: HashMap::new()}
	}
	pub fn clear(&mut self) {
		self.acc.clear();
	}
	pub fn record(&mut self, pos:u16) {
		*self.acc.entry(pos).or_insert(0) += 1;
	}
	pub fn print(&self) {
		println!("Distinct memory positions: {}", self.acc.keys().len());
		let mut key_vec:Vec<u16> = Vec::new();
		for k in self.acc.keys() {
			key_vec.push(*k);
		}
		key_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
		let mut sum = 0;
		for k in key_vec {
			println!("{}:\t{}", k, self.acc[&k]);
			sum += self.acc[&k];
		}
		println!("Sum of positions: {}", sum);	
	}
}

#[derive(Clone)]
pub struct SynacorVM {
	mem: HashMap<u16,u16>,
	reg: [u16; REG_N],
	stack: Vec<u16>,
	mem_ptr: u16,
	halt: bool,
	halt_code: String,
	halt_err: bool,
	interactive: bool,
	output_buff: Vec<char>,
	output_buff_index: usize,
	input_buff: Vec<char>,
	input_buff_index: usize,
	awaiting_input:bool,
	input_ready:bool,
}
impl SynacorVM {
	pub fn new(interactive:bool) -> SynacorVM {
		SynacorVM { mem: HashMap::new(), reg:[0;REG_N], stack: Vec::new(), mem_ptr: 0, halt: false, halt_code:String::new(), halt_err: false, interactive: interactive, input_buff:Vec::new(), input_buff_index:0, output_buff:Vec::new(), output_buff_index: 0, awaiting_input:false, input_ready:false}
	}
	// set the input buffer and continue execution
	pub fn input_line_string(&mut self, input: String) {
		if !self.awaiting_input {
			return;
		}
		self.input_buff.clear();
		let input_chars:Vec<char> = input.chars().collect();
		for c in input_chars {
			self.input_buff.push(c);
		}
		if !input.contains("\n") {
			self.input_buff.push('\n');
		}
		self.input_buff_index = self.input_buff.len();
		self.input_ready = true;
		self.awaiting_input = false;
		self.execute();
	}
	pub fn input_line(&mut self, input:&str) {
		let mut s:String = input.to_string();
		if !s.contains("\n") {
			s.push_str("\n");
		}
		self.input_line_string(s);
	}
	pub fn set_input_line(&mut self, input:&str) {
		let mut s:String = input.to_string();
		if !s.contains("\n") {
			s.push_str("\n");
		}
		if !self.awaiting_input {
			return;
		}
		self.input_buff.clear();
		let input_chars:Vec<char> = input.chars().collect();
		for c in input_chars {
			self.input_buff.push(c);
		}
		if !input.contains("\n") {
			self.input_buff.push('\n');
		}
		self.input_buff_index = self.input_buff.len();
		self.input_ready = true;
		self.awaiting_input = false;
	}
	pub fn output_line(&mut self, reset:bool) -> String {
		let s: String = self.output_buff.iter().collect();
		if reset {
			self.output_buff.clear();
			self.output_buff_index = 0;
		}
		return s;
	}
	pub fn set_interactive(&mut self, interactive:bool) {
		self.interactive = interactive;
		if self.interactive {
			self.awaiting_input = false;
			self.input_ready = false;
		}
	}
	pub fn set_register(&mut self, index:usize, val:u16) {
		if index >= REG_N {
			return;
		}
		self.reg[index] = val;
	}
	pub fn get_register(&mut self, index:usize) -> u16 {
		if index >= REG_N {
			return 0;
		}
		return self.reg[index];
	}
	pub fn get_mem_ptr(&mut self) -> u16 {
		return self.mem_ptr;
	}
	pub fn set_mem_ptr(&mut self, ptr:u16) {
		self.mem_ptr = ptr;
	}
	pub fn load_mem(&mut self, input:Vec<u16>) -> VMResult<bool> {	
		if input.len() >= MEM_MAX as usize {
			return Err(format!("LOAD MEM: input length {} exceeds address space capacity {}", input.len(), MEM_MAX + 1))
		}
		self.mem.clear();
		for i in 0..input.len() {
			self.mem.insert(i as u16, input[i]);
		}
		return Ok(true);
	}
	pub fn get_mem(&mut self, addr:u16) -> VMResult<u16> {
		return self.mem_read(addr);
	}
	pub fn is_halted(&mut self) -> bool {
		return self.halt;
	}
	pub fn is_awaiting_input(&mut self) -> bool {
		return self.awaiting_input;
	}
	pub fn execute(&mut self) {
		while !self.halt && (!self.awaiting_input) {
			self.execute_one();
		}
	}
	pub fn execute_once(&mut self) {
		if self.halt {
			println!("EXECUTE_ONCE: HALTED");
			return;
		}
		if self.awaiting_input && !self.input_ready {
			println!("EXECUTE_ONCE: AWAITING INPUT");
			return;
		}
		self.execute_one();
	}
	fn execute_one(&mut self) {
		if self.halt {
			println!("{}", self.halt_code);
			return;
		}
		if self.awaiting_input && !self.input_ready {
			return;
		}
		let opcode_result = self.mem_read(self.mem_ptr);
		if !opcode_result.is_ok() {
			self.set_halt_with_error(format!("Exception while reading opcode at index {} - {}", self.mem_ptr, opcode_result.unwrap_err()));
			return;
		}
		let opcode = opcode_result.unwrap();
		match opcode {
			0 => self.op_halt(),
			1 => self.op_set(),
			2 => self.op_push(),
			3 => self.op_pop(),
			4 => self.op_eq(),
			5 => self.op_gt(),
			6 => self.op_jmp(),
			7 => self.op_jt(),
			8 => self.op_jf(),
			9 => self.op_add(),
			10 => self.op_mult(),
			11 => self.op_mod(),
			12 => self.op_and(),
			13 => self.op_or(),
			14 => self.op_not(),
			15 => self.op_rmem(),
			16 => self.op_wmem(),
			17 => self.op_call(),
			18 => self.op_ret(),
			19 => self.op_out(),
			20 => self.op_in(),
			21 => self.op_noop(),
			_ => self.op_undefined(opcode),
		}
	}
	fn val(&mut self, val:u16) -> VMResult<u16> {
			if val <= LITERAL_MAX {
				return Ok(val);
			}
			else if val > LITERAL_MAX && val <= REG_ADDR_MAX {
				return Ok(self.reg[(val - LITERAL_MAX - 1) as usize]);
			}
			else {
				self.set_halt_with_error(format!("VAL: Invalid value {}", val));
				return Err(format!("VAL: Invalid value {}", val));
			}
	}
	fn mem_read(&mut self, addr:u16) -> VMResult<u16> {
		if addr > MEM_MAX {
			self.set_halt_with_error(format!("Invalid memory read access at address {} (out of range)", addr));
			return Err(format!("Invalid memory write access at address {} (out of range)", addr));
		}
		if !self.mem.contains_key(&addr) {
			return Ok(0);
		}
		return Ok(self.mem[&addr]);
	}
	fn mem_write(&mut self, addr:u16, val:u16) -> VMResult<bool> {
		if addr > MEM_MAX {
			self.set_halt_with_error(format!("Invalid memory write access at address {} (out of range)", addr));
			return Err(format!("Invalid memory write access at address {} (out of range)", addr));
		}
		self.mem.insert(addr, val);
		return Ok(true);
	}
	fn is_reg_addr(&mut self, addr: u16) -> bool {
		return addr > LITERAL_MAX && addr <= REG_ADDR_MAX;
	}
	fn reg_index(&mut self, addr: u16) -> VMResult<usize> {
		if addr > LITERAL_MAX && addr <= REG_ADDR_MAX {
			return Ok((addr - LITERAL_MAX - 1) as usize);
		}
		return Err(format!("Register address {} ({}) out of range", addr, addr - LITERAL_MAX - 1));
	}
	fn reg_set(&mut self, reg_addr: u16, val: u16) -> VMResult<bool> {
		let reg_index = self.reg_index(reg_addr);
		if reg_index.is_ok() {
			self.reg[reg_index.unwrap()] = val;
			return Ok(true);
		}
		else {
			return Err(format!("REG SET: Register index {} is out of range", reg_addr));
		}
	}
	fn set_halt_with_error(&mut self, err_str:String) {
		self.halt = true;
		self.halt_code = err_str;
		self.halt_err = true;
	}
	fn op_halt(&mut self) {
		
		self.halt = true;
		self.halt_code = "Halted successfully".to_string();
		self.halt_err = false;
		self.mem_ptr += 1;
	}
	fn op_set(&mut self) {
		let op_name = "OP SET";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}	
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{} error: arg1 {} is not a register address", op_name, arg1));
		}
		let val_result = self.val(arg2);
		if !val_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg2 - {}", op_name, val_result.unwrap_err()));
			return;
		}
		let val = val_result.unwrap();
		let reg_set = self.reg_set(arg1, val);
		if !reg_set.is_ok() {
			self.set_halt_with_error(reg_set.unwrap_err().to_string());
		}
		self.mem_ptr += 3;
	}
	fn op_push(&mut self) {
		let op_name = "OP PUSH";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		let val_result = self.val(arg1);
		if !val_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val_result.unwrap_err()));
			return;
		}
		let val = val_result.unwrap();
		self.stack.push(val);		
		self.mem_ptr += 2;
	}
	fn op_pop(&mut self) {
		let op_name = "OP POP";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{} error: arg1 {} is not a register address", op_name, arg1));
			return;
		}
		
		let pop_val = self.stack.pop();
		match pop_val {
			Some(x) => {
					let reg_set_result = self.reg_set(arg1, x);
					if !reg_set_result.is_ok() {
						self.halt = true; self.halt_code = format!("{} error: {}", op_name, reg_set_result.unwrap_err().to_string());
					}
				},
			None => {
					self.halt = true; 
					self.halt_code = format!("{} error: empty stack", op_name)
				},
		}
		self.mem_ptr += 2;
	}
	fn op_eq(&mut self) {
		let op_name = "OP EQ";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		if val2 == val3 {
			let reg_set_result = self.reg_set(arg1, 1);
			if !reg_set_result.is_ok() {
				self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
				return;
			}
		}
		else {
			let reg_set_result = self.reg_set(arg1, 0);
			if !reg_set_result.is_ok() {
				self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
				return;
			}
		}
		self.mem_ptr += 4;
	}
	fn op_gt(&mut self) {
		let op_name = "OP GT";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		if val2 > val3 {
			let reg_set_result = self.reg_set(arg1, 1);
			if !reg_set_result.is_ok() {
				self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
				return;
			}
		}
		else {
			let reg_set_result = self.reg_set(arg1, 0);
			if !reg_set_result.is_ok() {
				self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
				return;
			}
		}
		self.mem_ptr += 4;
	}
	fn op_jmp(&mut self) {
		let op_name = "OP JMP";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		let val_result = self.val(arg1);
		if !val_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val_result.unwrap_err()));
			return;
		}
		let val = val_result.unwrap();
		self.mem_ptr = val;
	}
	fn op_jt(&mut self) {
		let op_name = "OP JT";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let val1_result = self.val(arg1);
		if !val1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val1_result.unwrap_err()));
			return;
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val1 = val1_result.unwrap();
		let val2 = val2_result.unwrap();
		if val1 != 0 {
			self.mem_ptr = val2;
		}
		else {
			self.mem_ptr += 3;
		}
	}
	fn op_jf(&mut self) {
		let op_name = "OP JF";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let val1_result = self.val(arg1);
		if !val1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val1_result.unwrap_err()));
			return;
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val1 = val1_result.unwrap();
		let val2 = val2_result.unwrap();
		if val1 == 0 {
			self.mem_ptr = val2;
		}
		else {
			self.mem_ptr += 3;
		}
	}
	fn op_add(&mut self) {
		let op_name = "OP ADD";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		let val2_u32 = val2 as u32;
		let val3_u32 = val3 as u32;
		let sum_u32 = (val2_u32 + val3_u32) % (LITERAL_MAX as u32 + 1);
		let sum = sum_u32 as u16;
		
		let reg_set_result = self.reg_set(arg1, sum);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 4;
	}
	fn op_mult(&mut self) {
		let op_name = "OP MULT";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		let val2_u32 = val2 as u32;
		let val3_u32 = val3 as u32;
		let product_u32 = (val2_u32 * val3_u32) % (LITERAL_MAX as u32 + 1);
		let product = product_u32 as u16;
		let reg_set_result = self.reg_set(arg1, product);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 4;
	}
	fn op_mod(&mut self) {
		let op_name = "OP MOD";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		let remainder = (val2 % val3) % (LITERAL_MAX + 1);
		let reg_set_result = self.reg_set(arg1, remainder);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 4;
	}
	fn op_and(&mut self) {
		let op_name = "OP AND";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		let and_val = val2 & val3;
		let reg_set_result = self.reg_set(arg1, and_val);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 4;
	}
	fn op_or(&mut self) {
		let op_name = "OP OR";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		let arg3_result = self.mem_read(self.mem_ptr + 3);
		if !arg3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg3 - {}", op_name, arg3_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		let arg3 = arg3_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		let val3_result = self.val(arg3);
		if !val3_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg3 - {}", op_name, val3_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let val3 = val3_result.unwrap();
		
		let or_val = val2 | val3;
		let reg_set_result = self.reg_set(arg1, or_val);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 4;
	}
	fn op_not(&mut self) {
		let op_name = "OP NOT";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		
		let not_val = (!val2) & LITERAL_MAX;
		let reg_set_result = self.reg_set(arg1, not_val);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 3;
	}
	fn op_rmem(&mut self) {
		let op_name = "OP RMEM";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		
		let val2 = val2_result.unwrap();
		let mem_val_result = self.mem_read(val2);
		if !mem_val_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value from memory at address {} - {}", op_name, val2, mem_val_result.unwrap_err()));
			return;
		}
		
		let mem_val = mem_val_result.unwrap();
		let reg_set_result = self.reg_set(arg1, mem_val);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		self.mem_ptr += 3;
	}
	fn op_wmem(&mut self) {
		let op_name = "OP WMEM";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg2_result = self.mem_read(self.mem_ptr + 2);
		if !arg2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg2 - {}", op_name, arg2_result.unwrap_err()));
			return;
		}
		
		let arg1 = arg1_result.unwrap();
		let arg2 = arg2_result.unwrap();
		
		let val1_result = self.val(arg1);
		if !val1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg1 - {}", op_name, val1_result.unwrap_err()));
			return;
		}
		let val2_result = self.val(arg2);
		if !val2_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error retrieving value for arg2 - {}", op_name, val2_result.unwrap_err()));
			return;
		}
		
		let val1 = val1_result.unwrap();
		let val2 = val2_result.unwrap();
		let mem_write_result = self.mem_write(val1, val2);
		if !mem_write_result.is_ok() {
			self.set_halt_with_error(format!("{} error: error writing value {} to memory at address {} - {}", op_name, val2, val1, mem_write_result.unwrap_err()));
			return;
		}
		
		self.mem_ptr += 3;
	}
	fn op_call(&mut self) {
		let op_name = "OP CALL";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		let val_result = self.val(arg1);
		if !val_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val_result.unwrap_err()));
			return;
		}
		let val = val_result.unwrap();
		let next = self.mem_ptr + 2;
		self.stack.push(next);
		self.mem_ptr = val;
	}
	fn op_ret(&mut self) {
		let op_name = "OP RET";
		
		let pop_val = self.stack.pop();
		match pop_val {
			Some(x) => {
					self.mem_ptr = x;
				},
			None => {
					// this may not need to be an error state, just a halt
					self.set_halt_with_error(format!("{} error: empty stack", op_name));
				},
		}
	}
	// should maybe include a check for valid ascii values, but eh.
	fn op_out(&mut self) {
		let op_name = "OP OUT";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		let val_result = self.val(arg1);
		if !val_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid val for arg1 - {}", op_name, val_result.unwrap_err()));
			return;
		}
		let val = val_result.unwrap();
		let chr = (val as u8) as char;
		if self.interactive {
			print!("{}", chr);
		}
		else {
			self.output_buff.push(chr);
			self.output_buff_index += 1;
		}
		self.mem_ptr += 2;
	}
	fn op_in(&mut self) {
		let op_name = "OP IN";
		let arg1_result = self.mem_read(self.mem_ptr + 1);
		if !arg1_result.is_ok() {
			self.set_halt_with_error(format!("{} error: invalid arg1 - {}", op_name, arg1_result.unwrap_err()));
			return;
		}
		let arg1 = arg1_result.unwrap();
		if !self.is_reg_addr(arg1) {
			self.set_halt_with_error(format!("{}: arg1 {} is not a register address", op_name, arg1));
			return
		}
		// if the input buffer is empty or exhausted, read a line from stdin
		if self.input_buff.len() == 0 || self.input_buff.len() <= self.input_buff_index {
			if self.interactive {
				let mut line = String::new();
				let stdin = io::stdin();
				stdin.read_line(&mut line).unwrap();
				let char_vec:Vec<char> = line.chars().collect();
				self.input_buff.clear();
				for c in char_vec {
					self.input_buff.push(c);
				}
				self.input_buff_index = 0;
			}
			else {
				self.awaiting_input = true;
				if self.input_ready  {
					self.input_buff_index = 0;
					self.awaiting_input = false;
					self.input_ready = false;
				}
				else {
					// resume execution after input ready
					return;
				}
			}
		}
		let input_char:char = self.input_buff[self.input_buff_index];
		self.input_buff_index += 1;
		let input = input_char as u16;
		
		let reg_set_result = self.reg_set(arg1, input);
		if !reg_set_result.is_ok() {
			self.set_halt_with_error(format!("{} error: {}", op_name, reg_set_result.unwrap_err()));
			return;
		}
		
		self.mem_ptr += 2;
	}
	fn op_noop(&mut self) {
		self.mem_ptr += 1;
	}
	fn op_undefined(&mut self, opcode:u16) {
		self.set_halt_with_error(format!("Undefined opcode {}", opcode));
	}
}