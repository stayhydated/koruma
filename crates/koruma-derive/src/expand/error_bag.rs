#[derive(Default)]
pub(crate) struct ErrorBag {
    error: Option<syn::Error>,
}

impl ErrorBag {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, error: syn::Error) {
        if let Some(existing) = &mut self.error {
            existing.combine(error);
        } else {
            self.error = Some(error);
        }
    }

    pub(crate) fn push_result<T>(&mut self, result: Result<T, syn::Error>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.push(error);
                None
            },
        }
    }

    pub(crate) fn finish(self) -> Result<(), syn::Error> {
        match self.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
