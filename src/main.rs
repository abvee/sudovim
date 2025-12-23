use std::env;
use std::io;
use std::io::Read;
use std::fs;
use std::fs::File;

use std::path;
use path::Path;
use path::PathBuf;

use std::process::Command;
use std::os::unix::fs::symlink;

mod xxhash;

const ROOT_PATH: &str = "/sudovim";

fn main() -> Result<(), io::Error> {
	// get the path
	let mut root_path = env::var("XDG_DATA_HOME")
		.expect("XDG_DATA_HOME environment variable not set, please set it");
	root_path.push_str(ROOT_PATH);
	let root_path = Path::new(&root_path);

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
				return list(&root_path);
			}

			if &arg[0..1] != "-" {
				break cmdline.collect()
			}
		} else { break cmdline.collect() };
		// NOTE: this ^ else breaks null
	};

	let mut real_paths: Vec<Option<PathBuf>> = Vec::with_capacity(argc);
	// None => files doesn't exist yet
	let mut paths: Vec<&Path> = Vec::with_capacity(argc);
	let mut existing: Vec<bool> = Vec::with_capacity(argc);
	// store if flag has symlink already under &root
	let mut sizes: Vec<usize> = Vec::with_capacity(argc);
	let mut hashes: Vec<u64> = Vec::with_capacity(argc);

	let mut buffer: Vec<u8> = Vec::new();
	for i in 0..file_names.len() {
		let name = &file_names[i];
		println!("Found file name: {}", name);
		existing.push(false);
		real_paths.push(None);
		buffer.clear();

		// get path of file
		let p = Path::new(name);
		paths.push(p); // slices get copied
		if !p.exists() {
			println!("File doesn't exist yet");
			continue;
		}
		let p = p.canonicalize()?;

		// check if file already exists
		let exists = check_subdir(&root_path, &p)?;
		existing[i] = exists;
		if exists {
			println!("{} already exists under {}",
				p.display(),
				root_path.display()
			);
			continue;
		}

		// get size of file and it's hash
		let mut file = File::open(&p)?;
		sizes.push(
			file.read_to_end(&mut buffer)?
		);
		println!("{} size: {}", name, sizes.last().unwrap());

		hashes.push(hash(&buffer));
		println!("{} hash: {}", name, hashes.last().unwrap());

		real_paths[i] = Some(p);
		println!("full path: {}", real_paths.last()
			.unwrap()
			.as_ref()
			.unwrap()
			.display()
		);
	}

	assert_eq!(file_names.len(), real_paths.len());
	assert_eq!(file_names.len(), existing.len());

	// start vim
	Command::new("doas")
		.arg(editor)
		.args(&file_names)
		.status()?;

	// check the hashes and everything again
	let mut sizes = sizes.into_iter();
	let mut hashes = hashes.into_iter();
	println!("");
	for i in 0..file_names.len() {
		let name = &file_names[i];

		// check if it exists
		if existing[i] {
			println!("{} already has a symlink", name);
			continue;
		}

		// check if a new file has been created
		if let None = real_paths[i] {
			if paths[i].exists() {
				println!("Creating symlink for new file {}", paths[i].display());
				add(root_path, &paths[i].canonicalize()?)?;
			} else {
				println!("file {} not created", paths[i].display());
			}
			continue;
		}

		// The file exists
		buffer.clear();
		let real_path = real_paths[i].take().unwrap();
		// NOTE: take() ^ transfers ownership, does not allocate again

		// get the size and hash
		let mut file = File::open(&real_path)?;
		let size = file.read_to_end(&mut buffer)?;

		// I hope that rust has short circuting
		if size != sizes.next().unwrap()
			||
		hash(&buffer) != hashes.next().unwrap() {
			println!("{} modified, creating symlink", real_path.display());
			add(root_path, &real_path)?;
		} else {
			println!("{} not modified, symlink not created", real_path.display());
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

#[inline]
fn convert_u64(bytes: &[u8]) -> u64 {
	let mut target: u64 = 0;

	let mut i = 0;
	while i < 8 && i < bytes.len() {
		target += (bytes[i] as u64) << (i * 8);
		i += 1;
	}
	target
}

// check if subdir is a subdirectory of path
// assume both paths are canonicalized, will fail if not
fn check_subdir(path: &Path, subdir: &Path) -> Result<bool, io::Error> {
	assert!(subdir.is_absolute());

	let check_path = path.join(
		subdir.strip_prefix("/")
			.expect("file name did not canonicalize")
	);
	Ok(check_path.exists())
}

// add the path under sudovim
fn add(path: &Path, subdir: &Path) -> Result<(), io::Error> {
	assert!(subdir.is_absolute());

	let add_path = path.join(
		subdir.strip_prefix("/")
			.expect("file isn't canonicalized")
	);

	fs::create_dir_all(
		match add_path.parent() {
			Some(p) => p,
			None => return Err(io::Error::new(io::ErrorKind::Other, "Empty string or root dir passed to add()")),
		}
	)?;

	symlink(subdir, add_path)?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn convert_u64_test() {
		let result = convert_u64(&[0x78, 0x56, 0x34, 0x12]);
		assert!(result == 305419896);
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
		println!("{}", check_subdir(&path, Path::new("/etc/portage"))?);
		Ok(())
	}

	#[test]
	fn add_test() -> Result<(), io::Error> {
		println!("ADD TEST");
		let cwd = PathBuf::from(".").canonicalize()
			.unwrap();
		let p = Path::new("/tests/testing/other");

		match add(&cwd, p) {
			Err(e) => match e.kind() {
				io::ErrorKind::AlreadyExists => {},
				_ => return Err(e),
			},
			_ => {},
		};
		println!("ADD TEST END");
		Ok(())
	}
}
