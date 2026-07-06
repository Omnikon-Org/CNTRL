## Description

Please include a summary of the changes and the related issue. List any dependencies that are required for this change.

Fixes # (issue)

> **Reminder:** This PR must target the `main` branch. PRs targeting other branches will not be reviewed.

## Type of Change
Please delete options that are not relevant.
- [ ] Bug fix (`fix/` branch — non-breaking change that fixes an issue)
- [ ] New feature (`feat/` branch — non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update (`docs/` branch)
- [ ] Chore / tooling (`chore/` branch — no production code change)

## How Has This Been Tested?
Please describe the tests that you ran to verify your changes.
- [ ] Frontend: `npx tsc --noEmit`, `npx eslint . --max-warnings 0`, `npx vitest run`
- [ ] Backend: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all`

## Checklist
- [ ] My branch is named following the convention (`feat/`, `fix/`, `docs/`, `chore/`, `test/`)
- [ ] My branch is up to date with `upstream/main`
- [ ] My commits follow [Conventional Commits](https://www.conventionalcommits.org/) guidelines
- [ ] My code follows the style guidelines of this project
- [ ] My changes generate no new warnings or compiler errors
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] I have updated relevant documentation if needed
