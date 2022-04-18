
#[derive(Clone)]
struct OrbVaultMap {
	width:usize,
	height:usize,
	symbols:Vec<String>,
	values:Vec<i32>,
}
// return the node symbol 
fn parse_vault_node(vm: &super::synacor_vm::SynacorVM, east:usize, north:usize) -> String {
	let mut vm_clone = vm.clone();
	for _y in 0..north {
		vm_clone.input_line_string(format!("go north\n"));
	}
	for _x in 0..east {
		vm_clone.input_line_string(format!("go east\n"));
	}
	// clear output
	let _ = vm_clone.output_line(true);
	vm_clone.input_line("look");
	let look = vm_clone.output_line(true);
	
	// silly way to do this - split on single-quote
	let look_split:Vec<&str> = look.split(39 as char).collect();
	if look_split.len() < 3 {
		return "?".to_string();
	}
	return look_split[1].to_string();
}
fn map_vault(vm: &super::synacor_vm::SynacorVM) -> OrbVaultMap {
	// determine the dimensions
	let mut dim_clone = vm.clone();
	let mut width = 1;
	let mut height = 1;
	// clear output
	let _ = dim_clone.output_line(true);
	loop {
		dim_clone.input_line("go east");
		let output = dim_clone.output_line(true);
		if output.contains("I don't understand") {
			break;
		}
		width += 1;
	}
	loop {
		dim_clone.input_line("go north");
		let output = dim_clone.output_line(true);
		if output.contains("I don't understand") {
			break;
		}
		height += 1;
	}
	let mut map = OrbVaultMap { width:width, height:height, symbols:Vec::new(), values:Vec::new() };
	for y in 0..width {
		for x in 0..height {
			let symbol = parse_vault_node(vm, x,y);
			map.symbols.push(symbol);
		}
	}
	for i in 0..map.symbols.len() {
		if map.symbols[i] == "*".to_string() || 
			map.symbols[i] == "+".to_string() ||
			map.symbols[i] == "-".to_string() {
				map.values.push(0);
		}
		else {
			map.values.push(map.symbols[i].parse::<i32>().unwrap());
		}
	}
	return map;
}
fn evaluate_route(map: &OrbVaultMap, route:&Vec<char> ) -> i32 {
	let mut val = map.values[0];
	let mut x:isize = 0;
	let mut y:isize = 0;
	for i in (0..route.len()).step_by(2) {
		match route[i] {
			'n' => y += 1,
			'e' => x += 1,
			's' => y -= 1,
			'w' => x -= 1,
			_ => return 0
		}
		if x < 0 || y < 0 || x >= map.width as isize|| y >= map.height as isize {
			return 0;
		}			
		let operation = &map.symbols[(x as usize) + map.width * (y as usize)];
		match route[i + 1] {
			'n' => y += 1,
			'e' => x += 1,
			's' => y -= 1,
			'w' => x -= 1,
			_ => return 0
		}
		if x < 0 || y < 0 || x >= map.width as isize || y >= map.height as isize {
			return 0;
		}
		let value = map.values[(x as usize) + map.width * (y as usize)];
		match operation.as_str() {
			"+" => val += value,
			"-" => val -= value,
			"*" => val *= value,
			_ => return 0
		}
	}
	if x == map.width as isize - 1 && y == map.height as isize - 1 {
		return val;
	}
	return 0;
}
fn route_pos(route: &Vec<char>) -> (isize,isize) {
	let mut x = 0;
	let mut y = 0;
	for i in 0..route.len() {
		match route[i] {
			'n' => y += 1,
			'e' => x += 1,
			's' => y -= 1,
			'w' => x -= 1,
			_ => return (0,0)
		}
	}
	return (x,y)
}
// I think a bfs would be smarter, but using a dfs for progressivly larger maximum search depths 
// is fine for this without too much repeated work.
fn recurse(map: &OrbVaultMap, route: &Vec<char>, desired_result:i32, max_steps:usize) -> (bool,Vec<char>) {
	let max_eval = 1<<16; // cutoff if we get stuck in a multiplication loop
	if route.len() >= max_steps {
		return (false,Vec::new());
	}
	let (x1, y1) = route_pos(route);
	// append new random walk steps - two at a time
	for i in 1..=4 {
		let step1:char;
		if i == 1 { 
			if y1 < map.height as isize - 1 {
				step1 = 'n';
			}
			else {
				continue;
			}
		}
		else if i == 2 {
			if x1 < map.width as isize - 1 {
				step1 = 'e';
			}
			else {
				continue;
			}
		}
		else if i == 3 {
			if y1 > 0 {
				step1 = 's';
			}
			else {
				continue;
			}
		}
		else {
			if x1 > 0 {
				step1 = 'w';
			}
			else {
				continue;
			}
		}
		
		for j in 1..=4 {
			let mut new_route = route.clone();
			new_route.push(step1);
			let (x2,y2) = route_pos(&new_route);
			let step2:char;
			
			if j == 1 { 
				if y2 < map.height as isize - 1 {
					step2 = 'n';
				}
				else {
					continue;
				}
			}
			else if j == 2 {
				if x2 < map.width as isize - 1 {
					step2 = 'e';
				}
				else {
					continue;
				}
			}
			else if j == 3 {
				if y2 > 0 {
					step2 = 's';
				}
				else {
					continue;
				}
			}
			else {
				if x2 > 0 {
					step2 = 'w';
				}
				else {
					continue;
				}
			}
			
			
			new_route.push(step2);
			let (x3, y3) = route_pos(&new_route);
			let eval = evaluate_route(map, &new_route);
			if  eval > max_eval {
				continue;
			}
			// the orb resets when we reach the goal node, so it can only be visited once
			if x3 == map.width as isize - 1 && y3 == map.width as isize - 1 && eval != desired_result {
				continue;
			}
			if eval == desired_result {
				return (true, new_route);
			}
			let (success,next) = recurse(map, &new_route, desired_result, max_steps);
			if success {
				return (true, next)
			}
		}
	}
	return (false,Vec::new());
}
fn try_route(vm:&mut super::synacor_vm::SynacorVM, route:&Vec<char>) {
	vm.input_line("take orb");
	for c in route {
		match c {
			'n' => vm.input_line("go north"),
			'e' => vm.input_line("go east"),
			's' => vm.input_line("go south"),
			'w' => vm.input_line("go west"),
			_ => {
					println!("Unknown direction {}", c); 
					return; 
				}
		}
	}
	vm.input_line("go vault");
}
fn reverse(code:&String) -> String {
	let code_chars:Vec<char> = code.chars().collect();
	let mut result_chars:Vec<char> = Vec::new();
	let mut temp:char;
	for i in 0..code_chars.len() {
		temp = code_chars[i];
		// possibly not an exhaustive list of reversal replacements
		match temp {
			'p' => temp = 'q',
			'q' => temp = 'p',
			'b' => temp = 'd',
			'd' => temp = 'b',
			_ => ()
		}
		result_chars.insert(0, temp);
	}
	return result_chars.into_iter().collect();
}
fn mirror_code(mirror_use:&String) -> String {
	// dumb way to extract the code string - split on double quote
	let code_split:Vec<&str> = mirror_use.split(34 as char).collect();
	if code_split.len() < 3 {
		return "Code not found!".to_string();
	}
	return reverse(&code_split[1].to_string());
}
// requires a vm placed at the orb position in the vault antechamber
pub fn solve(vm:&mut super::synacor_vm::SynacorVM) -> bool {
	println!("Exploring the rooms around the vault...");
	let map = map_vault(vm);
	let desired_result = 1;
	let steps_cutoff = 16;
	println!("Pondering a route for the orb...");
	for steps in 6..=steps_cutoff {
		let start_route:Vec<char> = Vec::new();
		let (success, route) = recurse(&map, &start_route, desired_result, steps); 
		if !success {
			continue;
		}
		println!("A route has been discovered! Taking the orb and proceeding to the vault...");
		try_route(vm, &route);
		vm.input_line("go vault");
		vm.input_line("take mirror");
		// clear output
		let _ = vm.output_line(true);
		vm.input_line("use mirror");
		let use_mirror = vm.output_line(true);
		println!("{}", use_mirror);
		println!("Thinking quickly, you realize that the writing on your forehead is reversed in the mirror! It must actually say \"{}\" ", mirror_code(&use_mirror));
		return true;
	}
	println!("Pondering failed to find a route for the orb.");
	return false;
}
