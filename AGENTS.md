# AGENTS.md

AIR3 rust-agent-rules webchecked=2026-06-01
SRC:RustAPI,StyleGuide,Clippy,AsyncBook,PerfBook,RustPatterns,Nomicon,CargoBook
B=origin/main;BRANCH_RENAME!user;COMMIT/PUSH!user
DISCOVER:trust Cargo.toml/module tree/tests over this file for implementation facts

SCOPE:min_diff;read touched crate manifests first;preserve existing architecture;no drive-by refactor;no README!user;no secrets/.env;ignore legacy root src unless user says otherwise
RUN:deps="docker compose up -d";api="cargo run";batch="cargo run -p batch";stack="docker compose --profile app up -d --build"
VFY:fmt="cargo fmt --all --check";check="cargo check --workspace --all-targets";test="cargo test --workspace";lint="cargo clippy --workspace --all-targets -- -D warnings"

FMT:rustfmt default;4sp;100col;spaces not tabs;trailing commas multiline;outer doc comments;one derive attr;no manual style wars;format before final
CARGO:centralize shared deps in workspace if pattern exists;member deps use workspace=true when available;avoid new deps if std/local helper enough;min features;intentional Cargo.lock;do not change MSRV/edition/profile casually
CLIPPY:fix correctness/suspicious/perf;local allow only narrow+reason;style allow rare;pedantic/restriction/nursery cherry-pick only;no blanket allow to make CI green

NAMING:RFC430 casing;getters no get_ unless needed;as_=cheap ref/view,to_=cheap owned/borrow copy,into_=consume;iter/iter_mut/into_iter exact;feature names meaningful;consistent word order
API:constructors inherent new;Default if natural zero/empty config;builders for many/optional params;methods when receiver is clear;no out params;operator overload unsurprising;Deref only smart-pointer-like
TRAITS:derive/impl Debug,Clone,Eq,Hash,Ord,Default,Display selectively;public error types Display+Debug(+Error if std boundary);From/TryFrom/AsRef/AsMut over ad-hoc conversion;serde only at IO/DTO boundaries
FLEX:accept impl Trait/generics when it lowers assumptions;prefer &str/&[T]/Path over owned args;return impl Iterator when no allocation needed;expose intermediate results when it avoids duplicate work;object-safe traits if dyn use likely
FUTURE:private fields by default;sealed traits if downstream impls would constrain evolution;newtypes hide representation;avoid duplicating derive bounds;non_exhaustive for public enums when future variants plausible

OWNERSHIP:borrow before clone;clone deliberately not to appease borrow checker;clone Arc/Rc explicitly at ownership boundary;use mem::take/replace/split scopes to satisfy borrows;avoid needless lifetime params
ERROR:Result recoverable;Option absence-only;? over match boilerplate;panic/unwrap/expect only tests/prototypes/proven invariants with message;map errors at layer boundaries;do not leak internals to users
TYPES:encode invariants in types;newtype IDs/secrets/units;avoid bool/Option flag params;bitflags for combinable flags;validate untrusted input at edge;prefer NonZero/Duration/PathBuf/etc over primitive strings/ints when fitting
MATCH:prefer exhaustive match over stringly branching;use let-else/? for early exits;avoid partial state mutation before fallible steps unless rollback/transaction exists

ASYNC:futures do nothing until awaited/spawned;never block async worker;sync/cpu work=>spawn_blocking;no std::thread::sleep in async;no std::sync guard across await;prefer tokio sync in async;avoid nested runtimes/block_on in async
TASKS:spawned futures Send+'static unless LocalSet;JoinHandle awaited/logged/aborted intentionally;propagate cancellation;select! branches cancellation-safe;timeout external IO when local pattern exists;backpressure over unbounded fanout
CONCURRENCY:Arc for shared cross-task/thread state;Rc/RefCell only single-thread local;Mutex/RwLock scope tiny;avoid nested locks;prefer message passing for ownership transfer;manual Send/Sync unsafe only with proof

PERF:measure before complex optimization;avoid N+1 IO/queries;paginate/stream large data;avoid collect-then-iterate;preallocate Vec/String/Map when size known;reuse buffers in hot loops;format! allocates;Cow for mixed borrowed/owned if worth it
ALLOC:heap clone usually allocates except Arc/Rc;to_string/to_owned may allocate;SmallVec/ArrayVec only after profiling;do not trade clarity for micro-opts outside hot paths

HTTP:handlers/controllers thin;extract/validate/map at edge;business logic outside transport;return concrete error convertible to response;log internals with tracing;client messages sanitized;authz close to protected action
DB:migrations for schema;transactions for multi-write invariants;parameterized query/ORM builders;raw SQL only clearer/needed+tested;avoid long transactions across await-heavy external work;no generated entity edits unless project pattern
SEC:no log tokens/passwords/secrets/PII;fail closed;constant-time helpers for secret compare if present;env defaults local-dev only;redact debug output;least privilege for external calls
OBS:tracing over println;structured fields;instrument boundaries not hot loops;include IDs/status not secrets;errors logged once at boundary

TEST:unit tests near pure/domain code;tests/ for integration;doc tests for public examples;deterministic clocks/randomness/ports;no live services unless existing harness;regression test before fix when feasible;assert behavior not implementation
DOCS:comment why/invariants/tradeoffs;rustdoc public reusable API;docs mention errors/panics/safety;examples use ? not unwrap;keep comments current or delete
MACROS:avoid unless clear win;input syntax mirrors output;compose with attrs/visibility;hygiene;prefer functions/traits first
UNSAFE:forbid unless user explicitly asks or existing module requires;small unsafe blocks;private module boundary;document SAFETY invariants;safe wrapper;tests/Miri if available;never manual Send/Sync without invariant proof
