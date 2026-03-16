---
title: "locus"
description: ""
summary: ""
date: 2026-03-15T12:00:00+00:00
lastmod: 2026-03-15T12:00:00+00:00
draft: false
menu:
  docs:
    parent: ""
    identifier: "locus-module"
weight: 750
toc: true
seo:
  title: "" # custom title (optional)
  description: "" # custom description (recommended)
  canonical: "" # custom canonical URL (optional)
  noindex: false # false (default) or true
---

The `locus` module helps you express proximity relationships between pattern
matches.

When you pass patterns like `$a`, `$b`, or the current `for .. of` pattern `$`,
the module checks all match combinations for you. Numeric functions return the
minimum value across all valid combinations, and boolean functions return true
as soon as any combination satisfies the condition.

You can also build reusable pattern groups with `locus.any(...)` and compare
those groups directly.

Functions like `start(...)` and `end(...)` return the bounds of the smallest
window that contains one match from each input. If more than one such window
exists, the first one means the leftmost smallest window. `end(...)` returns
the exclusive end offset, so `end(...) - start(...) == window(...)`.

Lower-level overloads that accept offsets and lengths are also available when
you want to reason about a specific match selected with `@a[i]` and `!a[i]`.

-------

## Functions

### any($a)

Creates a pattern group containing `$a`.

### any($a, $b)

Creates a pattern group containing `$a` and `$b`.

### any($a, $b, $c)

Creates a pattern group containing `$a`, `$b`, and `$c`.

Examples:

`locus.any($a, $b, $c)`

### distance($a, $b)

Returns the minimum distance between the start offsets of any `$a` and `$b`
match.

Examples:

`locus.distance($a, $b) <= 64`

### distance(offset_a, offset_b)

Returns the absolute distance between two offsets.

### gap($a, $b)

Returns the minimum gap between any `$a` and `$b` match. Overlapping or
adjacent matches return `0`.

Examples:

`locus.gap($loader, $url) <= 32`

### gap(offset_a, length_a, offset_b, length_b)

Returns the number of bytes between two detections. Overlapping or adjacent
detections return `0`.

### overlap($a, $b)

Returns true if any `$a` match overlaps any `$b` match.

### overlap(offset_a, length_a, offset_b, length_b)

Returns true if the two detections share at least one byte.

### covers($outer, $inner)

Returns true if any `$outer` match fully contains any `$inner` match.

### covers(offset_a, length_a, offset_b, length_b)

Returns true if the first detection fully contains the second one.

### window($a, $b)

Returns the size of the smallest window that contains one `$a` match and one
`$b` match.

Examples:

`locus.window($a, $b) <= 128`

### start($a, $b)

Returns the start offset of the first smallest window that contains one `$a`
match and one `$b` match.

### end($a, $b)

Returns the exclusive end offset of the first smallest window that contains
one `$a` match and one `$b` match.

### window($a, $b, $c)

Returns the size of the smallest window that contains one match from each
pattern.

Examples:

`locus.window($a, $b, $c) <= 256`

### start($a, $b, $c)

Returns the start offset of the first smallest window that contains one match
from each pattern.

### end($a, $b, $c)

Returns the exclusive end offset of the first smallest window that contains
one match from each pattern.

### window(offset_a, length_a, offset_b, length_b)

Returns the size of the smallest byte window that contains both detections.

### start(offset_a, length_a, offset_b, length_b)

Returns the start offset of the byte window that contains both detections.

### end(offset_a, length_a, offset_b, length_b)

Returns the exclusive end offset of the byte window that contains both
detections.

### window(offset_a, length_a, offset_b, length_b, offset_c, length_c)

Returns the size of the smallest byte window that contains three detections.

### start(offset_a, length_a, offset_b, length_b, offset_c, length_c)

Returns the start offset of the byte window that contains three detections.

### end(offset_a, length_a, offset_b, length_b, offset_c, length_c)

Returns the exclusive end offset of the byte window that contains three
detections.

### within($a, $b, tolerance)

Returns true if some `$a` and `$b` match fit within a `tolerance`-byte window.

Examples:

`locus.within($dom, $append, 128)`

### within(locus.any(...), locus.any(...), tolerance)

Returns true if any match from the first group and any match from the second
group fit within a `tolerance`-byte window.

Examples:

```yara
import "locus"

rule grouped_proximity {
    strings:
        $a = "document.createElement"
        $b = "appendChild"
        $c = "createElement(\"script\")"
        $d = "fetch("
        $e = "XMLHttpRequest"

    condition:
        locus.within(
            locus.any($a, $b, $c),
            locus.any($d, $e),
            128
        )
}
```

The same grouped arguments are accepted by `start(...)` and `end(...)`:

```yara
locus.start(locus.any($a, $b, $c), locus.any($d, $e))
locus.end(locus.any($a, $b, $c), locus.any($d, $e))
```

### within($a, $b, $c, tolerance)

Returns true if some combination of `$a`, `$b`, and `$c` fits within a
`tolerance`-byte window.

Examples:

```yara
import "locus"

rule clustered_js {
    strings:
        $dom = "document.createElement"
        $append = ".appendChild("
        $script = "createElement(\"script\")"

    condition:
        locus.within($dom, $append, $script, 256)
}
```

### within(offset_a, offset_b, tolerance)

Returns true if two offsets are at most `tolerance` bytes apart.

### within(offset_a, length_a, offset_b, length_b, tolerance)

Returns true if two detections fit within a `tolerance`-byte window.

### within(offset_a, length_a, offset_b, length_b, offset_c, length_c, tolerance)

Returns true if three detections fit within a `tolerance`-byte window.
