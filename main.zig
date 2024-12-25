const std = @import("std");
const process = std.process;
const posix = std.posix;
const assert = std.debug.assert;

const MAX_BYTES = 10000; // TODO: change to size of largest file

pub fn main() !void {
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	// convert std.os.argv to slices
	// Then get it's size and checksum value
	const cwd = std.fs.cwd();
	const files: [][]const u8 = try allocator.alloc([]const u8, std.os.argv.len - 1);
	const sizes: []u64 = try allocator.alloc(u64, std.os.argv.len - 1);
	const checksums: []u32 = try allocator.alloc(u32, std.os.argv.len - 1);
	const new_files: []bool = try allocator.alloc(bool, std.os.argv.len - 1);

	// assign values
	for (std.os.argv[1..], 0..) |argv, i| {
		files[i] = argv[0..strlen(argv)];
		new_files[i] = false;

		const file = cwd.openFile(files[i], .{})
			catch |err| switch (err) {
				error.FileNotFound => {
					new_files[i] = true;
					continue;
				},
				else => return err,
			};
		defer file.close();

		sizes[i] = (try file.stat()).size; // sizes
		checksums[i] = std.hash.Crc32.hash(try file.readToEndAlloc(allocator, MAX_BYTES)); // hashes
	}

	// $EDITOR
	// TODO: set a default editor if one isn't set in the env
	const editor = posix.getenv("EDITOR") orelse unreachable;
	const sudo = blk: { // check if sudo or doas is installed and use them
		std.fs.accessAbsolute("/bin/sudo", .{}) catch {
			std.fs.accessAbsolute("/bin/doas", .{}) catch |e| {
				// if nothing is found, error
				std.debug.print("/bin/sudo and /bin/doas not found, aborting\n", .{});
				return e;
			};
			break :blk "doas";
		};
		break :blk "sudo";
	};
	assert(@TypeOf(sudo) == *const [4:0]u8); // ensure we get something

	// pass argv file list to the child process
	const child_args =
		try allocator.alloc([]const u8, std.os.argv.len + 1);
	// set argument values
	child_args[0] = sudo; // casts pointer to slice implicitly
	child_args[1] = editor;
	for (files, 2..) |f, i| { // pass files to editor
		child_args[i] = f;
	}

	// process spawning
	var child: process.Child = process.Child.init(
		child_args,
		allocator,
	);
	try child.spawn();
	_ = try child.wait();

	// get all real paths and file names
	const paths: [][]const u8 =
		try allocator.alloc([]const u8, std.os.argv.len - 1);
	for (files, paths) |f, *p| {
		p.* = std.fs.realpathAlloc(allocator, f)
			catch &.{0}; // TODO: deal with files that do not exist yet
	}

	// create the paths after the process exits
	// This allows us to check if anything changed at all

	// TODO: check if something changed
	const root = try std.fs.openDirAbsolute(
		try strcat(
			allocator,
			posix.getenv("XDG_DATA_HOME").?, // TODO: handle if this variable doesn't exist
			"/sudovim",
		),
		.{},
	);

	// create the symlinks after checking if the file has changed or not
	for (paths, 0..) |p, i| {
		if (root.access(p[1..], .{})) |value| {
			assert(@TypeOf(value) == void);
			continue; // file already exists
		}
		else |err| switch (err) {
			posix.AccessError.FileNotFound => {},
			else => return err,
		}

		const file = try std.fs.openFileAbsolute(p, .{});
		defer file.close();
		if ((try file.stat()).size == sizes[i])
			continue;
		if (
			std.hash.Crc32.hash(try file.readToEndAlloc(allocator, MAX_BYTES))
			==
			checksums[i]
		)
			continue;

		// create path
		try root.makePath(p[1..file_name_index(p)]);

		// create symlinks
		try root.symLink(p, p[1..], .{});
	}
}

test "argv" {
	std.debug.print("--ARGV--\n", .{});
	for (std.os.argv) |argv|
		std.debug.print("{s}\n", .{argv});
}

test "realpath" {
	std.debug.print("--REALPATH--\n", .{});
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	const path = try std.fs.realpathAlloc(allocator, "/bin/ls");
	std.debug.print("{s}\n", .{path});
}

test "makePath" {
	std.debug.print("--MAKE PATH--\n", .{});
	const tmp = try std.fs.openDirAbsolute("/tmp", .{});
	try tmp.makePath("etc/default");
	// try tmp.makePath("/etc/default"); // this doesn't work
}

inline fn strlen(s: [*:0]const u8) u8 {
	var i: u8 = 0;
	while (s[i] != 0) {
		i += 1;
	}
	return i;
}
test "strlen" {
	std.debug.print("--STRLEN--\n", .{});
	std.debug.print("{}\n", .{strlen(@ptrCast("Hello"))});
}

fn strcat(allocator: std.mem.Allocator, s1: []const u8, s2: []const u8) ![]const u8 {
	var i: u8 = 0;
	const ret: []u8 = try allocator.alloc(u8, s1.len + s2.len);
	for (s1) |s| {
		ret[i] = s;
		i += 1;
	}

	for (s2) |s| {
		ret[i] = s;
		i += 1;
	}
	return ret;
}
test "strcat" {
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	std.debug.print("--STRCAT--\n", .{});
	std.debug.print("{s}\n", .{
		try strcat(allocator, "Hello ", "World"),
	});
}

inline fn file_name_index(s: []const u8) usize {
	var i: usize = s.len - 1;
	while (i >= 0) : (i -= 1)
		if (s[i] == '/') // std.os.path.sep not needed
			break;
	return i + 1;
}
test "file name" {
	std.debug.print("--FILE NAME--\n", .{});
	std.debug.print("{s}\n{s}\n", .{
		"/etc/default/grub"[file_name_index("/etc/default/grub")..],
		"/etc/default/grub"[0..file_name_index("/etc/default/grub")],
	});
}
