const std = @import("std");
const posix = std.posix;

const stdout = std.io.getStdOut().writer();

pub fn main() !void {

	var gpa = std.heap.GeneralPurposeAllocator(.{}){};
	const allocator = gpa.allocator();
	defer {
		const deinit_status = gpa.deinit();
		if (deinit_status == .leak) unreachable;
	}

	// NOTE: editor is sentinel terminated.
	const editor = strcat(
		allocator,
		"/bin/",
		posix.getenvZ("EDITOR") orelse blk: {
			// we don't want to break the program is stdout is closed
			stdout.print("$EDITOR not found, using vim\n", .{}) catch {};
			break :blk "vim";
		}
	);
	defer allocator.free(editor);
	const sudo = "/bin/sudo"; // Yes, I'm hardcoding sudo, what are you going to do about it.

	const pid = try posix.fork();
	// NOTE: don't use catch |err| here, posix.execveZ returns an error set
	// and as of zig 13, you cannot use capture groups with them. `try` is
	// also a form of catch |err|.
	if (pid == 0)
		posix.execveZ(sudo, &.{std.os.argv[0], @ptrCast(editor), null}, &.{null}) catch
			std.debug.print("Something went wrong with calling exec.\n", .{});

	stdout.print("editor pid: {}\n", .{pid}) catch {};
	stdout.print("editor {s}\n", .{editor}) catch {};
	_ = posix.waitpid(pid, 0);
}

fn strcat(allocator: std.mem.Allocator, str1: []const u8, str2: []const u8) []const u8 {

	const ret: []u8 = allocator.alloc(u8, str1.len + str2.len + 1) catch unreachable;
	ret[ret.len - 1] = 0; // sentinel value

	var i: u8 = 0;
	for (str1) |s| {
		ret[i] = s;
		i += 1;
	}

	for (str2) |s| {
		ret[i] = s;
		i += 1;
	}
	return ret;
}

test "strcat" {
	var gpa = std.heap.GeneralPurposeAllocator(.{}){};
	const allocator = gpa.allocator();
	defer {
		const deinit_status = gpa.deinit();
		if (deinit_status == .leak) unreachable;
	}

	const e = strcat(allocator, "/bin/", "vim");
	defer allocator.free(e);

	for (e) |d|
		std.debug.print("{d} ", .{d});
	std.debug.print("\n", .{});
}
