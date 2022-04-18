use std::collections::HashMap;

#[derive(Clone)]
struct TwistyPassagesLink {
	name:String,
	id:u16,
	explored:bool,
}

#[derive(Clone)]
struct TwistyPassagesNode {
	id:u16,
	links:Vec<TwistyPassagesLink>,
	items: Vec<String>,
	look:String,
	halts:bool,
	path:Vec<String>,
}

// requires a vm at the ladder position
fn explore_link(vm: &super::synacor_vm::SynacorVM, path:Vec<String>, nodes:&mut HashMap<u16, TwistyPassagesNode>, parent:u16) -> u16 {
	let path_clone = path.clone();
	let node = parse_node(vm, path_clone);
	let node_id = node.id;
	if nodes.contains_key(&node_id) {
		// shorten the path if possible
		if path.len() < nodes.get(&node.id).unwrap().path.len() {
			let update_node = nodes.get_mut(&node.id).unwrap();
			update_node.path.clear();
			for i in 0..path.len() {
				update_node.path.push(path[i].clone());
			}
		}
	}
	else {
		nodes.insert(node.id, node);
	}
	if nodes.contains_key(&parent) {
		let parent_node = nodes.get_mut(&parent).unwrap();
		for i in 0..parent_node.links.len() {
			if parent_node.links[i].name == path[path.len() - 1] {
				parent_node.links[i].id = node_id;
				parent_node.links[i].explored = true;
				break;
			}
		}
	}
	else {
		println!("Explore_link: node list does not contain an entry for parent node {}", parent);
	}
	return node_id;
}
// requires a vm at the ladder position
fn parse_node(vm: &super::synacor_vm::SynacorVM, path:Vec<String>) -> TwistyPassagesNode {
	// by manual inspection, it looks like the current node id is set at 
	// memory address 2733
	
	let mut vm_clone = vm.clone();
	for i in 0..path.len() {
		vm_clone.input_line_string(format!("go {}\n", path[i]));
	}
	// clear output buffer and look
	let _ = vm_clone.output_line(true);
	vm_clone.input_line("look");
	let look = vm_clone.output_line(true);
	let lines:Vec<&str> = look.split(10 as char).collect();
	let id = vm_clone.get_mem(2733).unwrap();
	let halts = vm_clone.is_halted();
	
	
	let mut node = TwistyPassagesNode { id: id, links:Vec::new(), items:Vec::new(), look:look.clone(), halts:halts, path:path.clone()};
	let mut at_items:bool = false;
	let mut at_exits:bool = false;
	for i in 0..lines.len() {
		if lines[i].contains("Things of interest here:") {
			at_items = true;
			continue;
		}
		if lines[i].contains("exits:") {
			at_exits = true;
			continue;
		}
		if at_items {
			if lines[i].len() < 2 {
				at_items = false;
				continue;
			}
			node.items.push(lines[i][2..].to_string());
		}
		if at_exits {
			if lines[i].len() < 2 {
				at_exits = false;
				continue;
			}
			node.links.push(TwistyPassagesLink { name:lines[i][2..].to_string(), id:0, explored:false });
		}
	}
	
	return node;
}
// requires a vm in non-interactive mode that has been placed 
// at the ladder node in the maze of twisty passages
pub fn solve(vm:&mut super::synacor_vm::SynacorVM) -> bool {
	println!("Begin exploring...");
	let vm_clone = vm.clone();
	let mut nodes:HashMap<u16, TwistyPassagesNode> = HashMap::new();
	let mut explored:Vec<u16> = Vec::new();
	let mut frontier:Vec<u16> = Vec::new();
	let mut frontier_next:Vec<u16> = Vec::new();

	let start_node = parse_node(&vm_clone, Vec::new());
	
	frontier_next.push(start_node.id);
	nodes.insert(start_node.id, start_node);
	
	while frontier_next.len() > 0 {
		frontier.clear();
		for i in 0..frontier_next.len() {
			frontier.push(frontier_next[i]);
		}
		frontier_next.clear();
		
		for i in 0..frontier.len() {
			let node = nodes.get(&frontier[i]).unwrap().clone();
			for j in 0..node.links.len() {
				if !node.links[j].explored {
					let mut path = node.path.clone();
					path.push(node.links[j].name.clone());
					let link_id = explore_link(&vm_clone, path, &mut nodes, node.id);
					if !explored.contains(&link_id) {
						frontier_next.push(link_id);
					}
				}					
			}
			if !explored.contains(&node.id) {
				explored.push(node.id);
			}
		}
	}
	println!("Explored {} locations.", explored.len());
	let mut solution:bool = false;
	for (_, node) in nodes.iter() {
		if node.items.len() > 0 {
			println!("An important location has been discovered!");
			solution = true;
			for i in 0..node.path.len() {
				vm.input_line_string(format!("go {}", node.path[i]));
				if i < node.path.len() - 1 {
					let _ = vm.output_line(true);
				}
				// the output at the destination yields a challenge code
				println!("{}", vm.output_line(true));
			}
			println!("Taking the {}...", node.items[0]);
			vm.input_line_string(format!("take {}", node.items[0]));
			if node.links.len() > 0 {
				vm.input_line_string(format!("go {}", node.links[0].name));
			}
			break;
		}
	}
	return solution;
}