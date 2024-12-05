// General file for tests
const std = @import("std");
const process = std.process;
const posix = std.posix;
const print =  std.debug.print;

// test "zig process spawning" {
// 
// 	// allocator
// 	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
// 	defer arena.deinit();
// 	const allocator = arena.allocator();
// 
// 	// init a process
// 	var child: process.Child = process.Child.init(
// 		&.{"bash", "./tests/spawn-test.sh"},
// 		allocator,
// 	);
// 	std.debug.print("{any}\n", .{child});
// 
// 	// spawn it
// 	try child.spawn();
// 	std.debug.print("After spawning\n", .{});
// 
// 	// Wait for it to finish
// 	_ = try child.wait();
// }


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
test "other" {
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();
	const root = try std.fs.openDirAbsolute(
		try strcat(
			allocator,
			posix.getenv("XDG_DATA_HOME").?,
			"/sudovim",
		),
		.{},
	);
	print("{any}\n", .{root});

	try root.makePath("tmp/other");
}

test "checksum" {
	print("--CHECKSUM--\n", .{});
	// allocator
	var arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
	defer arena.deinit();
	const allocator = arena.allocator();

	const file = try std.fs.openFileAbsolute("/tmp/file", .{});
	const bytes = try file.readToEndAlloc(allocator, 100000);
	const w = std.hash.Crc32.hash(bytes);
	// @compileLog(@TypeOf(w));
	print("{}\n", .{w});
}

test "misc" {
	for (0..10) |i| {
		std.debug.print("{}\n", .{i});
		defer print("Something\n", .{});
	}
}
