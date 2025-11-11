use std::env;
use std::io;
use std::fs;
use std::path::Path;
use std::process::Command;

const ROOT_PATH: &str = "/sudovim";

fn main() -> Result<(), io::Error> {
	let mut cmdline = env::args();

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

	cmdline.next(); // get rid of argv[0]
	let mut cmdline = cmdline.peekable();

	let files: Vec<String> = loop {
		if let Some(arg) = cmdline.peek() {
			if arg == "-l" {
				println!("found argument: {}", arg);
				return list(&path);
			}

			if &arg[0..1] != "-" {
				break cmdline.collect()
			}
		} else { break cmdline.collect() };
		// NOTE: this ^ else breaks null
	};
	
	// print out files
	for i in &files {
		println!("Found file: {}", i);
	}

	// start vim
	Command::new("vim")
		.args(&files)
		.status()?;
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
