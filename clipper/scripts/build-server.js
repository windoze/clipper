#!/usr/bin/env node

import { execSync } from "child_process";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

// Get directory paths (ES module equivalent of __dirname)
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const scriptDir = __dirname;
const clipperDir = path.dirname(scriptDir);
const projectRoot = path.dirname(clipperDir);

// Get target triple(s) from environment or argument
// Can be comma-separated for universal builds (e.g., "aarch64-apple-darwin,x86_64-apple-darwin")
function getTargetTriples() {
  let targets = null;

  // Check Tauri environment variable first
  if (process.env.TAURI_ENV_TARGET_TRIPLE) {
    targets = process.env.TAURI_ENV_TARGET_TRIPLE.split(",").map(t => t.trim());
  }
  // Check command line argument
  else if (process.argv[2]) {
    targets = process.argv[2].split(",").map(t => t.trim());
  }

  // Handle universal-apple-darwin by expanding to both architectures
  if (targets && targets.length === 1 && targets[0] === "universal-apple-darwin") {
    return ["aarch64-apple-darwin", "x86_64-apple-darwin"];
  }

  // Return targets if specified, otherwise detect current host
  if (targets) {
    return targets;
  }

  const hostTuple = execSync("rustc --print host-tuple", { encoding: "utf8" }).trim();
  return [hostTuple];
}

// Build for a single target and return the path to the binary
function buildForTarget(targetTriple) {
  console.log(`Building clipper-server for target: ${targetTriple}`);

  const isWindows = targetTriple.includes("windows");
  const binaryName = isWindows ? "clipper-server.exe" : "clipper-server";

  // Binary is always in target/{triple}/release/ when using --target
  const sourceBinary = path.join(projectRoot, "target", targetTriple, "release", binaryName);

  // Skip building if binary already exists
  if (fs.existsSync(sourceBinary)) {
    console.log(`Binary already exists: ${sourceBinary}, skipping build`);
    return sourceBinary;
  }

  // Build clipper-server with explicit target
  const cargoArgs = ["build", "--release", "-p", "clipper-server", "--target", targetTriple];

  console.log(`Running: cargo ${cargoArgs.join(" ")}`);
  execSync(`cargo ${cargoArgs.join(" ")}`, {
    cwd: projectRoot,
    stdio: "inherit",
  });

  return sourceBinary;
}

// Create a universal binary using lipo (macOS only)
function createUniversalBinary(binaries, outputPath) {
  console.log(`Creating universal binary from: ${binaries.join(", ")}`);
  const lipoArgs = ["-create", "-output", outputPath, ...binaries];
  console.log(`Running: lipo ${lipoArgs.join(" ")}`);
  execSync(`lipo ${lipoArgs.join(" ")}`, { stdio: "inherit" });
  console.log(`Created universal binary: ${outputPath}`);
}

function main() {
  const targetTriples = getTargetTriples();
  const binariesDir = path.join(clipperDir, "src-tauri", "binaries");

  // Ensure binaries directory exists
  fs.mkdirSync(binariesDir, { recursive: true });

  // Check if this is a macOS universal build
  const isMacOSUniversal = targetTriples.length === 2 &&
    targetTriples.includes("aarch64-apple-darwin") &&
    targetTriples.includes("x86_64-apple-darwin");

  if (isMacOSUniversal) {
    // Build both architectures separately
    const builtBinaries = [];
    for (const target of targetTriples) {
      const sourceBinary = buildForTarget(target);
      builtBinaries.push(sourceBinary);

      // Also copy individual arch binaries (needed during Tauri's cargo build phase)
      const destBinaryName = `clipper-server-${target}`;
      const destBinary = path.join(binariesDir, destBinaryName);
      fs.copyFileSync(sourceBinary, destBinary);
      console.log(`Copied clipper-server to: ${destBinary}`);
    }

    // Create universal binary with lipo (needed during Tauri's bundle phase)
    const universalBinaryPath = path.join(binariesDir, "clipper-server-universal-apple-darwin");
    createUniversalBinary(builtBinaries, universalBinaryPath);
  } else {
    // Single target build (Linux, Windows, or single macOS arch)
    const targetTriple = targetTriples[0];
    const isWindows = targetTriple.includes("windows");
    const sourceBinary = buildForTarget(targetTriple);

    const destBinaryName = isWindows
      ? `clipper-server-${targetTriple}.exe`
      : `clipper-server-${targetTriple}`;
    const destBinary = path.join(binariesDir, destBinaryName);

    // Copy binary
    fs.copyFileSync(sourceBinary, destBinary);
    console.log(`Copied clipper-server to: ${destBinary}`);
  }
}

main();
