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

// Get target triple from environment or argument
function getTargetTriple() {
  // Check Tauri environment variable first
  if (process.env.TAURI_ENV_TARGET_TRIPLE) {
    return process.env.TAURI_ENV_TARGET_TRIPLE;
  }

  // Check command line argument
  if (process.argv[2]) {
    return process.argv[2];
  }

  // Detect current host
  const hostTuple = execSync("rustc --print host-tuple", { encoding: "utf8" }).trim();
  return hostTuple;
}

// Get host tuple for comparison
function getHostTuple() {
  return execSync("rustc --print host-tuple", { encoding: "utf8" }).trim();
}

function main() {
  const targetTriple = getTargetTriple();
  const hostTuple = getHostTuple();

  console.log(`Building clipper-server for target: ${targetTriple}`);

  // Determine if this is a cross-compile
  const isNativeBuild = targetTriple === hostTuple;

  // Build clipper-server
  const cargoArgs = ["build", "--release", "-p", "clipper-server"];
  if (!isNativeBuild) {
    cargoArgs.push("--target", targetTriple);
  }

  console.log(`Running: cargo ${cargoArgs.join(" ")}`);
  execSync(`cargo ${cargoArgs.join(" ")}`, {
    cwd: projectRoot,
    stdio: "inherit",
  });

  // Determine source and destination paths
  const isWindows = targetTriple.includes("windows");
  const binaryName = isWindows ? "clipper-server.exe" : "clipper-server";

  let sourceBinary;
  if (isNativeBuild) {
    sourceBinary = path.join(projectRoot, "target", "release", binaryName);
  } else {
    sourceBinary = path.join(projectRoot, "target", targetTriple, "release", binaryName);
  }

  const destBinaryName = isWindows
    ? `clipper-server-${targetTriple}.exe`
    : `clipper-server-${targetTriple}`;
  const binariesDir = path.join(clipperDir, "src-tauri", "binaries");
  const destBinary = path.join(binariesDir, destBinaryName);

  // Ensure binaries directory exists
  fs.mkdirSync(binariesDir, { recursive: true });

  // Copy binary
  fs.copyFileSync(sourceBinary, destBinary);

  console.log(`Copied clipper-server to: ${destBinary}`);
}

main();
