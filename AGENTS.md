# Global Workspace Directives & Agent Rules

## Git & Version Control Directives
- **MANDATORY ATOMIC COMMITS**: Always perform discrete, atomic git commits! Every discrete feature, bugfix, refactor, test addition, or configuration update must be staged and committed immediately upon completion and verification.
- **Conventional Commit Messages**: Use standard conventional commit format (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`, `chore:`, `perf:`).
- **Zero Uncommitted Stray State**: Never leave multiple unrelated changes bundled into a single monolithic commit. Keep commit histories clean, granular, and reversible.

## Engineering & Testing Directives
- **Test-Driven Development (TDD)**: Write failing unit/integration tests before writing implementation code.
- **Full-Spectrum Verification**: Verify all changes with test suites, linters (`cargo clippy -- -D warnings`), and formatters (`cargo fmt --check`) before considering any task complete.
- **UI & View Layer Testing**: Never assume visual or formatting code works without dedicated tests covering degraded and edge-case states.
