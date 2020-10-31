#!/usr/bin/env node
const { execFileSync } = require("child_process");
const os = require("os");
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
const shfmtVersion = "v3.2.0";
const shfmtDownloadUrl = `https://github.com/mvdan/sh/releases/download/${shfmtVersion}/shfmt_${shfmtVersion}_${platform}_${arch}`;

if (platform !== "windows") {
  console.log(
    "About to download shfmt from",
    shfmtDownloadUrl,
    "to /usr/local/bin"
  );
  execFileSync("curl", ["-L", shfmtDownloadUrl, "-o", "/usr/local/bin/shfmt"]);
  console.log("Done");
} else {
  console.log("Please download shfmt manually from", shfmtDownloadUrl);
}
