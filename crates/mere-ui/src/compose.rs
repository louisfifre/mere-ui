//! The input method's composition inside a focused text field. The composition (preedit) is what the input
//! method is still building: it is shown at the caret but is not part of the value, so the application's text
//! changes only at a commit. This departs from the browser's input events during a composition, on purpose:
//! the preview must not see half-composed kana.

use mere_base::{Ime, observe};

use crate::edit::Outcome;
use crate::field::FieldState;

pub(crate) struct Compose {
    pub text: String,
    /// The input method's caret inside `text`, in chars.
    pub cursor: usize,
}

impl Compose {
    /// An empty composition shows nothing: the field looks as it does without one.
    pub fn shown(&self) -> bool {
        !self.text.is_empty()
    }
}

/// Applies one message of the input method. Edited when a commit changed the text. The logs carry char
/// counts only, never the text itself: it is what the person types.
pub(crate) fn apply(st: &mut FieldState, ime: &Ime, max_chars: usize) -> Outcome {
    match ime {
        Ime::Start => {
            st.compose = Some(Compose { text: String::new(), cursor: 0 });
            observe!(Debug, "ime: composition starts");
            Outcome::None
        }
        Ime::Update { text, cursor } => {
            let n = text.chars().count();
            let cursor = (*cursor).min(n);
            st.compose = Some(Compose { text: text.clone(), cursor });
            observe!(Debug, "ime: composition of {n} chars, caret {cursor}");
            Outcome::None
        }
        Ime::Commit { text } => {
            st.compose = None;
            let changed = st.edit.commit(text, max_chars);
            observe!(Debug, "ime: commit of {} chars, changed {changed}", text.chars().count());
            if changed { Outcome::Edited } else { Outcome::None }
        }
        Ime::End => {
            st.compose = None;
            observe!(Debug, "ime: composition ends");
            Outcome::None
        }
    }
}
