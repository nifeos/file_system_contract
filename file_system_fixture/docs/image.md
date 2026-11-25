# Why the test image has the layout it has

This page holds the reasons behind the files and directories the test image carries, and behind
the two disks that are not the shared image. The doc comments in `src/lib.rs` say what each
constant is and what reads it.

## The root

```text
/            motd  scratch  fs-witness-victim  fs-witness-usurpr
             sub/  other/  redir/  rmtree/  globset/  globmany/  walk/
```

`MOTD` is longer than eight bytes so the head word of the client's report is a real prefix of it.
`SCRATCH_NAME` ships with a placeholder that the client's write test overwrites with
`WRITE_PATTERN`; a host check reads it back after the run to show the write reached the image.

`tree::ROOT_ENTRIES` is the set of names the root must still hold after a run. A host check makes
this half of the assertion from outside the confined program, which no in-guest verdict can report.
It checks containment, not equality. Every test in a boot shares the root, so an exact comparison
would couple the check to whatever unrelated tests write. The upward-escape half checks
`tree::MADE` and `tree::MADE_DIRECTORY` instead, names that only the attacker writes.

## The granted subtree

```text
/sub         inner  deeper/          <- the directory grant designates this
/sub/deeper  leaf
/other       secret                  <- never reachable from the grant
```

`tree::OTHER` is a real sibling, so "it cannot reach a sibling" is about a real directory, and
`tree::DEEPER` gives a second descent somewhere to go. The host check pins `INNER` and `SECRET`, so
nothing may damage them. `ABSOLUTE_INNER` and `ABSOLUTE_SECRET` are constants rather than bytes
built at the call site because the navigating witness runs on two stack pages, and a static string
costs it nothing.

The attacker and the navigating shell leave names behind on purpose, each with a run index so that
runs sharing one image do not collide:

- `tree::MADE` stays so the host check can find it in `SUB` and not in the root. `tree::MOVED`
  beside an unrenamed `MADE` shows `REMOVE` enforced from outside the guest: one capability moved a
  name and another could not.
- `tree::NAVIGATION_KEPT` stays so that the absence of `NAVIGATION_GONE` is a fact about `unlink`
  and not about a shell that created nothing.
- `tree::NAVIGATION_DIRECTORY` stays because removing a directory with a name in it takes two
  deliberate steps (`UNLINK` refuses a directory, `RMDIR` refuses a non-empty one), and the shell
  takes neither.
- `tree::NAVIGATION_TOUCH` stays so the host check can confirm the second `touch` left the first
  write's bytes alone.

`tree::NAVIGATION_TOUCH_AT_RFC3339` is 2030-01-01, far enough past any real clock reading that
neither the server's logical clock nor a wall clock could produce it by chance. Its seconds value
`NAVIGATION_TOUCH_AT_UNIX_SECONDS` comes from two independent tools, `date -u` and Python's
`datetime`, rather than from either end of the wire.

`tree::NAVIGATION_BIND_ALIAS` is a fresh top-level word, never a real entry, so reaching it is the
bind and not a coincidence of the tree's shape.

## The trees the shell programs work in

```text
/redir                        alpha  beta
/rmtree                       rm-keep  rm-solo  rm-doomed/
/rmtree/rm-doomed             rm-one  rm-two  rm-nested/
/rmtree/rm-doomed/rm-nested   rm-leaf
/globset                      gl-one.txt  gl-two.txt  gl-three.log  gl-dir/
/globset/gl-dir               gl-inner
/globmany                     m-00.txt ... m-10.txt  m-keep.log
```

Each is a sibling of `SUB`, so a capability to one is provably not a capability to another, and a
shell that writes files has somewhere its writes cannot pass for another test's.

- `redir` holds two files so that `ls > out.txt` writes a listing with something in it, and the `wc`
  that counts it counts more than its own output's name.
- In `rmtree`, `rm-keep` is the control: the grant reaches it, and it stays because nothing names it.
  `rm-doomed` has two files at the top and a nested directory, so `rm -r` has to recurse and remove
  bottom-up. `rm-solo` falls to a plain `rm`, which must print nothing and exit zero: `rm`'s `-v`
  exists because the default prints nothing. `RM_MISSING` is not on the image, so `rm` of it is a
  diagnostic and `rm -f` of it is silence, which is the whole of what `-f` means.
- In `globset`, `GLOB_PATTERN` matches exactly the two `.txt` files. `gl-three.log` is the escape the
  set caretaker faces: it exists, and the caretaker one hop up could remove it. `gl-dir` is a
  directory, so the type bit a name set carries has something to be wrong about, and `gl-inner`
  keeps it non-empty. `GLOB_NOMATCH` shares the prefix and fails on the extension alone, the case a
  matcher that gave up early gets wrong. `GLOB_PLAIN` has two consecutive spaces, so a shell that
  split words would change it.
- `globmany` is a sibling of `globset` and not an extension of it: the globbing tests assert that
  `gl-*.txt` matches exactly two names, and adding files there would change that claim by accident.
  Its eleven names make one full batch of eight and one short batch, since a sweep that always ran
  full batches would pass a test with sixteen. The names are zero-padded so their byte order is
  their numeric order. `MANY_FIRST_BATCH` and `MANY_SECOND_BATCH` are spelled out, because a test
  that derives its expected answer from the rule it tests checks nothing. `m-keep.log` makes "the
  batches are the match" a claim about which names and not only about how many.

## The walk tree

```text
/walk/chain      f  c/                 (c/f, c/c/f, and so on, eight levels down)
/walk/wide       n000 ... n127
/walk/narrow     n000
/walk/sizes      s4k  s64k  s256k
```

[benchmarks.md](benchmarks.md) gives the reasons.

## The disks that are not the image

The crash test runs on its own disk, so nothing it does can pass for a change to the shared
image's `scratch`. Its file starts as `crash::INITIAL`, and recovering that would mean an
acknowledged write vanished. [witnesses.md](witnesses.md) gives the reasons behind the payloads.

The blank disk is the one the partitioner and the filesystem maker write. Each run makes it anew,
because a test that writes a partition table cannot share a disk other tests read. It arrives as 64
MiB of zeros with no table, so a program refused entropy that wrote nothing leaves a disk that still
parses as nothing. Its three partitions start at the 1 MiB alignment every real tool uses. The boot
and Linux partitions stay unformatted; they give the table more than one entry and more than one
identifier. The data partition's start and length are whole multiples of the 4096-byte RedoxFS
block, which the filesystem maker requires. A filesystem whose blocks straddle the partition's first
byte would read through the offset disk that made it and through nothing else. `blank::MADE_NAME`
is the file the maker writes on the new filesystem. An empty filesystem that opens proves the
header; a file the guest wrote proves the tree, the allocator and the root node too.
