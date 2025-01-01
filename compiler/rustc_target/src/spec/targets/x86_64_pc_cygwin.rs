use crate::spec::{Target, base};

pub(crate) fn target() -> Target {
    base::cygwin::x86_64_target(base::cygwin::CygwinVariant::Cygwin)
}
