# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.1] - 2026-01-27

### Changed

- **BREAKING CHANGE:** Entire codebase translated to English
- All variable names Portuguese → English
- All function names Portuguese → English  
- All user-facing messages Portuguese → English
- All dynamic titles and error messages now in English
- Achieve full consistency with English documentation
- Updated function names: `extract_priority`, etc.

### Added

- `search <term>` command to search tasks by term
- `display_task()` function for atomic rendering
- `display_lists()` function for list rendering with statistics

### Changed

- Complete refactoring: separation of parsing vs rendering
- Better code reuse without duplication

### Fixed

- Correct numbering in search command

## [Unreleased]

### Planned

- Tags/categories (`#work`, `#home`)
- Edit command
- Due dates
- Sort by date (`--sort date`)
- JSON export
- Unit tests
- Refactoring with structs

## [0.4.0] - 2026-01-23

### Added

- `undone` command to unmark tasks

## [0.3.0] - 2026-01-23

### Added

- `remove` command to delete specific tasks
- Index validation
- Comprehensive error handling

## [0.2.0] - 2026-01-23

### Added

- `done` command to mark tasks as completed
- String manipulation with `.replace()`, `.map()`, `.collect()`

## [0.1.0] - 2026-01-23

### Added

- `add` command to add tasks
- `list` command to list all tasks
- Basic file operations with `OpenOptions`
- Pattern matching for subcommands
- Error handling with `?` operator

[Unreleased]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.9.0...v1.0.0
[0.9.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.2...v0.5.0
[0.4.2]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/joaofelipegalvao/todo-cli/releases/tag/v0.1.0
