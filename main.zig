const std = @import("std");
const process = std.process;

pub fn main() !void {
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	// pass argv to the child process
	const child_args =
		try allocator.alloc([]const u8, std.os.argv.len + 1);
	// set values
	child_args[0] = "doas";
	child_args[1] = "vim";
	for (std.os.argv[1..], 2..) |argv, i| { // pass files to vim
		child_args[i] = argv[0..strlen(argv)];
	}

	// init a process
	var child: process.Child = process.Child.init(
		child_args,
		allocator,
	);

	// spawn process
	try child.spawn();

	// wait for it to finish
	_ = try child.wait();
}

test "argv" {
	for (std.os.argv) |argv|
		std.debug.print("{s}", .{argv});
}

inline fn strlen(s: [*:0]const u8) u8 {
	var i: u8 = 0;
	while (s[i] != 0) {
		i += 1;
	}
	return i;
}
test "strlen" {
	std.debug.print("{}\n", .{strlen(@ptrCast("Hello"))});
}
