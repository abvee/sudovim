use std::env;
use std::io;
use std::fs;
use std::path::Path;

const ROOT_PATH: &str = "/sudovim";

fn main() -> Result<(), io::Error> {

	// get the path
	let mut path = match env::var("XDG_DATA_HOME") {
		Ok(home) => home,
		Err(_) => match env::var("HOME") {
			Ok(path) => path,
			Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "HOME is not set variables")),
		},
	};
	path.push_str(ROOT_PATH);
	let path = Path::new(&path);

	for i in env::args() {
		if i == "-l" {
			list(&path)?; // list out directory
			return Ok(());
		}
	}
	Ok(())
}

fn list(path: &Path) -> Result<(), io::Error> {
	let dir = fs::read_dir(path)?;

	for entry in dir {
		let entry = entry?;
		let file_type = entry.file_type()?;

		/*
		If the file is directory, recall the function
		If the file is a symlink, println it
		*/
		if file_type.is_dir() {
			list(&entry.path())?;
		}
		else if file_type.is_symlink() {
			println!("{}", &entry.path().display());
		}
	}
	Ok(())
}
