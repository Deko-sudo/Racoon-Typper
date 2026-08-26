#!/usr/bin/env node
// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 Racoon Typper Contributors

import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const repositoryRoot = resolve(scriptDirectory, "..");
const licensesDirectory = join(repositoryRoot, "licenses");
const assetInventoryPath = join(licensesDirectory, "assets.json");
const assetProvenancePath = join(licensesDirectory, "ASSET_PROVENANCE.md");
const dependencyInventoryPath = join(licensesDirectory, "dependencies.json");
const noticesPath = join(repositoryRoot, "THIRD_PARTY_NOTICES.md");
const sbomPath = join(licensesDirectory, "sbom.cdx.json");
const projectLicense = "Apache-2.0";
const generatedContentProvenance = "Generated specifically for Racoon Typper using GLM-5.2 and GPT-5.6 (Codex) during project development; no external repository, website, dataset, book, or application was used";
const approvedThirdPartyLicenseExceptions = new Map([
  ["r-efi", "MIT OR Apache-2.0 OR LGPL-2.1-or-later"],
]);

function read(path) {
  return readFileSync(join(repositoryRoot, path), "utf8");
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function relativePath(path) {
  return relative(repositoryRoot, path).split("\\").join("/");
}

function walk(directory) {
  if (!existsSync(directory)) return [];
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...walk(path));
    else if (entry.isFile()) files.push(path);
  }
  return files.sort();
}

function sha256(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function assetProvenance(path) {
  const relative = relativePath(path);
  if (relative === "assets/branding/racoon-typper-icon.png") {
    return {
      asset_name: "Racoon Typper source icon",
      owner: "Racoon Typper Contributors",
      source: "Approved source icon supplied by the Racoon Typper project owner",
      license: projectLicense,
      provenance_status: "original-project-asset",
      modified: false,
      generation: null,
      notes: "Canonical 1024x1024 branding source; platform icon assets are generated from this file.",
    };
  }
  if (relative.startsWith("crates/app/icons/")) {
    return {
      asset_name: `Racoon Typper generated icon (${relative.split("/").at(-1)})`,
      owner: "Racoon Typper Contributors",
      source: "assets/branding/racoon-typper-icon.png",
      license: projectLicense,
      provenance_status: "generated-from-original-project-asset",
      modified: true,
      generation: "Tauri 2 CLI icon generator from the approved PNG source",
      notes: "Generated platform icon asset; do not edit independently.",
    };
  }
  if (relative.startsWith("resources/themes/")) {
    return {
      asset_name: relative.split("/")[2],
      owner: "Racoon Typper Contributors",
      source: "Original theme authored for Racoon Typper in this repository",
      license: projectLicense,
      provenance_status: "original-project-asset",
      modified: false,
      generation: null,
      notes: "Structured theme metadata and CSS are shipped together.",
    };
  }
  if (relative.startsWith("resources/quotes/")) {
    return {
      asset_name: relative.split("/").at(-1),
      owner: "Racoon Typper Contributors",
      source: generatedContentProvenance,
      license: projectLicense,
      provenance_status: "original-project-generated-content",
      modified: false,
      generation: "GLM-5.2 and GPT-5.6 (Codex) during project development",
      notes: "No external quotation or attribution source is used.",
    };
  }
  if (relative.startsWith("resources/words/")) {
    return {
      asset_name: relative.split("/").at(-1),
      owner: "Racoon Typper Contributors",
      source: generatedContentProvenance,
      license: projectLicense,
      provenance_status: "original-project-generated-content",
      modified: false,
      generation: "GLM-5.2 and GPT-5.6 (Codex) during project development",
      notes: "Word entries are project content and are covered by the project license.",
    };
  }
  if (relative.startsWith("resources/courses/")) {
    return {
      asset_name: relative.split("/").at(-1),
      owner: "Racoon Typper Contributors",
      source: generatedContentProvenance,
      license: projectLicense,
      provenance_status: "original-project-generated-content",
      modified: false,
      generation: "GLM-5.2 and GPT-5.6 (Codex) during project development",
      notes: "Lesson text and structure are project content and are covered by the project license.",
    };
  }
  if (relative.startsWith("resources/fonts/")) {
    return {
      asset_name: relative.split("/").at(-1),
      owner: "DejaVu Fonts project; Bitstream, Inc. (original Vera glyphs)",
      source: "DejaVu Fonts 2.37 (https://dejavu-fonts.github.io/)",
      license: "DejaVu Font License (Bitstream Vera + public domain DejaVu changes)",
      provenance_status: "third-party-font",
      modified: false,
      generation: null,
      notes: "Embedded at build time (include_bytes!) for the PNG result share card; full license text shipped in resources/fonts/DejaVu-LICENSE.",
    };
  }
  throw new Error(`No provenance rule for asset: ${relative}`);
}

function generateAssetInventory() {
  const roots = [
    join(repositoryRoot, "assets/branding"),
    join(repositoryRoot, "resources"),
    join(repositoryRoot, "crates/app/icons"),
  ];
  const files = roots.flatMap(walk).sort();
  const entries = files.map((path) => ({
    path: relativePath(path),
    checksum_sha256: sha256(path),
    ...assetProvenance(path),
  }));
  const unlicensedThirdParty = entries.some(
    (entry) => entry.license !== projectLicense && entry.provenance_status !== "third-party-font",
  );
  if (unlicensedThirdParty) {
    throw new Error("Every distributed project-owned asset must be Apache-2.0");
  }
  return {
    schema_version: 1,
    project_license: projectLicense,
    generated_by: "scripts/license-policy.mjs",
    entries,
  };
}

function markdownCell(value) {
  return String(value ?? "").replaceAll("|", "\\|").replaceAll("\n", " ");
}

function generateAssetProvenance(assets) {
  const lines = [
    "# Asset Provenance Inventory",
    "",
    "This generated inventory covers project-owned resources and application icons distributed by Racoon Typper. Project-owned assets are Apache-2.0; embedded third-party fonts (resources/fonts/) retain their original license and are marked third-party-font. The machine-readable source, including SHA-256 checksums and all provenance fields, is `licenses/assets.json`.",
    "",
    "Word, course, and quote packs are project-owned generated content. They were generated specifically for Racoon Typper using GLM-5.2 and GPT-5.6 (Codex) during project development; no external repository, website, dataset, book, or application was used.",
    "",
    "| Path | Asset | Owner | Source | Status | License | SHA-256 | Generation / modification notes |",
    "|---|---|---|---|---|---|---|---|",
    ...assets.entries.map((entry) => {
      const row = [
        entry.path,
        entry.asset_name,
        entry.owner,
        entry.source,
        entry.provenance_status,
        entry.license,
        entry.checksum_sha256,
        [entry.generation, entry.modified ? "modified/generated" : "unmodified", entry.notes].filter(Boolean).join("; "),
      ].map(markdownCell).join(" | ");
      return `| ${row} |`;
    }),
    "",
  ];
  return `${lines.join("\n")}\n`;
}

function packageLicense(packageJson, lockEntry) {
  if (typeof packageJson.license === "string") return packageJson.license;
  if (Array.isArray(packageJson.licenses)) {
    return packageJson.licenses.map((value) => value.type || value).join(" OR ");
  }
  if (typeof lockEntry?.license === "string") return lockEntry.license;
  return null;
}

function generateNpmInventory(lock) {
  const packages = [];
  for (const [location, lockEntry] of Object.entries(lock.packages || {})) {
    if (!location.startsWith("node_modules/")) continue;
    const packagePath = join(repositoryRoot, "frontend", location, "package.json");
    let packageJson = {};
    if (existsSync(packagePath)) packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
    const license = packageLicense(packageJson, lockEntry);
    packages.push({
      name: packageJson.name || location.slice("node_modules/".length),
      version: packageJson.version || lockEntry.version,
      license: license || "UNKNOWN",
      source: lockEntry.resolved || null,
      integrity: lockEntry.integrity || null,
      path: location,
    });
  }
  packages.sort((left, right) => `${left.name}@${left.version}`.localeCompare(`${right.name}@${right.version}`));
  return packages;
}

function generateRustInventory() {
  const output = execFileSync("cargo", ["metadata", "--format-version", "1", "--locked"], {
    cwd: repositoryRoot,
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  });
  const metadata = JSON.parse(output);
  const packages = metadata.packages
    .filter((packageJson) => packageJson.source)
    .map((packageJson) => ({
      name: packageJson.name,
      version: packageJson.version,
      license: packageJson.license || "UNKNOWN",
      source: packageJson.source,
      repository: packageJson.repository || null,
      authors: packageJson.authors || [],
    }))
    .sort((left, right) => `${left.name}@${left.version}`.localeCompare(`${right.name}@${right.version}`));
  return packages;
}

function validateThirdPartyLicenses(packages, ecosystem) {
  const failures = [];
  for (const packageJson of packages) {
    if (packageJson.license === "UNKNOWN") {
      failures.push(`${ecosystem} ${packageJson.name}@${packageJson.version} has an unknown license`);
    }
    if (/GPL|LGPL|AGPL/i.test(packageJson.license)) {
      const approved = ecosystem === "rust"
        && approvedThirdPartyLicenseExceptions.get(packageJson.name) === packageJson.license;
      if (!approved) {
        failures.push(`${ecosystem} ${packageJson.name}@${packageJson.version} has an unapproved copyleft expression: ${packageJson.license}`);
      }
    }
  }
  return failures;
}

function validateProjectMetadata() {
  const failures = [];
  const cargoManifestPaths = ["Cargo.toml", ...walk(join(repositoryRoot, "crates")).filter((path) => path.endsWith("Cargo.toml")).map(relativePath)];
  for (const path of cargoManifestPaths) {
    const content = read(path);
    if (path === "Cargo.toml" && !/license\s*=\s*"Apache-2\.0"/.test(content)) failures.push(`${path} lacks Apache-2.0 workspace license`);
    if (path !== "Cargo.toml" && !/license\.workspace\s*=\s*true/.test(content)) failures.push(`${path} does not inherit the Apache-2.0 workspace license`);
  }
  const frontend = JSON.parse(read("frontend/package.json"));
  if (frontend.license !== projectLicense) failures.push("frontend/package.json must declare Apache-2.0");
  const lockRoot = JSON.parse(read("frontend/package-lock.json")).packages?.[""];
  if (lockRoot?.license !== projectLicense) failures.push("frontend/package-lock.json root must declare Apache-2.0");
  if (!/license=\('Apache-2\.0'\)/.test(read("PKGBUILD"))) failures.push("PKGBUILD must declare Apache-2.0");
  const tauri = JSON.parse(read("crates/app/tauri.conf.json"));
  if (tauri.bundle?.license !== projectLicense) failures.push("Tauri bundle metadata must declare Apache-2.0");
  const requiredBundleNotices = ["../../LICENSE", "../../THIRD_PARTY_NOTICES.md"];
  for (const resource of requiredBundleNotices) {
    if (!tauri.bundle?.resources?.includes(resource)) failures.push(`Tauri bundle resources must include ${resource}`);
  }
  if (!read("PKGBUILD").includes("THIRD_PARTY_NOTICES.md")) failures.push("PKGBUILD must install third-party notices");
  if (!read("com.racoon.typper.json").includes("/app/share/licenses/com.racoon.typper/LICENSE")) failures.push("Flatpak manifest must install the Apache-2.0 license");
  if (!read("com.racoon.typper.json").includes("THIRD_PARTY_NOTICES.md")) failures.push("Flatpak manifest must install third-party notices");
  if (!read("build-appimage.sh").includes("THIRD_PARTY_NOTICES.md")) failures.push("AppImage script must include third-party notices");

  const scanRoots = ["crates", "frontend/src", "scripts", "resources", ".github"];
  const excluded = new Set([
    "Cargo.lock",
    "frontend/package-lock.json",
    // This policy source necessarily contains the names of disallowed license families.
    "scripts/license-policy.mjs",
  ]);
  const forbidden = /(?:MIT License|GNU General Public License|\b(?:GPL|LGPL|AGPL)(?:[- ]\d(?:\.\d)?)?\b)/i;
  for (const root of scanRoots) {
    for (const path of walk(join(repositoryRoot, root))) {
      const relative = relativePath(path);
      if (excluded.has(relative) || relative.startsWith("crates/app/gen/")) continue;
      const content = readFileSync(path);
      if (content.includes(0)) continue;
      if (forbidden.test(content.toString("utf8"))) failures.push(`${relative} contains an obsolete project license reference`);
    }
  }
  return failures;
}

function makeDependencyInventory() {
  const lock = JSON.parse(read("frontend/package-lock.json"));
  const rust = generateRustInventory();
  const npm = generateNpmInventory(lock);
  return {
    schema_version: 1,
    project_license: projectLicense,
    generated_by: "scripts/license-policy.mjs",
    policy_exceptions: [
      {
        ecosystem: "rust",
        package: "r-efi",
        expression: approvedThirdPartyLicenseExceptions.get("r-efi"),
        rationale: "The dependency offers MIT and Apache-2.0 alternatives; the LGPL alternative is not selected as project content.",
      },
    ],
    rust,
    npm,
  };
}

function licenseCounts(packages) {
  const counts = new Map();
  for (const packageJson of packages) counts.set(packageJson.license, (counts.get(packageJson.license) || 0) + 1);
  return [...counts.entries()].sort(([left], [right]) => left.localeCompare(right));
}

function packageNoticeRows(packages) {
  return packages.map((packageJson) => {
    const source = packageJson.repository || packageJson.source || "package distribution";
    const row = [
      `${packageJson.name}@${packageJson.version}`,
      packageJson.license,
      source,
      packageJson.authors?.join(", ") || "",
    ].map(markdownCell).join(" | ");
    return `| ${row} |`;
  });
}

function generateNotices(dependencies) {
  const lines = [
    "# Third-Party Notices",
    "",
    "This file inventories third-party packages distributed or embedded by Racoon Typper. Racoon Typper project-owned code, resources, and metadata are Apache-2.0; these dependencies retain their original licenses and are not relicensed.",
    "",
    "The authoritative machine-readable report is `licenses/dependencies.json`. License expressions and source URLs are captured from the locked Rust metadata and npm installation. Each package's original notice/license remains authoritative at its source distribution.",
    "",
    "## Policy",
    "",
    "- Unknown licenses fail the policy check.",
    "- GPL/LGPL/AGPL expressions fail unless an explicit, reviewed permissive-choice exception is recorded.",
    "- `r-efi` is the only current exception: its expression offers MIT and Apache-2.0 alternatives; the LGPL alternative is not selected.",
    "",
    "## Rust dependency license summary",
    "",
    "| License expression | Packages |",
    "|---|---:|",
    ...licenseCounts(dependencies.rust).map(([license, count]) => `| ${license} | ${count} |`),
    "",
    `Total Rust packages: ${dependencies.rust.length}`,
    "",
    "## npm dependency license summary",
    "",
    "| License expression | Packages |",
    "|---|---:|",
    ...licenseCounts(dependencies.npm).map(([license, count]) => `| ${license} | ${count} |`),
    "",
    `Total npm packages: ${dependencies.npm.length}`,
    "",
    "## Human-readable package inventory",
    "",
    "The following tables identify every locked package, its original license expression, and its source reference. Rust package authors are included when declared by Cargo metadata. The machine-readable inventory additionally records npm integrity hashes and all available metadata.",
    "",
    "### Rust packages",
    "",
    "| Package | License expression | Source | Declared authors |",
    "|---|---|---|---|",
    ...packageNoticeRows(dependencies.rust),
    "",
    "### npm packages",
    "",
    "| Package | License expression | Source | Declared authors |",
    "|---|---|---|---|",
    ...packageNoticeRows(dependencies.npm),
    "",
    "The complete machine-readable inventory is stored in `licenses/dependencies.json`.",
    "",
    "### Reviewed exception",
    "",
    "- `r-efi` — `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; the project policy selects the permissive MIT/Apache-2.0 alternatives and does not distribute the LGPL alternative as project content.",
    "",
  ];
  return `${lines.join("\n")}\n`;
}

function packageUrl(name, version, ecosystem) {
  if (ecosystem === "npm") return `pkg:npm/${encodeURIComponent(name).replaceAll("%2F", "/")}@${version}`;
  return `pkg:cargo/${name}@${version}`;
}

function sbomLicense(license) {
  // Cargo metadata contains a few legacy slash-separated dual-license values.
  // Preserve those raw expressions as CycloneDX license names rather than
  // silently rewriting their legal meaning into a different SPDX expression.
  if (license.includes("/")) return { license: { name: license } };
  return { expression: license };
}

function generateSbom(dependencies) {
  const components = [];
  for (const packageJson of dependencies.rust) {
    components.push({
      type: "library",
      name: packageJson.name,
      version: packageJson.version,
      purl: packageUrl(packageJson.name, packageJson.version, "rust"),
      licenses: [sbomLicense(packageJson.license)],
      externalReferences: packageJson.repository ? [{ type: "vcs", url: packageJson.repository }] : undefined,
    });
  }
  for (const packageJson of dependencies.npm) {
    components.push({
      type: "library",
      name: packageJson.name,
      version: packageJson.version,
      purl: packageUrl(packageJson.name, packageJson.version, "npm"),
      hashes: packageJson.integrity ? [{ alg: "SHA-512", content: packageJson.integrity.replace(/^sha512-/, "") }] : undefined,
      licenses: [sbomLicense(packageJson.license)],
      externalReferences: packageJson.source ? [{ type: "distribution", url: packageJson.source }] : undefined,
    });
  }
  return {
    bomFormat: "CycloneDX",
    specVersion: "1.5",
    serialNumber: "urn:uuid:00000000-0000-4000-8000-000000000001",
    version: 1,
    metadata: {
      tools: [{ vendor: "Racoon Typper Contributors", name: "scripts/license-policy.mjs" }],
      component: { type: "application", name: "racoon-typper", version: JSON.parse(read("frontend/package.json")).version, licenses: [{ license: { id: projectLicense } }] },
    },
    components,
  };
}

function assertGenerated(path, value) {
  if (!existsSync(path)) throw new Error(`Missing generated inventory: ${relativePath(path)}`);
  const actual = readFileSync(path, "utf8");
  const expected = JSON.stringify(value, null, 2) + "\n";
  if (actual !== expected) throw new Error(`Generated inventory is stale: ${relativePath(path)}`);
}

function main() {
  const write = process.argv.includes("--write");
  mkdirSync(licensesDirectory, { recursive: true });
  const projectFailures = validateProjectMetadata();
  const assets = generateAssetInventory();
  const assetProvenance = generateAssetProvenance(assets);
  const dependencies = makeDependencyInventory();
  const dependencyFailures = [
    ...validateThirdPartyLicenses(dependencies.rust, "rust"),
    ...validateThirdPartyLicenses(dependencies.npm, "npm"),
  ];
  const notices = generateNotices(dependencies);
  const sbom = generateSbom(dependencies);

  if (write) {
    writeJson(assetInventoryPath, assets);
    writeFileSync(assetProvenancePath, assetProvenance);
    writeJson(dependencyInventoryPath, dependencies);
    writeJson(sbomPath, sbom);
    writeFileSync(noticesPath, notices);
  } else {
    assertGenerated(assetInventoryPath, assets);
    if (!existsSync(assetProvenancePath) || readFileSync(assetProvenancePath, "utf8") !== assetProvenance) throw new Error("ASSET_PROVENANCE.md is stale");
    assertGenerated(dependencyInventoryPath, dependencies);
    assertGenerated(sbomPath, sbom);
    if (!existsSync(noticesPath) || readFileSync(noticesPath, "utf8") !== notices) throw new Error("THIRD_PARTY_NOTICES.md is stale");
  }

  const failures = [...projectFailures, ...dependencyFailures];
  if (failures.length > 0) {
    console.error(failures.join("\n"));
    process.exit(1);
  }
  console.log(`License policy passed: ${assets.entries.length} assets, ${dependencies.rust.length} Rust packages, ${dependencies.npm.length} npm packages.`);
}

main();
