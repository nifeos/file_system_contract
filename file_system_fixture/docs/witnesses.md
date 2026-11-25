# Why the witnesses report what they report

A witness is a program that runs inside the guest with some grant, tries things, and reports what
got through. This page holds the reasons behind each witness's words. The doc comments in
`src/lib.rs` say what each constant is and what reads it.

## A bitmap, not a pass or a fail

Every witness reports a bitmap, and each bit says one specific thing happened. The test asserts an
exact set rather than zero, so a failure names itself.

The exact set also lets one witness serve as its own negative control. Each witness carries at
least one control bit that a run sets when something it should be able to do fails
(`escape::GRANTED_READ_FAILED`, `directory_escape::GRANTED_ACCESS_FAILED` and their kin). A
capability that reaches nothing cannot escape either, and a test that only checked the refusals
would pass against one. The reaching bits prove the grant carried something, and only then do the
refusal bits mean anything.

A report leads with a fixed word (`VERDICT`, or the shared-frame words) so that a silent client, one
that trapped before it wrote anything, never reads as a clean verdict of zero.

Each witness module carries an `ALL` array of its bits in declaration order. A consumer may assert
that a word it received holds only those bits. The crate's tests check that every entry is a single
bit, that no two share one, and that the array names every bit constant in its module.

## The per-file attacker: `escape`

The attacker holds a grant to one file. Run against a read-only grant, every bit must be clear. Run
against a read and write grant of the same shape, the run must set the two write bits, which proves the
read-only refusals came from a narrowed capability rather than a caretaker that refuses everything.

`TRUNCATED` is its own bit because a truncate carries no bytes. A guard that covered only `WRITE`
would leave a way to destroy a file just as thoroughly. `WROTE_ATTRIBUTE` is the same twin for the
same reason: an extended attribute changes a file without file bytes. `GRANTED_ATTRIBUTES_FAILED`
catches a regression to a caretaker that answers `EOPNOTSUPP` to all four attribute verbs, which
would set it in both runs.

## The directory attacker: `directory_escape`

The attacker holds a grant to `tree::SUB`. The read-only run and the wide run are each other's
control.

- `REACHED_PARENT`, `REACHED_SIBLING` and `WALKED_UP` are the three ways out of a subtree: up by a
  file the parent holds, sideways into `tree::OTHER`, and up by `..`.
- `WIDENED` answers whether a child's rights can exceed its parent's.
- `ENUMERATED_A_STRANGER` treats a listing as a rendering of authority. A name from outside the
  grant in a listing is an escape even though the witness opened nothing.
- `RENAMED` needs `REMOVE` and `CREATE` together, which gives the `REMOVE` right something to fail.
- `OPENED_ITS_OWN` and `GRANTED_ACCESS_FAILED` are the controls.
- `READ_ATTRIBUTES` holds wherever `READ` does, since an attribute is part of what a file is.
  `SET_AN_ATTRIBUTE` holds only with `WRITE`. The subtree caretaker performs no check for either:
  a file handle inherits `READ` and `WRITE` from the directory it came through, so the server
  refuses a read-only grant's attribute write, not a branch in the caretaker.
- `REACHED_AN_UNMATCHED_NAME` asks the set question with `READ` rather than `REMOVE`. The set
  proxy serves four verbs that read, and closing an attribute gap by opening a naming one would be
  a poor trade.

## Two grants in one process: `endpoint_boundary` and `mounted_tree`

`endpoint_boundary` holds two subtree caretakers in two capability table slots. The witness learns
nothing about its grants beyond which slot each landed in. It names a file in each subtree, tries to
cross from one to the other by request and by client-side path resolution, and reports what it
found. The structural claim is that the endpoint is the boundary, witnessed with two live
caretakers. `DOT_DOT_CROSSED_MOUNTS` is the negative control that only a union of two grants can
state: a path such as `/a/../b` that ascends out of one grant and into the other.

`mounted_tree` is the shell's presentation of the same two grants: one tree, with the second
mounted at `mounted_tree::MOUNT` inside the first. `cd ..` at the mount point moves to its parent
in the first tree, as a Unix mount does. The reaching bits are the controls for the refusal bits,
and the four bits from 16 up are the never-allowed outcomes.

## The navigating shell: `navigation`

Two shells run the same script, each rooted in a different subtree and told nothing about which
beyond a run index. Each tries to name one file from each of the two roots and reports which it
reached. The claim that two shells cannot name each other's files comes from the pair of reports;
neither report claims it alone.

- `/` names the shell's own root. That is the Plan 9 answer rather than the drive-letter one, and
  it makes the path `pwd` prints one the user can type back. `ABSOLUTE_REACHED_INNER` and
  `ABSOLUTE_REACHED_SECRET` measure that an absolute path grants nothing: each shell reaches exactly
  the file its own root contains. A `/` rooted in a global namespace would let both shells reach
  both.
- `cd ..` at the root and `cd /..` both clamp, because the shell descends from what it holds and
  has nothing above it.
- `HOLDER_KEPT_READING` and `NAME_GONE_AFTER_UNLINK` together measure that unlink removes a name
  and not an object. Either alone would also hold for an `rm` that did nothing.
- `UNLINK_REFUSED_A_DIRECTORY` and `RMDIR_REFUSED_NON_EMPTY` are the two halves of "no single call
  takes a subtree away". `RMDIR_REMOVED_EMPTY` is the control for the second.
- `TOUCH_PRESERVED` is the control for `TOUCH_CREATED`: without it, a `touch` that truncates what
  it finds would pass.
- `TOUCH_AT_ROUND_TRIPPED` compares for an exact value. A `touch -t` that fell back to the current
  time would still read back later than the bare touch before it, so only the exact comparison
  shows the caller's time moved the file and not the server's clock.
- `TOUCH_NOW_NEEDS_ONLY_WRITE` and `TOUCH_AT_REFUSED_WITHOUT_SETTIME` run against one handle that
  holds `WRITE` and not `SET_MODIFICATION_TIME_AT`. A verb table row that let the explicit time
  through on `WRITE` alone would flip both bits, and a caretaker that refused everything would
  clear both. A test that called the server directly would miss a wrong row; this one goes through
  the wire.
- The three bind bits show that a bound name is a second name for a position already held. It
  reaches what that position reaches, `..` from inside it lands at the real parent, and a third
  `..` stops at the shell's own root exactly where a direct walk would.

## Globbing: `globbing`

The shell holds a real directory capability served by a real subtree caretaker, so the witness
tests the prompt's own expansion over a real `READDIR`. `AGREED` is the headline: the names `echo`
printed and the names the grant carries are the same bytes. `EXCLUDED_A_STRANGER` is its control,
since a shell that expanded a pattern to everything it could see would also agree.

The shell refuses an empty expansion rather than printing it as itself, as zsh does, because an
empty expansion is an empty grant. It also refuses a pattern in a leading path component, because
choosing directories to walk is an authority question, and a matcher does not answer those.

## Extended attributes: `attributes`

The attribute bitmap rides the third word of the filesystem client's report, beside the `MOTD` head
and the `SUCCESS` sentinel. The same server serves attributes over the same endpoint, so a second
run of the whole stack would prove nothing new.

- `SET_AND_READ_BACK` checks the type code as well as the bytes. A layer that dropped the kind
  would pass every other bit.
- `LISTED` checks that the listing names nothing else, which catches a store keyed on the wrong
  node.
- `SURVIVED_RENAME` is the property an AppleDouble sidecar file gets wrong. It holds because the
  store keys on the node and a rename changes only a directory entry.
- `GONE_AFTER_UNLINK` is the on-device half of deleting attributes in the same transaction as the
  file. A leaked blob is somebody else's metadata on a file that never asked for it.
- `STORE_UNNAMEABLE` and `STORE_UNLISTED` check that a client can neither name nor list the store's
  own directory. A client that could name it could read any file's attributes as plain bytes.
- `attributes::KIND` is non-zero on purpose, so a layer that wrote `RAW` silently fails.

## The shared-frame witness

The victim and the usurper files have names of equal length and bodies that differ in their first
eight bytes. The length a client sends travels in a register, while the name travels in the shared
page, which any holder of the frame can rewrite. With equal lengths, a server that read the name
after the attacker rewrote it opens the usurper's file and returns its body, so a single head word
tells the two outcomes apart. A build-time assertion holds the lengths equal.

## The crash test: `crash`

The crash test drives one crash through the real stack on its own disk. The driver writes payload A and
the server acknowledges it, then the server dies in the middle of writing payload B, and a recovery mount reads
the file. The two payloads have the same length because the contract's `WRITE` does not truncate:
a shorter second payload would leave the first one's tail behind, and "the file is exactly A or
exactly B" would have no answer. `crash::CUT` is the server's word just before it traps. It gives
the test a defined moment to start the recovery mount, and it shows the injector killed the server
and not some other fault.
