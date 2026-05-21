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
- [ ] Track files
- [ ] Store snapshots
- [ ] Implement commit history
- [ ] Implement checkout functionality

---

## Commands

```bash
ferrit init
ferrit add <file>
ferrit status
ferrit commit
ferrit checkout <commit_id>
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

1. Data and time information
2. snapshot / commits

```bash
.ferrit/
    HEAD
    Index
    commits/
        commit1/
        commit2/
            info.txt
            files/
                file1
                file2
                ..
            
```

HEAD stores the current commit.
