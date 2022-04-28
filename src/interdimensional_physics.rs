use std::collections::HashMap;
use std::thread;
use std::sync::{Arc,Mutex};

const U15_MAX: u16 = 32767;
const U15_MOD: u16 = 32768;
const REG_N:u16 = 8;

fn decompiler_val(program:&Vec<u16>, index:usize) -> String {
	if index >= program.len() {
		return "Invalid index".to_string();
	}
	let val = program[index];
	if val <= U15_MAX {
		return format!("{}", val);
	}
	else if val > U15_MAX && val <= U15_MAX + REG_N + 1 {
		return format!("reg{}", val - U15_MAX - 1);
	}
	else {
		return format!("INVALID {}", val);
	}
}
fn append_with_tabs(input:String, tab_pos:usize, append:String) -> String {
	let tab_len = 4;
	if tab_len * tab_pos < input.len() {
		return format!("{}\t{}", input, append);
	}
	let mut result = input.clone();
	let mut pos = result.len();
	while pos < tab_pos * tab_len {
		result.push_str("\t");
		pos += tab_len;
	}
	return format!("{}{}", result, append);
}
pub fn decompile(program:&Vec<u16>) -> Vec<String> {
	let mut index = 0;
	let mut lines:Vec<String> = Vec::new();
	let tab_pos = 6;
	while index < program.len() {
		match program[index] {
			0 => {
					lines.push(append_with_tabs(format!("HALT"), tab_pos, format!("#{}", index)));
					index += 1;
				},
			1 => {
					lines.push(append_with_tabs(format!("SET {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2)), tab_pos, format!("#{}", index)));
					index += 3;
				},
			2 => {
					lines.push(append_with_tabs(format!("PUSH {}", decompiler_val(program, index + 1)), tab_pos, format!("#{}", index)));
					index += 2;
				},
			3 => {
					lines.push(append_with_tabs(format!("POP {}", decompiler_val(program, index + 1)), tab_pos, format!("#{}", index)));
					index += 2;
				},
			4 => {
					lines.push(append_with_tabs(format!("EQ {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			5 => {
					lines.push(append_with_tabs(format!("GT {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			6 => {
					lines.push(append_with_tabs(format!("JMP {}", decompiler_val(program, index + 1)), tab_pos, format!("#{}", index)));
					index += 2;
				},
			7 => {
					lines.push(append_with_tabs(format!("JT {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2)), tab_pos, format!("#{}", index)));
					index += 3;
				},
			8 => {
					lines.push(append_with_tabs(format!("JF {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2)), tab_pos, format!("#{}", index)));
					index += 3;
				},
			9 => {
					lines.push(append_with_tabs(format!("ADD {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			10 => {
					lines.push(append_with_tabs(format!("MULT {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			11 => {
					lines.push(append_with_tabs(format!("MOD {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			12 => {
					lines.push(append_with_tabs(format!("AND {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			13 => {
					lines.push(append_with_tabs(format!("OR {} {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2), decompiler_val(program, index + 3)), tab_pos, format!("#{}", index)));
					index += 4;
				},
			14 => {
					lines.push(append_with_tabs(format!("NOT {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2)), tab_pos, format!("#{}", index)));
					index += 3;
				},
			15 => {
					lines.push(append_with_tabs(format!("RMEM {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2)), tab_pos, format!("#{}", index)));					
					index += 3;
				},
			16 => {
					lines.push(append_with_tabs(format!("WMEM {} {}", decompiler_val(program, index + 1), decompiler_val(program, index + 2)), tab_pos, format!("#{}", index)));					
					index += 3;
				},
			17 => {
					lines.push(append_with_tabs(format!("CALL {}", decompiler_val(program, index + 1)), tab_pos, format!("#{}", index)));
					index += 2;
				},
			18 => {
					lines.push(append_with_tabs(format!("RET"), tab_pos, format!("#{}", index)));
					index += 1;
				},
			19 =>  {
					let val = program[index+1];
					if val <= 255 {
						if val == 10 {
							lines.push(append_with_tabs(format!("OUT {}\t(LF)", decompiler_val(program, index + 1)), tab_pos - 1, format!("#{}", index)));		
						}
						else {
							lines.push(append_with_tabs(format!("OUT {}\t({})", decompiler_val(program, index + 1), val as u8 as char), tab_pos, format!("#{}", index)));
						}
						
					}
					else {
						lines.push(append_with_tabs(format!("OUT {}", decompiler_val(program, index + 1)), tab_pos, format!("#{}", index)));
					}
					index += 2;
				},
			20 =>  {
					lines.push(append_with_tabs(format!("IN {}", decompiler_val(program, index + 1)), tab_pos, format!("#{}", index)));
					index += 2;
				},
			21 => {
					lines.push(append_with_tabs(format!("NOOP"), tab_pos, format!("#{}", index)));
					index += 1;
				},
			_ => {
					lines.push(append_with_tabs(format!("DATA? {}", program[index]), tab_pos, format!("#{}", index)));
					index += 1;
				}
		}
	}
	return lines;
}

pub fn thread_6027_single(r7:u16) -> bool {
	let mut cache:HashMap<u32, u16> = HashMap::new();
	for r0 in 0..=4 {
		for r1 in 0..=U15_MAX {
			let compute_result = compute_6027(r0 as u16,r1 as u16,r7 as u16, &mut cache);
			cache.insert(tuple_key(r0 as u16, r1 as u16), compute_result);
			if r1 == 1 && r0 == 4 {
				break;
			}
		}
	}
	return cache[&tuple_key(4,1)] == 6;
}

pub fn thread_6027(lo:u16, hi:u16) -> u16 {
	let mut r7:u16 = lo;
	while r7 <= hi {
		if thread_6027_single(r7 as u16) {
			return r7 as u16;
		}
		r7+=1;
	}
	return 0 as u16;
}

// multi-thread search for r7
pub fn multithreaded_6027_result() -> u16 {
	println!("This may take some time...");
	let num_threads = 8; // number of search threads to batch at once
	let range = 20;	 // rumber of r7 values to test per spawned thread
	let mut index = 1;
	let mut done = false;
	let mut r7 = 0;
	let mut handles = Vec::new();
	let arc = Arc::new(Mutex::new(0));
	while !done {
		for _ in 0..num_threads {
			let arc_clone = Arc::clone(&arc);
			let handle = thread::spawn(move || {
				let lo:u16 = index;
				let hi:u16;
				if lo + range > U15_MAX {
					hi = U15_MAX;
				}
				else {
					hi = lo + range;
				}
				let result = thread_6027(lo, hi);
				if result != 0 {
					let mut r7_result = arc_clone.lock().unwrap();
					(*r7_result) = result;
				}
			});
			handles.push(handle);
			index = index + range;
			if index > U15_MAX {
				done = true;
				break;
			}
		}
		// execute batch
		while handles.len() > 0 {
			let handle = handles.remove(0);
			handle.join().unwrap();
		}
		
		let runresult = *arc.lock().unwrap();
		if runresult != 0 {
			r7 = runresult;
			done = true;
		}
	}
	
	if r7 != 0 {
		println!("Interdimensional physics analysis complete. Thank you for your patience.");
	}	
	return r7;
}

// single-thread search for r7
pub fn match_6027_result() -> u16 {
	println!("This may take some time...");
	for r7 in 1..=U15_MAX {
		if thread_6027_single(r7) {
			println!("Interdimensional physics analysis complete. Thank you for your patience.");
			return r7;
		}
	}
	return 0;
}

// heck if I can figure out exactly what this is doing, but we can speed it up
// by caching previous results. Returns the final state of r0
pub fn compute_6027(r0:u16, r1:u16, r7:u16, cache:&mut HashMap<u32, u16>) -> u16{
	if r0 == 0 {
		return (r1 + 1) % (U15_MOD);
	}
	else if r1 == 0 {
		return cache[&tuple_key(r0 - 1, r7)];
	}
	else {
		let r1_next = cache[&tuple_key(r0, (r1 - 1) % (U15_MOD))];
		return cache[&tuple_key(r0 - 1, r1_next)];
	}
}

fn tuple_key(r0:u16, r1:u16) -> u32 {
	return (r0 as u32) * (U15_MOD as u32) + (r1 as u32);
}

pub fn physics_analysis(vm:&mut super::synacor_vm::SynacorVM, skip_precomputed:bool, multithreaded:bool) -> bool {
	println!("Starting interdimensional physics analysis...");
	
	// after a lot of examination, I still don't know what the function at 6027 does exactly
	// and I definitely don't have anything like an algabraic simplification. But at least we
	// can compute it much faster by storing all the possible r1 results for one r0 level before 
	// moving on to the next. I bet there's a faster way to do this though.
	
	// Precomputed solution included due to the length of time needed to solve. 
	// Passing -t <SEARCH_TYPE> as a command line argument will enable the search anyway. 
	// Single-threaded and parallel mutli-threaded searches are available. Multi-threaded
	// is recommended for speed if multiple cores are available.
	let precomputed_solution = 25734;
	let r7;
	if precomputed_solution != 0 && !skip_precomputed {
		println!("Interdimensional physics analysis completed with the power of previously computing the solution at great length.");
		r7 = precomputed_solution;
	}
	else {
		if multithreaded {
			r7 = multithreaded_6027_result();
		}
		else {
			r7 = match_6027_result();
		}
	}	
	if r7 == 0 {
		println!("Interdimensional physics analysis failed!");
		return false;
	}
	println!("Modifying universe...");
	vm.set_register(7, r7);
	
	println!("Activating teleporter...");
	let _ = vm.output_line(true);
	
	vm.set_input_line("use teleporter");
	loop {	
		if vm.get_mem_ptr() == 5489 {
			println!("Bypassing teleporter activation check...");
			vm.set_register(0, 6);
			vm.set_register(1, 5);
			vm.set_mem_ptr(5491);
			println!("Teleporting...");
			break();
		}
		vm.execute_once();
		if vm.is_halted() || vm.is_awaiting_input() {
			break;
		}
	}
	// clear output
	let _ = vm.output_line(true);
	vm.execute();
	// arriving on the beach yields challenge code #7
	println!("{}", vm.output_line(true));
	return true;
}
