// Copyright (C) 2024 Ethan Uppal.
//
// This file is part of spadefmt.
//
// spadefmt is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version. spadefmt is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
// details. You should have received a copy of the GNU General Public License
// along with spadefmt. If not, see <https://www.gnu.org/licenses/>.

use std::{error::Error, fmt, io};

pub trait ProduceString {
    fn produce_string(&self) -> String;
}

impl ProduceString for String {
    fn produce_string(&self) -> String {
        self.clone()
    }
}

impl ProduceString for &'_ str {
    fn produce_string(&self) -> String {
        self.to_string()
    }
}

impl<T: Fn() -> String> ProduceString for T {
    fn produce_string(&self) -> String {
        self()
    }
}

pub trait WithContext {
    fn with_context<S: ProduceString>(self, context: S) -> Self;
}

#[derive(Debug)]
struct ContextualError {
    context: String,
    inner: Box<dyn Error + Send + Sync>,
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error: {}\n\nCaused by:\n    {}",
            self.context, self.inner
        )
    }
}

impl Error for ContextualError {}

impl<T> WithContext for io::Result<T> {
    fn with_context<S: ProduceString>(self, context: S) -> Self {
        match self {
            Ok(result) => Ok(result),
            Err(error) => Err(io::Error::other(ContextualError {
                context: context.produce_string(),
                inner: Box::new(error),
            })),
        }
    }
}
