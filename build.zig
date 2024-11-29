const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{}); // same as b.host
    const optimize = b.standardOptimizeOption(.{}); // Debug relsease

    const exe = b.addExecutable(.{
        .name = "sudovim",
        .root_source_file = b.path("main.zig"),
        .target = target,
        .optimize = optimize,
    });
    b.installArtifact(exe);

	// run
    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);
}
