const std = @import("std");
const process = std.process;

pub fn main() !void {
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	// init a process
	var child: process.Child = process.Child.init(
		&.{"/bin/doas", "vim"},
		allocator,
	);

	// spawn process
	try child.spawn();

	// wait for it to finish
	_ = try child.wait();
}

test "zig process spawning" {

	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	// init a process
	var child: process.Child = process.Child.init(
		&.{"bash", "./tests/spawn-test.sh"},
		allocator,
	);
	std.debug.print("{any}\n", .{child});

	// spawn it
	try child.spawn();
	std.debug.print("After spawning\n", .{});

	// Wait for it to finish
	_ = try child.wait();
}
