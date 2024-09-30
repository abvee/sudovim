const std = @import("std");

const posix = std.posix;

const stdout = std.io.getStdOut().writer();

pub fn main() !void {
	// @compileLog(@TypeOf(stdout));
	const editor = posix.getenvZ("EDITOR") orelse blk:{
		// we don't want to break the program if stdout is closed
		stdout.print("$EDITOR not found, using vim\n", .{}) catch {};
		break :blk "vim";
	};

	const pid = try posix.fork();
	if (pid == 0) {
		const x: ?[*:0]const u8 = null;
		posix.execvpeZ(editor, @ptrCast(&x), @ptrCast(&x)) catch {};
	}

	stdout.print("pid: {}\n", .{pid}) catch {};
	stdout.print("editor {s}\n", .{editor}) catch {};
	_ = posix.waitpid(pid, 0);
}
