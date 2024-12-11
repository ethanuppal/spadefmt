// // Copyright (C) 2024 Ethan Uppal.
// //
// // This file is part of spadefmt.
// //
// // spadefmt is free software: you can redistribute it and/or modify it under
// the // terms of the GNU General Public License as published by the Free
// Software // Foundation, either version 3 of the License, or (at your option)
// any later // version. spadefmt is distributed in the hope that it will be
// useful, but // WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or // FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
// Public License for more // details. You should have received a copy of the
// GNU General Public License // along with spadefmt. If not, see <https://www.gnu.org/licenses/>.
//
// use crate::{document::Document, interned_map::InternedMap};
//
// #[derive(Clone)]
// struct Layout(Vec<String>);
//
// impl Layout {
//     fn push(&mut self, c: char) {
//         if let Some(last) = self.0.last_mut() {
//             last.push(c)
//         } else {
//             self.0.push(c.to_string())
//         }
//     }
//     fn push_str(&mut self, string: &str) {
//         if let Some(last) = self.0.last_mut() {
//             last.push_str(string);
//         } else {
//             self.0.push(string.to_owned());
//         }
//     }
//
//     fn newline(&mut self) {
//         self.0.push(String::new());
//     }
// }
//
// #[derive(Clone)]
// struct Frame {
//     column: usize,
//     indent_level: usize,
//     should_flatten: bool,
// }
//
// struct Thread<'a> {
//     frames: Vec<Frame>,
//     indent: &'a str,
//     layout: Layout,
// }
//
// impl<'a> Thread<'a> {
//     fn new(indent: &'a str) -> Self {
//         Self {
//             frames: vec![Frame {
//                 column: 0,
//                 indent_level: 0,
//                 should_flatten: false,
//             }],
//             indent,
//             layout: Layout(Vec::new()),
//         }
//     }
// }
//
// impl Thread<'_> {
//     fn current_frame(&mut self) -> &mut Frame {
//         self.frames.last_mut().expect("thread with no frames")
//     }
//
//     fn push(&mut self) {
//         let next = self.current_frame().clone();
//         self.frames.push(next);
//     }
//
//     fn pop(&mut self) {
//         let _ = self.frames.pop();
//     }
// }
//
// struct PrintingContext<'a> {
//     threads: Vec<Thread<'a>>,
// }
//
// impl<'a> PrintingContext<'a> {
//     fn new(indent: &'a str) -> Self {
//         Self {
//             threads: vec![Thread::new(indent)],
//         }
//     }
//
//     fn evaluate(
//         &mut self,
//         document_context: &InternedMap<Document>,
//         document: &Document,
//     ) {
//         match document {
//             Document::Text(string) => {
//                 // Rules: Text, TextWiden
//                 for thread in &mut self.threads {
//                     thread.layout.push_str(string)
//                 }
//             }
//             Document::Newline => {
//                 for thread in &mut self.threads {
//                     if thread.current_frame().should_flatten {
//                         // Rules: LineFlatten, LineWiden
//                         thread.layout.push(' ');
//                     } else {
//                         // Rules: LineNoFlatten, LineWiden
//                         thread.layout.newline();
//                         for _ in 0..thread.current_frame().indent_level {
//                             thread.layout.push_str(&thread.indent);
//                         }
//                     }
//                 }
//             }
//             Document::Concat(children) => {
//                 // Rules: ConcatOne, ConcatMult, ConcatWiden
//                 for child in children {
//                     self.evaluate(document_context,
// &document_context[*child]);                 }
//             }
//             Document::Nest {
//                 add_indent_levels,
//                 child,
//             } => {
//                 // Rules: Nest
//                 for thread in &mut self.threads {
//                     thread.push();
//                     thread.current_frame().indent_level += add_indent_levels;
//                 }
//                 self.evaluate(document_context, &document_context[*child]);
//                 for thread in &mut self.threads {
//                     thread.pop();
//                 }
//             }
//             Document::Flatten(child) => {
//                 // Flatten
//                 for thread in &mut self.threads {
//                     thread.push();
//                     thread.current_frame().should_flatten = true;
//                 }
//                 self.evaluate(document_context, &document_context[*child]);
//                 for thread in &mut self.threads {
//                     thread.pop();
//                 }
//             }
//             Document::Union(a, b) => {}
//         }
//     }
//
//     // fn fork(&mut self) {
//     //     self.threads.clone
//     // }
// }
