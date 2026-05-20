# FERRIT

A Version Control System (VCS) implemented in Rust for tracking changes in project files.

---

## MVP Workflow

1. Initialize a repository
2. Add files for tracking
3. Save snapshots of project state (commits)
4. Restore the project to a previous snapshot

---

## TODO

- [ ] Implement CLI command parser
- [ ] Create repository metadata directory
- [ ] Track files
- [ ] Store snapshots
- [ ] Implement commit history
- [ ] Implement checkout functionality

---

## Commands

```bash
ferrit init
ferrit add <file>
ferrit commit
ferrit checkout <commit_id>