## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.8.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.7.0..v2.8.0) - 2026-02-21

### Features

- Add automatic tag normalization with fuzzy matching - ([4bcbfb2](https://github.com/joaofelipegalvao/todo-cli/commit/4bcbfb2b2f5c72f90865be21d8401f60c96f7ecf))

### Documentation

- *(changelog)* Update for 2.7.0 [skip ci] - ([0b71a1e](https://github.com/joaofelipegalvao/todo-cli/commit/0b71a1eaf71816072012d1f965b41ba614c5b48b))
## [2.7.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.6.1..v2.7.0) - 2026-02-21

### Features

- Add stats command for productivity statistics - ([9d235a9](https://github.com/joaofelipegalvao/todo-cli/commit/9d235a93d7f76bcdca3a5ff34c1937d0e6963199))

### Documentation

- *(changelog)* Update for 2.6.1 [skip ci] - ([92e0dbe](https://github.com/joaofelipegalvao/todo-cli/commit/92e0dbe1b8b1a9d479d63d591e157a6d01bff198))
## [2.6.1](https://github.com/joaofelipegalvao/todo-cli/compare/v2.6.0..v2.6.1) - 2026-02-21

### Refactoring

- Simplify error handling and if-let patterns - ([ff4ad5d](https://github.com/joaofelipegalvao/todo-cli/commit/ff4ad5d84dd03b833a96a32a67185df1f3f3cad6))

### Documentation

- *(changelog)* Update for 2.6.0 [skip ci] - ([0d5096e](https://github.com/joaofelipegalvao/todo-cli/commit/0d5096e0715b34a23d0630e35c5ce773321b83c1))
## [2.6.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.5.0..v2.6.0) - 2026-02-20

### Features

- Add task dependencies system with cycle detection - ([7b1f422](https://github.com/joaofelipegalvao/todo-cli/commit/7b1f4223155b37bf8b0a56b1fb0bdb768db3c651))

### Documentation

- *(changelog)* Update for 2.5.0 [skip ci] - ([49a9ac1](https://github.com/joaofelipegalvao/todo-cli/commit/49a9ac1f1cf070f43ac2580a01b50805b06babd3))
## [2.5.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.4.1..v2.5.0) - 2026-02-20

### Features

- Add project system for organizing tasks - ([7308a0b](https://github.com/joaofelipegalvao/todo-cli/commit/7308a0b5592d3e791fd2dde2efc34ebfd4fb8634))

### Documentation

- *(changelog)* Update for 2.4.1 [skip ci] - ([4881949](https://github.com/joaofelipegalvao/todo-cli/commit/48819496edb4d454984c818f4a8115e5563d2a9b))
## [2.4.1](https://github.com/joaofelipegalvao/todo-cli/compare/v2.4.0..v2.4.1) - 2026-02-20

### Performance

- Optimize date parsing with LazyLock and add past date validation - ([fb6c20a](https://github.com/joaofelipegalvao/todo-cli/commit/fb6c20afc879fc2faba64096766152e5af3484c3))

### Documentation

- *(changelog)* Update for 2.4.0 [skip ci] - ([8efc531](https://github.com/joaofelipegalvao/todo-cli/commit/8efc531eba5c57277044c9ae9329a858c1bd05ab))
## [2.4.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.4..v2.4.0) - 2026-02-20

### Features

- Add natural language date parsing (NLP dates) - ([ab9fa6e](https://github.com/joaofelipegalvao/todo-cli/commit/ab9fa6ef7dac76caed330ad5f9322a475b94660c))

### Bug Fixes

- *(ci)* Remove GitHub API dependency from git-cliff config - ([f64a7f8](https://github.com/joaofelipegalvao/todo-cli/commit/f64a7f8d252019aebc670bcbb28abeda47e6234b))
- *(ci)* Hardcode repo URL in cliff.toml template to fix Tera scope issue - ([df40c3e](https://github.com/joaofelipegalvao/todo-cli/commit/df40c3e88a7aecbd70b42f4a07daaa93be69c9a2))
- Collapse nested if-let and add GitHub token to git-cliff steps - ([ce24bfd](https://github.com/joaofelipegalvao/todo-cli/commit/ce24bfdb0e9bdf674727d7a850afa5aafef7798c))

### Documentation

- *(changelog)* Update for 2.3.4 [skip ci] - ([1d5e57a](https://github.com/joaofelipegalvao/todo-cli/commit/1d5e57aee589e84783d066a7cb57d1c113ffbc93))
## [2.3.4](https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.3..v2.3.4) - 2026-02-19

### Bug Fixes

- *(ci)* Generate CHANGELOG after tag creation so version is resolved correctly - ([50101d4](https://github.com/joaofelipegalvao/todo-cli/commit/50101d490fc2df9eece24877a5b5725cd4bc8698))
## [2.3.3](https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.2..v2.3.3) - 2026-02-19

### Bug Fixes

- *(ci)* Handle existing tags in release workflow - ([bc2b287](https://github.com/joaofelipegalvao/todo-cli/commit/bc2b28780042b20ec95ca40e5c1c3db750766203))
- *(ci)* Add comparison links to CHANGELOG footer - ([dda87f6](https://github.com/joaofelipegalvao/todo-cli/commit/dda87f696496bbdd870449ea395fb5d03a1343b4))
- *(ci)* Fix CHANGELOG footer template for git-cliff - ([8a4a14c](https://github.com/joaofelipegalvao/todo-cli/commit/8a4a14c28187fe091ee9b65dfc68aab2e19e34c8))
- *(ci)* Sync Cargo.toml version with last git tag before bump - ([743b94e](https://github.com/joaofelipegalvao/todo-cli/commit/743b94e5b787d63777a4f83c51928c371ebf7cd0))
- Test release pipeline - ([61b2197](https://github.com/joaofelipegalvao/todo-cli/commit/61b21975ca7c036fe062e785774fde47103ba83a))
## [2.3.2](https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.1..v2.3.2) - 2026-02-19

### Refactoring

- Remove unused storage functions and fix dead code warnings - ([cf16d2b](https://github.com/joaofelipegalvao/todo-cli/commit/cf16d2ba4f60cee8f86437e6ba32cb523a269103))
## [2.3.1](https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.0..v2.3.1) - 2026-02-19

### Bug Fixes

- Remove unused PathBuf import in storage module - ([0a4f37f](https://github.com/joaofelipegalvao/todo-cli/commit/0a4f37f8b0c26ab15002dfbcc29da3882b00921c))
## [2.3.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.2..v2.3.0) - 2026-02-19

### Features

- *(search)* Add status filter to search command - ([8f3deb4](https://github.com/joaofelipegalvao/todo-cli/commit/8f3deb4a69547eaea3d390d3ec956013f983921a))
## [2.2.2](https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.1..v2.2.2) - 2026-02-19

### Bug Fixes

- Update markdownlint config and remove mkdocs link from README - ([a2faad0](https://github.com/joaofelipegalvao/todo-cli/commit/a2faad0e559366831eb24f7e8cef66684e161383))
## [2.2.1](https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.0..v2.2.1) - 2026-02-19

### Bug Fixes

- Remove unused import and collapse nested if, fix README link - ([fd7646f](https://github.com/joaofelipegalvao/todo-cli/commit/fd7646f342eda0d3fa367eab93cec37b34540cb5))
## [2.2.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.1.0..v2.2.0) - 2026-02-18

### Features

- *(edit)* Add --add-tag and --remove-tag with comma support - ([ec151a4](https://github.com/joaofelipegalvao/todo-cli/commit/ec151a4c867af191f3b0362f8909694d92713d5c))

### Refactoring

- *(commands)* Update to use new validation and storage modules - ([120c93f](https://github.com/joaofelipegalvao/todo-cli/commit/120c93f1f9958a74f2763115e21f66dbac876df6))
- *(error)* Add validation error variants - ([2f5032a](https://github.com/joaofelipegalvao/todo-cli/commit/2f5032a9d9bf55988ef57e9b245f984d4c9d76c8))
- *(lib)* Create library crate structure - ([3a8695c](https://github.com/joaofelipegalvao/todo-cli/commit/3a8695cec7b2d88171c42125a5bc0ec896d993a7))
- *(main)* Update binary to use library - ([eb14fcb](https://github.com/joaofelipegalvao/todo-cli/commit/eb14fcbdcb8dfe215e842ce11b95ca806a7a5930))
- *(models)* Add Display impl for Recurrence and fix doc tests - ([c5598fc](https://github.com/joaofelipegalvao/todo-cli/commit/c5598fc06e0344ae17e8ad0c810034a61051a5e1))
- *(storage)* Extract storage trait with json/memory implementations - ([7bda7e3](https://github.com/joaofelipegalvao/todo-cli/commit/7bda7e3c54e13d47f82b16e515ade670290f75a4))
- *(validation)* Expand validation module with comprehensive checks - ([a5f3e6d](https://github.com/joaofelipegalvao/todo-cli/commit/a5f3e6dfadd45e42f32ca516344c9acf39e44db0))
## [2.1.0](https://github.com/joaofelipegalvao/todo-cli/compare/v2.0.0..v2.1.0) - 2026-02-12

### Features

- *(cli)* Add recur/norecur commands with examples - ([c342b83](https://github.com/joaofelipegalvao/todo-cli/commit/c342b83bc22705bddbda9be2530e31d9b2ecf7b6))
- *(commands)* Add recur and norecur commands - ([b3feb30](https://github.com/joaofelipegalvao/todo-cli/commit/b3feb3093112c695a65af5c9ab08e6eb950b197d))
- *(display)* Add recurrence column to table (D/W/M indicators) - ([7247390](https://github.com/joaofelipegalvao/todo-cli/commit/7247390590a51543a1215920b43a938e68d80f3a))
- *(done)* Auto-create next recurrence when completing recurring task - ([1e3ea78](https://github.com/joaofelipegalvao/todo-cli/commit/1e3ea78f7b29fa00e4dcec11c91d9fac7ce0b09e))
- *(list)* Add recurrence filters (daily/weekly/monthly/recurring/non-recurring) - ([7fbf722](https://github.com/joaofelipegalvao/todo-cli/commit/7fbf722ec8e8b25faf4f62012b9b57d6aa054892))
- *(models)* Add Recurrence enum and task recurrence support - ([b7d173a](https://github.com/joaofelipegalvao/todo-cli/commit/b7d173a57b6fc1d41bb095b1cdd74aeba0b63094))

### Refactoring

- *(commands)* Improve feedback messages and validation - ([e232ba6](https://github.com/joaofelipegalvao/todo-cli/commit/e232ba6da976c6a2c700ab68d1dc144ad2a9ed7a))

### Documentation

- *(advanced)* Add v2.1.0 recurring tasks guide - ([f5ce3d1](https://github.com/joaofelipegalvao/todo-cli/commit/f5ce3d118f2a9b1ab765ae4987080d7b50b374c0))
- *(changelog)* Add v2.1.0 release notes - ([b5eb6b6](https://github.com/joaofelipegalvao/todo-cli/commit/b5eb6b6dadfc5241134b5f87f2afa1d6ef95a7a7))
- *(guide)* Document recurring tasks - ([208c38e](https://github.com/joaofelipegalvao/todo-cli/commit/208c38ed00226c7b7fcc2a3552a761576a25264a))
- *(readme)* Update with recurring tasks features - ([c70fe76](https://github.com/joaofelipegalvao/todo-cli/commit/c70fe76dadee2b46e380e371f45bf42daae82d27))
- Update mkdocs navigation - ([d574d78](https://github.com/joaofelipegalvao/todo-cli/commit/d574d787c07ea5f1770c513054b963fbe60963ba))
## [2.0.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.9.0..v2.0.0) - 2026-02-10

### Refactoring

- [**breaking**] Modularize architecture and split monolithic main.rs - ([b5b2d7c](https://github.com/joaofelipegalvao/todo-cli/commit/b5b2d7c51be2a8f7e4e861964018e1fb299c6fe2))

### Documentation

- *(changelog)* Add v2.0.0 release notes - ([a57d855](https://github.com/joaofelipegalvao/todo-cli/commit/a57d85504d2bea73912431e2de2d814a5cf45483))
- *(mkdocs)* Document v2.0 modular architecture refactor - ([7e731d2](https://github.com/joaofelipegalvao/todo-cli/commit/7e731d2196a84084b62d90f6aa5e41daf62aa84b))
- *(readme)* V2.0.0 modular architecture refactor - ([ed37b35](https://github.com/joaofelipegalvao/todo-cli/commit/ed37b351436b5bfca97fa5d3349e51c92eb30b8c))
## [1.9.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.8.0..v1.9.0) - 2026-02-10

### Features

- *(edit)* Add edit command and interactive confirmation prompts - ([d201bda](https://github.com/joaofelipegalvao/todo-cli/commit/d201bdadc47ef8fce3564c829a5fe1dd4beb2c1e))

### Refactoring

- *(ui)* Centralize table layout with TableLayout - ([32a88b6](https://github.com/joaofelipegalvao/todo-cli/commit/32a88b61717f0cc75a7c7a838550b080d49065ca))

### Documentation

- *(changelog)* Add v1.9.0 release notes - ([49ace3c](https://github.com/joaofelipegalvao/todo-cli/commit/49ace3cc95885a88db0d6145f4068479bbedaa31))
- *(guide)* Document edit command and confirmation prompts - ([027bf6c](https://github.com/joaofelipegalvao/todo-cli/commit/027bf6c89732b2050e6b74adf95d12ec127eb44a))
- *(mkdocs)* Document TableLayout architecture and layout decisions - ([131528f](https://github.com/joaofelipegalvao/todo-cli/commit/131528f0ef9cf46cd713e5d3dc6e0c5ba7153f91))
- *(readme)* Highlight TableLayout-based display architecture - ([00d6e75](https://github.com/joaofelipegalvao/todo-cli/commit/00d6e75ed1c44ec2c3db4ced124cdfadcd208e60))
## [1.8.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.7.0..v1.8.0) - 2026-02-09

### Features

- *(info)* Add command to display data file location - ([7565418](https://github.com/joaofelipegalvao/todo-cli/commit/7565418e0eeb2430cea4910296fb9f185eaee1f5))
- *(info)* Add command to display data file location and doc comments - ([71182e2](https://github.com/joaofelipegalvao/todo-cli/commit/71182e2bb1c0626d71f614599d8fda4d5b58a604))
- [**breaking**] Migrate task storage to OS configuration directory - ([d1a0dd0](https://github.com/joaofelipegalvao/todo-cli/commit/d1a0dd0b871ca1042cba4201a88e3f6cb0e407a6))

### Documentation

- Document v1.8.0 global data directory feature - ([a155523](https://github.com/joaofelipegalvao/todo-cli/commit/a155523ba27e0b53d3e3db3965342482cec80464))
- Fix examples and explanations for global data directory - ([b0affd7](https://github.com/joaofelipegalvao/todo-cli/commit/b0affd7780e8e2878fd87e5f3417cdb7c7a58a46))
- Document global data directory and info command - ([70abd90](https://github.com/joaofelipegalvao/todo-cli/commit/70abd909b8e5568a0d32088c8806d9deaacf9324))
## [1.7.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.6.0..v1.7.0) - 2026-02-07

### Features

- [**breaking**] Professional error handling with anyhow and thiserror - ([da4b40d](https://github.com/joaofelipegalvao/todo-cli/commit/da4b40d1217d04cc6f7d99d4543c460233dbf496))

### Documentation

- Migrate project documentation to MkDocs - ([d65332b](https://github.com/joaofelipegalvao/todo-cli/commit/d65332b08d31270f886957c10158b43f931f218a))
## [1.6.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.5.0..v1.6.0) - 2026-02-04

### Features

- [**breaking**] V1.6.0 - professional CLI with clap - ([9ac0ca8](https://github.com/joaofelipegalvao/todo-cli/commit/9ac0ca8d50fffc32aef8f54d67089e1013c66509))
## [1.5.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.4.0..v1.5.0) - 2026-02-04

### Features

- [**breaking**] V1.5.0 - due dates, sorting, and tabular task display - ([5c6da3a](https://github.com/joaofelipegalvao/todo-cli/commit/5c6da3a797eca3040aa1e90b771b0f771e14ae64))
## [1.4.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.3.0..v1.4.0) - 2026-02-02

### Features

- V1.4.0 - tags system and correct task numbering - ([3f0faa5](https://github.com/joaofelipegalvao/todo-cli/commit/3f0faa533e2298c658a670bd67a3f6843677f01d))
## [1.3.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.2.0..v1.3.0) - 2026-01-30

### Features

- V1.3.0 - JSON serialization with serde - ([d8f5ea9](https://github.com/joaofelipegalvao/todo-cli/commit/d8f5ea981707e056b1ef5a1c6c1b1d116587a67f))
## [1.2.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.1.0..v1.2.0) - 2026-01-30

### Features

- V1.2.0 - type-safe task architecture with structs and enums - ([f4e9385](https://github.com/joaofelipegalvao/todo-cli/commit/f4e9385cdfb7451b3c6725bb95944a412d123f98))
## [1.1.0](https://github.com/joaofelipegalvao/todo-cli/compare/v1.0.0..v1.1.0) - 2026-01-28

### Features

- [**breaking**] V1.0.1 - translate entire codebase to English - ([e231654](https://github.com/joaofelipegalvao/todo-cli/commit/e2316540ad83118558fc4ae6bc70815dd848472d))
- V1.1.0 - add --medium priority filter - ([c83dd94](https://github.com/joaofelipegalvao/todo-cli/commit/c83dd949c00f54df0af6e19698d51c0aa7098db2))
## [1.0.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.9.0..v1.0.0) - 2026-01-27

### Features

- V1.0.0 - search command + architectural refactoring - ([8af90a4](https://github.com/joaofelipegalvao/todo-cli/commit/8af90a404cb9503236137e0e124d3205cf3d37bd))
## [0.9.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.8.0..v0.9.0) - 2026-01-27

### Features

- V0.9.0 - priority sorting with --sort flag - ([0ae4962](https://github.com/joaofelipegalvao/todo-cli/commit/0ae49622620d94dffd76e8d167abcae3d371444f))
## [0.8.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.7.0..v0.8.0) - 2026-01-27

### Features

- V0.8.0 - priority system with advanced filters - ([a72e487](https://github.com/joaofelipegalvao/todo-cli/commit/a72e487b958ee99d0b5dce9acc2249a7d6901d6b))
## [0.7.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.6.0..v0.7.0) - 2026-01-26

### Features

- V0.7.0 - advanced filters with flags - ([696511f](https://github.com/joaofelipegalvao/todo-cli/commit/696511f2093e716490228f3295250f88bcead284))

### Documentation

- Update README for v0.6.0 colored interface - ([75295a9](https://github.com/joaofelipegalvao/todo-cli/commit/75295a92d360c03aa95836c412a5ed58a41eb1ef))
## [0.6.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.5.0..v0.6.0) - 2026-01-25

### Features

- V0.6.0 - colored interface with progress tracking - ([475e0c0](https://github.com/joaofelipegalvao/todo-cli/commit/475e0c0eccfcdb7feaf01fde12b53c454367b091))

### Documentation

- Complete README restructure with comprehensive documentation - ([7f5135c](https://github.com/joaofelipegalvao/todo-cli/commit/7f5135c88e5c8f9ece2a80d00f4ac84f06bcab67))
## [0.5.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.2..v0.5.0) - 2026-01-24

### Features

- V5 - clear command to remove all tasks - ([d443e1b](https://github.com/joaofelipegalvao/todo-cli/commit/d443e1b026358bd287ea08f94fe64466046f86b9))
## [0.4.2](https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.1..v0.4.2) - 2026-01-24

### Features

- V4.2 - add state validation for task operations - ([29f1b0a](https://github.com/joaofelipegalvao/todo-cli/commit/29f1b0a2ec6c84f176f6decc61086e553470a705))
## [0.4.1](https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.0..v0.4.1) - 2026-01-24

### Bug Fixes

- Display bug in list command showing empty lines - ([9a84730](https://github.com/joaofelipegalvao/todo-cli/commit/9a84730445b6a8cda7e733307b07fb0af7da27c5))
## [0.4.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.3.0..v0.4.0) - 2026-01-23

### Features

- V4 - undone command for task completion toggle - ([630f2e1](https://github.com/joaofelipegalvao/todo-cli/commit/630f2e1ec8ba118d36e7e89c2e2e3e1157e9416a))
## [0.3.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.2.0..v0.3.0) - 2026-01-23

### Features

- V3 - remove command for task deletion - ([17bbf1a](https://github.com/joaofelipegalvao/todo-cli/commit/17bbf1ac3d4f5568a14f102a534560fda9288e95))
## [0.2.0](https://github.com/joaofelipegalvao/todo-cli/compare/v0.1.0..v0.2.0) - 2026-01-23

### Features

- Add v2 - done command with task completion - ([a562656](https://github.com/joaofelipegalvao/todo-cli/commit/a5626567103c1faa69c96caea1cab27ad6f89b14))

### Bug Fixes

- Add input validation for done command - ([9c09fea](https://github.com/joaofelipegalvao/todo-cli/commit/9c09fead1060c16c70f3ff746000a1304ecab9ed))

### Documentation

- V1 - basic todo CLI with add/list commands - ([f2354c9](https://github.com/joaofelipegalvao/todo-cli/commit/f2354c9d7cfda27dd4068954fd72ab7f44a11c3b))
- V2 - done command with task completion - ([26d6abe](https://github.com/joaofelipegalvao/todo-cli/commit/26d6abe4f1f9db639e1794b58e9b9d509bb6d754))
## [0.1.0] - 2026-01-23

### Features

- V1 - basic todo CLI with add/list commands - ([9580ae2](https://github.com/joaofelipegalvao/todo-cli/commit/9580ae297837c9a6c5d4b18868d2f3abac1b1b9e))
[2.8.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.7.0...v2.8.0
[2.7.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.6.1...v2.7.0
[2.6.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.6.0...v2.6.1
[2.6.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.5.0...v2.6.0
[2.5.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.4.1...v2.5.0
[2.4.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.4.0...v2.4.1
[2.4.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.4...v2.4.0
[2.3.4]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.3...v2.3.4
[2.3.3]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.2...v2.3.3
[2.3.2]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.1...v2.3.2
[2.3.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.3.0...v2.3.1
[2.3.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.2...v2.3.0
[2.2.2]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.1...v2.2.2
[2.2.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.2.0...v2.2.1
[2.2.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.9.0...v2.0.0
[1.9.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.8.0...v1.9.0
[1.8.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.7.0...v1.8.0
[1.7.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.6.0...v1.7.0
[1.6.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.5.0...v1.6.0
[1.5.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.4.0...v1.5.0
[1.4.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.2.0...v1.3.0
[1.2.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v1.0.0...v1.1.0
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

