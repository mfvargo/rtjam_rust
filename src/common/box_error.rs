//! type created so that the errors could work multi-threaded.
//!
//! To be very honest, I don't really know why I needed this, but it allows
//! me to "move" things into threads that I could not otherwise.
//!
//! TODO: Figure out why I need this.
pub type BoxError = std::boxed::Box<
    dyn std::error::Error // must implement Error to satisfy ?
        + std::marker::Send // needed for threads
        + std::marker::Sync, // needed for threads
>;
