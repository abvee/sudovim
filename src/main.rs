use std::env;
use std::io;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::Command;
use std::io::ErrorKind;

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

	// get editor. If EDITOR not set, use vim
	let editor = match env::var("EDITOR") {
		Ok(editor) => editor,
		Err(_) => String::from("vim"),
	};

	let mut cmdline = env::args();
	let argc: usize = cmdline.len();
	cmdline.next(); // get rid of argv[0]
	let mut cmdline = cmdline.peekable();

	let file_names: Vec<String> = loop {
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
	
	let mut crcs: Vec<u64> = Vec::with_capacity(argc);
	let mut files: Vec<Option<File>> = Vec::with_capacity(argc);

	for i in 0..file_names.len() {
		println!("Found file: {}", i);

		files[i] = match File::open(&file_names[i]) {
			Ok(file) => Some(file),
			Err(e) => match e.kind() {
				ErrorKind::NotFound => None,
				_ => return Err(e),
			},
		};

		// hash the files
		if let Some(file) = &files[i] { crcs[i] = hash(file) };
	}

	// start vim
	Command::new("doas")
		.arg(editor)
		.args(&file_names)
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

fn hash(file: &File) -> u64 {
	1
}
