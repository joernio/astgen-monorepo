#!/usr/bin/env node
const { spawnSync, execFileSync } = require("child_process");
const os = require("os");
const fs = require("fs");
const path = require("path");

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
const shfmtVersion = "v3.2.1";
const shfmtDownloadUrl = `https://github.com/mvdan/sh/releases/download/${shfmtVersion}/shfmt_${shfmtVersion}_${platform}_${arch}`;

if (platform !== "windows") {
  let binDir = "/usr/local/bin";
  try {
    const cp = spawnSync("npm", ["config", "get", "prefix"], {
      encoding: "utf-8",
    });
    if (cp && !cp.error && cp.stdout) {
      tmpBinDir = cp.stdout.replace("\n", "") + path.sep + "bin";
      try {
        if (fs.accessSync(tmpBinDir, fs.constants.W_OK)) {
          binDir = tmpBinDir;
        }
      } catch (err) {}
    }
  } catch (err) {
    console.log(err);
  }
  try {
    if (fs.accessSync(binDir, fs.constants.W_OK)) {
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
