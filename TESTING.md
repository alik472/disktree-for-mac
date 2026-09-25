# Testing disktree

Three layers, from fastest to most real. All of them must pass before a
change is called done.

## 1. The gate: lint and tests

```sh
make ci
```

`cargo xtask lint` (rustfmt, then clippy with `all` and `pedantic` as errors)
and `cargo xtask test`: the scanner, layout and removal tests in
`disktree-core`, and the window-harness tests in
`crates/disktree-app/src/tests.rs`, which open a real window, draw frames and
press keys. macOS-only safety tests (the Trash, protected paths, the mount
table) run only on a Mac.

## 2. Performance budgets

```sh
make perf                                  # scans $HOME for the real test
DISKTREE_PERF_ROOT=/some/big/dir make perf # or any directory
```

Always a release build. Two tests:

- `crates/disktree-core/tests/perf.rs` builds a synthetic tree of four
  million entries and times each operation that runs on or feeds the UI
  thread: layout at depth 3 and 6 (one frame, 16 ms), *Worth a look*
  (250 ms), the full copy, re-sort and free that a rescan implies, and a
  filter over every name.
- `a_real_home_directory_stays_responsive` in the app tests scans a real
  directory, opens the window over it and walks through every interaction —
  moving, opening, depth, zoom, Size → Files → Age, marking, typing in the
  filter, help — timing each key to the next drawn frame. Budget: 100 ms per
  key. Filter results, which are computed in the background, must arrive
  within 3 s.

Numbers from a MacBook (Apple silicon), 3.5M-file home folder: every key
5–16 ms; Size ↔ Files 10 ms (was 45 s before the fix); typing in the filter
6 ms per key.

## 3. End to end, by hand or by an agent

The harness cannot see what a person sees: the menu bar, the Dock icon,
permission prompts, the Trash. Build and install, then run through the
script below. To have a coding agent do it, give it this prompt:

> You are testing the macOS app disktree end to end. Build and install it
> with `make install-app`, then open `/Applications/disktree.app` and drive
> it with screenshots, clicks and keystrokes. Work only inside a scratch
> directory you create for the removal steps; never mark or delete anything
> outside it.
>
> 1. Launch. The Dock icon is the treemap icon, not a blank square. The home
>    folder scan finishes and the treemap, the selection panel, *Worth a
>    look* and the disk meter all render with no overlapping text.
> 2. Menu bar: the disktree menu has Hide, Hide Others, Show All and Quit.
>    ⌘H hides the window, ⌘Q quits, and closing the window quits the app.
> 3. Keyboard: arrows and hjkl move the selection; Enter opens, Backspace goes
>    up; `[` and `]` change depth; `=` `-` `0` zoom and reset; `t` cycles
>    Size → Files → Age and each switch redraws immediately, with the
>    selection staying on the same entry; `?` shows help and Escape closes
>    it. ⌘C, ⌘X and ⌘R must do nothing; ⌘F opens the filter.
> 4. Mouse: clicking Size / Files / Age switches instantly; hovering shows a
>    tooltip; breadcrumbs navigate; dragging the panel edge resizes it.
> 5. Filter: type a common name like `cache` quickly. Every keystroke shows
>    up at once; the results arrive within a few seconds; Enter applies;
>    Escape clears.
> 6. Removal, inside the scratch directory only (`disktree <scratch-dir>`):
>    mark two files with Space, press `c`, choose Move to Trash, commit. Both
>    appear in the Finder Trash and can be put back. Then mark one and delete
>    permanently: the confirmation appears, and after it the file is gone and
>    its neighbours are not.
> 7. Protection: run `disktree --disk`. `/System/Volumes/Data` is not in
>    the map (your files appear once, under `/Users`). Mark `/Library`,
>    `~/Library` and `~/Library/Keychains` and press `c`: the review screen
>    must block all three with a reason, and nothing is removed.
> 8. Background: start a rescan (`r`), switch to another app, come back.
>    The scan finished at full speed.
>
> Report each step as pass or fail, with a screenshot for every failure and
> anything that felt slow or looked wrong.
