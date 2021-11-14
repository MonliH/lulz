use std::sync::RwLock;

use codespan_reporting::files::SimpleFiles;
use once_cell::sync::Lazy;

pub static SOURCEMAP: Lazy<RwLock<SimpleFiles<String, String>>> = Lazy::new(|| {
    RwLock::new(SimpleFiles::new())
});
