struct Coin {
	name: String,
	val: u16,
}

// in principle one could parse the monument inscription, but...
fn formula(a:u16, b:u16, c:u16, d:u16, e:u16) -> u16{
	return a + b * (c*c) + (d*d*d) - e;
}
fn solution() -> Vec<u16> {
	let coins:Vec<u16> = vec![2,7,3,9,5];
	let solution = 399;
	// there's probably a more elegant way to do n choose k
	for a in 0..coins.len() {
		for b in 0..coins.len()  {
			for c in 0..coins.len()  {
				for d in 0..coins.len() {
					for e in 0..coins.len()  {
						if b == a || c == a || d == a || e == a ||
							c == b || d == b || e == b ||
							d == c || e == c ||
							e == d {
								continue;
							}
							if formula(coins[a], coins[b], coins[c], coins[d], coins[e]) == solution {
								return vec![coins[a],coins[b],coins[c],coins[d],coins[e]];
							}
					}
				}
			}
		}
	}
	return Vec::new();
}
fn examine_coin(vm:&mut super::synacor_vm::SynacorVM, coin_name:String) -> Coin {
	vm.input_line_string(format!("look {}", coin_name));
	let look = vm.output_line(true);
	let mut val = 0;
	if look.contains("two") {
		val = 2;
	}
	else if look.contains("triangle") {
		val = 3;
	}
	else if look.contains("pentagon") {
		val = 5;
	}
	else if look.contains("seven") {
		val = 7;
	}
	else if look.contains("nine") {
		val = 9;
	}
	let coin = Coin{name: coin_name, val: val};
	return coin;
}
fn parse_inventory(inventory:String) -> Vec<String> {
	let lines:Vec<&str> = inventory.split(10 as char).collect();
	let mut inv:Vec<String> = Vec::new();
	let mut at_inv:bool = false;
	for i in 0..lines.len() {
		if lines[i].contains("Your inventory:") {
			at_inv = true;
			continue;
		}
		if at_inv {
			if lines[i].len() < 2 {
				break;
			}
			inv.push(lines[i][2..].to_string());
		}
	}
	return inv;
}
// requires a vm in non-interactive mode that has been placed 
// at the strange monument with all coins collected
pub fn solve(vm:&mut super::synacor_vm::SynacorVM) -> bool {
	println!("Pondering deeply...");
	println!("Examining inventory...");
	let _ = vm.output_line(true);
	vm.input_line("inv");
	let inv = parse_inventory(vm.output_line(true));
	
	let mut coins:Vec<Coin> = Vec::new();
	for item in inv {
		if item.contains("coin") {
			coins.push(examine_coin(vm, item));
		}
	}
	let solution = solution();
	if solution.len() != 5 {
		println!("Pondering failed to reveal a solution...");
		return false;
	}
	for val in solution {
		for i in 0..coins.len() {
			if coins[i].val == val {
				let _ = vm.output_line(true);
				println!("Using the {}...", coins[i].name);
				vm.input_line_string(format!("use {}", coins[i].name));
				break
			}
		}
	}
	println!("{}", vm.output_line(true));
	println!("Pondering successful!");
	return true;
}