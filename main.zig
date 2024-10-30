const std = @import("std");
const posix = std.posix;
const stdout = std.io.getStdOut().writer();

pub fn main() !void {

	// allocator
	var gpa = std.heap.GeneralPurposeAllocator(.{}){};
	const allocator = gpa.allocator();
	defer {
		const deinit_status = gpa.deinit();
		if (deinit_status == .leak) unreachable;
	}

	// Get $EDITOR from env
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

	// fork
	const pid = try posix.fork();
	if (pid == 0) {

		// Allocate space for the argument list to execve
		// arg list is: argv[0] + editor + files + null.
		// Thus, arg_list.len = std.os.argv.len + 2 (1 for editor, 1 for null)
		const arg_list: [*:null] ?[*:0]const u8 = @ptrCast(try allocator.alloc(
				?[*:0]const u8,
				std.os.argv.len + 2
			)
		);
		defer allocator.free(arg_list[0..std.os.argv.len + 2]);

		// populate argument list
		arg_list[0] = std.os.argv[0];
		arg_list[1] = @ptrCast(editor);
		// add file list to arg_list
		var i: u8 = 2;
		for (std.os.argv[1..]) |argv| {
			arg_list[i] = argv;
			i += 1;
		}
		arg_list[i] = null;

		const err = posix.execveZ("/bin/sudo", arg_list, &.{null});

		// if we get here, we have a problem
		std.debug.print("Something went wrong with calling exec.\n", .{});
		return err;
	}

	// get sizes of all the files
	const files: []std.fs.File = try allocator.alloc(std.fs.File, std.os.argv.len - 1);
	defer allocator.free(files);
	const sizes: []u64 = try allocator.alloc(u64, std.os.argv.len - 1);
	defer allocator.free(sizes);

	const cwd = std.fs.cwd();
	for (std.os.argv[1..], 0..) |argv, i| {
		files[i] = try cwd.openFile(@ptrCast(argv[0..strlen(argv)]), .{});
		sizes[i] = (try files[i].stat()).size;
		stdout.print("{}\n", .{sizes[i]});
	}

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

fn strlen(s: [*:0]u8) u8 {
	var i: u8 = 0;
	while (s[i] != 0) {
		i += 1;
	}
	return i;
}
