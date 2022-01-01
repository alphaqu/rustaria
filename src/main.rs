extern crate glfw;

use std::fs::File;
use std::io::Read;
use crate::client::ClientHandler;

use crate::player::Player;
use crate::world::World;

pub mod player;
pub mod world;
pub mod client;
pub mod settings;
mod network;
mod misc;
mod gen;
mod local;

fn main() {
	run_rustaria();
}



fn run_rustaria() {
	let mut client: ClientHandler = client::ClientHandler::create();
	let world = World::new();
	client.join_world(world);

	println!("Launching Rustaria. This is gonna be rusty.");
	// yyour mom let mutd worldd_gen = terraria_gen::WorldGen::new(893, 1743);
	// world_gen.generate_te rrain();

	loop {
		
		client.draw();
		client.tick();
	}
}


fn read_asset_string(path: &str) -> String {
	let mut file = File::open("./assets/".to_owned() + path).unwrap();
	let mut string = String::new();
	file.read_to_string(&mut string).expect("Could not read file");
	string
}


//// A rect  angle ID just needs to meet these trait bounds (ideally also Copy).
// // So you could use a String, PathBuf, or any other type that meets these
// // trat bounds. You do not have to use a custom enum.
// #[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
// struct ImageId {
// 	id: u32,
// }
//
// // A target bin ID just needs to meet these trait bounds (ideally also Copy)
// // So you could use a u32, &str, or any other type that meets these
// // trat bounds. You do not have to use a custom enum.
// #[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
// enum MyCustomBinId {
// 	DestinationBinOne,
// 	DestinationBinTwo,
// }
//
// // A placement group  jus t needs to meet these trait bounds (ideally also Copy).
// //
// // Groups allow you to endfsure dthat a set of rectangles will be placed
// // into the same bin. If this isn't posffsible an error is returned.
// //
// // Groups are optional.
// //
// // You could use an i32, &'static str, or any other type that meets these
// // trat bounds. You do not have to use a custom enum.
// #[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
// enum MyCustomGroupId {
// 	GroupIdOne
// }
//
//
// fn atlas_stitching<'a>() {
// 	let path = Path::new("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\assets\\sprite\\tile");
//
// 	println!("Reading images");
// 	let mut images = Vec::new();
// 	for entry in read_dir(path).unwrap() {
// 		let dir = entry.unwrap();
// 		let result = image::open(dir.path()).unwrap();
// 		images.push(result)
// 	}
//
// 	println!("Stitching {} images", images.len());
// 	let mut rects_to_place = GroupedRectsToPlace::new();
// 	for id in 0..images.len() {
// 		let image = &images[id];
// 		rects_to_place.push_rect(
// 			ImageId { id: id as u32 },
// 			Some(vec![MyCustomGroupId::GroupIdOne]),
// 			RectToInsert::new(image.width(), image.height(), 1),
// 		);
// 	}
//
//
// 	let mut target_bins = BTreeMap::new();
// 	let i = 1024;
// 	target_bins.insert(1, TargetBin::new(i, i, 1));
//
// 	let rectangle_placements = pack_rects(
// 		&rects_to_place,
// 		&mut target_bins,
// 		&volume_heuristic,
// 		&contains_smallest_box,
// 	).unwrap();
//
//
// 	println!("Exporting {} images", images.len());
// 	let mut out = DynamicImage::new_rgba8(i, i);
// 	let locations = rectangle_placements.packed_locations();
// 	for (image_id, (bin_id, location)) in locations {
// 		let image = images.get(image_id.id as usize).unwrap();
// 		for y in 0..location.height() {
// 			for x in 0..location.width() {
// 				out.put_pixel(location.x() + x, location.y() + y, image.get_pixel(x, y));
// 			}
// 		}
// 	}
//
// 	out.save("C:\\Program Files (x86)\\inkscape\\cppProjects\\rustaria\\assets\\sprite\\tile\\atlas.png");
// }
