// SPDX-License-Identifier: Apache-2.0
//! PHT-0028 slice 2: the budgeted renderer. For every type carrying
//! annotations, codegen emits `render_budgeted(&self, &Budget) ->
//! Budgeted` beside the untouched `render()` — a SECOND deterministic
//! renderer, never a nondeterministic one. The measure is the
//! caller's; the whole candidate text is re-measured after every
//! degradation step, so the budget guarantee holds for any
//! deterministic measure; truncation floors arrive at render time in
//! measure units (`Budget::floors`).

use std::collections::{HashMap, HashSet};

use crate::ir::{Elastic, PhenotypeModule, PhenotypeType, RenderNode, ValueType};
use crate::symbol::FieldId;

use super::emit::{emit_guarded_render_node, emit_render_node};
use super::naming;

/// Does any type in the module carry annotations? (The runtime block
/// is emitted once, and only when someone needs it.)
pub fn module_annotated(module: &PhenotypeModule) -> bool {
    module.types.iter().any(|pt| {
        pt.fields
            .iter()
            .any(|f| f.annotations != Default::default())
    })
}

fn type_annotated(pt: &PhenotypeType) -> bool {
    pt.fields
        .iter()
        .any(|f| f.annotations != Default::default())
}

/// One degradation step, in the order the design fixes: drops first
/// (priority ascending, later-in-template first among equals), then
/// truncations (later-in-template first).
struct Step {
    field: FieldId,
    name: String,
    kind: StepKind,
}

#[derive(PartialEq)]
enum StepKind {
    Drop,
    DropTail,
    Truncate,
}

/// The per-type instrumentation plan.
struct Plan {
    steps: Vec<Step>,
    /// field → index into `steps`/cfg for elastic fields.
    idx: HashMap<FieldId, usize>,
    /// Effectively volatile fields (own annotation, or a same-module
    /// volatile type).
    volatile: HashSet<FieldId>,
    /// field → segment name.
    segments: HashMap<FieldId, String>,
    /// field → truncation marker.
    markers: HashMap<FieldId, String>,
}

fn first_ref_positions(pt: &PhenotypeType) -> HashMap<FieldId, usize> {
    let mut pos = HashMap::new();
    let mut n = 0usize;
    fn walk(nodes: &[RenderNode], pos: &mut HashMap<FieldId, usize>, n: &mut usize) {
        for node in nodes {
            let mut touch = |id: FieldId, n: &mut usize| {
                pos.entry(id).or_insert(*n);
                *n += 1;
            };
            match node {
                RenderNode::Text(_) | RenderNode::ParentFieldRef { .. } => {}
                RenderNode::Emit(id) => touch(*id, n),
                RenderNode::Join { field, .. } => touch(*field, n),
                RenderNode::Eol { field } => {
                    if let Some(id) = field {
                        touch(*id, n)
                    }
                }
                RenderNode::IfSet { field, body } | RenderNode::IfNotEmpty { field, body } => {
                    touch(*field, n);
                    walk(body, pos, n);
                }
            }
        }
    }
    walk(&pt.render, &mut pos, &mut n);
    pos
}

fn plan(pt: &PhenotypeType, module: &PhenotypeModule) -> Plan {
    let positions = first_ref_positions(pt);
    let volatile_types = crate::ir::volatile_type_set(module);
    let mut drops: Vec<(u32, usize, Step)> = Vec::new();
    let mut cuts: Vec<(usize, Step)> = Vec::new();
    let mut volatile = HashSet::new();
    let mut segments = HashMap::new();
    let mut markers = HashMap::new();
    for f in &pt.fields {
        let pos = positions.get(&f.id).copied().unwrap_or(usize::MAX);
        if f.annotations.volatile
            || crate::ir::referenced_type_id_of(&f.ty).is_some_and(|t| volatile_types.contains(&t))
        {
            volatile.insert(f.id);
        }
        if let Some(name) = &f.annotations.segment {
            segments.insert(f.id, name.clone());
        }
        match &f.annotations.elastic {
            Some(Elastic::Drop { priority }) => {
                let kind = if matches!(f.ty, ValueType::UserPlural { .. })
                    || f.cardinality != crate::symbol::Cardinality::One
                {
                    StepKind::DropTail
                } else {
                    StepKind::Drop
                };
                drops.push((
                    *priority,
                    pos,
                    Step {
                        field: f.id,
                        name: f.name.clone(),
                        kind,
                    },
                ));
            }
            Some(Elastic::Truncate { marker }) => {
                markers.insert(f.id, marker.clone());
                cuts.push((
                    pos,
                    Step {
                        field: f.id,
                        name: f.name.clone(),
                        kind: StepKind::Truncate,
                    },
                ));
            }
            None => {}
        }
    }
    // Priority ascending; among equals, later in the template first.
    drops.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    cuts.sort_by_key(|(pos, _)| std::cmp::Reverse(*pos));
    let steps: Vec<Step> = drops
        .into_iter()
        .map(|(_, _, s)| s)
        .chain(cuts.into_iter().map(|(_, s)| s))
        .collect();
    let idx = steps
        .iter()
        .enumerate()
        .map(|(i, s)| (s.field, i))
        .collect();
    Plan {
        steps,
        idx,
        volatile,
        segments,
        markers,
    }
}

/// The module-level runtime, emitted once when any type is annotated.
pub fn emit_budget_runtime(out: &mut String) {
    out.push_str(RUNTIME);
}

/// Emit `render_budgeted` and its instrumented render for one type.
/// Parent-context types are skipped (their render needs a parent; the
/// budgeted twin lands with the composition work, not before).
pub fn emit_budgeted_impl(pt: &PhenotypeType, module: &PhenotypeModule, out: &mut String) {
    if !type_annotated(pt) || pt.parent_context.is_some() {
        return;
    }
    let plan = plan(pt, module);
    let rust_name = naming::to_rust_type_name(&pt.singular_name);

    out.push_str(&format!("impl {rust_name} {{\n"));

    // ── render_budgeted ──
    out.push_str(
        "    /// PHT-0028: the budgeted render — deterministic for any\n\
         \x20\x20\x20\x20/// deterministic measure; annotations stay inert under `render()`.\n",
    );
    out.push_str("    pub fn render_budgeted(&self, budget: &Budget<'_>) -> Budgeted {\n");
    out.push_str("        let steps: Vec<__PhtStep> = vec![\n");
    for (i, s) in plan.steps.iter().enumerate() {
        let kind = match s.kind {
            StepKind::Drop => "Drop",
            StepKind::DropTail => "DropTail",
            StepKind::Truncate => "Truncate",
        };
        out.push_str(&format!(
            "            __PhtStep {{ idx: {i}, name: \"{}\", kind: __PhtStepKind::{kind} }},\n",
            s.name
        ));
    }
    out.push_str("        ];\n");
    out.push_str("        let cfg: Vec<__PhtFieldCfg> = vec![\n");
    for s in &plan.steps {
        let f = pt.fields.iter().find(|f| f.id == s.field).unwrap();
        let fname = naming::to_rust_field_name(&f.name);
        match s.kind {
            StepKind::Drop => out.push_str("            __PhtFieldCfg::scalar(),\n"),
            StepKind::DropTail => {
                let len = if matches!(f.ty, ValueType::UserPlural { .. }) {
                    format!("self.{fname}.as_slice().len()")
                } else {
                    format!("self.{fname}.len()")
                };
                out.push_str(&format!("            __PhtFieldCfg::tail({len}),\n"));
            }
            StepKind::Truncate => {
                let chars = match f.requiredness {
                    crate::symbol::Requiredness::Optional => {
                        format!("self.{fname}.as_deref().map(|v| v.chars().count()).unwrap_or(0)")
                    }
                    _ => format!("self.{fname}.chars().count()"),
                };
                out.push_str(&format!("            __PhtFieldCfg::cuttable({chars}),\n"));
            }
        }
    }
    out.push_str("        ];\n");
    out.push_str(
        "        __pht_drive(budget, &steps, cfg, &|c| self.__pht_render_cfg(c))\n    }\n\n",
    );

    // ── the instrumented render ──
    out.push_str("    #[allow(unused_variables, unused_mut)]\n");
    out.push_str("    fn __pht_render_cfg(&self, __cfg: &[__PhtFieldCfg]) -> __PhtOut {\n");
    out.push_str("        let mut __text = String::new();\n");
    out.push_str("        let mut __stable: Option<usize> = None;\n");
    out.push_str(
        "        let mut __segs: Vec<(&'static str, std::ops::Range<usize>)> = Vec::new();\n",
    );
    out.push_str(&format!(
        "        let mut __spans: Vec<Option<std::ops::Range<usize>>> = vec![None; {}];\n",
        plan.steps.len()
    ));
    out.push_str("        {\n            let out = &mut __text;\n");
    for node in &pt.render {
        emit_budget_node(node, pt, module, &plan, None, out, "            ");
    }
    out.push_str("        }\n");
    out.push_str(
        "        __PhtOut { text: __text, stable: __stable, segments: __segs, spans: __spans }\n",
    );
    out.push_str("    }\n");
    out.push_str("}\n\n");
}

/// Instrumented mirror of the render walk. Plain nodes delegate to the
/// standard emitters (they write through the `out` alias); nodes that
/// touch annotated fields grow their hooks. `guard` carries the
/// enclosing `@ifset` field whose value is bound to `val`.
#[allow(clippy::too_many_arguments)]
fn emit_budget_node(
    node: &RenderNode,
    pt: &PhenotypeType,
    module: &PhenotypeModule,
    plan: &Plan,
    guard: Option<FieldId>,
    out: &mut String,
    indent: &str,
) {
    let field_of = |id: &FieldId| pt.fields.iter().find(|f| f.id == *id);
    let mark_volatile = |id: &FieldId, out: &mut String| {
        if plan.volatile.contains(id) {
            out.push_str(&format!(
                "{indent}if __stable.is_none() {{ __stable = Some(out.len()); }}\n"
            ));
        }
    };
    match node {
        RenderNode::Emit(id)
            if plan.volatile.contains(id)
                || plan.segments.contains_key(id)
                || plan
                    .idx
                    .get(id)
                    .is_some_and(|i| plan.steps[*i].kind == StepKind::Truncate) =>
        {
            let Some(f) = field_of(id) else { return };
            let fname = naming::to_rust_field_name(&f.name);
            let value_expr = if guard == Some(*id) {
                "val".to_string()
            } else {
                format!("self.{fname}")
            };
            mark_volatile(id, out);
            out.push_str(&format!(
                "{indent}{{\n{indent}    let __start = out.len();\n"
            ));
            if let Some(i) = plan
                .idx
                .get(id)
                .filter(|i| plan.steps[**i].kind == StepKind::Truncate)
            {
                let marker = plan
                    .markers
                    .get(id)
                    .cloned()
                    .unwrap_or_default()
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                out.push_str(&format!(
                    "{indent}    let __v: &str = &{value_expr};\n\
                     {indent}    match __cfg[{i}].cut {{\n\
                     {indent}        Some(c) => match __v.char_indices().nth(c) {{\n\
                     {indent}            Some((b, _)) => {{ out.push_str(&__v[..b]); out.push_str(\"{marker}\"); }}\n\
                     {indent}            None => out.push_str(__v),\n\
                     {indent}        }},\n\
                     {indent}        None => out.push_str(__v),\n\
                     {indent}    }}\n"
                ));
                out.push_str(&format!(
                    "{indent}    if __spans[{i}].is_none() {{ __spans[{i}] = Some(__start..out.len()); }}\n"
                ));
            } else {
                // volatile/segment-only: plain write of the value.
                out.push_str(&format!("{indent}    out.push_str(&{value_expr});\n"));
            }
            if let Some(name) = plan.segments.get(id) {
                let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
                out.push_str(&format!(
                    "{indent}    __segs.push((\"{escaped}\", __start..out.len()));\n"
                ));
            }
            out.push_str(&format!("{indent}}}\n"));
        }
        RenderNode::Join { field, .. }
            if plan.idx.contains_key(field)
                || plan.volatile.contains(field)
                || plan.segments.contains_key(field) =>
        {
            let Some(f) = field_of(field) else { return };
            let fname = naming::to_rust_field_name(&f.name);
            mark_volatile(field, out);
            out.push_str(&format!(
                "{indent}{{\n{indent}    let __start = out.len();\n"
            ));
            // Re-emit the join with an optional take() — a faithful copy
            // of the standard join with the keep limit applied.
            let take = plan
                .idx
                .get(field)
                .filter(|i| plan.steps[**i].kind == StepKind::DropTail)
                .map(|i| format!(".take(__cfg[{i}].keep)"))
                .unwrap_or_default();
            let joined = render_join_with_take(node, pt, module, &fname, &take, indent);
            out.push_str(&joined);
            if let Some(name) = plan.segments.get(field) {
                let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
                out.push_str(&format!(
                    "{indent}    __segs.push((\"{escaped}\", __start..out.len()));\n"
                ));
            }
            out.push_str(&format!("{indent}}}\n"));
        }
        RenderNode::IfSet { field, body } => {
            let Some(f) = field_of(field) else { return };
            let fname = naming::to_rust_field_name(&f.name);
            if let Some(i) = plan
                .idx
                .get(field)
                .filter(|i| plan.steps[**i].kind == StepKind::Drop)
            {
                out.push_str(&format!("{indent}if !__cfg[{i}].dropped {{\n"));
                emit_ifset_body(
                    field,
                    &fname,
                    body,
                    pt,
                    module,
                    plan,
                    out,
                    &format!("{indent}    "),
                );
                out.push_str(&format!("{indent}}}\n"));
            } else {
                emit_ifset_body(field, &fname, body, pt, module, plan, out, indent);
            }
        }
        RenderNode::IfNotEmpty { field, body } => {
            let Some(f) = field_of(field) else { return };
            let fname = naming::to_rust_field_name(&f.name);
            let is_plural = matches!(f.ty, ValueType::UserPlural { .. });
            let base = if is_plural {
                format!("!self.{fname}.as_slice().is_empty()")
            } else {
                format!("!self.{fname}.is_empty()")
            };
            let keep = plan
                .idx
                .get(field)
                .map(|i| format!("__cfg[{i}].keep > 0 && "))
                .unwrap_or_default();
            out.push_str(&format!("{indent}if {keep}{base} {{\n"));
            for child in body {
                emit_budget_node(
                    child,
                    pt,
                    module,
                    plan,
                    guard,
                    out,
                    &format!("{indent}    "),
                );
            }
            out.push_str(&format!("{indent}}}\n"));
        }
        RenderNode::Emit(id) | RenderNode::Eol { field: Some(id) } => {
            mark_volatile(id, out);
            match guard {
                Some(g) => emit_guarded_render_node(node, pt, module, g, out, indent),
                None => emit_render_node(node, pt, module, out, indent),
            }
        }
        other => match guard {
            Some(g) => emit_guarded_render_node(other, pt, module, g, out, indent),
            None => emit_render_node(other, pt, module, out, indent),
        },
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_ifset_body(
    field: &FieldId,
    fname: &str,
    body: &[RenderNode],
    pt: &PhenotypeType,
    module: &PhenotypeModule,
    plan: &Plan,
    out: &mut String,
    indent: &str,
) {
    out.push_str(&format!("{indent}if let Some(ref val) = self.{fname} {{\n"));
    for child in body {
        emit_budget_node(
            child,
            pt,
            module,
            plan,
            Some(*field),
            out,
            &format!("{indent}    "),
        );
    }
    out.push_str(&format!("{indent}}}\n"));
}

/// A faithful copy of the standard join emission with `take` applied.
fn render_join_with_take(
    node: &RenderNode,
    pt: &PhenotypeType,
    module: &PhenotypeModule,
    fname: &str,
    take: &str,
    indent: &str,
) -> String {
    use crate::ir::SeparatorExpr;
    let RenderNode::Join { field, separator } = node else {
        return String::new();
    };
    let f = pt.fields.iter().find(|f| f.id == *field).unwrap();
    let sep_expr = match separator {
        SeparatorExpr::Literal(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
        SeparatorExpr::Field(sep_id) => pt
            .fields
            .iter()
            .find(|f| f.id == *sep_id)
            .map(|sf| format!("&self.{}", naming::to_rust_field_name(&sf.name)))
            .unwrap_or_else(|| "\"\"".to_string()),
    };
    let inner = format!("{indent}    ");
    let mut s = String::new();
    s.push_str(&format!("{inner}let mut first = true;\n"));
    s.push_str(&format!(
        "{inner}for item in self.{fname}.iter(){take} {{\n"
    ));
    s.push_str(&format!(
        "{inner}    if !first {{ out.push_str({sep_expr}); }}\n"
    ));
    s.push_str(&format!("{inner}    first = false;\n"));
    if let ValueType::Imported(imp) = &f.ty {
        let prefix = naming::imported_module_prefix(&module.namespace, &imp.namespace);
        s.push_str(&format!(
            "{inner}    {prefix}Render::render_into(item, out);\n"
        ));
    } else if super::emit::needs_render_into(&f.ty) {
        s.push_str(&format!("{inner}    item.render_into(out);\n"));
    } else {
        s.push_str(&format!("{inner}    out.push_str(&item.to_string());\n"));
    }
    s.push_str(&format!("{inner}}}\n"));
    s
}

const RUNTIME: &str = r#"
// ─── PHT-0028: the budgeted-render runtime (emitted once) ───────────────────

/// A render budget: the limit and the measure are the caller's; floors
/// are truncation minimums in MEASURE units.
pub struct Budget<'a> {
    pub limit: usize,
    pub measure: &'a dyn Fn(&str) -> usize,
    pub floors: Floors<'a>,
}

#[derive(Clone, Copy)]
pub struct Floors<'a> {
    pub default: usize,
    pub per_field: &'a [(&'static str, usize)],
}

impl<'a> Floors<'a> {
    pub fn none() -> Self {
        Floors { default: 0, per_field: &[] }
    }
    pub fn uniform(default: usize) -> Self {
        Floors { default, per_field: &[] }
    }
    fn for_field(&self, name: &str) -> usize {
        self.per_field
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| *v)
            .unwrap_or(self.default)
    }
}

/// A budgeted render's honest report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Budgeted {
    pub text: String,
    /// measure(text) — may exceed `limit` only when the floor render
    /// itself does; the caller decides what to do with that truth.
    pub cost: usize,
    pub full_cost: usize,
    pub floor_cost: usize,
    pub dropped: Vec<&'static str>,
    pub truncated: Vec<&'static str>,
    /// Bytes before the first volatile content (text.len() when none).
    pub stable_len: usize,
    pub segments: Vec<(&'static str, std::ops::Range<usize>)>,
}

#[derive(Clone)]
pub struct __PhtFieldCfg {
    dropped: bool,
    keep: usize,
    cut: Option<usize>,
    chars: usize,
}

impl __PhtFieldCfg {
    pub fn scalar() -> Self {
        __PhtFieldCfg { dropped: false, keep: usize::MAX, cut: None, chars: 0 }
    }
    pub fn tail(len: usize) -> Self {
        __PhtFieldCfg { dropped: false, keep: len, cut: None, chars: 0 }
    }
    pub fn cuttable(chars: usize) -> Self {
        __PhtFieldCfg { dropped: false, keep: usize::MAX, cut: None, chars }
    }
}

pub enum __PhtStepKind { Drop, DropTail, Truncate }

pub struct __PhtStep {
    pub idx: usize,
    pub name: &'static str,
    pub kind: __PhtStepKind,
}

pub struct __PhtOut {
    pub text: String,
    pub stable: Option<usize>,
    pub segments: Vec<(&'static str, std::ops::Range<usize>)>,
    pub spans: Vec<Option<std::ops::Range<usize>>>,
}

/// The driver: deterministic for any deterministic measure. The whole
/// candidate is re-measured after every step; binary searches keep the
/// render count at O(steps · log n).
pub fn __pht_drive(
    budget: &Budget<'_>,
    steps: &[__PhtStep],
    mut cfg: Vec<__PhtFieldCfg>,
    render: &dyn Fn(&[__PhtFieldCfg]) -> __PhtOut,
) -> Budgeted {
    let mut cur = render(&cfg);
    let mut cost = (budget.measure)(&cur.text);
    let full_cost = cost;
    let mut dropped: Vec<&'static str> = Vec::new();
    let mut truncated: Vec<&'static str> = Vec::new();

    // The floor cut for a truncatable field: the smallest cut whose
    // field span still measures at least the floor (0 when no floor).
    let floor_cut = |cfg: &[__PhtFieldCfg], i: usize, floor: usize| -> usize {
        if floor == 0 {
            return 0;
        }
        let (mut lo, mut hi) = (0usize, cfg[i].chars);
        while lo < hi {
            let mid = (lo + hi) / 2;
            let mut probe = cfg.to_vec();
            probe[i].cut = Some(mid);
            let r = render(&probe);
            let span_cost = r.spans[i]
                .as_ref()
                .map(|s| (budget.measure)(&r.text[s.clone()]))
                .unwrap_or(0);
            if span_cost >= floor {
                hi = mid;
            } else {
                lo = mid + 1;
            }
        }
        lo
    };

    for step in steps {
        if cost <= budget.limit {
            break;
        }
        match step.kind {
            __PhtStepKind::Drop => {
                cfg[step.idx].dropped = true;
                cur = render(&cfg);
                cost = (budget.measure)(&cur.text);
                dropped.push(step.name);
            }
            __PhtStepKind::DropTail => {
                let full = cfg[step.idx].keep;
                // The largest keep that fits, else zero.
                let (mut lo, mut hi, mut best) = (0usize, full, 0usize);
                while lo <= hi {
                    let mid = (lo + hi) / 2;
                    let mut probe = cfg.clone();
                    probe[step.idx].keep = mid;
                    let r = render(&probe);
                    if (budget.measure)(&r.text) <= budget.limit {
                        best = mid;
                        lo = mid + 1;
                    } else if mid == 0 {
                        break;
                    } else {
                        hi = mid - 1;
                    }
                }
                cfg[step.idx].keep = best;
                cur = render(&cfg);
                cost = (budget.measure)(&cur.text);
                if best < full {
                    dropped.push(step.name);
                }
            }
            __PhtStepKind::Truncate => {
                let chars = cfg[step.idx].chars;
                let floor = floor_cut(&cfg, step.idx, budget.floors.for_field(step.name));
                // The largest cut that fits, clamped up to the floor.
                let (mut lo, mut hi, mut best) = (0usize, chars, 0usize);
                while lo <= hi {
                    let mid = (lo + hi) / 2;
                    let mut probe = cfg.clone();
                    probe[step.idx].cut = Some(mid);
                    let r = render(&probe);
                    if (budget.measure)(&r.text) <= budget.limit {
                        best = mid;
                        lo = mid + 1;
                    } else if mid == 0 {
                        break;
                    } else {
                        hi = mid - 1;
                    }
                }
                let chosen = best.max(floor);
                if chosen < chars {
                    cfg[step.idx].cut = Some(chosen);
                    cur = render(&cfg);
                    cost = (budget.measure)(&cur.text);
                    truncated.push(step.name);
                } else {
                    cfg[step.idx].cut = None;
                }
            }
        }
    }

    // The floor render: everything elastic spent.
    let mut floor_cfg = cfg.clone();
    for step in steps {
        match step.kind {
            __PhtStepKind::Drop => floor_cfg[step.idx].dropped = true,
            __PhtStepKind::DropTail => floor_cfg[step.idx].keep = 0,
            __PhtStepKind::Truncate => {
                let fc = floor_cut(&floor_cfg, step.idx, budget.floors.for_field(step.name));
                floor_cfg[step.idx].cut = Some(fc);
            }
        }
    }
    let floor_render = render(&floor_cfg);
    let floor_cost = (budget.measure)(&floor_render.text);

    let stable_len = cur.stable.unwrap_or(cur.text.len());
    Budgeted {
        cost,
        full_cost,
        floor_cost,
        dropped,
        truncated,
        stable_len,
        segments: cur.segments,
        text: cur.text,
    }
}
"#;
