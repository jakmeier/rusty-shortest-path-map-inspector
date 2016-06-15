extern crate piston_window;

use piston_window::*;
use std::env;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::process::Command;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1920;


fn main() {
    
	
	let mut args = Vec::new();
	for argument in env::args() {
		args.push( argument.to_string() );
	}
	
	if args.len() < 2 { println!("No file given."); error(); return; }
	
	println!("Exe: {}", &args[0]);
	println!("File: {}", &args[1]);
	println!("#Arguments: {}", args.len());
	
	let mut map_string = String::new();
	match File::open(&args[1]) {
		Err(e) => {
			println!("{}", e);
			println!("Path of  file: {}", &args[1]);
			error();
			return;
		},
		Ok(mut file) => {
			match file.read_to_string(&mut map_string) {
				Err(e) => {
					println!("Exe: {}", &args[0]);
					println!("File: {}", &args[1]);
					println!("#Arguments: {}", args.len());
					
					println!("{}", e);
					error();
					return;
				},
				Ok(_) => {}
			}
		}
	}
	
	let mut max_x = 1.0;
	let mut max_y = 1.0;
	let mut min_x = 0.0;
	let mut min_y = 0.0;
	
	let mut graph = Vec::new();
	let mut obstacles = Vec::new();
	let mut nodes_finished = false;
	// read graph from string
	'line: for s in map_string.lines() {
		let mut buf = s.chars();
		
		if !nodes_finished {
			// x|y|N|E|S|W|ShortestPath|Cost 
			// 0|0|-|12|13|-|1|140 
			let mut words : [String; 8] = ["".to_string(),"".to_string(),"".to_string(),"".to_string(),"".to_string(),"".to_string(),"".to_string(),"".to_string(),];
			
			for w in words.iter_mut() {	
				loop {
					if let Some(c) = buf.next() {
						if c == '#' {nodes_finished = true; continue 'line;}
						if c == '|' || c == '\n' {break;}
						w.push(c);
						//print!("{}",c);
					}
					else { break; }	
				}
			}
			
			let x = if let Ok(x) = words[0].parse::<f64>(){x}
					else if let Ok(x) = words[0].parse::<i64>(){x as f64}
					else { println!("File corrupted: No x coordinate."); error(); return; };  // exit if it doesn't work, every node needs coordinates
			let y = if let Ok(y) = words[1].parse::<f64>(){y}
					else if let Ok(y) = words[1].parse::<i64>(){y as f64}
					else { println!("File corrupted: No y coordinate."); error(); return; };
			let north = match words[2].parse::<usize>() { Ok(u) => Some(u), _ => None};
			let east = match words[3].parse::<usize>() { Ok(u) => Some(u), _ => None};
			let south = match words[4].parse::<usize>() { Ok(u) => Some(u), _ => None};
			let west = match words[5].parse::<usize>() { Ok(u) => Some(u), _ => None};
			let shortest_path = match words[6].parse::<usize>() { Ok(u) => Some(u), _ => None};
			let cost = if let Ok(cost) = words[7].parse::<f64>(){cost}
						else if let Ok(cost) = words[7].parse::<i64>(){cost as f64}
						else { println!("File corrupted: No cost given for node{}. Displaying -1.0 for it.", graph.len()); -1.0 };
							
			if x > max_x && x < std::f64::INFINITY {max_x = x;}
			if x < min_x && x > std::f64::NEG_INFINITY {min_x = x;}
			if y > max_y && y < std::f64::INFINITY {max_y = y;}
			if y < min_y && y > std::f64::NEG_INFINITY {min_y = y;}
			graph.push( GraphNode{neighbours: [north, east, south, west], x:x, y:y, shortest_path: shortest_path, cost: cost} );
		}
		else {
			// x|y|w|h
			let mut words : [String; 4] = ["".to_string(),"".to_string(),"".to_string(),"".to_string(),];
			
			for w in words.iter_mut() {	
				loop {
					if let Some(c) = buf.next() {
						if c == '|' || c == '\n' {break;}
						w.push(c);
					}
					else { break; }
				}
			}
			
			let x = if let Ok(x) = words[0].parse::<f64>(){x}
					else if let Ok(x) = words[0].parse::<i64>(){x as f64}
					else { println!("File corrupted: No x coordinate for obstacle."); error(); return; }; 
			let y = if let Ok(y) = words[1].parse::<f64>(){y}
					else if let Ok(y) = words[1].parse::<i64>(){y as f64}
					else { println!("File corrupted: No y coordinate for obstacle."); error(); return; };
			let w = if let Ok(w) = words[2].parse::<f64>(){w}
					else if let Ok(w) = words[0].parse::<i64>(){w as f64}
					else { println!("File corrupted: No width for obstacle."); error(); return; };
			let h = if let Ok(h) = words[3].parse::<f64>(){h}
					else if let Ok(h) = words[1].parse::<i64>(){h as f64}
					else { println!("File corrupted: No height for obstacle."); error(); return; };
			
			if x < min_x {min_x = x;}
			if x+w > max_x {max_x = x+w;}
			if y < min_y {min_y = y;}
			if y+h > max_y {max_y = y+h;}
			obstacles.push( (x,y,w,h) );
		
		}
		
	}


	
	let line_color = [0.0,0.0,0.0,1.0];
	let node_color = [1.0, 0.0, 0.0, 1.0];
	let obstacle_color = [0.0,0.0,0.5,0.7];
	let shortest_path_color = [0.0,0.9,0.0,1.0];
	
	let scale = match graph.len()
		{
			0 ... 20 => 5.0,
			21 ... 50 => 3.0,
			51 ... 100 => 2.0,
			_ => 1.0
		};
	let dx:f64 = WIDTH as f64 / (max_x - min_x);
	let dy:f64 = HEIGHT as f64 / (max_y - min_y);
	
	let mut window: PistonWindow = 
	WindowSettings::new("Inspector", [WIDTH, HEIGHT])
	.exit_on_esc(true).build().unwrap();
	
	window.set_max_fps(1);
	
	for e in window {
		e.draw_2d(|c, g| {
			clear([1.0; 4], g);
			
			// Draw graph	
			
			//draw obstacles
			for &(x,y,w,h) in obstacles.iter() {
			    let x = x - min_x;
				let y = y - min_y;
				rectangle(obstacle_color, [x*dx,y*dy,w*dx,h*dy], c.transform, g);
			}
			
			// draw edges
			for node in graph.iter() {
				for d in 0..4 {
					if let Some(neighbour) = node.neighbours[d] {
						line(line_color, scale*3.0, [(node.x - min_x)*dx, (node.y - min_y)*dy, (graph[neighbour].x - min_x)*dx, (graph[neighbour].y - min_y)*dy ], c.transform, g);
					}
				}		
			}
			//draw nodes and shortest paths
			for node in graph.iter() {
				ellipse(node_color, [(node.x - min_x)*dx - scale*10.0, (node.y - min_y)*dy-scale*10.0, scale*20.0, scale*20.0], c.transform, g);
				match node.shortest_path {
					Some(0) => ellipse(shortest_path_color, [(node.x - min_x)*dx - scale*4.0, (node.y - min_y)*dy - scale*18.0, scale*8.0, scale*8.0], c.transform, g),
					Some(1) => ellipse(shortest_path_color, [(node.x - min_x)*dx + scale*10.0, (node.y - min_y)*dy - scale*4.0, scale*8.0, scale*8.0], c.transform, g),
					Some(2) => ellipse(shortest_path_color, [(node.x - min_x)*dx - scale*4.0, (node.y - min_y)*dy + scale*10.0, scale*8.0, scale*8.0], c.transform, g),
					Some(3) => ellipse(shortest_path_color, [(node.x - min_x)*dx - scale*18.0, (node.y - min_y)*dy - scale*4.0, scale*8.0, scale*8.0], c.transform, g),
					_ => ellipse(shortest_path_color, [(node.x - min_x)*dx - scale*4.0, (node.y - min_y)*dy - scale*4.0, scale*8.0, scale*8.0], c.transform, g)
				}
			}
		});
	}
    
}

fn error () {
	let window: PistonWindow = 
	WindowSettings::new("Inspector error", [300, 300])
	.exit_on_esc(true).build().unwrap();
	
	for e in window {
		e.draw_2d(|c, g| {
			clear([1.0; 4], g);
			
			//maybe draw something
			});
		}
}

struct GraphNode {
	neighbours: [Option<usize>;4],
	x: f64, y: f64, 
	shortest_path: Option<usize>, cost: f64,
}