## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Bug Fixes

- Test release pipeline
- **ci:** Handle existing tags in release workflow

## [2.3.2] - 2026-02-19

### Refactoring

- Remove unused storage functions and fix dead code warnings

## [2.3.1] - 2026-02-19

### Bug Fixes

- Remove unused PathBuf import in storage module

## [2.3.0] - 2026-02-19

### Features

- **search:** Add status filter to search command

## [2.2.2] - 2026-02-19

### Bug Fixes

- Update markdownlint config and remove mkdocs link from README

## [2.2.1] - 2026-02-19

### Bug Fixes

- Remove unused import and collapse nested if, fix README link

## [2.2.0] - 2026-02-18

### Features

- **edit:** Add --add-tag and --remove-tag with comma support

### Refactoring

- **lib:** Create library crate structure
- **storage:** Extract storage trait with json/memory implementations
- **validation:** Expand validation module with comprehensive checks
- **error:** Add validation error variants
- **commands:** Update to use new validation and storage modules
- **main:** Update binary to use library
- **models:** Add Display impl for Recurrence and fix doc tests

## [2.1.0] - 2026-02-12

### Documentation

- **changelog:** Add v2.1.0 release notes
- **readme:** Update with recurring tasks features
- **guide:** Document recurring tasks
- **advanced:** Add v2.1.0 recurring tasks guide
- Update mkdocs navigation

### Features

- **models:** Add Recurrence enum and task recurrence support
- **commands:** Add recur and norecur commands
- **list:** Add recurrence filters (daily/weekly/monthly/recurring/non-recurring)
- **done:** Auto-create next recurrence when completing recurring task
- **display:** Add recurrence column to table (D/W/M indicators)
- **cli:** Add recur/norecur commands with examples

### Refactoring

- **commands:** Improve feedback messages and validation

## [2.0.0] - 2026-02-10

### Documentation

- **mkdocs:** Document v2.0 modular architecture refactor
- **readme:** V2.0.0 modular architecture refactor
- **changelog:** Add v2.0.0 release notes

### Refactoring

- Modularize architecture and split monolithic main.rs

## [1.9.0] - 2026-02-10

### Documentation

- **mkdocs:** Document TableLayout architecture and layout decisions
- **readme:** Highlight TableLayout-based display architecture
- **changelog:** Add v1.9.0 release notes
- **guide:** Document edit command and confirmation prompts

### Features

- **edit:** Add edit command and interactive confirmation prompts

### Refactoring

- **ui:** Centralize table layout with TableLayout

## [1.8.0] - 2026-02-09

### Documentation

- Document v1.8.0 global data directory feature
- Fix examples and explanations for global data directory
- Document global data directory and info command

### Features

- Migrate task storage to OS configuration directory
- **info:** Add command to display data file location
- **info:** Add command to display data file location and doc comments

## [1.7.0] - 2026-02-07

### Documentation

- Migrate project documentation to MkDocs

### Features

- Professional error handling with anyhow and thiserror

## [1.6.0] - 2026-02-04

### Features

- V1.6.0 - professional CLI with clap

## [1.5.0] - 2026-02-04

### Features

- V1.5.0 - due dates, sorting, and tabular task display

## [1.4.0] - 2026-02-02

### Features

- V1.4.0 - tags system and correct task numbering

## [1.3.0] - 2026-01-30

### Features

- V1.3.0 - JSON serialization with serde

## [1.2.0] - 2026-01-30

### Features

- V1.2.0 - type-safe task architecture with structs and enums

## [1.1.0] - 2026-01-28

### Features

- V1.0.1 - translate entire codebase to English
- V1.1.0 - add --medium priority filter

## [1.0.0] - 2026-01-27

### Features

- V1.0.0 - search command + architectural refactoring

## [0.9.0] - 2026-01-27

### Features

- V0.9.0 - priority sorting with --sort flag

## [0.8.0] - 2026-01-27

### Features

- V0.8.0 - priority system with advanced filters

## [0.7.0] - 2026-01-26

### Documentation

- Update README for v0.6.0 colored interface

### Features

- V0.7.0 - advanced filters with flags

## [0.6.0] - 2026-01-25

### Documentation

- Complete README restructure with comprehensive documentation

### Features

- V0.6.0 - colored interface with progress tracking

## [0.5.0] - 2026-01-24

### Features

- V5 - clear command to remove all tasks

## [0.4.2] - 2026-01-24

### Features

- V4.2 - add state validation for task operations

## [0.4.1] - 2026-01-24

### Bug Fixes

- Display bug in list command showing empty lines

## [0.4.0] - 2026-01-23

### Features

- V4 - undone command for task completion toggle

## [0.3.0] - 2026-01-23

### Features

- V3 - remove command for task deletion

## [0.2.0] - 2026-01-23

### Bug Fixes

- Add input validation for done command

### Documentation

- V1 - basic todo CLI with add/list commands
- V2 - done command with task completion

### Features

- Add v2 - done command with task completion

## [0.1.0] - 2026-01-23

### Features

- V1 - basic todo CLI with add/list commands


