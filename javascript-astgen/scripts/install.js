#!/usr/bin/env node
import { spawnSync, execFileSync } from "child_process";
import os from "os";
import fs from "fs";
import path from "path";

let platform = os.platform();
if (platform == "win32") {
  platform = "windows";
}

let arch = os.arch();
switch (arch) {
  case "x32":
    arch = "386";
    break;
  case "x64":
    arch = "amd64";
    break;
}
const shfmtVersion = "v3.4.0";
const shfmtDownloadUrl = `https://github.com/mvdan/sh/releases/download/${shfmtVersion}/shfmt_${shfmtVersion}_${platform}_${arch}`;

if (platform !== "windows") {
  let binDir = "/usr/local/bin";
  try {
    const cp = spawnSync("npm", ["config", "get", "prefix"], {
      encoding: "utf-8",
    });
    if (cp && !cp.error && cp.stdout) {
      const tmpBinDir = cp.stdout.replace("\n", "") + path.sep + "bin";
      try {
        if (fs.lstatSync(tmpBinDir).isDirectory()) {
          binDir = tmpBinDir;
        }
      } catch (err) {}
    }
  } catch (err) {
    console.log(err);
  }
  try {
    if (fs.lstatSync(binDir).isDirectory()) {
      console.log(
        "About to download shfmt from",
        shfmtDownloadUrl,
        "to",
        binDir
      );
      try {
        execFileSync("curl", ["-L", shfmtDownloadUrl, "-o", binDir + "/shfmt"]);
        execFileSync("chmod", ["+x", binDir + "/shfmt"]);
      } catch (err) {
        console.log("Please download shfmt manually from", shfmtDownloadUrl);
      }
      console.log("Done");
    } else {
      console.log(
        `Unable to write to ${binDir}.\nEither execute this command with sudo privileges or manually download shfmt from ${shfmtDownloadUrl}`
      );
    }
  } catch (err) {
    console.log("Please download shfmt manually from", shfmtDownloadUrl);
  }
} else {
  console.log("Please download shfmt manually from", shfmtDownloadUrl);
}
