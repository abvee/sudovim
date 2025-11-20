use std::env;
use std::io;
use std::io::Read;
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
	cmdline.next(); // get rid of argv[0]
	let argc: usize = cmdline.len();
	println!("Commandline len: {}", argc);
	if argc == 0 { return Ok(()) }
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
	
	let mut files: Vec<Option<File>> = Vec::with_capacity(argc);
	let mut sizes: Vec<usize> = Vec::with_capacity(argc);
	let mut hashes: Vec<u64> = Vec::with_capacity(argc);

	let mut buffer: Vec<u8> = Vec::new(); // general purpose buffer

	for name in &file_names {
		println!("Found file: {}", name);

		if check_subdir(&path, Path::new(name))? {
			continue
		}

		files.push(match File::open(name) {
			Ok(mut file) => {
				sizes.push(file.read_to_end(&mut buffer)?);
				hashes.push(hash(&buffer));
				Some(file)
			},
			Err(e) => match e.kind() {
				ErrorKind::NotFound => None, // new file
				_ => return Err(e),
			},
		});
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

// Just a hash by XORing the 8 bytes together.
// There is a very small chance that if the file sizes are the same, the hashes
// would also be the same.
fn hash(bytes: &[u8]) -> u64 {
	let mut hash_: u64 = 0;

	let mut i = 0;
	while i+8 < bytes.len() {
		hash_ ^= convert_u64(&bytes[i..i+8]);
		i += 8;
	}
	hash_
}

fn convert_u64(bytes: &[u8]) -> u64 {
	let mut target: u64 = 0;

	let mut i = 0;
	while i < 8 && i < bytes.len() {
		target += (bytes[i] as u64) << (i * 8);
		i += 1;
	}
	target
}

fn check_subdir(path: &Path, subdir: &Path) -> Result<bool, io::Error> {
	// get the path to check
	// if that path doesn't exist at all, return false'
	let subdir = match subdir.canonicalize() {
		Ok(existing_dir) => existing_dir,
		Err(e) => return match e.kind() {
			ErrorKind::NotFound => Ok(false),
			_ => Err(e),
		},
	};

	let check_path = path.join(
		subdir.canonicalize()?
			.as_path()
			.strip_prefix("/")
			.expect("file name is did not canonicalize")
	);
	Ok(check_path.exists())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_u64_test() {
		let result = convert_u64(&[0x78, 0x56, 0x34, 0x12]);
		assert_eq!(result, 305419896);
	}

	#[test]
	fn hash_test() -> Result<(), io::Error> {
		let bytes = fs::read(Path::new("./src/main.rs"))?;
		println!("{:x}", hash(&bytes));
		Ok(())
	}

	#[test]
	fn check_subdir_test() -> Result<(), io::Error> {
		let mut path = match env::var("XDG_DATA_HOME") {
			Ok(home) => home,
			Err(_) => match env::var("HOME") {
				Ok(path) => path,
				Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "HOME is not set variables")),
			},
		};
		path.push_str(ROOT_PATH);
		let path = Path::new(&path);

		println!("{}", check_subdir(&path, Path::new("/tmp"))?);
		Ok(())
	}
}
