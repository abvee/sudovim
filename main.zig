const std = @import("std");
const process = std.process;
const posix = std.posix;
const assert = std.debug.assert;

pub fn main() !void {
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	// $EDITOR
	// TODO: set a default editor if one isn't set in the env
	const editor = posix.getenv("EDITOR") orelse unreachable;
	const sudo = blk: { // check if sudo or doas is installed and use them
		std.fs.accessAbsolute("/bin/sudo", .{}) catch {
			std.fs.accessAbsolute("/bin/doas", .{}) catch |e| {
				// if nothing is found, use error
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
	for (std.os.argv[1..], 2..) |argv, i| { // pass files to editor
		child_args[i] = argv[0..strlen(argv)];
	}

	// init a process
	var child: process.Child = process.Child.init(
		child_args,
		allocator,
	);

	// spawn process
	try child.spawn();

	// get all real paths
	const paths: [][]const u8 =
		try allocator.alloc([]const u8, std.os.argv.len - 1);
	for (std.os.argv[1..], paths) |argv, *p| {
		p.* = std.fs.realpathAlloc(allocator, argv[0..strlen(argv)])
			catch &.{0}; // TODO: deal with files that do not exist yet
	}

	// wait for it to finish
	_ = try child.wait();
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
