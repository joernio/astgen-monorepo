#!/usr/bin/env node

import { readFileSync } from 'fs';
import { minimatch } from 'minimatch';
import yaml from 'js-yaml';
import { execSync } from 'child_process';

/**
 * Detects which language workflows should be triggered based on changed files.
 * Reads config from .github/monorepo-ci-config.yml and compares against git diff.
 *
 * Outputs JSON array of language names to trigger.
 */

function main() {
  try {
    // Read configuration file
    const configPath = '.github/monorepo-ci-config.yml';
    const configContent = readFileSync(configPath, 'utf8');
    const config = yaml.load(configContent);

    if (!config.languages || !Array.isArray(config.languages)) {
      throw new Error('Config file must contain "languages" array');
    }

    // Get changed files from git
    const baseBranch = process.env.GITHUB_BASE_REF || 'main';
    const changedFilesOutput = execSync(
      `git diff --name-only origin/${baseBranch}...HEAD`,
      { encoding: 'utf8' }
    );
    const changedFiles = changedFilesOutput.trim().split('\n').filter(f => f.length > 0);

    if (changedFiles.length === 0) {
      console.error('No changed files detected');
      console.log('[]');
      return;
    }

    console.error(`Changed files (${changedFiles.length}):`);
    changedFiles.forEach(f => console.error(`  - ${f}`));

    // Determine which workflows to trigger
    const triggeredLanguages = new Set();
    let triggerAll = false;

    for (const file of changedFiles) {
      // Check trigger_none patterns (skip this file)
      if (config.shared?.trigger_none) {
        const matchesNone = config.shared.trigger_none.some(pattern =>
          minimatch(file, pattern, { dot: true })
        );
        if (matchesNone) {
          console.error(`  ${file} matches trigger_none pattern, skipping`);
          continue;
        }
      }

      // Check trigger_all patterns (trigger all languages)
      if (config.shared?.trigger_all) {
        const matchesAll = config.shared.trigger_all.some(pattern =>
          minimatch(file, pattern, { dot: true })
        );
        if (matchesAll) {
          console.error(`  ${file} matches trigger_all pattern`);
          triggerAll = true;
          break;
        }
      }

      // Check language-specific patterns
      for (const language of config.languages) {
        const matchesLanguage = language.paths.some(pattern =>
          minimatch(file, pattern, { dot: true })
        );
        if (matchesLanguage) {
          console.error(`  ${file} matches ${language.name}`);
          triggeredLanguages.add(language.name);
        }
      }
    }

    // Build result
    let result;
    if (triggerAll) {
      console.error('Triggering ALL languages due to shared infrastructure change');
      result = config.languages.map(l => l.name);
    } else {
      result = Array.from(triggeredLanguages);
    }

    console.error(`\nTriggered workflows: ${JSON.stringify(result)}`);
    console.log(JSON.stringify(result));

  } catch (error) {
    console.error('Error detecting changes:', error.message);
    process.exit(1);
  }
}

main();
