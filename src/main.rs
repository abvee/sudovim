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
use xxhash::XXhash64;

const ROOT_PATH: &str = "/sudovim";

#[derive(Debug)]
enum State {
	Existing, // already has a symlink
	New, // new file, dne yet
	Process, // file has to be processed
}

// all the relevant info on the file
struct FileInfo {
	state: State,
	path: PathBuf,
	size: usize,
	hash: u64,
}
impl FileInfo {
	fn new(state: State, path: PathBuf) -> FileInfo {
		FileInfo {
			state,
			path,
			size: 0,
			hash: 0,
		}
	}
}

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

	let mut infos: Vec<FileInfo> = Vec::with_capacity(argc);

	let mut buffer: Vec<u8> = Vec::new();
	for i in 0..file_names.len() {
		let name = &file_names[i];
		println!("Found file name: {}", name);

		buffer.clear();

		// new info struct
		infos.push(FileInfo::new(
			State::Process,
			PathBuf::from(name)
		));
		let info: &mut FileInfo = infos.last_mut().unwrap();

		// check if new file
		if !info.path.exists() {
			println!("File doesn't exist yet");
			info.state = State::New;
			continue;
		}
		info.path = info.path.canonicalize()?;

		// check if path already exists
		if check_subdir(&root_path, &info.path)? {
			info.state = State::Existing;
			println!("{} already exists under {}",
				info.path.display(),
				root_path.display()
			);
			continue;
		}

		// get size of file and it's hash
		let mut file = File::open(&info.path)?;
		info.size = file.read_to_end(&mut buffer)?;
		println!("{} size: {}", name, info.size);

		info.hash = buffer.hash();
		println!("{} hash: {}", name, info.hash);

		println!("full path: {}", info.path.display());
	}

	assert_eq!(file_names.len(), infos.len());

	// start vim
	Command::new("doas")
		.arg(editor)
		.args(&file_names)
		.status()?;

	// check the hashes and everything again
	println!("");
	for info in infos {
		match info.state {
			State::Existing => {
				println!("{} already has a symlink", info.path.display())
			},
			State::New => {
				if info.path.exists() {
					println!("Creating symlink for new file {}", info.path.display());
					add(root_path, &info.path.canonicalize()?)?;
				} else {
					println!("file {} not created", info.path.display());
				}
			},
			State::Process => {
				buffer.clear();
				// get the size and hash
				let mut file = File::open(&info.path)?;
				let size = file.read_to_end(&mut buffer)?;

				// I hope that rust has short circuting
				if size != info.size
					||
				buffer.hash() != info.hash {
					println!("{} modified, creating symlink", info.path.display());
					add(root_path, &info.path)?;
				} else {
					println!("{} not modified, symlink not created", info.path.display());
				}
			}
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
