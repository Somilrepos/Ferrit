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

- [X] Implement CLI command parser
- [X] Create repository metadata directory
- [X] Track files
- [X] Store snapshots
- [X] Implement commit history
- [X] Implement checkout functionality
- [X] Implement status command

---

## Commands

```bash
ferrit init
ferrit add <file>
ferrit status
ferrit commit <message>
ferrit checkout <commit_id>
ferrit log
```

## Design Decisions

Assumptions:
1. We assume that ferrit is tracking a flat directory project (i.e no subdirectory)


### How does ferrit works ?

Each file in the repository is either __Tracked__ or __Untracked__.

For tracked files, ferrit compares the working tree with the latest snapshot. A tracked file can be:

1. __UnModified__ - the file matches the latest snapshot
2. __Modified__ - the file existed in the latest snapshot, but its contents changed
3. __Added__ - the file did not exist in the latest snapshot, but exists now
4. __Deleted__ - the file existed in the latest snapshot, but does not exist now

__Added__, __Modified__, and __Deleted__ are specialized forms of a broader idea: the working tree differs from the latest snapshot.

All changed files are further divided into two states - __Staged__ or __Unstaged__ i.e whether we want the changes to be snapshotted in the next commit.

```ferrit add <file>``` will either __Stage__ an existing changed ```<file>``` or track and stage an __Untracked__ ```<file>```.

All files on initialization are __Untracked__. On ```ferrit add```, a new file becomes __Added__ and __Staged__. Once we perform ```ferrit commit```, staged files transition to __UnModified__ and __Unstaged__. Such files need to be __Staged__ again to be included in the next commit.


What metadata are we storing ?

1. Current commit information
2. Staged file information
3. Content-addressed file objects
4. Commit metadata
5. Append-only commit log

```bash
.ferrit/
    HEAD
    index
    log
    objects/
        <file_hash>
        <file_hash>
    commits/
        <commit_id>/
            index
            metadata
```

```HEAD``` stores the current commit id. Before the first commit, it is empty.

```index``` stores the staged files for the next commit. Each line stores:

```bash
<file_path> <file_hash>
```

```objects/``` stores file contents by hash. When ```ferrit add <file>``` runs, ferrit hashes the file contents, copies the file into ```objects/<file_hash>```, and records the staged file in ```index```.

Each commit directory stores:

1. ```index``` - the files included in that commit, with their object hashes
2. ```metadata``` - commit id, parent commit, commit time, and commit message

Commit metadata has this shape:

```bash
commit <commit_id>
parent <parent_commit_id_or_None>
time <YYYY-MM-DD HH:MM:SS UTC>
message <commit_message>
```