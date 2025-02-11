// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of pretty-expressive.
//
// pretty-expressive is free software: you can redistribute it and/or modify it
// under the terms of the GNU Lesser General Public License as published by the
// Free Software Foundation, either version 3 of the License, or (at your
// option) any later version. pretty-expressive is distributed in the hope that
// it will be useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// Lesser General Public License for more details. You should have received a
// copy of the GNU Lesser General Public License along with pretty-expressive.
// If not, see <https://www.gnu.org/licenses/>.

use crate::{
    cost::Cost,
    document::{Document, DocumentRef},
    interned_map::{InternedInfoMap, InternedMap},
};

struct Measure<C: Cost> {
    last_line_width: usize,
    cost: C,
    document: DocumentRef,
}

impl<C: Cost> Measure<C> {
    fn dominates(&self, other: &Self) -> bool {
        self.last_line_width <= other.last_line_width && self.cost <= other.cost
    }
}

impl<C: Cost> Clone for Measure<C> {
    fn clone(&self) -> Self {
        Self {
            last_line_width: self.last_line_width,
            cost: self.cost.clone(),
            document: self.document,
        }
    }
}

enum MeasureSet<C: Cost> {
    /// The measures in a measure set are strictly decreasing in order of
    /// [`Measure::last_line_width`] and they are all Pareto-optimal.
    Set(Vec<Measure<C>>),
    Tainted(Measure<C>),
}

impl<C: Cost> MeasureSet<C> {
    fn tainted(self) -> Self {
        match self {
            Self::Set(set) => Self::Tainted(
                set.into_iter().next().expect("empty measure set"),
            ),
            Self::Tainted(measure) => Self::Tainted(measure),
        }
    }

    fn lift<F: Fn(&mut InternedMap<Document>, Measure<C>) -> Measure<C>>(
        self,
        document_context: &mut InternedMap<Document>,
        f: F,
    ) -> Self {
        match self {
            Self::Set(set) => Self::Set(
                set.into_iter()
                    .map(|measure| f(document_context, measure))
                    .collect(),
            ),
            Self::Tainted(measure) => {
                Self::Tainted(f(document_context, measure))
            }
        }
    }

    /// Merges two measure sets (p. 261:20).
    fn merge_with(self, other: Self) -> Self {
        match (self, other) {
            (set, Self::Tainted(_)) => set,
            (Self::Tainted(_), Self::Set(set)) => Self::Set(set),
            (Self::Set(a), Self::Set(b)) => {
                let mut merged = Vec::new();
                let mut a = a.into_iter().peekable();
                let mut b = b.into_iter().peekable();
                while a.peek().is_some() && b.peek().is_some() {
                    let measure_a = a.peek().unwrap();
                    let measure_b = b.peek().unwrap();
                    if measure_a.dominates(measure_b) {
                        a.next();
                    } else if measure_b.dominates(measure_a) {
                        b.next();
                    } else if measure_a.last_line_width
                        > measure_b.last_line_width
                    {
                        merged.push(a.next().unwrap());
                    } else {
                        merged.push(b.next().unwrap());
                    }
                }
                merged.extend(a);
                merged.extend(b);
                Self::Set(merged)
            }
        }
    }
}

struct PrintingConfig {
    max_width: usize,
    indent_amount: usize,
}

#[derive(Clone)]
struct PrintingContext {
    column: usize,
    current_indent: usize,
    should_flatten: bool,
}

struct ChoicelessContext<C: Cost> {
    measures: InternedInfoMap<DocumentRef, Measure<C>>,
}

impl<C: Cost> ChoicelessContext<C> {
    /// Implements _TextM_ (p. 261:18).
    fn measure_text(
        &mut self,
        config: &PrintingConfig,
        printing_context: &PrintingContext,
        document: DocumentRef,
        text: &str,
    ) -> Measure<C> {
        Measure {
            last_line_width: printing_context.column + text.len(),
            cost: C::for_text(
                printing_context.column,
                text.len(),
                config.max_width,
            ),
            document,
        }
    }

    ///  Implements _LineM_ (p. 261:18).
    fn measure_newline(
        &mut self,
        printing_context: &PrintingContext,
        document: DocumentRef,
    ) -> Measure<C> {
        Measure {
            last_line_width: printing_context.current_indent,
            cost: C::NEWLINE_COST,
            document,
        }
    }

    /// Implements _ConcatM_ and the "dot" operator on measures (p. 261:18).
    fn measure_concat(
        &mut self,
        config: &PrintingConfig,
        mut printing_context: PrintingContext,
        document_context: &mut InternedMap<Document>,
        children: &[DocumentRef],
    ) -> Measure<C> {
        let mut children_measures = Vec::new();
        for child in children {
            let measure = self.measure_document_memoized(
                config,
                printing_context.clone(),
                document_context,
                *child,
            );
            printing_context.column = measure.last_line_width;
            children_measures.push(measure);
        }
        let last_line_width = children_measures.last().unwrap().last_line_width;

        let mut cost = children_measures[0].cost.clone();
        for i in 1..children_measures.len() {
            // No AddAssign :(
            cost = cost + children_measures[i].cost.clone();
        }

        let new_parent = document_context.add(Document::Concat(
            children_measures
                .into_iter()
                .map(|measure| measure.document)
                .collect(),
        ));

        Measure {
            last_line_width,
            cost,
            document: new_parent,
        }
    }

    fn measure_nest(
        &mut self,
        config: &PrintingConfig,
        mut printing_context: PrintingContext,
        document_context: &mut InternedMap<Document>,
        add_indent_amount: usize,
        child: DocumentRef,
    ) -> Measure<C> {
        printing_context.current_indent += add_indent_amount;

        let child_measure = self.measure_document_memoized(
            config,
            printing_context,
            document_context,
            child,
        );

        Measure {
            last_line_width: child_measure.last_line_width,
            cost: child_measure.cost,
            document: document_context.add(Document::Nest {
                add_indent_amount,
                child: child_measure.document,
            }),
        }
    }

    pub fn measure_document_memoized(
        &mut self,
        config: &PrintingConfig,
        printing_context: PrintingContext,
        document_context: &mut InternedMap<Document>,
        document: DocumentRef,
    ) -> Measure<C> {
        if self.measures.contains_key(document) {
            self.measures[document].clone()
        } else {
            let measure = match document_context[document].clone() {
                Document::Text(text) => self.measure_text(
                    config,
                    &printing_context,
                    document,
                    &text,
                ),
                Document::Newline => {
                    self.measure_newline(&printing_context, document)
                }
                Document::Concat(children) => self.measure_concat(
                    config,
                    printing_context,
                    document_context,
                    &children,
                ),
                Document::Nest {
                    add_indent_amount,
                    child,
                } => todo!(),
                Document::Flatten(interned_key) => todo!(),
                Document::Union(interned_key, interned_key1) => todo!(),
            };
            self.measures.init(document, measure.clone());
            measure
        }
    }

    /// Implements _TextRSSet_ and _TextRSTnt_ (p. 261:21).
    pub fn resolve_text(
        &mut self,
        config: &PrintingConfig,
        printing_context: PrintingContext,
        document: DocumentRef,
        text: &str,
    ) -> MeasureSet<C> {
        let measure =
            self.measure_text(config, &printing_context, document, text);
        if printing_context.column + text.len() <= config.max_width
            && printing_context.current_indent <= config.max_width
        {
            MeasureSet::Set(vec![measure])
        } else {
            MeasureSet::Tainted(measure)
        }
    }

    /// Implements _LineRSSet_ and _LineRSTnt_ (p. 261:21).
    pub fn resolve_newline(
        &mut self,
        config: &PrintingConfig,
        printing_context: PrintingContext,
        document: DocumentRef,
    ) -> MeasureSet<C> {
        let measure = self.measure_newline(&printing_context, document);
        if printing_context.column <= config.max_width
            && printing_context.current_indent <= config.max_width
        {
            MeasureSet::Set(vec![measure])
        } else {
            MeasureSet::Tainted(measure)
        }
    }

    /// Implements _ConcatRS_ and _ConcatRSTnt_ (p. 261:21).
    pub fn resolve_concat(
        &mut self,
        config: &PrintingConfig,
        printing_context: PrintingContext,
        document_context: &mut InternedMap<Document>,
        document: DocumentRef,
        children: Vec<DocumentRef>,
    ) -> MeasureSet<C> {
        todo!()
    }

    /// Implements _NestRS_ (p. 261:21).
    pub fn resolve_nest(
        &mut self,
        config: &PrintingConfig,
        printing_context: PrintingContext,
        document_context: &mut InternedMap<Document>,
        document: DocumentRef,
        add_indent_amount: usize,
        child: DocumentRef,
    ) -> MeasureSet<C> {
        todo!()
    }

    pub fn resolve_document(
        &mut self,
        config: &PrintingConfig,
        document_context: &mut InternedMap<Document>,
        printing_context: PrintingContext,
        document: DocumentRef,
    ) -> MeasureSet<C> {
        match document_context[document].clone() {
            Document::Text(text) => {
                self.resolve_text(config, printing_context, document, &text)
            }
            Document::Newline => {
                self.resolve_newline(config, printing_context, document)
            }
            Document::Concat(children) => self.resolve_concat(
                config,
                printing_context,
                document_context,
                document,
                children,
            ),
            Document::Nest {
                add_indent_amount,
                child,
            } => self.resolve_nest(
                config,
                printing_context,
                document_context,
                document,
                add_indent_amount,
                child,
            ),
            Document::Flatten(interned_key) => todo!(),
            Document::Union(interned_key, interned_key1) => todo!(),
        }
    }
}
