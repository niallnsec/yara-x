use crate::compiler::PatternId;
use crate::mods::prelude::*;
use crate::modules::protos::locus::*;
use crate::scanner::{
    Match, MatchList, PatternSet, PatternSetHandle, RuntimeObjectHandle,
};

fn main(_ctx: &mut ModuleContext, _data: &[u8]) -> Result<Locus, ModuleError> {
    Ok(Locus::new())
}

#[derive(Clone, Copy)]
struct Span {
    start: i128,
    end: i128,
}

impl Span {
    fn new(start: i64, length: i64) -> Option<Self> {
        if start < 0 || length < 0 {
            return None;
        }

        let start = i128::from(start);
        let end = start.checked_add(i128::from(length))?;

        Some(Self { start, end })
    }

    fn from_match(m: &Match) -> Option<Self> {
        let start = i128::try_from(m.range.start).ok()?;
        let end = i128::try_from(m.range.end).ok()?;

        Some(Self { start, end })
    }

    fn gap(self, other: Self) -> i128 {
        if self.end < other.start {
            other.start - self.end
        } else if other.end < self.start {
            self.start - other.end
        } else {
            0
        }
    }

    fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }

    fn contains(self, other: Self) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    fn size(self) -> i128 {
        self.end - self.start
    }
}

fn checked_non_negative(value: i64) -> Option<i128> {
    if value < 0 { None } else { Some(i128::from(value)) }
}

fn checked_i64(value: i128) -> Option<i64> {
    value.try_into().ok()
}

fn distance_between_offsets(a: i64, b: i64) -> Option<i64> {
    let a = checked_non_negative(a)?;
    let b = checked_non_negative(b)?;

    checked_i64((a - b).abs())
}

fn window_bounds(spans: &[Span]) -> Option<Span> {
    let start = spans.iter().map(|span| span.start).min()?;
    let end = spans.iter().map(|span| span.end).max()?;

    Some(Span { start, end })
}

fn window_size(spans: &[Span]) -> Option<i64> {
    checked_i64(window_bounds(spans)?.size())
}

fn span_start(span: Option<Span>) -> Option<i64> {
    checked_i64(span?.start)
}

fn span_end(span: Option<Span>) -> Option<i64> {
    checked_i64(span?.end)
}

fn is_better_window(candidate: Span, current: Span) -> bool {
    candidate.size() < current.size()
        || (candidate.size() == current.size()
            && (candidate.start < current.start
                || (candidate.start == current.start
                    && candidate.end < current.end)))
}

fn match_list<'a>(
    ctx: &'a ScanContext<'_, '_>,
    pattern: PatternId,
) -> Option<&'a MatchList> {
    let matches = ctx.tracker.pattern_matches.get(pattern)?;

    if matches.len() == 0 { None } else { Some(matches) }
}

fn pattern_set(
    ctx: &ScanContext<'_, '_>,
    pattern_set: PatternSetHandle,
) -> Option<std::rc::Rc<PatternSet>> {
    let handle: RuntimeObjectHandle = pattern_set.into();
    ctx.runtime_objects.get(&handle).map(|object| object.as_pattern_set())
}

fn best_window_between_pattern_groups(
    ctx: &ScanContext<'_, '_>,
    left: &[PatternId],
    right: &[PatternId],
) -> Option<Span> {
    let mut best: Option<Span> = None;

    for &left_pattern in left {
        for &right_pattern in right {
            if let Some(window) =
                best_window_patterns(ctx, left_pattern, right_pattern)
                && best
                    .is_none_or(|current| is_better_window(window, current))
            {
                best = Some(window);
            }
        }
    }

    best
}

fn min_window_between_pattern_groups(
    ctx: &ScanContext<'_, '_>,
    left: &[PatternId],
    right: &[PatternId],
) -> Option<i64> {
    checked_i64(best_window_between_pattern_groups(ctx, left, right)?.size())
}

fn min_distance_between_matches(
    a_matches: &MatchList,
    b_matches: &MatchList,
) -> Option<i64> {
    let mut a_index = 0;
    let mut b_index = 0;
    let mut best = i128::MAX;

    while a_index < a_matches.len() && b_index < b_matches.len() {
        let a = Span::from_match(a_matches.get(a_index)?)?;
        let b = Span::from_match(b_matches.get(b_index)?)?;

        best = best.min((a.start - b.start).abs());

        if best == 0 {
            break;
        }

        if a.start <= b.start {
            a_index += 1;
        } else {
            b_index += 1;
        }
    }

    checked_i64(best)
}

fn min_gap_between_matches(
    a_matches: &MatchList,
    b_matches: &MatchList,
) -> Option<i64> {
    let mut a_index = 0;
    let mut b_index = 0;
    let mut best = i128::MAX;

    while a_index < a_matches.len() && b_index < b_matches.len() {
        let a = Span::from_match(a_matches.get(a_index)?)?;
        let b = Span::from_match(b_matches.get(b_index)?)?;

        best = best.min(a.gap(b));

        if best == 0 {
            break;
        }

        if a.end <= b.end {
            a_index += 1;
        } else {
            b_index += 1;
        }
    }

    checked_i64(best)
}

fn any_overlap(a_matches: &MatchList, b_matches: &MatchList) -> Option<bool> {
    let mut a_index = 0;
    let mut b_index = 0;

    while a_index < a_matches.len() && b_index < b_matches.len() {
        let a = Span::from_match(a_matches.get(a_index)?)?;
        let b = Span::from_match(b_matches.get(b_index)?)?;

        if a.overlaps(b) {
            return Some(true);
        }

        if a.end <= b.start {
            a_index += 1;
        } else if b.end <= a.start {
            b_index += 1;
        } else if a.end <= b.end {
            a_index += 1;
        } else {
            b_index += 1;
        }
    }

    Some(false)
}

fn any_cover(a_matches: &MatchList, b_matches: &MatchList) -> Option<bool> {
    let mut a_index = 0;
    let mut max_end = i128::MIN;

    for b in b_matches {
        let b = Span::from_match(b)?;

        while a_index < a_matches.len() {
            let a = Span::from_match(a_matches.get(a_index)?)?;

            if a.start > b.start {
                break;
            }

            max_end = max_end.max(a.end);
            a_index += 1;
        }

        if max_end >= b.end {
            return Some(true);
        }
    }

    Some(false)
}

fn best_window<const N: usize>(lists: [&MatchList; N]) -> Option<Span> {
    let mut indexes = [0usize; N];
    let mut best: Option<Span> = None;

    loop {
        let mut min_list = 0usize;
        let mut min_start = i128::MAX;
        let mut max_end = i128::MIN;

        for (list_index, list) in lists.iter().enumerate() {
            let span = Span::from_match(list.get(indexes[list_index])?)?;

            if span.start < min_start {
                min_start = span.start;
                min_list = list_index;
            }

            max_end = max_end.max(span.end);
        }

        let candidate = Span { start: min_start, end: max_end };

        if best.is_none_or(|current| is_better_window(candidate, current)) {
            best = Some(candidate);
        }

        indexes[min_list] += 1;

        if indexes[min_list] == lists[min_list].len() {
            break;
        }
    }

    best
}

fn within_tolerance(value: Option<i64>, tolerance: i64) -> Option<bool> {
    let tolerance = checked_non_negative(tolerance)?;
    Some(i128::from(value?) <= tolerance)
}

#[module_export(name = "any")]
fn any_one(ctx: &mut ScanContext, a: PatternId) -> PatternSetHandle {
    ctx.store_pattern_set(vec![a])
}

#[module_export(name = "any")]
fn any_pair(
    ctx: &mut ScanContext,
    a: PatternId,
    b: PatternId,
) -> PatternSetHandle {
    ctx.store_pattern_set(vec![a, b])
}

#[module_export(name = "any")]
fn any_triplet(
    ctx: &mut ScanContext,
    a: PatternId,
    b: PatternId,
    c: PatternId,
) -> PatternSetHandle {
    ctx.store_pattern_set(vec![a, b, c])
}

#[module_export]
fn distance(_ctx: &ScanContext, a: i64, b: i64) -> Option<i64> {
    distance_between_offsets(a, b)
}

#[module_export(name = "distance")]
fn distance_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
) -> Option<i64> {
    min_distance_between_matches(match_list(ctx, a)?, match_list(ctx, b)?)
}

#[module_export]
fn gap(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;

    checked_i64(a.gap(b))
}

#[module_export(name = "gap")]
fn gap_patterns(ctx: &ScanContext, a: PatternId, b: PatternId) -> Option<i64> {
    min_gap_between_matches(match_list(ctx, a)?, match_list(ctx, b)?)
}

#[module_export]
fn overlap(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
) -> Option<bool> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;

    Some(a.overlaps(b))
}

#[module_export(name = "overlap")]
fn overlap_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
) -> Option<bool> {
    any_overlap(match_list(ctx, a)?, match_list(ctx, b)?)
}

#[module_export(name = "covers")]
fn covers(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
) -> Option<bool> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;

    Some(a.contains(b))
}

#[module_export(name = "covers")]
fn covers_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
) -> Option<bool> {
    any_cover(match_list(ctx, a)?, match_list(ctx, b)?)
}

#[module_export(name = "window")]
fn window_pair(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;

    window_size(&[a, b])
}

#[module_export(name = "start")]
fn start_pair(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;

    span_start(window_bounds(&[a, b]))
}

#[module_export(name = "end")]
fn end_pair(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;

    span_end(window_bounds(&[a, b]))
}

fn best_window_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
) -> Option<Span> {
    best_window([match_list(ctx, a)?, match_list(ctx, b)?])
}

#[module_export(name = "window")]
fn window_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
) -> Option<i64> {
    checked_i64(best_window_patterns(ctx, a, b)?.size())
}

#[module_export(name = "start")]
fn start_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
) -> Option<i64> {
    span_start(best_window_patterns(ctx, a, b))
}

#[module_export(name = "end")]
fn end_patterns(ctx: &ScanContext, a: PatternId, b: PatternId) -> Option<i64> {
    span_end(best_window_patterns(ctx, a, b))
}

#[module_export(name = "window")]
fn window_triplet(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
    c_offset: i64,
    c_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;
    let c = Span::new(c_offset, c_length)?;

    window_size(&[a, b, c])
}

#[module_export(name = "start")]
fn start_triplet(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
    c_offset: i64,
    c_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;
    let c = Span::new(c_offset, c_length)?;

    span_start(window_bounds(&[a, b, c]))
}

#[module_export(name = "end")]
fn end_triplet(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
    c_offset: i64,
    c_length: i64,
) -> Option<i64> {
    let a = Span::new(a_offset, a_length)?;
    let b = Span::new(b_offset, b_length)?;
    let c = Span::new(c_offset, c_length)?;

    span_end(window_bounds(&[a, b, c]))
}

fn best_window_triplet_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
    c: PatternId,
) -> Option<Span> {
    best_window([
        match_list(ctx, a)?,
        match_list(ctx, b)?,
        match_list(ctx, c)?,
    ])
}

#[module_export(name = "window")]
fn window_triplet_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
    c: PatternId,
) -> Option<i64> {
    checked_i64(best_window_triplet_patterns(ctx, a, b, c)?.size())
}

#[module_export(name = "start")]
fn start_triplet_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
    c: PatternId,
) -> Option<i64> {
    span_start(best_window_triplet_patterns(ctx, a, b, c))
}

#[module_export(name = "end")]
fn end_triplet_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
    c: PatternId,
) -> Option<i64> {
    span_end(best_window_triplet_patterns(ctx, a, b, c))
}

#[module_export(name = "start")]
fn start_pattern_and_set(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternSetHandle,
) -> Option<i64> {
    let left = [a];
    let right = pattern_set(ctx, b)?;
    span_start(best_window_between_pattern_groups(
        ctx,
        &left,
        right.pattern_ids(),
    ))
}

#[module_export(name = "end")]
fn end_pattern_and_set(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternSetHandle,
) -> Option<i64> {
    let left = [a];
    let right = pattern_set(ctx, b)?;
    span_end(best_window_between_pattern_groups(
        ctx,
        &left,
        right.pattern_ids(),
    ))
}

#[module_export(name = "start")]
fn start_set_and_pattern(
    ctx: &ScanContext,
    a: PatternSetHandle,
    b: PatternId,
) -> Option<i64> {
    let left = pattern_set(ctx, a)?;
    let right = [b];
    span_start(best_window_between_pattern_groups(
        ctx,
        left.pattern_ids(),
        &right,
    ))
}

#[module_export(name = "end")]
fn end_set_and_pattern(
    ctx: &ScanContext,
    a: PatternSetHandle,
    b: PatternId,
) -> Option<i64> {
    let left = pattern_set(ctx, a)?;
    let right = [b];
    span_end(best_window_between_pattern_groups(
        ctx,
        left.pattern_ids(),
        &right,
    ))
}

#[module_export(name = "start")]
fn start_pattern_sets(
    ctx: &ScanContext,
    a: PatternSetHandle,
    b: PatternSetHandle,
) -> Option<i64> {
    let left = pattern_set(ctx, a)?;
    let right = pattern_set(ctx, b)?;
    span_start(best_window_between_pattern_groups(
        ctx,
        left.pattern_ids(),
        right.pattern_ids(),
    ))
}

#[module_export(name = "end")]
fn end_pattern_sets(
    ctx: &ScanContext,
    a: PatternSetHandle,
    b: PatternSetHandle,
) -> Option<i64> {
    let left = pattern_set(ctx, a)?;
    let right = pattern_set(ctx, b)?;
    span_end(best_window_between_pattern_groups(
        ctx,
        left.pattern_ids(),
        right.pattern_ids(),
    ))
}

#[module_export(name = "within")]
fn within_offsets(
    _ctx: &ScanContext,
    a: i64,
    b: i64,
    tolerance: i64,
) -> Option<bool> {
    within_tolerance(distance_between_offsets(a, b), tolerance)
}

#[module_export(name = "within")]
fn within_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
    tolerance: i64,
) -> Option<bool> {
    within_tolerance(window_patterns(ctx, a, b), tolerance)
}

#[module_export(name = "within")]
fn within_pattern_and_set(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternSetHandle,
    tolerance: i64,
) -> Option<bool> {
    let left = [a];
    let right = pattern_set(ctx, b)?;
    within_tolerance(
        min_window_between_pattern_groups(ctx, &left, right.pattern_ids()),
        tolerance,
    )
}

#[module_export(name = "within")]
fn within_set_and_pattern(
    ctx: &ScanContext,
    a: PatternSetHandle,
    b: PatternId,
    tolerance: i64,
) -> Option<bool> {
    let left = pattern_set(ctx, a)?;
    let right = [b];
    within_tolerance(
        min_window_between_pattern_groups(ctx, left.pattern_ids(), &right),
        tolerance,
    )
}

#[module_export(name = "within")]
fn within_pattern_sets(
    ctx: &ScanContext,
    a: PatternSetHandle,
    b: PatternSetHandle,
    tolerance: i64,
) -> Option<bool> {
    let left = pattern_set(ctx, a)?;
    let right = pattern_set(ctx, b)?;
    within_tolerance(
        min_window_between_pattern_groups(
            ctx,
            left.pattern_ids(),
            right.pattern_ids(),
        ),
        tolerance,
    )
}

#[module_export(name = "within")]
fn within_pair(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
    tolerance: i64,
) -> Option<bool> {
    within_tolerance(
        window_pair(_ctx, a_offset, a_length, b_offset, b_length),
        tolerance,
    )
}

#[allow(clippy::too_many_arguments)]
#[module_export(name = "within")]
fn within_triplet(
    _ctx: &ScanContext,
    a_offset: i64,
    a_length: i64,
    b_offset: i64,
    b_length: i64,
    c_offset: i64,
    c_length: i64,
    tolerance: i64,
) -> Option<bool> {
    within_tolerance(
        window_triplet(
            _ctx, a_offset, a_length, b_offset, b_length, c_offset, c_length,
        ),
        tolerance,
    )
}

#[module_export(name = "within")]
fn within_triplet_patterns(
    ctx: &ScanContext,
    a: PatternId,
    b: PatternId,
    c: PatternId,
    tolerance: i64,
) -> Option<bool> {
    within_tolerance(window_triplet_patterns(ctx, a, b, c), tolerance)
}

#[cfg(test)]
mod tests {
    use crate::compiler::Compiler;
    use crate::scanner::Scanner;
    use crate::tests::rule_false;
    use crate::tests::rule_true;
    use crate::tests::test_rule;

    #[test]
    fn distance_and_within() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                condition:
                    locus.distance(10, 14) == 4 and
                    locus.within(10, 14, 4)
            }"#,
            &[]
        );

        rule_false!(
            r#"
            import "locus"
            rule test {
                condition:
                    locus.within(10, 15, 4)
            }"#,
            &[]
        );
    }

    #[test]
    fn pattern_arguments_choose_any_matching_pair() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                condition:
                    locus.distance($a, $b) == 4 and
                    locus.gap($a, $b) == 1 and
                    locus.window($a, $b) == 7 and
                    locus.within($a, $b, 7)
            }"#,
            b"AAAxxxxxxxxxxxxBBBxAAAxBBB"
        );

        rule_false!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                condition:
                    locus.within($a, $b, 6)
            }"#,
            b"AAAxxxxxxxxxxxxBBBxAAAxBBB"
        );
    }

    #[test]
    fn gap_window_overlap_and_covers() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                condition:
                    locus.gap(0, 4, 10, 3) == 6 and
                    locus.window(0, 4, 10, 3) == 13 and
                    locus.start(0, 4, 10, 3) == 0 and
                    locus.end(0, 4, 10, 3) == 13 and
                    locus.overlap(0, 5, 3, 4) and
                    locus.covers(0, 10, 3, 4)
            }"#,
            &[]
        );
    }

    #[test]
    fn grouped_patterns_can_be_compared() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                    $c = "CCC"
                    $d = "DDD"
                    $e = "EEE"
                condition:
                    locus.within(
                        locus.any($a, $b, $c),
                        locus.any($d, $e),
                        7
                    ) and
                    locus.start(
                        locus.any($a, $b, $c),
                        locus.any($d, $e)
                    ) == 0 and
                    locus.end(
                        locus.any($a, $b, $c),
                        locus.any($d, $e)
                    ) == 7
            }"#,
            b"AAAxDDDxxxxxxxxBBBzzzzzzEEE"
        );

        rule_false!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                    $c = "CCC"
                    $d = "DDD"
                    $e = "EEE"
                condition:
                    locus.within(
                        locus.any($a, $b, $c),
                        locus.any($d, $e),
                        6
                    )
            }"#,
            b"AAAxDDDxxxxxxxxBBBzzzzzzEEE"
        );
    }

    #[test]
    fn singleton_and_duplicate_groups_behave_as_expected() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                    $c = "CCC"
                condition:
                    locus.within(locus.any($a), locus.any($b), 7) and
                    locus.within(locus.any($a, $a), $b, 7) and
                    not locus.within(locus.any($a, $a), $b, 6) and
                    locus.start(locus.any($a, $a), $b) == 0 and
                    locus.end(locus.any($a, $a), $b) == 7 and
                    locus.within($a, locus.any($b, $c), 7) and
                    locus.start($a, locus.any($b, $c)) == 0 and
                    locus.end($a, locus.any($b, $c)) == 7 and
                    not locus.within($a, locus.any($c), 7)
            }"#,
            b"AAAxBBBxxxxxxxxxxxxCCC"
        );
    }

    #[test]
    fn start_and_end_choose_the_first_smallest_locus() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                condition:
                    locus.window($a, $b) == 7 and
                    locus.start($a, $b) == 0 and
                    locus.end($a, $b) == 7
            }"#,
            b"AAAxBBBxxxxAAAxBBB"
        );
    }

    #[test]
    fn window_for_three_detections() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                condition:
                    locus.window(10, 3, 20, 2, 25, 4) == 19 and
                    locus.start(10, 3, 20, 2, 25, 4) == 10 and
                    locus.end(10, 3, 20, 2, 25, 4) == 29 and
                    locus.within(10, 3, 20, 2, 25, 4, 19)
            }"#,
            &[]
        );

        rule_false!(
            r#"
            import "locus"
            rule test {
                condition:
                    locus.within(10, 3, 20, 2, 25, 4, 18)
            }"#,
            &[]
        );
    }

    #[test]
    fn pattern_triplets_scan_all_combinations() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                    $c = "CCC"
                condition:
                    locus.window($a, $b, $c) == 13 and
                    locus.start($a, $b, $c) == 19 and
                    locus.end($a, $b, $c) == 32 and
                    locus.within($a, $b, $c, 13)
            }"#,
            b"AAAxxxxxxxxBBBBBBxxAAAxxBBBxxCCC"
        );

        rule_false!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a = "AAA"
                    $b = "BBB"
                    $c = "CCC"
                condition:
                    locus.within($a, $b, $c, 12)
            }"#,
            b"AAAxxxxxxxxBBBBBBxxAAAxxBBBxxCCC"
        );
    }

    #[test]
    fn current_for_of_pattern_can_be_passed_directly() {
        rule_true!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a1 = "AAA"
                    $a2 = "AXA"
                    $b = "BBB"
                condition:
                    for any of ($a*) : (
                        locus.within($, $b, 10)
                    )
            }"#,
            b"AXAxxxxxxxxxxxxBBBxxAAAxxBBB"
        );

        rule_false!(
            r#"
            import "locus"
            rule test {
                strings:
                    $a1 = "AAA"
                    $a2 = "AXA"
                    $b = "BBB"
                condition:
                    for all of ($a*) : (
                        locus.within($, $b, 10)
                    )
            }"#,
            b"AXAxxxxxxxxxxxxBBBxxAAAxxBBB"
        );
    }

    #[test]
    fn optimized_current_for_of_pattern_stays_in_scope() {
        let mut compiler = Compiler::new();

        compiler
            .condition_optimization(true)
            .add_source(
                r#"
                import "locus"
                rule test {
                    strings:
                        $a = "A"
                        $b = "B"
                    condition:
                        for any i in (1..2) : (
                            for any of ($a, $b) : (
                                locus.distance($, $b) == 0
                            )
                        )
                }"#,
            )
            .unwrap();

        let rules = compiler.build();
        let mut scanner = Scanner::new(&rules);
        let results = scanner.scan(b"AB").unwrap();

        assert_eq!(results.matching_rules().len(), 1);
    }
}

register_module!("locus", Locus, main);
