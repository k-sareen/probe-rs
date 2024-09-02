//! This crate provides static instrumentation macros.
//!
//! With the `probe!` macro, programmers can place static instrumentation
//! points in their code to mark events of interest. These are compiled into
//! platform-specific implementations, e.g. SystemTap SDT on Linux. Probes are
//! designed to have negligible overhead during normal operation, so they can
//! be present in all builds, and only activated using those external tools.
//!
//! # Example
//!
//! This simple example instruments the beginning and end of program, as well
//! as every iteration through the loop with arguments for the counter and
//! intermediate total.
//!
//! ```rust
//! use probe::probe;
//! fn main() {
//!     probe!(foo, begin);
//!     let mut total = 0;
//!     for i in 0..100 {
//!         total += i;
//!         probe!(foo, loop, i, total);
//!     }
//!     assert_eq!(total, 4950);
//!     probe!(foo, end);
//! }
//! ```
//!
//! ## Using probes with SystemTap
//!
//! For the program above, a SystemTap script could double-check the totals:
//!
//! ```notrust
//! global check
//!
//! probe process.provider("foo").mark("loop") {
//!     check += $arg1;
//!     if (check != $arg2)
//!         printf("foo total is out of sync! (%d != %d)\n", check, $arg2);
//! }
//!
//! // .provider is optional
//! probe process.mark("begin"), process.mark("end") {
//!     printf("%s:%s\n", $$provider, $$name);
//! }
//! ```
//!
//! Since this program behaves as expected, this script will not have any complaint.
//!
//! ```notrust
//! $ stap --dyninst foo.stp -c ./foo
//! foo:begin
//! foo:end
//! ```
//!
//! ## Using probes with GDB
//!
//! Starting in version 7.5, GDB can set breakpoints on probes and read arguments.
//!
//! ```notrust
//! (gdb) info probes
//! Provider Name  Where              Semaphore Object
//! foo      begin 0x0000000000402e70           /tmp/foo
//! foo      end   0x000000000040315c           /tmp/foo
//! foo      loop  0x0000000000402f25           /tmp/foo
//! (gdb) break -probe foo:loop
//! Breakpoint 1 at 0x402f25
//! (gdb) condition 1 $_probe_arg1 > 1000
//! (gdb) run
//! Starting program: /tmp/foo
//! [Thread debugging using libthread_db enabled]
//! Using host libthread_db library "/lib64/libthread_db.so.1".
//!
//! Breakpoint 1, 0x0000000000402f25 in main::hd67360886023c1c6faa::v0.0 ()
//! (gdb) print $_probe_arg0
//! $1 = 45
//! (gdb) print $_probe_arg1
//! $2 = 1035
//! ```

#![no_std]

#[cfg(any(test, feature = "use_std"))]
extern crate std;

mod platform;

/// Define a static probe point.
///
/// This annotates a code location with a name and arguments, and compiles
/// in metadata to let debugging tools locate it.
///
/// # Arguments
///
/// * `provider` - An identifier for naming probe groups.
///
/// * `name`     - An identifier for this specific probe.
///
/// * `arg`...   - Optional data to provide with the probe. Any expression which
///   can be cast `as isize` is allowed as an argument. The arguments are always
///   evaluated, even on platforms that have a no-op implementation of probes.
///
/// # Example
///
/// ```
/// # use probe::probe;
/// // Probes are unit-typed expressions.
/// let () = probe!(foo, main);
///
/// let x = 42;
/// probe!(foo, show_x, x);
///
/// let y = Some(x);
/// probe!(foo, show_y, match y {
///     Some(n) => n,
///     None    => -1
/// });
///
/// let mut z = 0;
/// probe!(foo, inc_z, { z += 1; z });
/// assert_eq!(z, 1, "arguments are always evaluated");
/// ```
#[macro_export]
macro_rules! probe(
    ($provider:ident, $name:ident $(, $arg:expr)* $(,)?)
    => ($crate::platform_probe!($provider, $name, $($arg,)*));
);

/// Define a static probe point with lazy argument evaluation.
///
/// This annotates a code location with a name and arguments, and compiles
/// in metadata to let debugging tools locate it. This works the same way as
/// [`probe!`] except that arguments are only evaluated when a debugger or
/// tracing tool is attached to the probe. However, if a platform implementation
/// can't determine that, it might always evaluate arguments.
///
/// Returns `true` if the probe is executed (and its arguments evaluated).
///
/// # Example
///
/// ```
/// # use probe::probe_lazy;
/// let enabled = probe_lazy!(foo, main);
/// assert!(!enabled, "lazy probes only return true when they're active");
///
/// let mut z = 0;
/// probe_lazy!(foo, inc_z, { z += 1; z });
/// assert_eq!(z, 0, "arguments are not evaluated by default");
/// ```
#[macro_export]
macro_rules! probe_lazy(
    ($provider:ident, $name:ident $(, $arg:expr)* $(,)?)
    => ($crate::platform_probe_lazy!($provider, $name, $($arg,)*));
);
