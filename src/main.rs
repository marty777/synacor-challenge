pub mod synacor_vm;
pub mod twisty_passages;
pub mod strange_monument;
pub mod interdimensional_physics;

use std::env;
use std::process;
use std::io::prelude::*;
use std::fs;
use std::fs::File;
use std::io::Write;

fn read_bin(path:&String) -> Vec<u16>
{
	let file_result = File::open(&path);
	if !file_result.is_ok() {
		println!("Unable to open file at path {}", path);
		usage();
	}
	let metadata_result = fs::metadata(&path);
	if !metadata_result.is_ok() {
		println!("Unable to read metadata for file at path {}", path);
		usage();
	}
	let mut file = file_result.unwrap();
	let metadata = metadata_result.unwrap();
    let mut buff = vec![0; metadata.len() as usize];
	let file_read_result = file.read(&mut buff);
	if !file_read_result.is_ok() {
		println!("An error occurred while reading the file at path {}: {}", path, file_read_result.unwrap_err());
		usage();
	}
	let mut buff2:Vec<u16> = vec![0; ((metadata.len()/2) + 1) as usize];
	let mut index = 0;
	while index < buff.len() {
		let byte1 = buff[index];
		let mut byte2 = 0;
		if index + 1 < buff.len() {
			byte2 = buff[index + 1];
		}
		let mut val:u16 = 0;
		val = val | (byte1 as u16);
		val = val | (byte2 as u16) << 8;
		buff2[index/2] = val;
		index = index + 2;
	}
	return buff2;
}

fn usage() {
	println!("Usage:\tcargo run <binary_file_path>");
	println!("\tbinary_file_path:\tpath to input binary file");
	process::exit(0);
}

fn main() {
	// don't forget to examine the arch-spec file for challenge code #1
	let dump_decompile:bool = false;
	// get path to binary from args
	if env::args().len() < 2 {
		usage();
	}
	for argument in env::args() {
		println!("{}", argument);
	}
	// load the binary;
	let bin_path = env::args().nth(1).unwrap();
	let binary = read_bin(&bin_path);
	if dump_decompile {
		let decompiled = interdimensional_physics::decompile(&binary);
		let mut file = File::create("decompiled.txt").unwrap();
		for i in 0..decompiled.len() {
			writeln!(&mut file, "{}", decompiled[i]).unwrap();
		}
	}
	// initialize vm and load binary into memory
	let mut vm:synacor_vm::SynacorVM = synacor_vm::SynacorVM::new(false);
	let load_mem_result = vm.load_mem(binary);
	if !load_mem_result.is_ok() {
		println!("Load program error: {}", load_mem_result.unwrap_err())
	}	
	// run initial startup and self test
	vm.execute();
	// the output of startup and the self test yields challenge codes #2 and #3
	println!("{}",vm.output_line(true));
	
	println!("Suspending interactive mode. Beginning automatic traversal.");
	// challenge code #4 appears here when taking and using the tablet
	play_to_twisty_passages(&mut vm);
	println!("Automatic traversal has reached the maze of twisty little passages, all alike.");
	println!("Solving the maze of twisty little passages using the power of the multiverse...");
	// solving the maze of twisty little passages yields challenge code #5
	if !twisty_passages::solve(&mut vm) {
		println!("Unable to solve the maze of twisty little passages");
		process::exit(0);
	}
	println!("The maze of twisty passages has been solved. The can has been located.");
	println!("Resuming automatic traversal...");
	
	play_to_strange_monument(&mut vm);
	
	println!("Automatic traversal has reached the strange monument.");
	println!("Thinking about the solution to the strange monument...");
	if !strange_monument::solve(&mut vm) {
		println!("Unable to solve the mystery of the strange monument");
		process::exit(0);
	}
	println!("The mystery of the strange monument has been solved. The way forward has opened.");
	println!("Resuming automatic traversal...");
	
	// reaching Synacor HQ yields challenge code #6
	play_to_synacor_hq(&mut vm);
	
	println!("Automatic traversal has reached Synacor Headquarters.");
	println!("Delving into the secrets of the universe...");
	// activating the teleporter correctly to reach the second destination yields challenge code #7
	if !interdimensional_physics::physics_analysis(&mut vm) {
		println!("Unable to solve the secrets of the universe...")
	}
	println!("The secrets of the universe have been illuminated. The teleporter destination has been reached.");

	println!("Resuming interactive mode...");
	vm.set_interactive(true);	
	loop {
		vm.execute();
	}
	
	
}

fn play_to_twisty_passages(vm:&mut synacor_vm::SynacorVM ) {
	println!("Taking tablet...");
	vm.input_line("take tablet");
	let _ = vm.output_line(true);
	println!("Using tablet...");
	vm.input_line("use tablet");
	// the use tablet command yields challenge code #4
	println!("{}", vm.output_line(true));
	vm.input_line("go doorway");
	vm.input_line("go north");
	vm.input_line("go north");
	vm.input_line("go bridge");
	vm.input_line("go continue");
	vm.input_line("go down");
	vm.input_line("go east");
	println!("Taking empty lantern...");
	vm.input_line("take empty lantern");
	vm.input_line("go west");
	vm.input_line("go west");
	vm.input_line("go passage");
	vm.input_line("go ladder");
}


fn play_to_strange_monument(vm: &mut synacor_vm::SynacorVM) {
	vm.input_line("go west");
	vm.input_line("go ladder");
	vm.input_line("go darkness");
	println!("Using the can...");
	vm.input_line("use can");
	println!("Using the lantern...");
	vm.input_line("use lantern");
	vm.input_line("go continue");
	vm.input_line("go west");
	vm.input_line("go west");
	vm.input_line("go west");
	vm.input_line("go west");
	vm.input_line("go north");
	println!("Taking the red coin...");
	vm.input_line("take red coin");
	vm.input_line("go north");
	vm.input_line("go east");
	println!("Taking the concave coin...");
	vm.input_line("take concave coin");
	vm.input_line("go down");
	println!("Taking the corroded coin...");
	vm.input_line("take corroded coin");
	vm.input_line("go up");
	vm.input_line("go west");
	vm.input_line("go west");
	println!("Taking the blue coin...");
	vm.input_line("take blue coin");
	vm.input_line("go up");
	println!("Taking the shiny coin...");
	vm.input_line("take shiny coin");
	vm.input_line("go down");
	vm.input_line("go east");
	let _ = vm.output_line(true);
	//vm.input_line("look");
	//println!("{}",vm.output_line(true));
}

fn play_to_synacor_hq(vm: &mut synacor_vm::SynacorVM) {
	vm.input_line("go north");
	println!("Taking the teleporter...");
	vm.input_line("take teleporter");
	let _ = vm.output_line(true);
	println!("Using the teleporter...");
	vm.input_line("use teleporter");
	// using the teleporter to reach synacor hq yields challenge code #6
	println!("{}", vm.output_line(true));
	println!("Taking the business card...");
	vm.input_line("take business card");
	println!("Taking the strange book...");
	vm.input_line("take strange book");
}