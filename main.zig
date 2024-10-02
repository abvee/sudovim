const std = @import("std");
const posix = std.posix;

const stdout = std.io.getStdOut().writer();

pub fn main() !void {

	const editor = posix.getenvZ("EDITOR") orelse blk: {
		// we don't want to break the program is stdout is closed
		stdout.print("$EDITOR not found, using vim\n", .{}) catch {};
		break :blk "vim";
	};

	const pid = try posix.fork();
	// NOTE: don't use catch |err| here, posix.execvpeZ returns an error set
	// and as of zig 13, you cannot use capture groups with them. `try` is
	// also a form of catch |err|.
	if (pid == 0)
		posix.execvpeZ(editor, &.{null}, &.{null}) catch
			std.debug.print("Something went wrong with calling exec.\n", .{});

	stdout.print("editor pid: {}\n", .{pid}) catch {};
	stdout.print("editor {s}\n", .{editor}) catch {};
	_ = posix.waitpid(pid, 0);
}
